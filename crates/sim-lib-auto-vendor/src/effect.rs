//! Explicit automotive manifest operation policy.
use sim_kernel::{Error, Ref, Result, Symbol};
use sim_lib_auto_core::{AutoLane, SiteManifest};
use sim_lib_operation_gate::{ExecutionMode, OperationDeclaration};

/// Explicit operation policy adapted from an automotive site manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestOperation {
    /// Operation symbol text.
    pub operation: String,
    /// Explicit manifest lane.
    pub lane: AutoLane,
    /// Generic gate declaration.
    pub declaration: OperationDeclaration,
}

/// Looks up one fully declared operation. No policy is inferred from its name.
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
        .find(|item| item.operation == operation)
        .ok_or_else(|| {
            Error::Eval(format!(
                "auto vendor manifest {} operation {operation} has no explicit policy",
                manifest.site
            ))
        })?;
    let mode = match policy.effect_class.as_str() {
        "pure" | "observation" => ExecutionMode::Observation,
        "reversible" | "recorded" => ExecutionMode::Recorded,
        "irreversible" | "reviewed" => ExecutionMode::Reviewed,
        other => {
            return Err(Error::Eval(format!(
                "auto vendor manifest op {operation} has unknown execution mode {other}"
            )));
        }
    };
    let lane = manifest
        .lanes
        .iter()
        .find(|lane| operation.starts_with(&format!("{lane}/")))
        .or_else(|| match mode {
            ExecutionMode::Recorded => manifest
                .lanes
                .iter()
                .find(|lane| lane.as_str() == "service"),
            ExecutionMode::Reviewed => manifest
                .lanes
                .iter()
                .find(|lane| lane.as_str() == "control" || lane.as_str() == "flash"),
            ExecutionMode::Observation => None,
        })
        .or_else(|| manifest.lanes.first())
        .cloned()
        .ok_or_else(|| {
            Error::Eval(format!(
                "auto vendor manifest {} declares no lanes",
                manifest.site
            ))
        })?;
    Ok(ManifestOperation {
        operation: operation.to_owned(),
        lane: AutoLane::new(lane),
        declaration: OperationDeclaration {
            operation: operation.to_owned(),
            subject: Ref::Symbol(Symbol::qualified("auto", sanitize_symbol(&manifest.site))),
            capability: policy.capability.clone(),
            mode,
        },
    })
}

fn sanitize_symbol(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}
