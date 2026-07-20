//! Intent builders for the automotive bay surface.

use sim_kernel::{Error, Expr, Result, Symbol};
use sim_lib_auto_parts::PartLine;
use sim_lib_intent::{Origin, intent, validate_intent};

use crate::{BayDtc, BayState};

/// Bay actions exposed as validated Intent values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BayIntentOp {
    /// Run a modeled diagnostic scan.
    RunScan,
    /// Open the selected repair document.
    OpenProcedure,
    /// Add a replacement part to the cart.
    AddPart,
    /// Place the modeled supplier order.
    PlaceOrder,
    /// Request service coding.
    RequestCoding,
    /// Request a stock-map backup.
    RequestBackup,
    /// Request a gated flash write.
    RequestFlash,
    /// Restore the stock map.
    RestoreStockMap,
}

impl BayIntentOp {
    /// Stable operation name carried by `intent/invoke`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RunScan => "run-scan",
            Self::OpenProcedure => "open-procedure",
            Self::AddPart => "add-part",
            Self::PlaceOrder => "place-order",
            Self::RequestCoding => "request-coding",
            Self::RequestBackup => "request-backup",
            Self::RequestFlash => "request-flash",
            Self::RestoreStockMap => "restore-stock-map",
        }
    }

    fn symbol(self) -> Symbol {
        Symbol::qualified("auto", self.as_str())
    }
}

/// Builds an Intent selecting one DTC row.
pub fn select_dtc_intent(row: &BayDtc, tick: u64) -> Result<Expr> {
    checked(intent(
        "select",
        Origin::human(tick),
        vec![("targets", Expr::List(vec![dtc_target(row)]))],
    ))
}

/// Builds an Intent requesting a fresh modeled scan.
pub fn run_scan_intent(tick: u64) -> Result<Expr> {
    invoke(BayIntentOp::RunScan, vec![], tick)
}

/// Builds an Intent opening the repair procedure panel.
pub fn open_procedure_intent(procedure: &str, tick: u64) -> Result<Expr> {
    invoke(
        BayIntentOp::OpenProcedure,
        vec![string_field("procedure", procedure)],
        tick,
    )
}

/// Builds an Intent adding a part line to the cart.
pub fn add_part_intent(part: &PartLine, tick: u64) -> Result<Expr> {
    invoke(
        BayIntentOp::AddPart,
        vec![
            string_field("sku", &part.sku),
            string_field("description", &part.description),
            string_field("qty", &part.qty.to_string()),
        ],
        tick,
    )
}

/// Builds an Intent placing the modeled supplier order.
pub fn place_order_intent(tick: u64) -> Result<Expr> {
    invoke(
        BayIntentOp::PlaceOrder,
        vec![string_field("supplier", "mekonomen-pro-modeled")],
        tick,
    )
}

/// Builds an Intent requesting service coding.
pub fn request_coding_intent(ecu: &str, tick: u64) -> Result<Expr> {
    invoke(
        BayIntentOp::RequestCoding,
        vec![string_field("ecu", ecu)],
        tick,
    )
}

/// Builds an Intent requesting a stock-map backup.
pub fn request_backup_intent(ecu: &str, tick: u64) -> Result<Expr> {
    invoke(
        BayIntentOp::RequestBackup,
        vec![string_field("ecu", ecu)],
        tick,
    )
}

/// Builds an Intent requesting a gated flash write.
pub fn request_flash_intent(ecu: &str, tick: u64) -> Result<Expr> {
    invoke(
        BayIntentOp::RequestFlash,
        vec![string_field("ecu", ecu)],
        tick,
    )
}

/// Builds an Intent restoring the stock map.
pub fn restore_stock_map_intent(ecu: &str, tick: u64) -> Result<Expr> {
    invoke(
        BayIntentOp::RestoreStockMap,
        vec![string_field("ecu", ecu)],
        tick,
    )
}

/// Builds one valid Intent for every bay action exposed by the modeled state.
pub fn all_modeled_intents(state: &BayState, start_tick: u64) -> Result<Vec<Expr>> {
    let primary = state
        .primary_dtc()
        .ok_or_else(|| Error::Eval("modeled bay state has no selectable DTC".to_owned()))?;
    let part = state
        .parts_cart
        .first()
        .ok_or_else(|| Error::Eval("modeled bay state has no cart part".to_owned()))?;
    Ok(vec![
        select_dtc_intent(primary, start_tick)?,
        run_scan_intent(start_tick + 1)?,
        open_procedure_intent(&state.repair_title, start_tick + 2)?,
        add_part_intent(part, start_tick + 3)?,
        place_order_intent(start_tick + 4)?,
        request_coding_intent(&primary.ecu, start_tick + 5)?,
        request_backup_intent(&primary.ecu, start_tick + 6)?,
        request_flash_intent(&primary.ecu, start_tick + 7)?,
        restore_stock_map_intent(&primary.ecu, start_tick + 8)?,
    ])
}

fn invoke(op: BayIntentOp, args: Vec<Expr>, tick: u64) -> Result<Expr> {
    checked(intent(
        "invoke",
        Origin::human(tick),
        vec![
            ("target", Expr::Symbol(Symbol::qualified("auto", "bay"))),
            ("op", Expr::Symbol(op.symbol())),
            ("args", Expr::List(args)),
        ],
    ))
}

fn checked(value: Expr) -> Result<Expr> {
    validate_intent(&value)
        .map_err(|err| Error::Eval(format!("auto bay Intent invalid: {err}")))?;
    Ok(value)
}

fn dtc_target(row: &BayDtc) -> Expr {
    Expr::Map(vec![
        string_pair("target", "auto/bay/dtc"),
        string_pair("ecu", &row.ecu),
        string_pair("code", &row.dtc.code),
    ])
}

fn string_field(name: &str, value: &str) -> Expr {
    Expr::Map(vec![string_pair(name, value)])
}

fn string_pair(name: &str, value: &str) -> (Expr, Expr) {
    (
        Expr::Symbol(Symbol::new(name.to_owned())),
        Expr::String(value.to_owned()),
    )
}
