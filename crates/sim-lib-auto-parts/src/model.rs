//! Parts catalog and order status model values.

use sim_kernel::{Expr, NumberLiteral, Symbol};
use sim_lib_auto_core::text_read_construct_expr;

/// Modeled parts catalog family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartsCatalog {
    /// Mercedes EPC-shaped modeled catalog.
    EpcModeled,
    /// Aftermarket catalog-shaped modeled catalog.
    AftermarketModeled,
}

impl PartsCatalog {
    /// Parses a catalog label.
    pub fn parse(label: &str) -> Option<Self> {
        match normalize(label).as_str() {
            "epc" | "epc-modeled" | "mercedes-epc" => Some(Self::EpcModeled),
            "aftermarket" | "aftermarket-modeled" | "mekonomen" | "mekonomen-modeled" => {
                Some(Self::AftermarketModeled)
            }
            _ => None,
        }
    }

    /// Stable catalog label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EpcModeled => "epc-modeled",
            Self::AftermarketModeled => "aftermarket-modeled",
        }
    }
}

/// Supplier mode for automotive parts ordering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Supplier {
    /// Public modeled Mekonomen Pro fixture supplier.
    MekonomenProModeled,
    /// Host-owned live Mekonomen Pro placement.
    MekonomenProLive,
}

impl Supplier {
    /// Parses a supplier label.
    pub fn parse(label: &str) -> Option<Self> {
        match normalize(label).as_str() {
            "mekonomen-pro" | "mekonomen-pro-modeled" | "modeled" => {
                Some(Self::MekonomenProModeled)
            }
            "mekonomen-pro-live" | "live" => Some(Self::MekonomenProLive),
            _ => None,
        }
    }

    /// Stable supplier label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MekonomenProModeled => "mekonomen-pro-modeled",
            Self::MekonomenProLive => "mekonomen-pro-live",
        }
    }

    /// Whether this supplier needs a host-owned network placement.
    pub fn is_live(self) -> bool {
        matches!(self, Self::MekonomenProLive)
    }
}

/// One catalog or order line for a replacement part.
#[derive(Clone, Debug, PartialEq, Eq, sim_citizen_derive::Citizen)]
#[citizen(symbol = "auto/PartLine", version = 0)]
pub struct PartLine {
    /// Supplier SKU.
    pub sku: String,
    /// Optional OEM reference.
    pub oem: Option<String>,
    /// Synthetic public description.
    pub description: String,
    /// Requested quantity.
    pub qty: u32,
}

impl Default for PartLine {
    fn default() -> Self {
        Self::new(
            "SIM-COIL-1",
            Some("A0001500180"),
            "modeled ignition coil",
            1,
        )
    }
}

impl PartLine {
    /// Builds a part line.
    pub fn new(
        sku: impl Into<String>,
        oem: Option<impl Into<String>>,
        description: impl Into<String>,
        qty: u32,
    ) -> Self {
        Self {
            sku: sku.into(),
            oem: oem.map(Into::into),
            description: description.into(),
            qty,
        }
    }

    /// Encodes this part as explicit read-construct data.
    pub fn to_expr(&self) -> Expr {
        text_read_construct_expr(
            "auto/PartLine",
            vec![
                Expr::Symbol(Symbol::new("v0")),
                Expr::String(self.sku.clone()),
                self.oem
                    .as_ref()
                    .map(|oem| Expr::String(oem.clone()))
                    .unwrap_or(Expr::Nil),
                Expr::String(self.description.clone()),
                number_expr(self.qty),
            ],
        )
    }
}

/// Result of a supplier order request.
#[derive(Clone, Debug, PartialEq, Eq, sim_citizen_derive::Citizen)]
#[citizen(symbol = "auto/OrderStatus", version = 0)]
pub struct OrderStatus {
    /// Stable modeled order id.
    pub id: String,
    /// Supplier label.
    pub supplier: String,
    /// Whether the modeled supplier accepted the order.
    pub accepted: bool,
    /// Whether the operation is reversible.
    pub reversible: bool,
    /// Number of order lines.
    pub line_count: u32,
    /// Fixture ledger reference.
    pub ledger_ref: String,
}

impl Default for OrderStatus {
    fn default() -> Self {
        Self::accepted("SIM-ORDER-1", Supplier::MekonomenProModeled, 1)
    }
}

impl OrderStatus {
    /// Builds an accepted order status.
    pub fn accepted(id: impl Into<String>, supplier: Supplier, line_count: u32) -> Self {
        let id = id.into();
        Self {
            ledger_ref: format!("fixture-ledger/{id}"),
            id,
            supplier: supplier.as_str().to_owned(),
            accepted: true,
            reversible: true,
            line_count,
        }
    }

    /// Encodes this status as explicit read-construct data.
    pub fn to_expr(&self) -> Expr {
        text_read_construct_expr(
            "auto/OrderStatus",
            vec![
                Expr::Symbol(Symbol::new("v0")),
                Expr::String(self.id.clone()),
                Expr::String(self.supplier.clone()),
                Expr::Bool(self.accepted),
                Expr::Bool(self.reversible),
                number_expr(self.line_count),
                Expr::String(self.ledger_ref.clone()),
            ],
        )
    }
}

pub(crate) fn number_expr(value: u32) -> Expr {
    Expr::Number(NumberLiteral {
        domain: Symbol::qualified("core", "Number"),
        canonical: value.to_string(),
    })
}

pub(crate) fn normalize(label: &str) -> String {
    label.trim().to_ascii_lowercase().replace('_', "-")
}
