use std::sync::Arc;

use sim_kernel::{
    CapabilityName, Consistency, Cx, Error, EvalFabric, EvalMode, EvalReply, EvalRequest, Expr,
    Lib, Result, Value, realize_final, testing::bare_cx as cx,
};
use sim_lib_auto_core::{
    AUTO_CONTROL_EXEC, AUTO_DIAGNOSTICS_READ, AUTO_TRANSPORT_CONNECT, AutoSession, BrandCaps,
    TransportPlacement, VehicleId,
};
use sim_lib_stream_fabric::LedgeredRelayFabric;

use crate::{
    AutoDiagFabric, AutoDiagLib, auto_diag_site_symbol, auto_fabric, cassette_auto_fabric,
    code_expr, diagnostic_cassette, freeze_frame_expr, install_auto_diag_lib, read_dtcs_expr,
    read_pid_expr,
};

#[test]
fn modeled_fabric_reads_dtcs_pids_and_freeze_frames() {
    let mut cx = cx_with(&[AUTO_DIAGNOSTICS_READ]);
    let fabric = auto_fabric(read_session());

    let dtcs = realize_final(
        &mut cx,
        &fabric,
        request(read_dtcs_expr("ME97"), &[AUTO_DIAGNOSTICS_READ]),
    )
    .unwrap();
    assert!(expr_text(&mut cx, &dtcs.value).contains("P0301"));

    let pid = realize_final(
        &mut cx,
        &fabric,
        request(read_pid_expr("ME97", "rpm"), &[AUTO_DIAGNOSTICS_READ]),
    )
    .unwrap();
    assert!(expr_text(&mut cx, &pid.value).contains("1840"));

    let freeze = realize_final(
        &mut cx,
        &fabric,
        request(freeze_frame_expr("ME97"), &[AUTO_DIAGNOSTICS_READ]),
    )
    .unwrap();
    assert!(expr_text(&mut cx, &freeze.value).contains("freeze-frames"));
}

#[test]
fn cassette_replays_recorded_diagnostic_reply() {
    let cassette = diagnostic_cassette();
    let mut cx = cx_with(&[AUTO_DIAGNOSTICS_READ]);
    let fabric = cassette_auto_fabric(read_session(), Arc::clone(&cassette));
    let request = request(read_dtcs_expr("ME97"), &[AUTO_DIAGNOSTICS_READ]);

    let first = fabric.realize(&mut cx, request.clone()).unwrap();
    assert_eq!(cassette.len(), 1);

    let replay = LedgeredRelayFabric::new(DenyAll, cassette);
    let second = replay.realize(&mut cx, request).unwrap();
    assert_eq!(
        expr_text(&mut cx, &first.value),
        expr_text(&mut cx, &second.value)
    );
}

#[test]
fn controlled_operations_require_diminished_control_capability() {
    let mut cx = cx_with(&[AUTO_DIAGNOSTICS_READ, AUTO_CONTROL_EXEC]);
    let read_only = auto_fabric(read_session());
    let denied = read_only.realize(
        &mut cx,
        request(
            code_expr("ME97"),
            &[AUTO_DIAGNOSTICS_READ, AUTO_CONTROL_EXEC],
        ),
    );
    assert!(matches!(
        denied,
        Err(Error::CapabilityDenied { capability }) if capability.as_str() == AUTO_CONTROL_EXEC
    ));

    let controlled = auto_fabric(control_session());
    let accepted = controlled
        .realize(
            &mut cx,
            request(
                code_expr("ME97"),
                &[AUTO_DIAGNOSTICS_READ, AUTO_CONTROL_EXEC],
            ),
        )
        .unwrap();
    assert!(expr_text(&mut cx, &accepted.value).contains("accepted"));
}

