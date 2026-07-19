//! EvalFabric implementation for modeled automotive diagnostics.

use std::{any::Any, sync::Arc};

use sim_kernel::{
    CORE_LOCAL_EVAL_FABRIC_CLASS_ID, CapabilityName, ClassRef, Consistency, Cx, Error, EvalFabric,
    EvalMode, EvalReply, EvalRequest, Expr, Object, ObjectCompat, Result, Symbol,
};
use sim_lib_auto_core::{
    AUTO_DIAGNOSTICS_READ, AUTO_TRANSPORT_CONNECT, AutoSession, BrandCaps, TransportPlacement,
    VehicleId,
};
use sim_lib_stream_fabric::{EffectLedgerCassette, EvalCassette, LedgeredRelayFabric};

use crate::{
    model::{ModeledVehicle, dtc_expr, string_field},
    request::DiagnosticRequest,
};

/// Replay wrapper used for cassette-backed diagnostic sessions.
pub type AutoDiagReplayFabric = LedgeredRelayFabric<AutoDiagFabric>;

/// Local automotive diagnostic fabric backed by synthetic ECU data.
#[sim_citizen_derive::non_citizen(
    reason = "live diagnostic eval-fabric handle; reconstruct from auto/AutoSession descriptor data",
    kind = "handle",
    descriptor = "auto/AutoSession"
)]
#[derive(Clone, Debug)]
pub struct AutoDiagFabric {
    session: AutoSession,
    vehicle: ModeledVehicle,
}

impl AutoDiagFabric {
    /// Builds a diagnostic fabric over a session and modeled vehicle.
    pub fn new(session: AutoSession, vehicle: ModeledVehicle) -> Self {
        Self { session, vehicle }
    }

    /// Builds a diagnostic fabric using the default synthetic vehicle.
    pub fn fixture() -> Self {
        auto_fabric(modeled_session())
    }

    /// Returns the active automotive session.
    pub fn session(&self) -> &AutoSession {
        &self.session
    }

    /// Returns the modeled vehicle served by the fabric.
    pub fn vehicle(&self) -> &ModeledVehicle {
        &self.vehicle
    }

    /// Reads DTCs from a modeled ECU through the session grant set.
    pub fn read_dtcs(&self, ecu: &str) -> Result<Vec<sim_lib_auto_core::Dtc>> {
        self.require_session_capability(&CapabilityName::new(AUTO_DIAGNOSTICS_READ))?;
        Ok(self.vehicle.ecu(ecu)?.dtcs.clone())
    }

    /// Reads one PID value from a modeled ECU through the session grant set.
    pub fn read_pid(&self, ecu: &str, pid: &str) -> Result<crate::PidValue> {
        self.require_session_capability(&CapabilityName::new(AUTO_DIAGNOSTICS_READ))?;
        Ok(self.vehicle.ecu(ecu)?.pid(pid)?.clone())
    }

    /// Reads freeze-frame records from a modeled ECU through the session grant set.
    pub fn freeze_frames(&self, ecu: &str) -> Result<Vec<crate::FreezeFrame>> {
        self.require_session_capability(&CapabilityName::new(AUTO_DIAGNOSTICS_READ))?;
        Ok(self.vehicle.ecu(ecu)?.freeze_frames.clone())
    }

    fn require_session_capability(&self, capability: &CapabilityName) -> Result<()> {
        if self.session.has_grant(capability) {
            Ok(())
        } else {
            Err(Error::CapabilityDenied {
                capability: capability.clone(),
            })
        }
    }
}

impl EvalFabric for AutoDiagFabric {
    fn realize(&self, cx: &mut Cx, request: EvalRequest) -> Result<EvalReply> {
        answer_request(cx, self, request)
    }
}

