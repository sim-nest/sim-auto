use std::sync::Arc;

use sim_kernel::{EvalFabric, Expr, Lib, testing::bare_cx as cx};
use sim_lib_auto_core::{AUTO_DIAGNOSTICS_READ, AUTO_ORDER};

use crate::test_support::{cx_with, export_symbol, expr_text, request};
use crate::{
    AutoVendorLib, ModeledVendorBridge, VendorEffectClass, VendorSiteFabric,
    auto_vendor_site_symbol, biluppgifter_se_manifest, haynespro_manifest, install_auto_vendor_lib,
    manifest_operation, mekonomen_pro_manifest, supplier_site_cassettes, supplier_site_manifests,
    vendor_request_expr,
};

#[test]
fn supplier_manifests_install_through_single_vendor_library() {
    let manifests = supplier_site_manifests();
    assert_eq!(
        manifests
            .iter()
            .map(|manifest| manifest.site.as_str())
            .collect::<Vec<_>>(),
        vec!["haynespro", "biluppgifter-se", "mekonomen-pro"]
    );

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
            "missing installed supplier site {}",
            manifest.site
        );
    }
}

#[test]
fn supplier_manifest_ops_are_explicit_and_public_config_is_modeled_only() {
    let haynespro = haynespro_manifest();
    assert_eq!(haynespro.lanes, vec!["read", "info"]);
    assert_eq!(
        manifest_operation(&haynespro, "read/identity")
            .unwrap()
            .effect,
        VendorEffectClass::Pure
    );
    assert_eq!(
        manifest_operation(&haynespro, "info/procedure")
            .unwrap()
            .capability
            .as_str(),
        AUTO_DIAGNOSTICS_READ
    );

    let biluppgifter = biluppgifter_se_manifest();
    let plate_lookup = manifest_operation(&biluppgifter, "read/plate-lookup").unwrap();
    assert_eq!(plate_lookup.effect, VendorEffectClass::Pure);
    assert_eq!(plate_lookup.lane.name, "read");

    let mekonomen = mekonomen_pro_manifest();
    let catalog = manifest_operation(&mekonomen, "parts/catalog-lookup").unwrap();
    assert_eq!(catalog.effect, VendorEffectClass::Pure);
    assert_eq!(catalog.lane.name, "parts");
    let status = manifest_operation(&mekonomen, "order/status").unwrap();
    assert_eq!(status.effect, VendorEffectClass::Pure);
    assert_eq!(status.capability.as_str(), AUTO_ORDER);
    let order = manifest_operation(&mekonomen, "order/place").unwrap();
    assert_eq!(order.effect, VendorEffectClass::Reversible);
    assert_eq!(order.capability.as_str(), AUTO_ORDER);

    for manifest in supplier_site_manifests() {
        assert!(
            manifest
                .transports
                .iter()
                .all(|transport| transport.ends_with("-modeled") || transport == "cassette")
        );
        let public_descriptor = format!("{manifest:?}");
        assert!(!public_descriptor.contains("://"));
        assert!(!public_descriptor.to_ascii_lowercase().contains("account"));
    }
}

#[test]
fn modeled_supplier_cassettes_replay_and_order_uses_reversible_gate() {
    let bridge = Arc::new(ModeledVendorBridge::with_cassettes(
        supplier_site_cassettes(),
    ));
    let mut cx = cx_with(&[AUTO_ORDER]);
    let fabric = VendorSiteFabric::new(mekonomen_pro_manifest(), bridge);

    let status = fabric
        .realize(
            &mut cx,
            request(
                vendor_request_expr("order/status", Expr::Map(Vec::new())),
                &[AUTO_ORDER],
            ),
        )
        .unwrap();
    assert!(expr_text(&mut cx, &status.value).contains("modeled order status"));
    assert!(fabric.gate_ledger().records().unwrap().is_empty());
    assert_eq!(cx.effect_ledger().records().len(), 0);

    let order = fabric
        .realize(
            &mut cx,
            request(
                vendor_request_expr("order/place", Expr::Map(Vec::new())),
                &[AUTO_ORDER],
            ),
        )
        .unwrap();
    assert!(expr_text(&mut cx, &order.value).contains("modeled order accepted"));
    let gate_records = fabric.gate_ledger().records().unwrap();
    assert_eq!(gate_records.len(), 1);
    assert_eq!(gate_records[0].site, "mekonomen-pro");
    assert_eq!(gate_records[0].operation, "order/place");
    assert_eq!(gate_records[0].effect, VendorEffectClass::Reversible);
    assert_eq!(gate_records[0].capability.as_str(), AUTO_ORDER);
    assert_eq!(cx.effect_ledger().records().len(), 1);
}
