//! Request expression helpers and parser for the `auto/info` site.

use std::collections::BTreeMap;

use sim_kernel::{Error, Expr, Result, Symbol};
use sim_lib_auto_core::{AutoLane, VehicleId};

use crate::{InfoSource, RepairQuery};

/// Builds an `auto/info` request expression.
pub fn auto_info_expr(
    vehicle: &VehicleId,
    source: InfoSource,
    dtc: Option<&str>,
    ecu: Option<&str>,
    symptom: Option<&str>,
) -> Expr {
    let mut entries = vec![
        symbol_field("op", "auto/info"),
        string_field("namespace", &vehicle.namespace),
        string_field("vehicle", &vehicle.key),
        string_field("source", source.as_str()),
    ];
    if let Some(dtc) = dtc {
        entries.push(string_field("dtc", dtc));
    }
    if let Some(ecu) = ecu {
        entries.push(string_field("ecu", ecu));
    }
    if let Some(symptom) = symptom {
        entries.push(string_field("symptom", symptom));
    }
    Expr::Map(entries)
}

/// Parses a repair query from a map, list, or vector expression.
pub fn parse_repair_query(expr: &Expr) -> Result<RepairQuery> {
    let mut fields = match expr {
        Expr::Map(entries) => map_fields(entries)?,
        Expr::List(items) | Expr::Vector(items) => list_fields(items)?,
        _ => {
            return Err(Error::Eval(
                "auto/info request must be a map, list, or vector".to_owned(),
            ));
        }
    };
    if let Some(op) = fields.remove("op") {
        let op = string_value(&op)?;
        if normalize(&op) != "auto/info" {
            return Err(Error::Eval(format!("unsupported auto info op {op}")));
        }
    }
    let namespace = fields
        .remove("namespace")
        .map(|expr| string_value(&expr))
        .transpose()?
        .unwrap_or_else(|| "modeled-se".to_owned());
    let vehicle_key = fields
        .remove("vehicle")
        .map(|expr| string_value(&expr))
        .transpose()?
        .unwrap_or_else(|| "vehicle-alpha".to_owned());
    let mut query = RepairQuery::new(VehicleId::new(namespace, vehicle_key));
    if let Some(source) = fields.remove("source") {
        let source = string_value(&source)?;
        query = query.with_source(
            InfoSource::parse(&source)
                .ok_or_else(|| Error::Eval(format!("unknown auto info source {source}")))?,
        );
    }
    if let Some(dtc) = fields.remove("dtc") {
        query = query.with_dtc_code(string_value(&dtc)?);
    }
    if let Some(ecu) = fields.remove("ecu") {
        query = query.with_ecu(string_value(&ecu)?);
    }
    if let Some(symptom) = fields.remove("symptom") {
        query = query.with_symptom(string_value(&symptom)?);
    }
    if let Some(lane) = fields.remove("lane") {
        query = query.with_lane(AutoLane::new(string_value(&lane)?));
    }
    Ok(query)
}

fn map_fields(entries: &[(Expr, Expr)]) -> Result<BTreeMap<String, Expr>> {
    entries
        .iter()
        .map(|(key, value)| Ok((field_key(key)?, value.clone())))
        .collect()
}

fn list_fields(items: &[Expr]) -> Result<BTreeMap<String, Expr>> {
    let Some((head, tail)) = items.split_first() else {
        return Err(Error::Eval("empty auto/info request".to_owned()));
    };
    if !tail.len().is_multiple_of(2) {
        return Err(Error::Eval(
            "auto/info request arguments must be key/value pairs".to_owned(),
        ));
    }
    let mut fields = BTreeMap::new();
    fields.insert("op".to_owned(), head.clone());
    for pair in tail.chunks_exact(2) {
        fields.insert(field_key(&pair[0])?, pair[1].clone());
    }
    Ok(fields)
}

fn field_key(expr: &Expr) -> Result<String> {
    match expr {
        Expr::Symbol(symbol) => Ok(symbol.name.trim_start_matches(':').to_owned()),
        Expr::String(value) => Ok(value.trim_start_matches(':').to_owned()),
        _ => Err(Error::Eval(
            "auto/info request key must be a symbol or string".to_owned(),
        )),
    }
}

fn string_value(expr: &Expr) -> Result<String> {
    match expr {
        Expr::String(value) => Ok(value.clone()),
        Expr::Symbol(symbol) => Ok(symbol.as_qualified_str()),
        _ => Err(Error::Eval(
            "auto/info request field must be string or symbol".to_owned(),
        )),
    }
}

fn string_field(name: &str, value: &str) -> (Expr, Expr) {
    (
        Expr::Symbol(Symbol::new(name.to_owned())),
        Expr::String(value.to_owned()),
    )
}

fn symbol_field(name: &str, value: &str) -> (Expr, Expr) {
    (
        Expr::Symbol(Symbol::new(name.to_owned())),
        Expr::Symbol(Symbol::new(value.to_owned())),
    )
}

fn normalize(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}
