//! Ledgered modeled automotive work-order sessions.
//!
//! The crate ties the public automotive site manifests into one modeled
//! work-order replay. It keeps every concrete vehicle, vendor, and supplier
//! interaction synthetic while preserving the policy shape of a shop session:
//! parent grants are diminished before each delegated site call, accepted steps
//! are ledgered, and denied service or flash attempts stay visible in the
//! resulting report.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod conformance;
mod invoice;
mod model;
mod runtime;

#[cfg(test)]
mod tests;

pub use conformance::{
    ModeledWorkOrderEngine, expected_modeled_sites, run_modeled_conformance,
    run_modeled_conformance_with_parent_grants,
};
pub use invoice::{LedgerInvoiceEvidence, LedgerInvoiceExport, LedgerInvoicePosting};
pub use model::{ConformanceReport, WorkOrder, WorkOrderEvent, WorkOrderLedger};
pub use runtime::{
    AutoOrderLib, auto_order_citizen_symbols, auto_order_function_symbol, auto_order_shape_symbol,
    install_auto_order_lib,
};
