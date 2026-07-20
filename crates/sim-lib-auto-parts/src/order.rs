//! Mekonomen-style supplier ordering through the vendor effect gate.

use std::sync::{Mutex, MutexGuard};

use sim_kernel::{CapabilityName, Cx, Error, Expr, Result, Symbol};
use sim_lib_auto_core::{AUTO_ORDER, AutoLane, SiteManifest, VehicleId};
use sim_lib_auto_vendor::{VendorBridge, VendorBridgeRequest, VendorGateLedger, warranted_effect};

use crate::{OrderStatus, PartLine, Supplier};

/// Canonical network capability for live supplier placement.
pub const NET_HTTP_CAPABILITY: &str = "net/http";
/// Operation name declared in the Mekonomen modeled manifest.
pub const ORDER_OPERATION: &str = "order/place";

/// In-memory fixture ledger for modeled supplier orders.
#[derive(Default)]
pub struct ModeledOrderLedger {
    records: Mutex<Vec<OrderStatus>>,
}

impl ModeledOrderLedger {
    /// Builds an empty modeled order ledger.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns recorded modeled order statuses.
    pub fn records(&self) -> Result<Vec<OrderStatus>> {
        Ok(lock(&self.records, "modeled order ledger")?.clone())
    }

    fn push(&self, status: OrderStatus) -> Result<()> {
        lock(&self.records, "modeled order ledger")?.push(status);
        Ok(())
    }
}

/// Places a parts order through the shared warranted-effect gate.
pub fn place_order(
    cx: &mut Cx,
    supplier: Supplier,
    lines: Vec<PartLine>,
    ledger: &ModeledOrderLedger,
) -> Result<OrderStatus> {
    let gate_ledger = VendorGateLedger::new();
    place_order_with_gate(cx, supplier, lines, ledger, &gate_ledger)
}

/// Places a parts order with an explicit vendor gate ledger.
pub fn place_order_with_gate(
    cx: &mut Cx,
    supplier: Supplier,
    lines: Vec<PartLine>,
    ledger: &ModeledOrderLedger,
    gate_ledger: &VendorGateLedger,
) -> Result<OrderStatus> {
    validate_lines(&lines)?;
    let bridge = OrderBridge::new(supplier, lines.clone());
    let request = VendorBridgeRequest::new(
        "mekonomen-pro",
        AutoLane::new("parts"),
        ORDER_OPERATION,
        VehicleId::new("fixture", "mekonomen-order"),
        order_args_expr(supplier, &lines),
    );
    let required = required_capabilities(supplier);
    warranted_effect(
        cx,
        &mekonomen_order_manifest(),
        request,
        &required,
        gate_ledger,
        &bridge,
    )?;
    let status = bridge.status()?.ok_or_else(|| {
        Error::Eval("modeled order bridge did not produce an order status".to_owned())
    })?;
    ledger.push(status.clone())?;
    Ok(status)
}

/// Manifest used for Mekonomen Pro modeled ordering.
pub fn mekonomen_order_manifest() -> SiteManifest {
    SiteManifest::new(
        "mekonomen-pro",
        "mekonomen-order",
        "mekonomen",
        vec!["parts".to_owned(), "service".to_owned()],
        vec!["modeled".to_owned(), "http-dir".to_owned()],
        vec![ORDER_OPERATION.to_owned()],
    )
}

fn required_capabilities(supplier: Supplier) -> Vec<CapabilityName> {
    let mut capabilities = vec![CapabilityName::new(AUTO_ORDER)];
    if supplier.is_live() {
        capabilities.push(CapabilityName::new(NET_HTTP_CAPABILITY));
    }
    capabilities
}

fn validate_lines(lines: &[PartLine]) -> Result<()> {
    if lines.is_empty() {
        return Err(Error::Eval(
            "auto parts order requires at least one line".to_owned(),
        ));
    }
    for line in lines {
        if line.sku.trim().is_empty() {
            return Err(Error::Eval(
                "auto parts order line has empty SKU".to_owned(),
            ));
        }
        if line.qty == 0 {
            return Err(Error::Eval(format!(
                "auto parts order line {} has zero quantity",
                line.sku
            )));
        }
    }
    Ok(())
}

struct OrderBridge {
    supplier: Supplier,
    lines: Vec<PartLine>,
    status: Mutex<Option<OrderStatus>>,
}

impl OrderBridge {
    fn new(supplier: Supplier, lines: Vec<PartLine>) -> Self {
        Self {
            supplier,
            lines,
            status: Mutex::new(None),
        }
    }

    fn status(&self) -> Result<Option<OrderStatus>> {
        Ok(lock(&self.status, "modeled order status")?.clone())
    }
}

impl VendorBridge for OrderBridge {
    fn call(&self, _cx: &mut Cx, request: &VendorBridgeRequest) -> Result<Expr> {
        if self.supplier.is_live() {
            return Err(Error::Eval(
                "live Mekonomen Pro ordering requires a host-owned HttpDir; this public crate commits no endpoint, key, order account, or network transport"
                    .to_owned(),
            ));
        }
        let id = format!("SIM-ORDER-{}", stable_order_suffix(&self.lines));
        let status = OrderStatus::accepted(id, self.supplier, self.lines.len() as u32);
        *lock(&self.status, "modeled order status")? = Some(status.clone());
        Ok(Expr::Map(vec![
            string_field("site", &request.site),
            string_field("operation", &request.op),
            string_field("supplier", self.supplier.as_str()),
            (Expr::Symbol(Symbol::new("status")), status.to_expr()),
        ]))
    }
}

fn order_args_expr(supplier: Supplier, lines: &[PartLine]) -> Expr {
    Expr::Map(vec![
        string_field("supplier", supplier.as_str()),
        (
            Expr::Symbol(Symbol::new("lines")),
            Expr::List(lines.iter().map(PartLine::to_expr).collect()),
        ),
    ])
}

fn stable_order_suffix(lines: &[PartLine]) -> String {
    lines
        .iter()
        .map(|line| format!("{}x{}", line.sku, line.qty))
        .collect::<Vec<_>>()
        .join("-")
}

fn string_field(name: &str, value: &str) -> (Expr, Expr) {
    (
        Expr::Symbol(Symbol::new(name.to_owned())),
        Expr::String(value.to_owned()),
    )
}

fn lock<'a, T>(mutex: &'a Mutex<T>, name: &str) -> Result<MutexGuard<'a, T>> {
    mutex
        .lock()
        .map_err(|_| Error::Eval(format!("{name} mutex poisoned")))
}
