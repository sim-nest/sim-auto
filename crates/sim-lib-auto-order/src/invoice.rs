//! Balanced invoice export data for modeled work orders.

use sim_citizen::{CitizenField, field_error};
use sim_kernel::{Error, Expr, NumberLiteral, Result, Symbol};
use sim_lib_auto_core::text_read_construct_expr;
use sim_lib_auto_parts::{OrderStatus, PartLine};

/// Reference-only invoice evidence compatible with ledger draft attachments.
#[derive(Clone, Debug, PartialEq, Eq, sim_citizen_derive::Citizen)]
#[citizen(symbol = "auto/LedgerInvoiceEvidence", version = 0)]
pub struct LedgerInvoiceEvidence {
    /// Source namespace for the evidence.
    pub backend: String,
    /// Source-local evidence id.
    pub external_id: String,
    /// Optional immutable version or digest marker.
    pub version: Option<String>,
    /// Optional operator-facing URL.
    pub web_url: Option<String>,
    /// Optional voucher or capture digest.
    pub immutable_hint: Option<String>,
}

impl Default for LedgerInvoiceEvidence {
    fn default() -> Self {
        Self::new(
            "auto/work-order",
            "SIM-WO-1",
            Some("modeled-v0"),
            None::<String>,
            None::<String>,
        )
    }
}

impl LedgerInvoiceEvidence {
    /// Builds an invoice evidence reference.
    pub fn new(
        backend: impl Into<String>,
        external_id: impl Into<String>,
        version: Option<impl Into<String>>,
        web_url: Option<impl Into<String>>,
        immutable_hint: Option<impl Into<String>>,
    ) -> Self {
        Self {
            backend: backend.into(),
            external_id: external_id.into(),
            version: version.map(Into::into),
            web_url: web_url.map(Into::into),
            immutable_hint: immutable_hint.map(Into::into),
        }
    }

    /// Encodes this evidence reference as explicit read-construct data.
    pub fn to_expr(&self) -> Expr {
        text_read_construct_expr(
            "auto/LedgerInvoiceEvidence",
            vec![
                Expr::Symbol(Symbol::new("v0")),
                Expr::String(self.backend.clone()),
                Expr::String(self.external_id.clone()),
                option_string_expr(&self.version),
                option_string_expr(&self.web_url),
                option_string_expr(&self.immutable_hint),
            ],
        )
    }
}

impl CitizenField for LedgerInvoiceEvidence {
    fn encode_field(&self) -> Expr {
        self.to_expr()
    }

    fn decode_field_expr(expr: &Expr, field: &'static str) -> Result<Self> {
        let args = read_construct_args(expr, "auto/LedgerInvoiceEvidence", field, 5)?;
        Ok(Self {
            backend: string_arg(&args[0], field)?,
            external_id: string_arg(&args[1], field)?,
            version: option_string_arg(&args[2], field)?,
            web_url: option_string_arg(&args[3], field)?,
            immutable_hint: option_string_arg(&args[4], field)?,
        })
    }
}

/// One exact minor-unit invoice posting.
#[derive(Clone, Debug, PartialEq, Eq, sim_citizen_derive::Citizen)]
#[citizen(symbol = "auto/LedgerInvoicePosting", version = 0)]
pub struct LedgerInvoicePosting {
    /// Ledger account text.
    pub account: String,
    /// Signed amount in currency minor units.
    pub amount_minor: i64,
    /// Operator-facing posting text.
    pub text: String,
}

impl Default for LedgerInvoicePosting {
    fn default() -> Self {
        Self::new("1510", 1_000, "modeled customer receivable")
    }
}

impl LedgerInvoicePosting {
    /// Builds an invoice posting.
    pub fn new(account: impl Into<String>, amount_minor: i64, text: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            amount_minor,
            text: text.into(),
        }
    }

    /// Encodes this posting as explicit read-construct data.
    pub fn to_expr(&self) -> Expr {
        text_read_construct_expr(
            "auto/LedgerInvoicePosting",
            vec![
                Expr::Symbol(Symbol::new("v0")),
                Expr::String(self.account.clone()),
                number_expr(self.amount_minor),
                Expr::String(self.text.clone()),
            ],
        )
    }
}

