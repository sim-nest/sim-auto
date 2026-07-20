use std::sync::Arc;

use sim_kernel::{
    CapabilityName, Consistency, Cx, Error, EvalFabric, EvalMode, EvalRequest, Expr, Lib, Symbol,
    Value, testing::bare_cx as cx,
};
use sim_lib_auto_core::{
    AUTO_CONTROL_EXEC, AUTO_DIAGNOSTICS_READ, AUTO_ORDER, AUTO_SERVICE_WRITE, SiteManifest,
};

use crate::{
    AutoVendorLib, ModeledVendorBridge, VendorEffectClass, VendorSiteFabric, VendorWarrant,
    auto_vendor_site_symbol, cassette_vendor_fabric, install_auto_vendor_lib, manifest_operation,
    vendor_cassette, vendor_irreversible_request_expr, vendor_request_expr,
};

#[test]
fn library_exports_manifest_sites() {
    let manifest = fixture_manifest();
    let lib = AutoVendorLib::modeled(vec![manifest.clone()]);
    let export_symbol = auto_vendor_site_symbol(&manifest);
    assert!(lib.manifest().exports.iter().any(|export| {
        matches!(export, sim_kernel::Export::Site { symbol, .. } if symbol == &export_symbol)
    }));

    let mut cx = cx();
    install_auto_vendor_lib(&mut cx, vec![manifest.clone()]).unwrap();
    let value = cx
        .registry()
        .site_by_symbol(&export_symbol)
        .cloned()
        .unwrap();
    assert!(value.object().as_eval_fabric().is_some());
}

#[test]
fn pure_read_needs_only_read_capability_and_no_ledger() {
    let mut cx = cx_with(&[AUTO_DIAGNOSTICS_READ]);
    let fabric = vendor_fabric();

    let reply = fabric
        .realize(
            &mut cx,
            request(
                vendor_request_expr("read/inventory", Expr::Map(Vec::new())),
                &[AUTO_DIAGNOSTICS_READ],
            ),
        )
        .unwrap();

    assert!(expr_text(&mut cx, &reply.value).contains("read/inventory"));
    assert_eq!(cx.effect_ledger().records().len(), 0);
    assert!(fabric.gate_ledger().records().unwrap().is_empty());
}

#[test]
fn reversible_op_needs_capability_and_records_ledgers() {
    let mut cx = cx_with(&[AUTO_SERVICE_WRITE]);
    let fabric = vendor_fabric();

    let denied = fabric.realize(
        &mut cx,
        request(
            vendor_request_expr("service/write-note", Expr::String("note".to_owned())),
            &[AUTO_DIAGNOSTICS_READ],
        ),
    );
    assert!(matches!(
        denied,
        Err(Error::CapabilityDenied { capability }) if capability.as_str() == AUTO_DIAGNOSTICS_READ
    ));

    let reply = fabric
        .realize(
            &mut cx,
            request(
                vendor_request_expr("service/write-note", Expr::String("note".to_owned())),
                &[AUTO_SERVICE_WRITE],
            ),
        )
        .unwrap();

    assert!(expr_text(&mut cx, &reply.value).contains("service/write-note"));
    assert_eq!(cx.effect_ledger().records().len(), 1);
    let records = fabric.gate_ledger().records().unwrap();
    assert_eq!(records[0].effect, VendorEffectClass::Reversible);
    assert_eq!(records[0].capability.as_str(), AUTO_SERVICE_WRITE);
}

#[test]
fn irreversible_op_requires_reversal_warrant_and_human_gate() {
    let mut cx = cx_with(&[AUTO_CONTROL_EXEC]);
    let fabric = vendor_fabric();

    let missing_artifact = fabric.realize(
        &mut cx,
        request(
            vendor_request_expr("control/code", Expr::String("coding".to_owned())),
            &[AUTO_CONTROL_EXEC],
        ),
    );
    assert!(matches!(missing_artifact, Err(Error::Eval(message)) if message.contains("reversal")));

    let missing_human = fabric.realize(
        &mut cx,
        request(
            vendor_request_expr("control/code", Expr::String("coding".to_owned())),
            &[AUTO_CONTROL_EXEC],
        )
        .with_expr(
            vendor_irreversible_request_expr(
                "control/code",
                Expr::String("coding".to_owned()),
                Expr::String("stock-map".to_owned()),
                "warrant-1",
            )
            .without_human_gate(),
        ),
    );
    assert!(matches!(missing_human, Err(Error::Eval(message)) if message.contains("human")));

    let reply = fabric
        .realize(
            &mut cx,
            request(
                vendor_irreversible_request_expr(
                    "control/code",
                    Expr::String("coding".to_owned()),
                    Expr::String("stock-map".to_owned()),
                    "warrant-1",
                ),
                &[AUTO_CONTROL_EXEC],
            ),
        )
        .unwrap();

    assert!(expr_text(&mut cx, &reply.value).contains("control/code"));
    assert_eq!(cx.effect_ledger().records().len(), 1);
    let records = fabric.gate_ledger().records().unwrap();
    assert_eq!(records[0].effect, VendorEffectClass::Irreversible);
    assert!(records[0].reversal_artifact);
    assert_eq!(records[0].warrant.as_deref(), Some("warrant-1"));
    assert!(records[0].human_approved);
}

