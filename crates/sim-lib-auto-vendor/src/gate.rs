//! Capability and warrant gate for vendor operations.

use std::cell::RefCell;
use std::sync::{Mutex, MutexGuard};

use sim_kernel::{
    CapabilityName, CapabilitySet, Cx, Datum, DatumStore, Error, Expr, Ref, Result, Symbol,
    diminish,
    effect::{Effect, effect_abort_op_key, effect_resume_op_key, resolve_effect},
};
use sim_lib_auto_core::SiteManifest;

use crate::{
    ManifestOperation, VendorBridge, VendorBridgeRequest, VendorEffectClass, manifest_operation,
};

/// Human-review warrant attached to an irreversible vendor action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VendorWarrant {
    /// Stable warrant identifier.
    pub id: String,
    /// Human-readable reason bound to the action.
    pub reason: String,
}

impl VendorWarrant {
    /// Builds a warrant.
    pub fn new(id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            reason: reason.into(),
        }
    }
}

/// One gate record proving policy that admitted a vendor operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VendorGateRecord {
    /// Manifest site label.
    pub site: String,
    /// Operation symbol text.
    pub operation: String,
    /// Required capability.
    pub capability: CapabilityName,
    /// Effect class enforced by the gate.
    pub effect: VendorEffectClass,
    /// Whether a reversal artifact was attached.
    pub reversal_artifact: bool,
    /// Content key carried by the reversal artifact.
    pub reversal_content_key: Option<String>,
    /// Warrant id, when one was required.
    pub warrant: Option<String>,
    /// Whether the human gate was open.
    pub human_approved: bool,
}

/// In-memory gate ledger for public modeled bridges.
#[derive(Default)]
pub struct VendorGateLedger {
    records: Mutex<Vec<VendorGateRecord>>,
}

impl VendorGateLedger {
    /// Builds an empty gate ledger.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns all recorded gate entries.
    pub fn records(&self) -> Result<Vec<VendorGateRecord>> {
        Ok(lock(&self.records, "vendor gate ledger")?.clone())
    }

    fn push(&self, record: VendorGateRecord) -> Result<()> {
        lock(&self.records, "vendor gate ledger")?.push(record);
        Ok(())
    }
}

/// Dispatches a vendor bridge request through the policy required by its effect class.
pub fn warranted_effect(
    cx: &mut Cx,
    manifest: &SiteManifest,
    request: VendorBridgeRequest,
    required_capabilities: &[CapabilityName],
    ledger: &VendorGateLedger,
    bridge: &dyn VendorBridge,
) -> Result<Expr> {
    let operation = manifest_operation(manifest, &request.op)?;
    require_diminished(cx, required_capabilities, &operation.capability)?;

    match operation.effect {
        VendorEffectClass::Pure => bridge.call(cx, &request),
        VendorEffectClass::Reversible => dispatch_ledgered(cx, ledger, bridge, request, operation),
        VendorEffectClass::Irreversible => {
            require_irreversible_gate(&request)?;
            dispatch_ledgered(cx, ledger, bridge, request, operation)
        }
    }
}

fn dispatch_ledgered(
    cx: &mut Cx,
    ledger: &VendorGateLedger,
    bridge: &dyn VendorBridge,
    request: VendorBridgeRequest,
    operation: ManifestOperation,
) -> Result<Expr> {
    let produced = RefCell::new(None);
    let effect = effect_for(&request, &operation);
    resolve_effect(cx, effect, |cx, _effect| {
        let reply = bridge.call(cx, &request)?;
        let reference = expr_ref(cx, &reply)?;
        produced.replace(Some(reply));
        Ok(reference)
    })?;
    let reply = produced.into_inner().ok_or_else(|| {
        Error::Eval("auto vendor cassette replay did not carry a reply".to_owned())
    })?;
    ledger.push(gate_record(&request, &operation))?;
    Ok(reply)
}