#[test]
fn local_bridge_is_default_denied_without_transport_grant() {
    let mut cx = cx_with(&[AUTO_DIAGNOSTICS_READ, AUTO_TRANSPORT_CONNECT]);
    let denied_session = read_session().with_transport(TransportPlacement::local_bridge("bench"));
    let denied = auto_fabric(denied_session).realize(
        &mut cx,
        request(
            read_dtcs_expr("ME97"),
            &[AUTO_DIAGNOSTICS_READ, AUTO_TRANSPORT_CONNECT],
        ),
    );
    assert!(matches!(
        denied,
        Err(Error::CapabilityDenied { capability }) if capability.as_str() == AUTO_TRANSPORT_CONNECT
    ));

    let allowed = bridge_session();
    let reply = auto_fabric(allowed)
        .realize(
            &mut cx,
            request(
                read_dtcs_expr("ME97"),
                &[AUTO_DIAGNOSTICS_READ, AUTO_TRANSPORT_CONNECT],
            ),
        )
        .unwrap();
    assert!(expr_text(&mut cx, &reply.value).contains("ME97"));
}

#[test]
fn diagnostic_lib_exports_modeled_site_value() {
    let manifest = AutoDiagLib.manifest();
    assert!(manifest.exports.iter().any(|export| {
        matches!(export, sim_kernel::Export::Site { symbol, .. } if symbol == &auto_diag_site_symbol())
    }));

    let mut cx = cx();
    install_auto_diag_lib(&mut cx).unwrap();
    let value = cx
        .registry()
        .site_by_symbol(&auto_diag_site_symbol())
        .cloned()
        .unwrap();
    assert!(value.object().as_eval_fabric().is_some());
}

#[test]
fn direct_modeled_methods_enforce_session_read_grant() {
    let fabric = AutoDiagFabric::new(
        AutoSession::modeled(
            VehicleId::new("fixture", "vehicle-alpha"),
            BrandCaps::new("fixture-brand", Vec::new()),
            Vec::new(),
        ),
        crate::ModeledVehicle::fixture(),
    );
    let denied = fabric.read_dtcs("ME97");
    assert!(matches!(
        denied,
        Err(Error::CapabilityDenied { capability }) if capability.as_str() == AUTO_DIAGNOSTICS_READ
    ));
}

fn read_session() -> AutoSession {
    session(&[AUTO_DIAGNOSTICS_READ], TransportPlacement::modeled())
}

fn control_session() -> AutoSession {
    session(
        &[AUTO_DIAGNOSTICS_READ, AUTO_CONTROL_EXEC],
        TransportPlacement::modeled(),
    )
}

fn bridge_session() -> AutoSession {
    session(
        &[AUTO_DIAGNOSTICS_READ, AUTO_TRANSPORT_CONNECT],
        TransportPlacement::local_bridge("bench"),
    )
}

fn session(capabilities: &[&'static str], transport: TransportPlacement) -> AutoSession {
    let grants = capabilities
        .iter()
        .copied()
        .map(CapabilityName::new)
        .collect::<Vec<_>>();
    AutoSession::new(
        VehicleId::new("fixture", "vehicle-alpha"),
        BrandCaps::new("fixture-brand", grants.clone()),
        transport,
        grants,
    )
}

fn request(expr: Expr, capabilities: &[&'static str]) -> EvalRequest {
    EvalRequest {
        expr,
        result_shape: None,
        required_capabilities: capabilities
            .iter()
            .copied()
            .map(CapabilityName::new)
            .collect(),
        deadline: None,
        consistency: Consistency::LocalFirst,
        mode: EvalMode::Eval,
        answer_limit: None,
        stream_buffer: None,
        stream: false,
        trace: false,
    }
}

fn cx_with(capabilities: &[&'static str]) -> Cx {
    let mut cx = cx();
    for capability in capabilities {
        cx.grant_named(capability);
    }
    cx
}

fn expr_text(cx: &mut Cx, value: &Value) -> String {
    format!("{:?}", value.object().as_expr(cx).unwrap())
}

struct DenyAll;

impl EvalFabric for DenyAll {
    fn realize(&self, _cx: &mut Cx, request: EvalRequest) -> Result<EvalReply> {
        Err(Error::CapabilityDenied {
            capability: request.required_capabilities[0].clone(),
        })
    }
}
