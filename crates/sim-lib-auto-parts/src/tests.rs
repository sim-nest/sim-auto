use sim_citizen::check_fixture;
use sim_kernel::{
    CapabilityName, Consistency, Cx, Error, EvalFabric, EvalMode, EvalRequest, Expr, Lib, Symbol,
    Value,
    table::{Dir, Table},
    testing::bare_cx as cx,
};
use sim_lib_auto_core::AUTO_ORDER;
use sim_lib_auto_vendor::{VendorEffectClass, VendorGateLedger, manifest_operation};

use crate::{
    AutoPartsFabric, AutoPartsLib, ModeledOrderLedger, NET_HTTP_CAPABILITY, ORDER_OPERATION,
    OrderStatus, PartLine, PartsCatalog, Supplier, auto_order_expr, auto_parts_citizen_symbols,
    auto_parts_dir_symbol, auto_parts_shape_symbol, auto_parts_site_symbol,
    auto_parts_suppliers_symbol, catalog_part, mekonomen_order_manifest, modeled_epc_dir,
    parts_catalog_get_expr, place_order_with_gate,
};

#[test]
fn parts_citizens_round_trip() {
    let mut cx = cx();
    crate::install_auto_parts_lib(&mut cx).unwrap();
    check_fixture(
        &mut cx,
        PartLine::new("SIM-COIL-1", Some("A0001500180"), "modeled coil", 1),
    )
    .unwrap();
    check_fixture(
        &mut cx,
        OrderStatus::accepted("SIM-ORDER-1", Supplier::MekonomenProModeled, 1),
    )
    .unwrap();
}

#[test]
fn epc_catalog_behaves_as_dir_table() {
    let mut cx = cx();
    let dir = modeled_epc_dir();

    assert!(dir.keys(&mut cx).unwrap().contains(&Symbol::new("engine")));
    assert!(dir.is_dir(&mut cx, Symbol::new("engine")).unwrap());

    let engine = dir
        .opendir(&mut cx, Symbol::new("engine"))
        .unwrap()
        .expect("engine directory");
    let ignition = engine
        .object()
        .as_dir()
        .unwrap()
        .opendir(&mut cx, Symbol::new("ignition"))
        .unwrap()
        .expect("ignition directory");
    let table = ignition.object().as_table_impl().unwrap();
    let coil = table.get(&mut cx, Symbol::new("coil-1")).unwrap();
    assert!(expr_text(&mut cx, &coil).contains("SIM-COIL-1"));
    assert_eq!(table.entries(&mut cx).unwrap().len(), 2);

    let by_path = catalog_part(
        &mut cx,
        PartsCatalog::EpcModeled,
        &[
            "engine".to_owned(),
            "ignition".to_owned(),
            "coil-1".to_owned(),
        ],
    )
    .unwrap();
    assert_eq!(expr_text(&mut cx, &coil), expr_text(&mut cx, &by_path));
}

#[test]
fn aftermarket_catalog_has_modeled_supplier_part() {
    let mut cx = cx();
    let part = catalog_part(
        &mut cx,
        PartsCatalog::AftermarketModeled,
        &[
            "engine".to_owned(),
            "ignition".to_owned(),
            "coil-1".to_owned(),
        ],
    )
    .unwrap();

    assert!(expr_text(&mut cx, &part).contains("MEK-SIM-COIL-1"));
}

#[test]
fn modeled_order_uses_reversible_gate_and_fixture_ledger() {
    let mut cx = cx_with(&[AUTO_ORDER]);
    let order_ledger = ModeledOrderLedger::new();
    let gate_ledger = VendorGateLedger::new();
    let line = PartLine::new("MEK-SIM-COIL-1", Some("A0001500180"), "modeled coil", 1);

    let status = place_order_with_gate(
        &mut cx,
        Supplier::MekonomenProModeled,
        vec![line],
        &order_ledger,
        &gate_ledger,
    )
    .unwrap();

    assert!(status.accepted);
    assert_eq!(status.supplier, "mekonomen-pro-modeled");
    assert_eq!(order_ledger.records().unwrap(), vec![status]);
    let gate_records = gate_ledger.records().unwrap();
    assert_eq!(gate_records.len(), 1);
    assert_eq!(gate_records[0].effect, VendorEffectClass::Reversible);
    assert_eq!(gate_records[0].capability.as_str(), AUTO_ORDER);
    assert_eq!(cx.effect_ledger().records().len(), 1);
}

