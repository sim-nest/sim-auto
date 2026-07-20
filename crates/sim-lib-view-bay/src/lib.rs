//! Automotive bay Scene and Intent projection.
//!
//! The crate composes existing modeled automotive diagnostics, repair
//! information, parts, vendor manifests, Scene, Intent, and SurfaceCaps
//! contracts into one bay-facing value surface. It adds no kernel surface type:
//! a bay is ordinary Scene data plus validated Intents over automotive targets.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod intent;
mod model;
mod scene;

#[cfg(test)]
mod tests;

pub use intent::{
    BayIntentOp, add_part_intent, all_modeled_intents, open_procedure_intent, place_order_intent,
    request_backup_intent, request_coding_intent, request_flash_intent, restore_stock_map_intent,
    run_scan_intent, select_dtc_intent,
};
pub use model::{
    BayDtc, BayPanelStatus, BayState, BayTimelineEntry, bay_surface_caps, dtc_status_label,
};
pub use scene::{bay_scene, bay_scene_symbol};
