use sim_kernel::Expr;
use sim_lib_intent::{intent_kind_of, validate_intent};
use sim_lib_scene::validate_scene;
use sim_lib_view::SurfaceCaps;

use crate::{BayState, all_modeled_intents, bay_scene, bay_surface_caps};

#[test]
fn bay_scene_covers_required_panels_and_validates() {
    let state = BayState::modeled_mercedes().expect("fixture builds");
    let scene = bay_scene(&state).expect("scene builds");

    validate_scene(&scene).expect("scene validates");
    let rendered = format!("{scene:?}");
    for required in [
        "vehicle-header",
        "site-selection",
        "dtc-list",
        "repair-document",
        "parts-cart",
        "coding-status",
        "flash-gate-status",
        "ledger-timeline",
    ] {
        assert!(rendered.contains(required), "missing panel {required}");
    }
}

#[test]
fn bay_intents_validate_and_cover_each_required_action() {
    let state = BayState::modeled_mercedes().expect("fixture builds");
    let intents = all_modeled_intents(&state, 10).expect("intents build");

    assert_eq!(intents.len(), 9);
    assert!(matches!(
        intent_kind_of(&intents[0]),
        Some(kind) if kind.as_qualified_str() == "intent/select"
    ));
    for intent in &intents {
        validate_intent(intent).expect("intent validates");
    }
    let rendered = intents
        .iter()
        .map(|intent| format!("{intent:?}"))
        .collect::<Vec<_>>()
        .join("\n");
    for op in [
        "run-scan",
        "open-procedure",
        "add-part",
        "place-order",
        "request-coding",
        "request-backup",
        "request-flash",
        "restore-stock-map",
    ] {
        assert!(rendered.contains(op), "missing intent op {op}");
    }
}

#[test]
fn bay_surface_caps_round_trip() {
    let caps = bay_surface_caps();
    let expr = caps.to_expr();

    let back = SurfaceCaps::from_expr(&expr).expect("surface caps parse");
    assert_eq!(caps, back);
    assert!(caps.input_flag("keyboard"));
}

#[test]
fn modeled_bay_primary_dtc_is_confirmed_me97() {
    let state = BayState::modeled_mercedes().expect("fixture builds");
    let primary = state.primary_dtc().expect("primary DTC");

    assert_eq!(primary.dtc.code, "P0301");
    assert_eq!(primary.ecu, "ME97");
    assert!(primary.dtc.status.confirmed);
}

fn _assert_expr(_: &Expr) {}
