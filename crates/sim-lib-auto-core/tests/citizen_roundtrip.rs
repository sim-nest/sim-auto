use std::sync::Arc;

use sim_citizen::check_fixture;
use sim_kernel::{Cx, DefaultFactory, Expr, Lib, NoopEvalPolicy, Symbol};

use sim_lib_auto_core::{
    AUTO_DIAGNOSTICS_READ, AutoCoreLib, AutoLane, BrandCaps, Dtc, EffectClass, OpCap, SiteManifest,
    TransportSpec, VehicleId, auto_capability_names, auto_caps_symbol, auto_citizen_registry,
    auto_citizen_symbols, auto_lanes_symbol, control_effect, diagnostic_effect, diagnostic_lane,
    install_auto_core_lib, manifest_shape_symbol, telemetry_lane, vehicle_read_construct,
};

const EXPECTED_CITIZENS: &[&str] = &[
    "auto/VehicleId",
    "auto/Dtc",
    "auto/BrandCaps",
    "auto/AutoLane",
    "auto/EffectClass",
    "auto/OpCap",
    "auto/TransportSpec",
    "auto/SiteManifest",
];

#[test]
fn core_citizens_round_trip() {
    let mut cx = cx();
    install_auto_core_lib(&mut cx).unwrap();
    assert_expected_citizens_registered();

    check_fixture(&mut cx, VehicleId::new("fixture", "vehicle-alpha")).unwrap();
    check_fixture(&mut cx, Dtc::new("body", "B0000", "modeled diagnostic")).unwrap();
    check_fixture(
        &mut cx,
        BrandCaps::new("fixture-brand", auto_capability_names()),
    )
    .unwrap();
    check_fixture(&mut cx, diagnostic_lane()).unwrap();
    check_fixture(&mut cx, AutoLane::new("service")).unwrap();
    check_fixture(&mut cx, diagnostic_effect()).unwrap();
    check_fixture(&mut cx, EffectClass::new("service-write")).unwrap();
    check_fixture(&mut cx, control_effect()).unwrap();
    check_fixture(
        &mut cx,
        OpCap::new(
            "diagnostics/read-dtc",
            auto_capability_names().remove(0),
            "diagnostic-read",
        ),
    )
    .unwrap();
    check_fixture(
        &mut cx,
        TransportSpec::new(
            "fixture-transport",
            "modeled-bus",
            telemetry_lane().name,
            auto_capability_names().remove(5),
            auto_capability_names().remove(3),
        ),
    )
    .unwrap();
    check_fixture(
        &mut cx,
        SiteManifest::new(
            "fixture-site",
            "vehicle-alpha",
            "fixture-brand",
            vec!["diagnostics".to_owned(), "telemetry".to_owned()],
            vec!["fixture-transport".to_owned()],
            vec!["diagnostics/read-dtc".to_owned()],
        ),
    )
    .unwrap();
}

#[test]
fn core_lib_exports_values_and_classes() {
    let mut cx = cx();
    install_auto_core_lib(&mut cx).unwrap();

    assert!(cx.registry().value_by_symbol(&auto_caps_symbol()).is_some());
    assert!(
        cx.registry()
            .value_by_symbol(&auto_lanes_symbol())
            .is_some()
    );
    assert!(
        cx.registry()
            .value_by_symbol(&manifest_shape_symbol())
            .is_some()
    );

    for symbol in auto_citizen_symbols() {
        assert!(
            cx.registry().class_by_symbol(&symbol).is_some(),
            "missing class {symbol}"
        );
    }

    let caps = cx
        .registry()
        .value_by_symbol(&auto_caps_symbol())
        .cloned()
        .unwrap();
    let expr = caps.object().as_expr(&mut cx).unwrap();
    let Expr::List(items) = expr else {
        panic!("expected capability list expression");
    };
    assert!(items.contains(&Expr::String(AUTO_DIAGNOSTICS_READ.to_owned())));
}

#[test]
fn manifest_declares_core_exports() {
    let manifest = AutoCoreLib.manifest();
    let declared = manifest
        .exports
        .iter()
        .map(export_symbol)
        .collect::<Vec<_>>();
    assert!(declared.contains(&"auto/caps".to_owned()));
    assert!(declared.contains(&"auto/lanes".to_owned()));
    assert!(declared.contains(&"auto/manifest-shape".to_owned()));
}

#[test]
fn vehicle_helper_emits_citizen_read_construct() {
    let expr = vehicle_read_construct(&VehicleId::new("fixture", "vehicle-alpha"));
    let Expr::Extension { tag, payload } = expr else {
        panic!("expected extension expression");
    };
    assert_eq!(tag, Symbol::qualified("citizen", "read-construct"));

    let Expr::Vector(items) = *payload else {
        panic!("expected vector payload");
    };
    assert_eq!(
        items.first(),
        Some(&Expr::Symbol(Symbol::qualified("auto", "VehicleId")))
    );
}

fn cx() -> Cx {
    Cx::new(Arc::new(NoopEvalPolicy), Arc::new(DefaultFactory))
}

fn assert_expected_citizens_registered() {
    let symbols = auto_citizen_registry()
        .into_iter()
        .map(|info| info.symbol)
        .collect::<Vec<_>>();
    for expected in EXPECTED_CITIZENS {
        assert!(
            symbols.contains(expected),
            "missing citizen inventory row {expected}"
        );
    }
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