impl Object for AutoDiagFabric {
    fn display(&self, _cx: &mut Cx) -> Result<String> {
        Ok(format!("#<auto-diag-fabric {}>", self.session.vehicle.key))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ObjectCompat for AutoDiagFabric {
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

/// Builds a diagnostic fabric for `session`.
pub fn auto_fabric(session: AutoSession) -> AutoDiagFabric {
    AutoDiagFabric::new(session, ModeledVehicle::fixture())
}

/// Builds a cassette-backed diagnostic fabric for `session`.
pub fn cassette_auto_fabric(
    session: AutoSession,
    cassette: Arc<EvalCassette>,
) -> AutoDiagReplayFabric {
    LedgeredRelayFabric::new(auto_fabric(session), cassette)
}

/// Builds an empty effect-ledger-backed diagnostic cassette.
pub fn diagnostic_cassette() -> Arc<EvalCassette> {
    Arc::new(EvalCassette::new(Arc::new(EffectLedgerCassette::new())))
}

pub(crate) fn modeled_session() -> AutoSession {
    let grants = vec![CapabilityName::new(AUTO_DIAGNOSTICS_READ)];
    AutoSession::modeled(
        VehicleId::new("fixture", "vehicle-alpha"),
        BrandCaps::new("fixture-brand", grants.clone()),
        grants,
    )
}

fn answer_request(cx: &mut Cx, fabric: &AutoDiagFabric, request: EvalRequest) -> Result<EvalReply> {
    validate_request_controls(&request)?;
    let diagnostic = DiagnosticRequest::parse(&request.expr)?;
    let required = CapabilityName::new(diagnostic.required_capability());
    require_transport(cx, fabric, &request)?;
    require_diminished(cx, &fabric.session, &request, &required)?;

    let trace = request.trace;
    let expr = response_expr(fabric, &diagnostic)?;
    let value = cx.factory().expr(expr)?;
    Ok(EvalReply {
        value,
        diagnostics: cx.take_diagnostics(),
        trace: trace
            .then(|| cx.factory().symbol(Symbol::qualified("auto", "diag")).ok())
            .flatten(),
    })
}

fn require_transport(cx: &mut Cx, fabric: &AutoDiagFabric, request: &EvalRequest) -> Result<()> {
    if matches!(
        fabric.session.transport,
        TransportPlacement::LocalBridge { .. }
    ) {
        require_diminished(
            cx,
            &fabric.session,
            request,
            &CapabilityName::new(AUTO_TRANSPORT_CONNECT),
        )?;
    }
    Ok(())
}

fn require_diminished(
    cx: &mut Cx,
    session: &AutoSession,
    request: &EvalRequest,
    capability: &CapabilityName,
) -> Result<()> {
    cx.require_all(&request.required_capabilities)?;
    cx.require(capability)?;
    let active = session.diminished_grants(&request.required_capabilities);
    if active.contains(capability) {
        Ok(())
    } else {
        Err(Error::CapabilityDenied {
            capability: capability.clone(),
        })
    }
}

fn response_expr(fabric: &AutoDiagFabric, request: &DiagnosticRequest) -> Result<Expr> {
    match request {
        DiagnosticRequest::ReadDtcs { ecu } => {
            let ecu_data = fabric.vehicle.ecu(ecu)?;
            Ok(Expr::Map(vec![
                string_field("operation", "auto/read-dtc"),
                string_field("vehicle-vin", &fabric.vehicle.vin),
                string_field("ecu", &ecu_data.name),
                (
                    Expr::Symbol(Symbol::new("dtcs")),
                    Expr::List(ecu_data.dtcs.iter().map(dtc_expr).collect()),
                ),
                (
                    Expr::Symbol(Symbol::new("inventory")),
                    Expr::List(fabric.vehicle.inventory()),
                ),
            ]))
        }
        DiagnosticRequest::ReadPid { ecu, pid } => {
            let ecu_data = fabric.vehicle.ecu(ecu)?;
            let value = ecu_data.pid(pid)?;
            Ok(Expr::Map(vec![
                string_field("operation", "auto/read-pid"),
                string_field("vehicle-vin", &fabric.vehicle.vin),
                string_field("ecu", &ecu_data.name),
                (Expr::Symbol(Symbol::new("pid")), value.to_expr()),
            ]))
        }
        DiagnosticRequest::FreezeFrame { ecu } => {
            let ecu_data = fabric.vehicle.ecu(ecu)?;
            Ok(Expr::Map(vec![
                string_field("operation", "auto/freeze-frame"),
                string_field("vehicle-vin", &fabric.vehicle.vin),
                string_field("ecu", &ecu_data.name),
                (
                    Expr::Symbol(Symbol::new("freeze-frames")),
                    Expr::List(
                        ecu_data
                            .freeze_frames
                            .iter()
                            .map(crate::FreezeFrame::to_expr)
                            .collect(),
                    ),
                ),
            ]))
        }
        DiagnosticRequest::Control { operation, ecu } => Ok(Expr::Map(vec![
            string_field("operation", operation),
            string_field("ecu", ecu),
            (Expr::Symbol(Symbol::new("accepted")), Expr::Bool(true)),
        ])),
    }
}

fn validate_request_controls(request: &EvalRequest) -> Result<()> {
    if request.mode != EvalMode::Eval {
        return Err(Error::Eval(format!(
            "auto diagnostic request: unsupported eval mode {}",
            request.mode.as_symbol()
        )));
    }
    if request.deadline.is_some() {
        return Err(Error::Eval(
            "auto diagnostic request: deadline is unsupported".to_owned(),
        ));
    }
    if request.consistency == Consistency::RemoteOnly {
        return Err(Error::Eval(
            "auto diagnostic request: remote-only consistency is unsupported".to_owned(),
        ));
    }
    if matches!(request.answer_limit, Some(0)) {
        return Err(Error::Eval(
            "auto diagnostic request: answer_limit must be greater than zero".to_owned(),
        ));
    }
    if request.stream || request.stream_buffer.is_some() {
        return Err(Error::Eval(
            "auto diagnostic request: streaming replies are unsupported".to_owned(),
        ));
    }
    Ok(())
}
