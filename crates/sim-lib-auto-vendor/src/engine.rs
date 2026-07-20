//! Manifest-backed vendor site fabric.

use std::sync::Arc;

use sim_kernel::{
    CORE_LOCAL_EVAL_FABRIC_CLASS_ID, ClassRef, Consistency, Cx, Error, EvalFabric, EvalMode,
    EvalReply, EvalRequest, Object, ObjectCompat, Result, Symbol,
};
use sim_lib_auto_core::SiteManifest;
use sim_lib_stream_fabric::{EffectLedgerCassette, EvalCassette, LedgeredRelayFabric};

use crate::{VendorBridge, VendorGateLedger, request::parse_vendor_request, warranted_effect};

/// Replay wrapper used for cassette-backed vendor sessions.
pub type VendorReplayFabric = LedgeredRelayFabric<VendorSiteFabric>;

/// Automotive vendor site backed by a manifest and bridge.
#[sim_citizen_derive::non_citizen(
    reason = "live vendor eval-fabric handle; reconstruct from auto/SiteManifest descriptor data",
    kind = "handle",
    descriptor = "auto/SiteManifest"
)]
#[derive(Clone)]
pub struct VendorSiteFabric {
    manifest: SiteManifest,
    bridge: Arc<dyn VendorBridge>,
    gate_ledger: Arc<VendorGateLedger>,
}

impl VendorSiteFabric {
    /// Builds a vendor site fabric from a manifest and bridge.
    pub fn new(manifest: SiteManifest, bridge: Arc<dyn VendorBridge>) -> Self {
        Self::with_gate_ledger(manifest, bridge, Arc::new(VendorGateLedger::new()))
    }

    /// Builds a vendor site fabric with an explicit gate ledger.
    pub fn with_gate_ledger(
        manifest: SiteManifest,
        bridge: Arc<dyn VendorBridge>,
        gate_ledger: Arc<VendorGateLedger>,
    ) -> Self {
        Self {
            manifest,
            bridge,
            gate_ledger,
        }
    }

    /// Returns the manifest this site serves.
    pub fn manifest(&self) -> &SiteManifest {
        &self.manifest
    }

    /// Returns the gate ledger.
    pub fn gate_ledger(&self) -> &Arc<VendorGateLedger> {
        &self.gate_ledger
    }
}

impl EvalFabric for VendorSiteFabric {
    fn realize(&self, cx: &mut Cx, request: EvalRequest) -> Result<EvalReply> {
        validate_request_controls(&request)?;
        let bridge_request = parse_vendor_request(&self.manifest, &request.expr)?;
        let trace = request.trace;
        let expr = warranted_effect(
            cx,
            &self.manifest,
            bridge_request,
            &request.required_capabilities,
            &self.gate_ledger,
            self.bridge.as_ref(),
        )?;
        let value = cx.factory().expr(expr)?;
        Ok(EvalReply {
            value,
            diagnostics: cx.take_diagnostics(),
            trace: trace
                .then(|| {
                    cx.factory()
                        .symbol(Symbol::qualified("auto", "vendor"))
                        .ok()
                })
                .flatten(),
        })
    }
}

impl Object for VendorSiteFabric {
    fn display(&self, _cx: &mut Cx) -> Result<String> {
        Ok(format!("#<auto-vendor-site {}>", self.manifest.site))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ObjectCompat for VendorSiteFabric {
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

/// Builds a cassette-backed vendor fabric for `manifest`.
pub fn cassette_vendor_fabric(
    manifest: SiteManifest,
    bridge: Arc<dyn VendorBridge>,
    cassette: Arc<EvalCassette>,
) -> VendorReplayFabric {
    LedgeredRelayFabric::new(VendorSiteFabric::new(manifest, bridge), cassette)
}

/// Builds an empty effect-ledger-backed vendor cassette.
pub fn vendor_cassette() -> Arc<EvalCassette> {
    Arc::new(EvalCassette::new(Arc::new(EffectLedgerCassette::new())))
}

fn validate_request_controls(request: &EvalRequest) -> Result<()> {
    if request.mode != EvalMode::Eval {
        return Err(Error::Eval(format!(
            "auto vendor request: unsupported eval mode {}",
            request.mode.as_symbol()
        )));
    }
    if request.deadline.is_some() {
        return Err(Error::Eval(
            "auto vendor request: deadline is unsupported".to_owned(),
        ));
    }
    if request.consistency == Consistency::RemoteOnly {
        return Err(Error::Eval(
            "auto vendor request: remote-only consistency is unsupported".to_owned(),
        ));
    }
    if matches!(request.answer_limit, Some(0)) {
        return Err(Error::Eval(
            "auto vendor request: answer_limit must be greater than zero".to_owned(),
        ));
    }
    if request.stream || request.stream_buffer.is_some() {
        return Err(Error::Eval(
            "auto vendor request: streaming replies are unsupported".to_owned(),
        ));
    }
    Ok(())
}