#[test]
fn live_order_denies_without_network_grant() {
    let mut cx = cx_with(&[AUTO_ORDER]);
    let order_ledger = ModeledOrderLedger::new();
    let gate_ledger = VendorGateLedger::new();

    let denied = place_order_with_gate(
        &mut cx,
        Supplier::MekonomenProLive,
        vec![PartLine::default()],
        &order_ledger,
        &gate_ledger,
    );

    assert!(
        matches!(denied, Err(Error::CapabilityDenied { capability }) if capability.as_str() == NET_HTTP_CAPABILITY)
    );
    assert!(order_ledger.records().unwrap().is_empty());
    assert!(gate_ledger.records().unwrap().is_empty());
}

#[test]
fn order_operation_is_declared_reversible() {
    let operation = manifest_operation(&mekonomen_order_manifest(), ORDER_OPERATION).unwrap();
    assert_eq!(operation.effect, VendorEffectClass::Reversible);
    assert_eq!(operation.capability.as_str(), AUTO_ORDER);
    assert_eq!(operation.lane.name, "parts");
}

#[test]
fn parts_lib_exports_site_dir_shape_and_citizens() {
    let manifest = AutoPartsLib.manifest();
    let exports = manifest
        .exports
        .iter()
        .map(export_symbol)
        .collect::<Vec<_>>();
    assert!(exports.contains(&"auto/parts-site".to_owned()));
    assert!(exports.contains(&"auto/parts-dir".to_owned()));
    assert!(exports.contains(&"auto/parts-shape".to_owned()));
    for symbol in auto_parts_citizen_symbols() {
        assert!(exports.contains(&symbol.to_string()));
    }

    let mut cx = cx();
    crate::install_auto_parts_lib(&mut cx).unwrap();
    assert!(
        cx.registry()
            .site_by_symbol(&auto_parts_site_symbol())
            .is_some()
    );
    assert!(
        cx.registry()
            .value_by_symbol(&auto_parts_dir_symbol())
            .is_some()
    );
    assert!(
        cx.registry()
            .value_by_symbol(&auto_parts_suppliers_symbol())
            .is_some()
    );
    assert!(
        cx.registry()
            .value_by_symbol(&auto_parts_shape_symbol())
            .is_some()
    );
}

#[test]
fn parts_fabric_serves_catalog_and_order_requests() {
    let fabric = AutoPartsFabric::fixture();
    let mut cx = cx_with(&[AUTO_ORDER]);

    let part = fabric
        .realize(
            &mut cx,
            request(parts_catalog_get_expr(&["engine", "ignition", "coil-1"])),
        )
        .unwrap();
    assert!(expr_text(&mut cx, &part.value).contains("SIM-COIL-1"));

    let order = fabric
        .realize(
            &mut cx,
            request(auto_order_expr(
                Supplier::MekonomenProModeled,
                &[PartLine::default()],
            )),
        )
        .unwrap();
    assert!(expr_text(&mut cx, &order.value).contains("SIM-ORDER"));
    assert_eq!(fabric.order_ledger().records().unwrap().len(), 1);
    assert_eq!(fabric.gate_ledger().records().unwrap().len(), 1);
}

fn cx_with(capabilities: &[&'static str]) -> Cx {
    let mut cx = cx();
    for capability in capabilities {
        cx.grant_named(capability);
    }
    cx
}

fn request(expr: Expr) -> EvalRequest {
    EvalRequest {
        expr,
        result_shape: None,
        required_capabilities: vec![CapabilityName::new(AUTO_ORDER)],
        deadline: None,
        consistency: Consistency::LocalFirst,
        mode: EvalMode::Eval,
        answer_limit: None,
        stream_buffer: None,
        stream: false,
        trace: false,
    }
}

fn expr_text(cx: &mut Cx, value: &Value) -> String {
    format!("{:?}", value.object().as_expr(cx).unwrap())
}

fn export_symbol(export: &sim_kernel::Export) -> String {
    match export {
        sim_kernel::Export::Class { symbol, .. }
        | sim_kernel::Export::Function { symbol, .. }
        | sim_kernel::Export::Macro { symbol, .. }
        | sim_kernel::Export::Shape { symbol, .. }
        | sim_kernel::Export::Codec { symbol, .. }
        | sim_kernel::Export::NumberDomain { symbol, .. }
        | sim_kernel::Export::Value { symbol }
        | sim_kernel::Export::Site { symbol, .. }
        | sim_kernel::Export::Open { symbol, .. } => symbol.to_string(),
    }
}
