//! Vehicle identity lookup and automotive data-source bridge contracts.
//!
//! The crate resolves modeled plate and VIN labels into the shared
//! [`VehicleId`] citizen, exports a loadable identity site, and advertises
//! host-owned HaynesPro and biluppgifter.se bridge contracts. Live paths require
//! `net/http` and fail closed unless the host installs its own HTTP bridge.
//!
//! [`VehicleId`]: sim_lib_auto_core::VehicleId

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod bridge;
mod contract;
mod fabric;
mod model;
mod request;
mod runtime;

#[cfg(test)]
mod tests;

pub use bridge::{
    LiveVehicleBridge, ModeledVehicleBridge, VehicleLookupBridge, VehicleLookupRouter,
    vehicle_by_plate, vehicle_by_vin, vehicle_record_by_plate, vehicle_record_by_vin,
};
pub use contract::{NET_HTTP_CAPABILITY, VehicleBridgeContract, vehicle_bridge_contracts};
pub use fabric::VehicleIdentityFabric;
pub use model::{VehicleRecord, VehicleSource, normalize_plate, normalize_vin};
pub use request::{
    VehicleLookupKind, VehicleLookupRequest, vehicle_by_plate_expr, vehicle_by_vin_expr,
};
pub use runtime::{
    AutoVehicleLib, auto_vehicle_contracts_symbol, auto_vehicle_site_symbol,
    install_auto_vehicle_lib, vehicle_lookup_shape_symbol,
};