#[test]
fn modeled_cassette_exercises_one_op_of_each_class() {
    let bridge = Arc::new(ModeledVendorBridge::new());
    let mut cx = cx_with(&[AUTO_DIAGNOSTICS_READ, AUTO_SERVICE_WRITE, AUTO_CONTROL_EXEC]);
    let fabric = cassette_vendor_fabric(fixture_manifest(), bridge.clone(), vendor_cassette());

    fabric
        .realize(
            &mut cx,
            request(
                vendor_request_expr("read/inventory", Expr::Map(Vec::new())),
                &[AUTO_DIAGNOSTICS_READ],
            ),
        )
        .unwrap();
    fabric
        .realize(
            &mut cx,
            request(
                vendor_request_expr("service/write-note", Expr::String("note".to_owned())),
                &[AUTO_SERVICE_WRITE],
            ),
        )
        .unwrap();
    fabric
        .realize(
            &mut cx,
            request(
                vendor_irreversible_request_expr(
                    "control/code",
                    Expr::String("coding".to_owned()),
                    Expr::String("stock-map".to_owned()),
                    "warrant-1",
                ),
                &[AUTO_CONTROL_EXEC],
            ),
        )
        .unwrap();

    let calls = bridge.calls().unwrap();
    assert_eq!(calls.len(), 3);
    assert_eq!(calls[0].op, "read/inventory");
    assert_eq!(calls[1].op, "service/write-note");
    assert_eq!(calls[2].op, "control/code");
}

#[test]
fn operation_classification_fails_closed_for_unknown_ops() {
    let manifest = fixture_manifest();
    assert_eq!(
        manifest_operation(&manifest, "read/inventory")
            .unwrap()
            .effect,
        VendorEffectClass::Pure
    );
    assert_eq!(
        manifest_operation(&manifest, "service/write-note")
            .unwrap()
            .effect,
        VendorEffectClass::Reversible
    );
    assert!(manifest_operation(&manifest, "undeclared/read").is_err());

    let request = crate::VendorBridgeRequest::new(
        "fixture",
        manifest_operation(&manifest, "control/code").unwrap().lane,
        "control/code",
        sim_lib_auto_core::VehicleId::new("fixture", "vehicle-alpha"),
        Expr::String("coding".to_owned()),
    )
    .with_reversal_artifact(Expr::String("stock-map".to_owned()))
    .with_warrant(VendorWarrant::new("warrant-1", "fixture"))
    .with_human_approval(true);
    assert_eq!(request.op, "control/code");
}

#[test]
fn order_ops_are_reversible_and_use_order_capability() {
    let manifest = SiteManifest::new(
        "fixture-parts",
        "vehicle-alpha",
        "fixture-brand",
        vec!["parts".to_owned(), "service".to_owned()],
        vec!["modeled".to_owned()],
        vec!["order/place".to_owned()],
    );

    let operation = manifest_operation(&manifest, "order/place").unwrap();
    assert_eq!(operation.effect, VendorEffectClass::Reversible);
    assert_eq!(operation.capability.as_str(), AUTO_ORDER);
    assert_eq!(operation.lane.name, "parts");
}

fn vendor_fabric() -> VendorSiteFabric {
    VendorSiteFabric::new(fixture_manifest(), Arc::new(ModeledVendorBridge::new()))
}

fn fixture_manifest() -> SiteManifest {
    SiteManifest::new(
        "fixture-vendor",
        "vehicle-alpha",
        "fixture-brand",
        vec![
            "diagnostics".to_owned(),
            "service".to_owned(),
            "control".to_owned(),
        ],
        vec!["modeled".to_owned()],
        vec![
            "read/inventory".to_owned(),
            "service/write-note".to_owned(),
            "control/code".to_owned(),
        ],
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

trait RequestEdit {
    fn with_expr(self, expr: Expr) -> Self;
}

impl RequestEdit for EvalRequest {
    fn with_expr(mut self, expr: Expr) -> Self {
        self.expr = expr;
        self
    }
}

trait WithoutHumanGate {
    fn without_human_gate(self) -> Expr;
}

impl WithoutHumanGate for Expr {
    fn without_human_gate(self) -> Expr {
        let Expr::Map(fields) = self else {
            return self;
        };
        Expr::Map(
            fields
                .into_iter()
                .map(|(key, value)| {
                    if key == Expr::Symbol(Symbol::new("human-approved")) {
                        (key, Expr::Bool(false))
                    } else {
                        (key, value)
                    }
                })
                .collect(),
        )
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
