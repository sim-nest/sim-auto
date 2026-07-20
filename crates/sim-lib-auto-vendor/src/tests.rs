use std::sync::Arc;

use sim_kernel::{CapabilityName, Error, EvalFabric, Expr, Lib, testing::bare_cx as cx};
use sim_lib_auto_core::{
    AUTO_CONTROL_EXEC, AUTO_DIAGNOSTICS_READ, AUTO_ORDER, AUTO_SERVICE_WRITE, SiteManifest,
    select_brand,
};

use crate::test_support::{
    RequestEdit, WithoutHumanGate, brand_need, cx_with, export_symbol, expr_text, request,
    reversal_artifact,
};
use crate::{
    AutoVendorLib, ModeledVendorBridge, VendorEffectClass, VendorSiteFabric, VendorWarrant,
    auto_vendor_site_symbol, cassette_vendor_fabric, install_auto_vendor_lib, manifest_operation,
    oem_site_cassettes, oem_site_manifests, vendor_cassette, vendor_irreversible_request_expr,
    vendor_request_expr, xentry_manifest,
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
fn oem_manifests_install_through_single_vendor_library() {
    let manifests = oem_site_manifests();
    let lib = AutoVendorLib::modeled(manifests.clone());
    let exports = lib
        .manifest()
        .exports
        .iter()
        .map(export_symbol)
        .collect::<Vec<_>>();

    for manifest in &manifests {
        assert!(
            exports.contains(&auto_vendor_site_symbol(manifest).to_string()),
            "missing site export for {}",
            manifest.site
        );
    }

    let mut cx = cx();
    install_auto_vendor_lib(&mut cx, manifests.clone()).unwrap();
    for manifest in &manifests {
        assert!(
            cx.registry()
                .site_by_symbol(&auto_vendor_site_symbol(manifest))
                .is_some(),
            "missing installed site {}",
            manifest.site
        );
    }
}

#[test]
fn brand_select_ranks_oem_sites_before_bosch_fallback() {
    let manifests = oem_site_manifests();

    let volvo = select_brand(&manifests, &brand_need("volvo", &["read", "info"])).unwrap();
    assert_eq!(volvo.manifest.site, "vida");
    assert!(volvo.exact_make);

    let saab = select_brand(&manifests, &brand_need("saab", &["read"])).unwrap();
    assert_eq!(saab.manifest.site, "esitronic");
    assert!(!saab.exact_make);

    let no_match = select_brand(&manifests, &brand_need("saab", &["parts"]));
    assert!(matches!(no_match, Err(Error::Eval(message)) if message.contains("missing parts")));

    let denied = select_brand(
        &manifests,
        &brand_need("bmw", &["service"]).requiring(CapabilityName::new(AUTO_CONTROL_EXEC)),
    );
    assert!(matches!(denied, Err(Error::Eval(message)) if message.contains("capability ceiling")));
}

#[test]
fn modeled_oem_cassettes_replay_and_denied_caps_fail_closed() {
    let bridge = Arc::new(ModeledVendorBridge::with_cassettes(oem_site_cassettes()));
    let mut cx = cx_with(&[AUTO_DIAGNOSTICS_READ, AUTO_SERVICE_WRITE]);
    let fabric = VendorSiteFabric::new(xentry_manifest(), bridge);

    let read = fabric
        .realize(
            &mut cx,
            request(
                vendor_request_expr("read/dtc", Expr::Map(Vec::new())),
                &[AUTO_DIAGNOSTICS_READ],
            ),
        )
        .unwrap();
    assert!(expr_text(&mut cx, &read.value).contains("mercedes modeled DTC read"));

    let code = fabric
        .realize(
            &mut cx,
            request(
                vendor_request_expr("code/sca", Expr::Map(Vec::new())),
                &[AUTO_SERVICE_WRITE],
            ),
        )
        .unwrap();
    assert!(expr_text(&mut cx, &code.value).contains("mercedes modeled service coding"));

    let mut denied_cx = cx_with(&[AUTO_DIAGNOSTICS_READ]);
    let denied_bridge = Arc::new(ModeledVendorBridge::with_cassettes(oem_site_cassettes()));
    let denied_fabric = VendorSiteFabric::new(crate::ista_manifest(), denied_bridge);
    let denied = denied_fabric.realize(
        &mut denied_cx,
        request(
            vendor_request_expr("code/coding", Expr::Map(Vec::new())),
            &[AUTO_SERVICE_WRITE],
        ),
    );
    assert!(
        matches!(denied, Err(Error::CapabilityDenied { capability }) if capability.as_str() == AUTO_SERVICE_WRITE)
    );
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
                reversal_artifact("stock-map"),
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
                    reversal_artifact("stock-map"),
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
    assert_eq!(
        records[0].reversal_content_key.as_deref(),
        Some("stock-map")
    );
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
                    reversal_artifact("stock-map"),
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
    .with_reversal_artifact(reversal_artifact("stock-map"))
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
