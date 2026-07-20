use sim_kernel::{
    CapabilityName, Consistency, Cx, EvalFabric, EvalMode, EvalRequest, Expr, Lib, Value,
    testing::bare_cx as cx,
};

use crate::{
    AutoInfoFabric, AutoInfoLib, InfoSource, RepairQuery, auto_info_expr, auto_info_shape_symbol,
    auto_info_site_symbol, auto_info_sources_symbol, fixture_vehicle, install_auto_info_lib,
    parse_repair_query, rank_repair_docs, repair_catalog, repair_document, repair_scene,
};

#[test]
fn catalog_covers_modeled_sources_with_public_fixture_text() {
    let catalog = repair_catalog();
    let sources = catalog
        .iter()
        .map(|procedure| procedure.source)
        .collect::<Vec<_>>();
    for source in InfoSource::all() {
        assert!(sources.contains(source), "missing source {source:?}");
    }
    for procedure in &catalog {
        let text = format!(
            "{} {} {} {:?} {:?}",
            procedure.title,
            procedure.summary,
            procedure.steps.join(" "),
            procedure.safety_notes,
            procedure.tags
        )
        .to_ascii_lowercase();
        for forbidden in [
            "screenshot",
            "wiring diagram",
            "dealer cookie",
            "vendor token",
            "customer name",
        ] {
            assert!(
                !text.contains(forbidden),
                "procedure {} contains forbidden fixture text {forbidden}",
                procedure.id
            );
        }
    }
}

#[test]
fn rank_uses_vehicle_dtc_ecu_symptom_lane_and_source() {
    let query = RepairQuery::new(fixture_vehicle())
        .with_source(InfoSource::WisModeled)
        .with_dtc_code("P0301")
        .with_ecu("ME97")
        .with_symptom("rough idle");

    let selected = rank_repair_docs(&query, &repair_catalog()).unwrap();

    assert_eq!(selected.id, "wis-misfire-modeled");
}

#[test]
fn every_modeled_source_projects_to_a_valid_document_scene() {
    for source in InfoSource::all() {
        let query = RepairQuery::new(fixture_vehicle()).with_source(*source);
        let doc = repair_document(&query).unwrap();
        let scene = repair_scene(&query).unwrap();

        let markdown = sim_lib_view_doc::export_markdown(&doc);
        assert!(markdown.contains(source.display_name()));
        sim_lib_scene::validate_scene(&scene).unwrap();
    }
}

#[test]
fn request_parser_accepts_map_and_list_forms() {
    let vehicle = fixture_vehicle();
    let map = auto_info_expr(
        &vehicle,
        InfoSource::EsiTronicModeled,
        Some("P0301"),
        Some("ME97"),
        Some("compression"),
    );
    let parsed = parse_repair_query(&map).unwrap();
    assert_eq!(parsed.source, Some(InfoSource::EsiTronicModeled));
    assert_eq!(parsed.dtc.as_deref(), Some("P0301"));

    let list = Expr::List(vec![
        Expr::Symbol(sim_kernel::Symbol::qualified("auto", "info")),
        Expr::Symbol(sim_kernel::Symbol::new(":vehicle")),
        Expr::String("vehicle-alpha".to_owned()),
        Expr::Symbol(sim_kernel::Symbol::new(":source")),
        Expr::String("vida-modeled".to_owned()),
        Expr::Symbol(sim_kernel::Symbol::new(":dtc")),
        Expr::String("B1000".to_owned()),
    ]);
    let parsed = parse_repair_query(&list).unwrap();
    assert_eq!(parsed.source, Some(InfoSource::VidaModeled));
    assert_eq!(parsed.dtc.as_deref(), Some("B1000"));
}

#[test]
fn library_exports_info_site_sources_and_shape_descriptor() {
    let lib = AutoInfoLib;
    assert!(lib.manifest().exports.iter().any(|export| {
        matches!(export, sim_kernel::Export::Site { symbol, .. } if symbol == &auto_info_site_symbol())
    }));
    assert!(lib.manifest().exports.iter().any(|export| {
        matches!(export, sim_kernel::Export::Value { symbol } if symbol == &auto_info_sources_symbol())
    }));
    assert!(lib.manifest().exports.iter().any(|export| {
        matches!(export, sim_kernel::Export::Value { symbol } if symbol == &auto_info_shape_symbol())
    }));

    let mut cx = cx();
    install_auto_info_lib(&mut cx).unwrap();
    assert!(
        cx.registry()
            .site_by_symbol(&auto_info_site_symbol())
            .is_some()
    );
    assert!(
        cx.registry()
            .value_by_symbol(&auto_info_sources_symbol())
            .is_some()
    );
    assert!(
        cx.registry()
            .value_by_symbol(&auto_info_shape_symbol())
            .is_some()
    );
}

#[test]
fn fabric_realizes_modeled_info_request_as_scene() {
    let mut cx = cx();
    let fabric = AutoInfoFabric::fixture();
    let reply = fabric
        .realize(
            &mut cx,
            request(auto_info_expr(
                &fixture_vehicle(),
                InfoSource::WisModeled,
                Some("P0301"),
                Some("ME97"),
                Some("rough idle"),
            )),
        )
        .unwrap();

    let text = expr_text(&mut cx, &reply.value);
    assert!(text.contains("Modeled misfire diagnosis"));
    assert!(text.contains("article"));
}

fn request(expr: Expr) -> EvalRequest {
    EvalRequest {
        expr,
        result_shape: None,
        required_capabilities: vec![CapabilityName::new("auto/diagnostics/read")],
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
