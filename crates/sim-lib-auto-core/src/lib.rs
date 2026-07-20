//! Automotive domain citizens, capability names, and loadable runtime exports.
//!
//! The crate keeps automotive concepts as open runtime data: identities,
//! lanes, effect classes, operation capabilities, transports, and site
//! manifests are citizens that round-trip through the shared read-construct
//! path. The kernel carries only generic library, capability, and citizen
//! contracts.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod capability;
pub mod manifest;
mod read_construct;
mod runtime;
pub mod session;

pub use capability::{
    AUTO_CONTROL_EXEC, AUTO_DIAGNOSTICS_READ, AUTO_MANIFEST_READ, AUTO_ORDER, AUTO_SERVICE_WRITE,
    AUTO_TELEMETRY_READ, AUTO_TRANSPORT_CONNECT, auto_capability_names, auto_capability_texts,
};
pub use manifest::{
    AutoLane, BrandCaps, Dtc, DtcStatus, EffectClass, OpCap, SiteManifest, TransportSpec,
    VehicleId, auto_lane, control_effect, diagnostic_effect, diagnostic_lane, manifest_lane,
    telemetry_lane,
};
pub use read_construct::{read_construct_expr, text_read_construct_expr, vehicle_read_construct};
pub use runtime::{
    AutoCoreLib, auto_caps_symbol, auto_citizen_registry, auto_citizen_symbols, auto_lanes_symbol,
    install_auto_core_lib, manifest_shape_symbol,
};
pub use session::{AutoSession, TransportPlacement};