fn require_diminished(
    cx: &Cx,
    required_capabilities: &[CapabilityName],
    capability: &CapabilityName,
) -> Result<()> {
    cx.require_all(required_capabilities)?;
    cx.require(capability)?;
    let allowed = capability_set(required_capabilities.iter().cloned());
    let active = diminish(cx.capabilities(), &allowed);
    if active.contains(capability) {
        Ok(())
    } else {
        Err(Error::CapabilityDenied {
            capability: capability.clone(),
        })
    }
}

fn require_irreversible_gate(request: &VendorBridgeRequest) -> Result<()> {
    let Some(reversal_artifact) = &request.reversal_artifact else {
        return Err(Error::Eval(format!(
            "auto vendor irreversible op {} requires a reversal artifact",
            request.op
        )));
    };
    reversal_content_key(reversal_artifact)?;
    let Some(warrant) = &request.warrant else {
        return Err(Error::Eval(format!(
            "auto vendor irreversible op {} requires a warrant",
            request.op
        )));
    };
    if warrant.id.trim().is_empty() {
        return Err(Error::Eval(
            "auto vendor warrant id must not be empty".to_owned(),
        ));
    }
    if !request.human_approved {
        return Err(Error::Eval(format!(
            "auto vendor irreversible op {} requires human approval",
            request.op
        )));
    }
    Ok(())
}

fn effect_for(request: &VendorBridgeRequest, operation: &ManifestOperation) -> Effect {
    Effect::new(
        Symbol::qualified("auto", "vendor-effect"),
        Ref::Symbol(Symbol::qualified("auto", sanitize_symbol(&request.site))),
        Ref::Symbol(Symbol::new(request.op.clone())),
        Ref::Symbol(Symbol::qualified("core", "Expr")),
        effect_resume_op_key(),
        effect_abort_op_key(),
    )
    .requiring(operation.capability.clone())
}

fn expr_ref(cx: &mut Cx, expr: &Expr) -> Result<Ref> {
    let id = cx
        .datum_store_mut()
        .intern(Datum::String(format!("{expr:?}")))?;
    Ok(Ref::Content(id))
}

fn gate_record(request: &VendorBridgeRequest, operation: &ManifestOperation) -> VendorGateRecord {
    VendorGateRecord {
        site: request.site.clone(),
        operation: request.op.clone(),
        capability: operation.capability.clone(),
        effect: operation.effect,
        reversal_artifact: request.reversal_artifact.is_some(),
        reversal_content_key: request
            .reversal_artifact
            .as_ref()
            .and_then(|artifact| reversal_content_key(artifact).ok()),
        warrant: request.warrant.as_ref().map(|warrant| warrant.id.clone()),
        human_approved: request.human_approved,
    }
}

fn reversal_content_key(artifact: &Expr) -> Result<String> {
    let Expr::Map(entries) = artifact else {
        return Err(Error::Eval(
            "auto vendor reversal artifact requires a content-key field".to_owned(),
        ));
    };
    for (key, value) in entries {
        if field_name(key).as_deref() == Some("content-key") {
            return field_text(value);
        }
    }
    Err(Error::Eval(
        "auto vendor reversal artifact requires a content-key field".to_owned(),
    ))
}

fn field_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::String(value) => Some(value.trim_start_matches(':').to_owned()),
        Expr::Symbol(symbol) => Some(symbol.name.trim_start_matches(':').to_owned()),
        _ => None,
    }
}

fn field_text(expr: &Expr) -> Result<String> {
    match expr {
        Expr::String(value) => Ok(value.clone()),
        Expr::Symbol(symbol) => Ok(symbol.as_qualified_str()),
        _ => Err(Error::Eval(
            "auto vendor reversal content-key must be string or symbol".to_owned(),
        )),
    }
}

fn capability_set(capabilities: impl IntoIterator<Item = CapabilityName>) -> CapabilitySet {
    capabilities
        .into_iter()
        .fold(CapabilitySet::new(), CapabilitySet::grant)
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

fn lock<'a, T>(mutex: &'a Mutex<T>, name: &str) -> Result<MutexGuard<'a, T>> {
    mutex
        .lock()
        .map_err(|_| Error::Eval(format!("{name} mutex poisoned")))
}
