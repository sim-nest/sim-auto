//! Loadable library wiring for vehicle identity lookup.

use std::sync::Arc;

use sim_kernel::{
    AbiVersion, Cx, Export, Expr, Lib, LibManifest, LibTarget, Linker, LoadCx, Result, Symbol,
    Version,
};

use crate::{VehicleIdentityFabric, vehicle_bridge_contracts};

/// Loadable library that contributes vehicle identity lookup exports.
#[derive(Clone, Copy, Debug, Default)]
pub struct AutoVehicleLib;

impl Lib for AutoVehicleLib {
    fn manifest(&self) -> LibManifest {
        LibManifest {
            id: Symbol::qualified("auto", "vehicle"),
            version: Version(env!("CARGO_PKG_VERSION").to_owned()),
            abi: AbiVersion { major: 0, minor: 1 },
            target: LibTarget::HostRegistered,
            requires: Vec::new(),
            capabilities: Vec::new(),
            exports: vec![
                Export::Site {
                    symbol: auto_vehicle_site_symbol(),
                    runtime_id: None,
                },
                Export::Value {
                    symbol: auto_vehicle_contracts_symbol(),
                },
                Export::Value {
                    symbol: vehicle_lookup_shape_symbol(),
                },
            ],
        }
    }

    fn load(&self, cx: &mut LoadCx, linker: &mut Linker<'_>) -> Result<()> {
        let fabric = cx
            .factory()
            .opaque(Arc::new(VehicleIdentityFabric::fixture()))?;
        linker.site_value(auto_vehicle_site_symbol(), fabric)?;
        linker.value(
            auto_vehicle_contracts_symbol(),
            cx.factory().expr(Expr::List(
                vehicle_bridge_contracts()
                    .into_iter()
                    .map(|contract| contract.to_expr())
                    .collect(),
            ))?,
        )?;
        linker.value(
            vehicle_lookup_shape_symbol(),
            cx.factory().string(
                "auto/VehicleLookupRequest op source market plate|vin; source in modeled haynespro biluppgifter.se; live requires net/http"
                    .to_owned(),
            )?,
        )?;
        Ok(())
    }
}

/// Installs the vehicle identity library into a context once.
pub fn install_auto_vehicle_lib(cx: &mut Cx) -> Result<()> {
    if cx
        .registry()
        .lib(&Symbol::qualified("auto", "vehicle"))
        .is_some()
    {
        return Ok(());
    }
    cx.load_lib(&AutoVehicleLib).map(|_| ())
}

/// Symbol for the vehicle identity site export.
pub fn auto_vehicle_site_symbol() -> Symbol {
    Symbol::qualified("auto", "vehicle-identity")
}

/// Symbol for the live bridge contract catalog.
pub fn auto_vehicle_contracts_symbol() -> Symbol {
    Symbol::qualified("auto", "vehicle-bridge-contracts")
}

/// Symbol for the vehicle lookup shape descriptor.
pub fn vehicle_lookup_shape_symbol() -> Symbol {
    Symbol::qualified("auto", "vehicle-lookup-shape")
}