impl CitizenField for LedgerInvoicePosting {
    fn encode_field(&self) -> Expr {
        self.to_expr()
    }

    fn decode_field_expr(expr: &Expr, field: &'static str) -> Result<Self> {
        let args = read_construct_args(expr, "auto/LedgerInvoicePosting", field, 3)?;
        Ok(Self {
            account: string_arg(&args[0], field)?,
            amount_minor: i64_arg(&args[1], field)?,
            text: string_arg(&args[2], field)?,
        })
    }
}

/// Balanced invoice draft export for ledger tooling.
#[derive(Clone, Debug, PartialEq, Eq, sim_citizen_derive::Citizen)]
#[citizen(symbol = "auto/LedgerInvoiceExport", version = 0)]
pub struct LedgerInvoiceExport {
    /// ISO-like voucher date text.
    pub voucher_date: String,
    /// Operator-facing voucher text.
    pub voucher_text: String,
    /// Currency code.
    pub currency: String,
    /// Exact posting lines.
    pub postings: Vec<LedgerInvoicePosting>,
    /// Reference-only evidence attachments.
    pub evidence: Vec<LedgerInvoiceEvidence>,
}

impl Default for LedgerInvoiceExport {
    fn default() -> Self {
        Self::for_work_order(
            "SIM-WO-1",
            &OrderStatus::default(),
            &[PartLine::default()],
            90,
        )
    }
}

impl LedgerInvoiceExport {
    /// Builds the modeled invoice export for one work order.
    pub fn for_work_order(
        work_order_id: &str,
        order: &OrderStatus,
        parts: &[PartLine],
        labor_minutes: u32,
    ) -> Self {
        let parts_minor = parts_minor(parts);
        let labor_minor = i64::from(labor_minutes) * 1_250;
        let total_minor = parts_minor + labor_minor;
        Self {
            voucher_date: "2026-01-15".to_owned(),
            voucher_text: format!("modeled work order {work_order_id} {}", order.id),
            currency: "SEK".to_owned(),
            postings: vec![
                LedgerInvoicePosting::new("1510", total_minor, "customer receivable"),
                LedgerInvoicePosting::new("3041", -parts_minor, "modeled parts revenue"),
                LedgerInvoicePosting::new("3051", -labor_minor, "modeled labor revenue"),
            ],
            evidence: vec![LedgerInvoiceEvidence::new(
                "auto/work-order",
                work_order_id,
                Some("modeled-v0"),
                None::<String>,
                Some(order.ledger_ref.clone()),
            )],
        }
    }

    /// Returns whether the posting lines sum to zero.
    pub fn is_balanced(&self) -> bool {
        self.postings
            .iter()
            .map(|posting| posting.amount_minor)
            .sum::<i64>()
            == 0
    }

    /// Returns the signed posting sum.
    pub fn minor_sum(&self) -> i64 {
        self.postings
            .iter()
            .map(|posting| posting.amount_minor)
            .sum()
    }

    /// Encodes this export as explicit read-construct data.
    pub fn to_expr(&self) -> Expr {
        text_read_construct_expr(
            "auto/LedgerInvoiceExport",
            vec![
                Expr::Symbol(Symbol::new("v0")),
                Expr::String(self.voucher_date.clone()),
                Expr::String(self.voucher_text.clone()),
                Expr::String(self.currency.clone()),
                Expr::List(
                    self.postings
                        .iter()
                        .map(LedgerInvoicePosting::to_expr)
                        .collect(),
                ),
                Expr::List(
                    self.evidence
                        .iter()
                        .map(LedgerInvoiceEvidence::to_expr)
                        .collect(),
                ),
            ],
        )
    }
}

impl CitizenField for LedgerInvoiceExport {
    fn encode_field(&self) -> Expr {
        self.to_expr()
    }

    fn decode_field_expr(expr: &Expr, field: &'static str) -> Result<Self> {
        let args = read_construct_args(expr, "auto/LedgerInvoiceExport", field, 5)?;
        Ok(Self {
            voucher_date: string_arg(&args[0], field)?,
            voucher_text: string_arg(&args[1], field)?,
            currency: string_arg(&args[2], field)?,
            postings: Vec::<LedgerInvoicePosting>::decode_field_expr(&args[3], field)?,
            evidence: Vec::<LedgerInvoiceEvidence>::decode_field_expr(&args[4], field)?,
        })
    }
}

