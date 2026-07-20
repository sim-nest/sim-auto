//! Manifest-driven automotive vendor site engine.
//!
//! The crate turns [`SiteManifest`] values into loadable kernel site exports.
//! Every site dispatches through one [`VendorBridge`] trait and the
//! [`warranted_effect`] gate, keeping concrete vendor behavior outside the
//! kernel and outside this public crate.
//!
//! [`SiteManifest`]: sim_lib_auto_core::SiteManifest

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod bridge;
mod effect;
mod engine;
mod gate;
mod request;
mod runtime;
mod sites;

#[cfg(test)]
mod flash_tests;
#[cfg(test)]
mod supplier_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

pub use bridge::{ModeledVendorBridge, ModeledVendorCassette, VendorBridge};
pub use effect::{ManifestOperation, VendorEffectClass, manifest_operation};
pub use engine::{VendorReplayFabric, VendorSiteFabric, cassette_vendor_fabric, vendor_cassette};
pub use gate::{VendorGateLedger, VendorGateRecord, VendorWarrant, warranted_effect};
pub use request::{VendorBridgeRequest, vendor_irreversible_request_expr, vendor_request_expr};
pub use runtime::{AutoVendorLib, auto_vendor_site_symbol, install_auto_vendor_lib};
pub use sites::{
    autotuner_manifest, biluppgifter_se_manifest, esitronic_manifest, flash_site_cassettes,
    flash_site_manifests, haynespro_manifest, ista_manifest, mekonomen_pro_manifest, odis_manifest,
    oem_site_cassettes, oem_site_manifests, supplier_site_cassettes, supplier_site_manifests,
    vida_manifest, xentry_manifest,
};
