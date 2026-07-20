//! Request expressions for the modeled parts site.

use sim_kernel::{Error, Expr, NumberLiteral, Result, Symbol};

use crate::{PartLine, Supplier};

/// Parsed request served by the modeled parts fabric.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PartsRequest {
    /// Fetch one catalog part by path.
    CatalogGet {
        /// Catalog path segments.
        path: Vec<String>,
    },
    /// Place a supplier order.
    Order {
        /// Supplier mode.
        supplier: Supplier,
        /// Order lines.
        lines: Vec<PartLine>,
    },
}

/// Builds a catalog lookup request expression.
pub fn parts_catalog_get_expr(path: &[&str]) -> Expr {
    Expr::Map(vec![
        string_field("op", "catalog/get"),
        (
            Expr::Symbol(Symbol::new("path")),
            Expr::List(
                path.iter()
                    .map(|segment| Expr::String((*segment).to_owned()))
                    .collect(),
            ),
        ),
    ])
}

/// Builds an order request expression.
pub fn auto_order_expr(supplier: Supplier, lines: &[PartLine]) -> Expr {
    Expr::Map(vec![
        string_field("op", "order/place"),
        string_field("supplier", supplier.as_str()),
        (
            Expr::Symbol(Symbol::new("lines")),
            Expr::List(lines.iter().map(part_line_map).collect()),
        ),
    ])
}

/// Parses a parts request expression.
pub fn parse_parts_request(expr: &Expr) -> Result<PartsRequest> {
    let Expr::Map(entries) = expr else {
        return Err(Error::Eval("auto parts request must be a map".to_owned()));
    };
    match string_field_value(entries, "op")?.as_str() {
        "catalog/get" => Ok(PartsRequest::CatalogGet {
            path: path_field(entries, "path")?,
        }),
        "order/place" | "auto/order" => {
            let supplier = Supplier::parse(&string_field_value(entries, "supplier")?)
                .ok_or_else(|| Error::Eval("unknown auto parts supplier".to_owned()))?;
            Ok(PartsRequest::Order {
                supplier,
                lines: line_list(entries, "lines")?,
            })
        }
        other => Err(Error::Eval(format!("unknown auto parts operation {other}"))),
    }
}

fn line_list(entries: &[(Expr, Expr)], field: &'static str) -> Result<Vec<PartLine>> {
    let Expr::List(items) = required(entries, field)? else {
        return Err(Error::Eval(format!(
            "auto parts request field {field} must be a list"
        )));
    };
    items.iter().map(part_line_from_expr).collect()
}

fn part_line_from_expr(expr: &Expr) -> Result<PartLine> {
    let Expr::Map(entries) = expr else {
        return Err(Error::Eval(
            "auto parts order line must be a map".to_owned(),
        ));
    };
    Ok(PartLine::new(
        string_field_value(entries, "sku")?,
        optional_string_field(entries, "oem"),
        string_field_value(entries, "description")?,
        u32_field(entries, "qty")?,
    ))
}

fn path_field(entries: &[(Expr, Expr)], field: &'static str) -> Result<Vec<String>> {
    let Expr::List(items) = required(entries, field)? else {
        return Err(Error::Eval(format!(
            "auto parts request field {field} must be a list"
        )));
    };
    items.iter().map(string_value).collect()
}

fn part_line_map(line: &PartLine) -> Expr {
    let mut fields = vec![
        string_field("sku", &line.sku),
        string_field("description", &line.description),
        (Expr::Symbol(Symbol::new("qty")), number_expr(line.qty)),
    ];
    if let Some(oem) = &line.oem {
        fields.push(string_field("oem", oem));
    }
    Expr::Map(fields)
}

fn required<'a>(entries: &'a [(Expr, Expr)], field: &'static str) -> Result<&'a Expr> {
    entries
        .iter()
        .find_map(|(key, value)| (field_key(key).as_deref() == Some(field)).then_some(value))
        .ok_or_else(|| Error::Eval(format!("missing auto parts request field {field}")))
}

fn string_field_value(entries: &[(Expr, Expr)], field: &'static str) -> Result<String> {
    string_value(required(entries, field)?)
}

fn optional_string_field(entries: &[(Expr, Expr)], field: &'static str) -> Option<String> {
    entries
        .iter()
        .find_map(|(key, value)| (field_key(key).as_deref() == Some(field)).then_some(value))
        .and_then(|value| string_value(value).ok())
}

fn string_value(expr: &Expr) -> Result<String> {
    match expr {
        Expr::String(value) => Ok(value.clone()),
        Expr::Symbol(symbol) => Ok(symbol.as_qualified_str()),
        _ => Err(Error::Eval(
            "auto parts request field must be string or symbol".to_owned(),
        )),
    }
}

fn u32_field(entries: &[(Expr, Expr)], field: &'static str) -> Result<u32> {
    match required(entries, field)? {
        Expr::Number(number) => number
            .canonical
            .parse::<u32>()
            .map_err(|_| Error::Eval(format!("auto parts request field {field} must be a u32"))),
        _ => Err(Error::Eval(format!(
            "auto parts request field {field} must be numeric"
        ))),
    }
}

fn field_key(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Symbol(symbol) => Some(symbol.name.trim_start_matches(':').to_owned()),
        Expr::String(value) => Some(value.trim_start_matches(':').to_owned()),
        _ => None,
    }
}

fn string_field(name: &str, value: &str) -> (Expr, Expr) {
    (
        Expr::Symbol(Symbol::new(name.to_owned())),
        Expr::String(value.to_owned()),
    )
}

fn number_expr(value: u32) -> Expr {
    Expr::Number(NumberLiteral {
        domain: Symbol::qualified("core", "Number"),
        canonical: value.to_string(),
    })
}
