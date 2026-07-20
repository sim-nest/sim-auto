//! EvalFabric implementation for vehicle identity lookup.

use std::sync::Arc;

use sim_kernel::{
    CORE_LOCAL_EVAL_FABRIC_CLASS_ID, ClassRef, Consistency, Cx, Error, EvalFabric, EvalMode,
    EvalReply, EvalRequest, Object, ObjectCompat, Result, Symbol,
};

use crate::{VehicleLookupBridge, VehicleLookupRequest, VehicleLookupRouter};

/// Local vehicle identity fabric backed by a lookup bridge.
#[derive(Clone)]
pub struct VehicleIdentityFabric {
    bridge: Arc<dyn VehicleLookupBridge>,
}

impl VehicleIdentityFabric {
    /// Builds a vehicle identity fabric over a bridge.
    pub fn new(bridge: Arc<dyn VehicleLookupBridge>) -> Self {
        Self { bridge }
    }

    /// Builds the default modeled identity fabric.
    pub fn fixture() -> Self {
        Self::new(Arc::new(VehicleLookupRouter::default()))
    }
}

impl Default for VehicleIdentityFabric {
    fn default() -> Self {
        Self::fixture()
    }
}

impl EvalFabric for VehicleIdentityFabric {
    fn realize(&self, cx: &mut Cx, request: EvalRequest) -> Result<EvalReply> {
        validate_request_controls(&request)?;
        let lookup = VehicleLookupRequest::parse(&request.expr)?;
        let trace = request.trace;
        let record = self.bridge.lookup(cx, &lookup)?;
        let value = cx.factory().expr(record.to_expr())?;
        Ok(EvalReply {
            value,
            diagnostics: cx.take_diagnostics(),
            trace: trace
                .then(|| {
                    cx.factory()
                        .symbol(Symbol::qualified("auto", "vehicle"))
                        .ok()
                })
                .flatten(),
        })
    }
}

impl Object for VehicleIdentityFabric {
    fn display(&self, _cx: &mut Cx) -> Result<String> {
        Ok("#<auto-vehicle-identity>".to_owned())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ObjectCompat for VehicleIdentityFabric {
    fn class(&self, cx: &mut Cx) -> Result<ClassRef> {
        cx.factory().class_stub(
            CORE_LOCAL_EVAL_FABRIC_CLASS_ID,
            Symbol::qualified("core", "LocalEvalFabric"),
        )
    }

    fn as_eval_fabric(&self) -> Option<&dyn EvalFabric> {
        Some(self)
    }
}

fn validate_request_controls(request: &EvalRequest) -> Result<()> {
    if request.mode != EvalMode::Eval {
        return Err(Error::Eval(format!(
            "auto vehicle request: unsupported eval mode {}",
            request.mode.as_symbol()
        )));
    }
    if request.deadline.is_some() {
        return Err(Error::Eval(
            "auto vehicle request: deadline is unsupported".to_owned(),
        ));
    }
    if request.consistency == Consistency::RemoteOnly {
        return Err(Error::Eval(
            "auto vehicle request: remote-only consistency is unsupported".to_owned(),
        ));
    }
    if matches!(request.answer_limit, Some(0)) {
        return Err(Error::Eval(
            "auto vehicle request: answer_limit must be greater than zero".to_owned(),
        ));
    }
    if request.stream || request.stream_buffer.is_some() {
        return Err(Error::Eval(
            "auto vehicle request: streaming replies are unsupported".to_owned(),
        ));
    }
    Ok(())
}
