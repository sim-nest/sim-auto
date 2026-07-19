//! DTC status-bit helpers shared by the UDS codec and tests.

use sim_kernel::{Expr, Symbol};
use sim_lib_auto_core::DtcStatus;

/// Decodes a UDS DTC status byte into the automotive status citizen.
pub fn decode_dtc_status(byte: u8) -> DtcStatus {
    DtcStatus::from_byte(byte)
}

/// Decodes a UDS DTC status byte into an expression map.
pub fn dtc_status_expr(byte: u8) -> Expr {
    let status = decode_dtc_status(byte);
    Expr::Map(vec![
        field("test_failed", status.test_failed),
        field(
            "test_failed_this_operation_cycle",
            status.test_failed_this_operation_cycle,
        ),
        field("pending", status.pending),
        field("confirmed", status.confirmed),
        field(
            "test_not_completed_since_clear",
            status.test_not_completed_since_clear,
        ),
        field("test_failed_since_clear", status.test_failed_since_clear),
        field(
            "test_not_completed_this_operation_cycle",
            status.test_not_completed_this_operation_cycle,
        ),
        field("warning_indicator", status.warning_indicator),
    ])
}

pub(crate) fn status_byte_from_expr(expr: &Expr) -> Option<u8> {
    let Expr::Map(entries) = expr else {
        return None;
    };
    let status = DtcStatus {
        test_failed: bool_field(entries, "test_failed")?,
        test_failed_this_operation_cycle: bool_field(entries, "test_failed_this_operation_cycle")?,
        pending: bool_field(entries, "pending")?,
        confirmed: bool_field(entries, "confirmed")?,
        test_not_completed_since_clear: bool_field(entries, "test_not_completed_since_clear")?,
        test_failed_since_clear: bool_field(entries, "test_failed_since_clear")?,
        test_not_completed_this_operation_cycle: bool_field(
            entries,
            "test_not_completed_this_operation_cycle",
        )?,
        warning_indicator: bool_field(entries, "warning_indicator")?,
    };
    Some(status.to_byte())
}

fn field(name: &str, value: bool) -> (Expr, Expr) {
    (Expr::Symbol(Symbol::new(name)), Expr::Bool(value))
}

fn bool_field(entries: &[(Expr, Expr)], name: &str) -> Option<bool> {
    entries.iter().find_map(|(key, value)| {
        if key == &Expr::Symbol(Symbol::new(name)) {
            match value {
                Expr::Bool(value) => Some(*value),
                _ => None,
            }
        } else {
            None
        }
    })
}
