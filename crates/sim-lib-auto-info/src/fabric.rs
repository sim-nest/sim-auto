//! EvalFabric site for modeled repair-information lookup.

use std::sync::Arc;

use sim_kernel::{
    CORE_LOCAL_EVAL_FABRIC_CLASS_ID, ClassRef, Consistency, Cx, Error, EvalFabric, EvalMode,
    EvalReply, EvalRequest, Object, ObjectCompat, Result, Symbol,
};

use crate::{RepairProcedure, parse_repair_query, repair_catalog, repair_scene_from_catalog};

/// Modeled repair-information site.
#[derive(Clone)]
pub struct AutoInfoFabric {
    catalog: Arc<Vec<RepairProcedure>>,
}

impl AutoInfoFabric {
    /// Builds a modeled info fabric from an explicit catalog.
    pub fn new(catalog: Vec<RepairProcedure>) -> Self {
        Self {
            catalog: Arc::new(catalog),
        }
    }

    /// Builds the default synthetic fixture fabric.
    pub fn fixture() -> Self {
        Self::new(repair_catalog())
    }

    /// Returns the catalog this fabric serves.
    pub fn catalog(&self) -> &[RepairProcedure] {
        self.catalog.as_ref().as_slice()
    }
}

impl EvalFabric for AutoInfoFabric {
    fn realize(&self, cx: &mut Cx, request: EvalRequest) -> Result<EvalReply> {
        validate_request_controls(&request)?;
        let query = parse_repair_query(&request.expr)?;
        let scene = repair_scene_from_catalog(&query, self.catalog.as_ref().as_slice())?;
        let value = cx.factory().expr(scene)?;
        Ok(EvalReply {
            value,
            diagnostics: cx.take_diagnostics(),
            trace: request
                .trace
                .then(|| cx.factory().symbol(Symbol::qualified("auto", "info")).ok())
                .flatten(),
        })
    }
}

impl Object for AutoInfoFabric {
    fn display(&self, _cx: &mut Cx) -> Result<String> {
        Ok(format!("#<auto-info-site {} docs>", self.catalog.len()))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ObjectCompat for AutoInfoFabric {
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
            "auto/info request: unsupported eval mode {}",
            request.mode.as_symbol()
        )));
    }
    if request.consistency == Consistency::RemoteOnly {
        return Err(Error::Eval(
            "auto/info request: remote-only consistency is unsupported".to_owned(),
        ));
    }
    if matches!(request.answer_limit, Some(0)) {
        return Err(Error::Eval(
            "auto/info request: answer_limit must be greater than zero".to_owned(),
        ));
    }
    if request.stream || request.stream_buffer.is_some() {
        return Err(Error::Eval(
            "auto/info request: streaming replies are unsupported".to_owned(),
        ));
    }
    Ok(())
}
