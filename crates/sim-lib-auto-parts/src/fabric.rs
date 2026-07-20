//! Eval-fabric wrapper for modeled parts lookup and ordering.

use std::sync::Arc;

use sim_kernel::{
    CORE_LOCAL_EVAL_FABRIC_CLASS_ID, ClassRef, Consistency, Cx, Error, EvalFabric, EvalMode,
    EvalReply, EvalRequest, Object, ObjectCompat, Result, Symbol,
};
use sim_lib_auto_vendor::VendorGateLedger;

use crate::{
    ModeledOrderLedger, PartsDir, PartsRequest, modeled_epc_dir, parse_parts_request,
    place_order_with_gate,
};

/// Modeled automotive parts fabric.
#[sim_citizen_derive::non_citizen(
    reason = "modeled parts eval-fabric handle; reconstruct from auto parts fixture data",
    kind = "handle",
    descriptor = "auto/PartLine"
)]
#[derive(Clone)]
pub struct AutoPartsFabric {
    catalog: PartsDir,
    order_ledger: Arc<ModeledOrderLedger>,
    gate_ledger: Arc<VendorGateLedger>,
}

impl AutoPartsFabric {
    /// Builds a fabric over a catalog and ledgers.
    pub fn new(
        catalog: PartsDir,
        order_ledger: Arc<ModeledOrderLedger>,
        gate_ledger: Arc<VendorGateLedger>,
    ) -> Self {
        Self {
            catalog,
            order_ledger,
            gate_ledger,
        }
    }

    /// Builds the public modeled fixture fabric.
    pub fn fixture() -> Self {
        Self::new(
            modeled_epc_dir(),
            Arc::new(ModeledOrderLedger::new()),
            Arc::new(VendorGateLedger::new()),
        )
    }

    /// Returns the modeled order ledger.
    pub fn order_ledger(&self) -> &Arc<ModeledOrderLedger> {
        &self.order_ledger
    }

    /// Returns the vendor gate ledger.
    pub fn gate_ledger(&self) -> &Arc<VendorGateLedger> {
        &self.gate_ledger
    }
}

impl EvalFabric for AutoPartsFabric {
    fn realize(&self, cx: &mut Cx, request: EvalRequest) -> Result<EvalReply> {
        validate_request_controls(&request)?;
        let trace = request.trace;
        let expr = match parse_parts_request(&request.expr)? {
            PartsRequest::CatalogGet { path } => {
                self.catalog.get_path(cx, &path)?.object().as_expr(cx)?
            }
            PartsRequest::Order { supplier, lines } => place_order_with_gate(
                cx,
                supplier,
                lines,
                self.order_ledger.as_ref(),
                self.gate_ledger.as_ref(),
            )?
            .to_expr(),
        };
        let value = cx.factory().expr(expr)?;
        Ok(EvalReply {
            value,
            diagnostics: cx.take_diagnostics(),
            trace: trace
                .then(|| cx.factory().symbol(Symbol::qualified("auto", "parts")).ok())
                .flatten(),
        })
    }
}

impl Object for AutoPartsFabric {
    fn display(&self, _cx: &mut Cx) -> Result<String> {
        Ok("#<auto-parts-fabric>".to_owned())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ObjectCompat for AutoPartsFabric {
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
            "auto parts request: unsupported eval mode {}",
            request.mode.as_symbol()
        )));
    }
    if request.deadline.is_some() {
        return Err(Error::Eval(
            "auto parts request: deadline is unsupported".to_owned(),
        ));
    }
    if request.consistency == Consistency::RemoteOnly {
        return Err(Error::Eval(
            "auto parts request: remote-only consistency is unsupported".to_owned(),
        ));
    }
    if request.stream || request.stream_buffer.is_some() {
        return Err(Error::Eval(
            "auto parts request: streaming replies are unsupported".to_owned(),
        ));
    }
    Ok(())
}