fn parts_minor(parts: &[PartLine]) -> i64 {
    parts
        .iter()
        .map(|part| i64::from(part.qty) * modeled_unit_minor(&part.sku))
        .sum()
}

fn modeled_unit_minor(sku: &str) -> i64 {
    if sku.contains("COIL") { 54_900 } else { 19_900 }
}

fn option_string_expr(value: &Option<String>) -> Expr {
    value
        .as_ref()
        .map(|text| Expr::String(text.clone()))
        .unwrap_or(Expr::Nil)
}

pub(crate) fn number_expr(value: i64) -> Expr {
    Expr::Number(NumberLiteral {
        domain: Symbol::qualified("core", "Number"),
        canonical: value.to_string(),
    })
}

pub(crate) fn read_construct_args<'a>(
    expr: &'a Expr,
    class: &'static str,
    field: &'static str,
    arity: usize,
) -> Result<&'a [Expr]> {
    let Expr::Extension { tag, payload } = expr else {
        return Err(field_error(field, "expected citizen read-construct"));
    };
    if tag != &Symbol::qualified("citizen", "read-construct") {
        return Err(field_error(
            field,
            format!("expected citizen read-construct, found {tag}"),
        ));
    }
    let Expr::Vector(items) = payload.as_ref() else {
        return Err(field_error(
            field,
            "read-construct payload must be a vector",
        ));
    };
    if items.len() != arity + 2 {
        return Err(field_error(
            field,
            format!(
                "expected {arity} constructor field(s), found {}",
                items.len().saturating_sub(2)
            ),
        ));
    }
    match &items[0] {
        Expr::Symbol(symbol) if symbol == &symbol_text(class) => {}
        Expr::Symbol(symbol) => {
            return Err(field_error(
                field,
                format!("expected class {class}, found {symbol}"),
            ));
        }
        other => {
            return Err(field_error(
                field,
                format!("expected class symbol, found {other:?}"),
            ));
        }
    }
    match &items[1] {
        Expr::Symbol(version) if version == &Symbol::new("v0") => Ok(&items[2..]),
        other => Err(field_error(field, format!("expected v0, found {other:?}"))),
    }
}

pub(crate) fn string_arg(expr: &Expr, field: &'static str) -> Result<String> {
    match expr {
        Expr::String(value) => Ok(value.clone()),
        Expr::Symbol(symbol) => Ok(symbol.as_qualified_str()),
        other => Err(field_error(
            field,
            format!("expected string or symbol, found {other:?}"),
        )),
    }
}

pub(crate) fn bool_arg(expr: &Expr, field: &'static str) -> Result<bool> {
    match expr {
        Expr::Bool(value) => Ok(*value),
        other => Err(field_error(
            field,
            format!("expected bool, found {other:?}"),
        )),
    }
}

pub(crate) fn i64_arg(expr: &Expr, field: &'static str) -> Result<i64> {
    let Expr::Number(number) = expr else {
        return Err(field_error(
            field,
            format!("expected number, found {expr:?}"),
        ));
    };
    number
        .canonical
        .parse::<i64>()
        .map_err(|err| Error::Eval(format!("citizen field {field}: invalid integer: {err}")))
}

pub(crate) fn u32_arg(expr: &Expr, field: &'static str) -> Result<u32> {
    let value = i64_arg(expr, field)?;
    u32::try_from(value)
        .map_err(|_| field_error(field, format!("integer {value} is out of range for u32")))
}

pub(crate) fn option_string_arg(expr: &Expr, field: &'static str) -> Result<Option<String>> {
    match expr {
        Expr::Nil => Ok(None),
        other => string_arg(other, field).map(Some),
    }
}

pub(crate) fn string_list_arg(expr: &Expr, field: &'static str) -> Result<Vec<String>> {
    match expr {
        Expr::List(items) => items.iter().map(|item| string_arg(item, field)).collect(),
        other => Err(field_error(
            field,
            format!("expected list, found {other:?}"),
        )),
    }
}

pub(crate) fn symbol_text(text: &str) -> Symbol {
    if let Some((namespace, name)) = text.split_once('/') {
        Symbol::qualified(namespace, name)
    } else {
        Symbol::new(text)
    }
}
