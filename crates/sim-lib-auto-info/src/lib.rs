//! Modeled automotive repair-information documents.
//!
//! The crate represents workshop information from WIS, ISTA, VIDA, ESI\[tronic\],
//! HaynesPro, and shop-authored sources as synthetic SIM documents. It ranks
//! candidates by vehicle identity, DTC, ECU, symptom, and lane, then projects
//! the selected document through the existing document view surface.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod document;
mod fabric;
mod fixtures;
mod model;
mod rank;
mod request;
mod runtime;

#[cfg(test)]
mod tests;

pub use document::{
    procedure_document, repair_document, repair_document_from_catalog, repair_scene,
    repair_scene_from_catalog,
};
pub use fabric::AutoInfoFabric;
pub use fixtures::{fixture_vehicle, repair_catalog};
pub use model::{InfoSource, RepairProcedure, RepairQuery};
pub use rank::{RepairCandidate, rank_repair_docs};
pub use request::{auto_info_expr, parse_repair_query};
pub use runtime::{
    AutoInfoLib, auto_info_shape_symbol, auto_info_site_symbol, auto_info_sources_symbol,
    install_auto_info_lib,
};
