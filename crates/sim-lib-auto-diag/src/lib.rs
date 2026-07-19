//! Modeled automotive diagnostic fabric.
//!
//! The crate exposes synthetic ECU data through the kernel [`EvalFabric`]
//! contract. Read operations require the automotive diagnostic-read capability;
//! coding, service, and actuation operations require the automotive control
//! capability after caller-side diminishment.
//!
//! [`EvalFabric`]: sim_kernel::EvalFabric

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod fabric;
mod model;
mod request;
mod runtime;

#[cfg(test)]
mod tests;

pub use fabric::{
    AutoDiagFabric, AutoDiagReplayFabric, auto_fabric, cassette_auto_fabric, diagnostic_cassette,
};
pub use model::{FreezeFrame, ModeledEcu, ModeledVehicle, PidValue};
pub use request::{DiagnosticRequest, code_expr, freeze_frame_expr, read_dtcs_expr, read_pid_expr};
pub use runtime::{AutoDiagLib, auto_diag_site_symbol, install_auto_diag_lib};

/// Cookbook recipes for this diagnostic fabric, embedded at build time.
pub static RECIPES: sim_cookbook::EmbeddedDir =
    include!(concat!(env!("OUT_DIR"), "/cookbook_recipes.rs"));
