use sim_kernel::{
    CapabilityName, Consistency, Cx, Error, EvalFabric, EvalMode, EvalRequest, Expr, Lib, Value,
    testing::bare_cx as cx,
};

use crate::{
    NET_HTTP_CAPABILITY, VehicleIdentityFabric, VehicleLookupRequest, VehicleSource,
    auto_vehicle_contracts_symbol, auto_vehicle_site_symbol, install_auto_vehicle_lib,
    vehicle_bridge_contracts, vehicle_by_plate, vehicle_by_plate_expr, vehicle_by_vin,
    vehicle_by_vin_expr, vehicle_lookup_shape_symbol,
};

#[test]
fn modeled_lookup_resolves_plate_and_vin_to_vehicle_id() {
    let mut cx = cx();

    let by_plate = vehicle_by_plate(&mut cx, VehicleSource::Modeled, "sim 123", "se").unwrap();
    let by_vin =
        vehicle_by_vin(&mut cx, VehicleSource::Modeled, "sim-vin-alpha-001", "SE").unwrap();

    assert_eq!(by_plate.namespace, "modeled-se");
    assert_eq!(by_plate.key, "vehicle-alpha");
    assert_eq!(by_vin, by_plate);
}

#[test]
fn request_parser_normalizes_synthetic_identifiers() {
    let plate = VehicleLookupRequest::parse(&vehicle_by_plate_expr(
        VehicleSource::Modeled,
        "Sim 45 X",
        "se",
    ))
    .unwrap();
    let vin = VehicleLookupRequest::parse(&vehicle_by_vin_expr(
        VehicleSource::Modeled,
        "sim vin beta 002",
        "SE",
    ))
    .unwrap();

    assert_eq!(plate.key, "SIM45X");
    assert_eq!(plate.market, "SE");
    assert_eq!(vin.key, "SIMVINBETA002");
}

#[test]
fn live_contracts_require_net_http_and_fail_closed() {
    let mut denied_cx = cx();
    let denied = vehicle_by_plate(&mut denied_cx, VehicleSource::HaynesPro, "sim 123", "SE");
    assert!(matches!(
        denied,
        Err(Error::CapabilityDenied { capability }) if capability.as_str() == NET_HTTP_CAPABILITY
    ));

    let mut granted_cx = cx();
    granted_cx.grant_named(NET_HTTP_CAPABILITY);
    let unconfigured = vehicle_by_vin(
        &mut granted_cx,
        VehicleSource::BiluppgifterSe,
        "sim vin alpha 001",
        "SE",
    );
    assert!(matches!(unconfigured, Err(Error::Eval(message)) if message.contains("host-owned")));
}

#[test]
fn bridge_contract_catalog_names_only_public_boundaries() {
    let contracts = vehicle_bridge_contracts();
    assert_eq!(contracts.len(), 2);
    assert!(contracts.iter().all(|contract| {
        contract.required_capability == NET_HTTP_CAPABILITY
            && contract.operations == ["vehicle/by-plate", "vehicle/by-vin"]
            && !contract.host_boundary.contains("://")
    }));
}

#[test]
fn library_exports_site_contracts_and_shape_descriptor() {
    let lib = crate::AutoVehicleLib;
    assert!(lib.manifest().exports.iter().any(|export| {
        matches!(export, sim_kernel::Export::Site { symbol, .. } if symbol == &auto_vehicle_site_symbol())
    }));
    assert!(lib.manifest().exports.iter().any(|export| {
        matches!(export, sim_kernel::Export::Value { symbol } if symbol == &auto_vehicle_contracts_symbol())
    }));
    assert!(lib.manifest().exports.iter().any(|export| {
        matches!(export, sim_kernel::Export::Value { symbol } if symbol == &vehicle_lookup_shape_symbol())
    }));

    let mut cx = cx();
    install_auto_vehicle_lib(&mut cx).unwrap();
    assert!(
        cx.registry()
            .site_by_symbol(&auto_vehicle_site_symbol())
            .is_some()
    );
    assert!(
        cx.registry()
            .value_by_symbol(&auto_vehicle_contracts_symbol())
            .is_some()
    );
    assert!(
        cx.registry()
            .value_by_symbol(&vehicle_lookup_shape_symbol())
            .is_some()
    );
}

#[test]
fn fabric_realizes_modeled_lookup() {
    let mut cx = cx();
    let fabric = VehicleIdentityFabric::fixture();

    let reply = fabric
        .realize(
            &mut cx,
            request(vehicle_by_plate_expr(
                VehicleSource::Modeled,
                "sim 123",
                "SE",
            )),
        )
        .unwrap();

    let text = expr_text(&mut cx, &reply.value);
    assert!(text.contains("vehicle-alpha"));
    assert!(text.contains("SIM123"));
}

#[test]
fn fabric_denies_live_lookup_without_net_http() {
    let mut cx = cx();
    let fabric = VehicleIdentityFabric::fixture();

    let denied = fabric.realize(
        &mut cx,
        request(vehicle_by_vin_expr(
            VehicleSource::BiluppgifterSe,
            "sim vin alpha 001",
            "SE",
        )),
    );

    assert!(matches!(
        denied,
        Err(Error::CapabilityDenied { capability }) if capability.as_str() == NET_HTTP_CAPABILITY
    ));
}

fn request(expr: Expr) -> EvalRequest {
    EvalRequest {
        expr,
        result_shape: None,
        required_capabilities: vec![CapabilityName::new("auto/vehicle/read")],
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
