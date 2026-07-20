//! Modeled automotive parts lookup and supplier ordering.
//!
//! The crate exposes synthetic Mercedes EPC-shaped and aftermarket catalog data
//! as SIM directory tables. Mekonomen Pro ordering is a reversible automotive
//! vendor operation: modeled orders record to a fixture ledger, while live
//! supplier mode requires `auto/order` and `net/http` and then fails closed
//! until a host-owned HTTP placement is installed.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod catalog;
mod fabric;
mod model;
mod order;
mod request;
mod runtime;

#[cfg(test)]
mod tests;

pub use catalog::{PartsDir, catalog_part, modeled_aftermarket_dir, modeled_epc_dir, parts_dir};
pub use fabric::AutoPartsFabric;
pub use model::{OrderStatus, PartLine, PartsCatalog, Supplier};
pub use order::{
    ModeledOrderLedger, NET_HTTP_CAPABILITY, ORDER_OPERATION, mekonomen_order_manifest,
    place_order, place_order_with_gate,
};
pub use request::{PartsRequest, auto_order_expr, parse_parts_request, parts_catalog_get_expr};
pub use runtime::{
    AutoPartsLib, auto_parts_citizen_symbols, auto_parts_dir_symbol, auto_parts_shape_symbol,
    auto_parts_site_symbol, auto_parts_suppliers_symbol, install_auto_parts_lib,
};
