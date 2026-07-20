//! Manifest operation classification.

use sim_kernel::{CapabilityName, Error, Result};
use sim_lib_auto_core::{
    AUTO_CONTROL_EXEC, AUTO_DIAGNOSTICS_READ, AUTO_ORDER, AUTO_SERVICE_WRITE, AutoLane, OpCap,
    SiteManifest,
};

/// Effect policy class enforced before vendor bridge dispatch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VendorEffectClass {
    /// Read-only request. No effect ledger entry is needed.
    Pure,
    /// Operation with a reversal path. The kernel effect ledger records it.
    Reversible,
    /// Operation that requires a reversal artifact, warrant, and human gate.
    Irreversible,
}

impl VendorEffectClass {
    /// Returns the stable effect class name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pure => "pure",
            Self::Reversible => "reversible",
            Self::Irreversible => "irreversible",
        }
    }
}

/// Operation policy derived from one [`SiteManifest`] operation name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestOperation {
    /// Operation symbol text.
    pub operation: String,
    /// Manifest lane selected for this operation.
    pub lane: AutoLane,
    /// Capability required by the operation.
    pub capability: CapabilityName,
    /// Effect class applied by the operation.
    pub effect: VendorEffectClass,
}

/// Looks up and classifies one operation declared by `manifest`.
pub fn manifest_operation(manifest: &SiteManifest, operation: &str) -> Result<ManifestOperation> {
    if !manifest.operations.iter().any(|item| item == operation) {
        return Err(Error::Eval(format!(
            "auto vendor manifest {} does not declare operation {operation}",
            manifest.site
        )));
    }
    let policy = manifest
        .op_caps
        .iter()
        .find(|item| item.operation == operation);
    let effect = policy
        .map(effect_from_policy)
        .transpose()?
        .unwrap_or_else(|| classify_operation(operation));
    Ok(ManifestOperation {
        operation: operation.to_owned(),
        lane: lane_for(manifest, operation, effect),
        capability: policy
            .map(|item| item.capability.clone())
            .unwrap_or_else(|| capability_for(operation, effect)),
        effect,
    })
}

fn effect_from_policy(policy: &OpCap) -> Result<VendorEffectClass> {
    match policy.effect_class.to_ascii_lowercase().as_str() {
        "pure" | "read" | "diagnostic-read" => Ok(VendorEffectClass::Pure),
        "reversible" | "service-write" | "control-write" => Ok(VendorEffectClass::Reversible),
        "irreversible" | "control-exec" => Ok(VendorEffectClass::Irreversible),
        other => Err(Error::Eval(format!(
            "auto vendor manifest op {} has unknown effect class {other}",
            policy.operation
        ))),
    }
}

fn classify_operation(operation: &str) -> VendorEffectClass {
    let lower = operation.to_ascii_lowercase();
    if lower.contains("read") || lower.contains("query") || lower.starts_with("brand/") {
        VendorEffectClass::Pure
    } else if lower.contains("service")
        || lower.contains("write")
        || lower.contains("reset")
        || lower.contains("order")
    {
        VendorEffectClass::Reversible
    } else {
        VendorEffectClass::Irreversible
    }
}

fn capability_for(operation: &str, effect: VendorEffectClass) -> CapabilityName {
    let name = match effect {
        VendorEffectClass::Pure => AUTO_DIAGNOSTICS_READ,
        VendorEffectClass::Reversible if operation.to_ascii_lowercase().contains("order") => {
            AUTO_ORDER
        }
        VendorEffectClass::Reversible => AUTO_SERVICE_WRITE,
        VendorEffectClass::Irreversible => AUTO_CONTROL_EXEC,
    };
    CapabilityName::new(name)
}

fn lane_for(manifest: &SiteManifest, operation: &str, effect: VendorEffectClass) -> AutoLane {
    let lower = operation.to_ascii_lowercase();
    let preferred = match effect {
        VendorEffectClass::Pure => "diagnostics",
        VendorEffectClass::Reversible if lower.contains("order") => "parts",
        VendorEffectClass::Reversible => "service",
        VendorEffectClass::Irreversible => "control",
    };
    let lane = manifest
        .lanes
        .iter()
        .find(|lane| operation.contains(lane.as_str()))
        .or_else(|| {
            manifest
                .lanes
                .iter()
                .find(|lane| lane.as_str() == preferred)
        })
        .or_else(|| {
            manifest
                .lanes
                .iter()
                .find(|lane| lower.contains(&lane.to_ascii_lowercase()))
        })
        .or_else(|| manifest.lanes.first())
        .cloned()
        .unwrap_or_else(|| preferred.to_owned());
    AutoLane::new(lane)
}
