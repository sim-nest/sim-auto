//! Automotive adapters for the generic operation gate.
use crate::{ManifestOperation, VendorBridge, VendorBridgeRequest};
use sim_kernel::{
    Cx, Datum, DatumStore, Error, Ref, Result, Symbol,
    effect::{Effect, effect_abort_op_key, effect_resume_op_key},
};
use sim_lib_operation_gate::{
    Approval, ApprovalUse, ApprovalVerifier, GateContext, GateRecord, GateRecordSink,
    SinkFailurePolicy, guard_operation,
};
use std::sync::{Mutex, MutexGuard};

/// In-memory modeled record sink used by automotive fixtures.
#[derive(Default)]
pub struct AutomotiveGateRecords(Mutex<Vec<GateRecord>>);
impl AutomotiveGateRecords {
    /// Creates an empty sink.
    pub fn new() -> Self {
        Self::default()
    }
    /// Returns all records.
    pub fn records(&self) -> Result<Vec<GateRecord>> {
        Ok(lock(&self.0)?.clone())
    }
}
impl GateRecordSink for AutomotiveGateRecords {
    fn record(&self, record: GateRecord) -> Result<()> {
        lock(&self.0)?.push(record);
        Ok(())
    }
}

/// Modeled verifier that accepts non-empty approval identities.
pub struct AutomotiveApprovalVerifier;
impl ApprovalVerifier for AutomotiveApprovalVerifier {
    fn verify(&self, approval: &Approval) -> Result<()> {
        if approval.id.trim().is_empty() {
            Err(Error::Eval(
                "automotive approval id must not be empty".to_owned(),
            ))
        } else {
            Ok(())
        }
    }
}

/// In-memory atomic approval-use adapter.
#[derive(Default)]
pub struct AutomotiveApprovalUse(Mutex<Vec<String>>);
impl AutomotiveApprovalUse {
    /// Creates an empty use store.
    pub fn new() -> Self {
        Self::default()
    }
    /// Number of successful consumptions.
    pub fn use_count(&self) -> Result<usize> {
        Ok(lock(&self.0)?.len())
    }
}
impl ApprovalUse for AutomotiveApprovalUse {
    fn consume(&self, approval: &Approval) -> Result<()> {
        let mut used = lock(&self.0)?;
        if used.iter().any(|id| id == &approval.id) {
            return Err(Error::Eval(format!(
                "approval {} already used",
                approval.id
            )));
        }
        used.push(approval.id.clone());
        Ok(())
    }
}

/// Dispatches one explicitly declared automotive operation through the generic gate.
pub fn guard_vendor_operation(
    cx: &mut Cx,
    request: &VendorBridgeRequest,
    operation: &ManifestOperation,
    records: &AutomotiveGateRecords,
    approval_use: &AutomotiveApprovalUse,
    bridge: &dyn VendorBridge,
) -> Result<Ref> {
    let effect = Effect::new(
        Symbol::qualified("auto", "vendor-effect"),
        operation.declaration.subject.clone(),
        input_ref(cx, request)?,
        Ref::Symbol(Symbol::qualified("core", "Expr")),
        effect_resume_op_key(),
        effect_abort_op_key(),
    )
    .requiring(operation.declaration.capability.clone());
    guard_operation(
        cx,
        &operation.declaration,
        effect,
        GateContext {
            approval: request.approval.as_ref(),
            verifier: &AutomotiveApprovalVerifier,
            approval_use,
            sink: records,
            sink_failure: SinkFailurePolicy::FailClosed,
        },
        |cx, _| {
            let datum = Datum::try_from(bridge.call(cx, request)?)?;
            Ok(Ref::Content(cx.datum_store_mut().intern(datum)?))
        },
    )
}

fn input_ref(cx: &mut Cx, request: &VendorBridgeRequest) -> Result<Ref> {
    let datum = Datum::Node {
        tag: Symbol::qualified("auto", "VendorOperationInput"),
        fields: vec![
            (Symbol::new("site"), Datum::String(request.site.clone())),
            (
                Symbol::new("lane"),
                Datum::String(request.lane.name.clone()),
            ),
            (Symbol::new("operation"), Datum::String(request.op.clone())),
            (
                Symbol::new("vehicle-namespace"),
                Datum::String(request.vehicle.namespace.clone()),
            ),
            (
                Symbol::new("vehicle-key"),
                Datum::String(request.vehicle.key.clone()),
            ),
            (Symbol::new("args"), Datum::try_from(request.args.clone())?),
        ],
    };
    Ok(Ref::Content(cx.datum_store_mut().intern(datum)?))
}
fn lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>> {
    mutex
        .lock()
        .map_err(|_| Error::Eval("automotive gate mutex poisoned".to_owned()))
}
