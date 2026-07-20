//! Vehicle lookup request expression helpers and parser.

use std::collections::BTreeMap;

use sim_kernel::{Error, Expr, Result, Symbol};

use crate::{VehicleSource, normalize_plate, normalize_vin};

/// Lookup key kind for a vehicle request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VehicleLookupKind {
    /// Lookup by plate label.
    Plate,
    /// Lookup by VIN label.
    Vin,
}

impl VehicleLookupKind {
    /// Returns the operation symbol for this lookup kind.
    pub fn operation(self) -> &'static str {
        match self {
            Self::Plate => "auto/vehicle/by-plate",
            Self::Vin => "auto/vehicle/by-vin",
        }
    }

    /// Returns the request field that carries this lookup key.
    pub fn key_field(self) -> &'static str {
        match self {
            Self::Plate => "plate",
            Self::Vin => "vin",
        }
    }
}

/// A parsed vehicle identity lookup request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VehicleLookupRequest {
    /// Source that should answer the request.
    pub source: VehicleSource,
    /// Lookup key kind.
    pub kind: VehicleLookupKind,
    /// Normalized lookup key.
    pub key: String,
    /// Market for plate lookups.
    pub market: String,
}

impl VehicleLookupRequest {
    /// Builds a lookup request by plate label.
    pub fn by_plate(source: VehicleSource, plate: &str, market: &str) -> Result<Self> {
        Ok(Self {
            source,
            kind: VehicleLookupKind::Plate,
            key: normalize_plate(plate, market)?,
            market: market.trim().to_ascii_uppercase(),
        })
    }

    /// Builds a lookup request by VIN label.
    pub fn by_vin(source: VehicleSource, vin: &str, market: &str) -> Result<Self> {
        Ok(Self {
            source,
            kind: VehicleLookupKind::Vin,
            key: normalize_vin(vin)?,
            market: market.trim().to_ascii_uppercase(),
        })
    }

    /// Parses a request expression.
    pub fn parse(expr: &Expr) -> Result<Self> {
        match expr {
            Expr::Call { operator, args } => parse_parts(operator_symbol(operator)?, args),
            Expr::List(items) | Expr::Vector(items) => {
                let Some((head, args)) = items.split_first() else {
                    return Err(Error::Eval("empty vehicle lookup request".to_owned()));
                };
                parse_parts(operator_symbol(head)?, args)
            }
            Expr::Map(entries) => parse_map_request(entries),
            _ => Err(Error::Eval(
                "vehicle lookup request must be a call, list, vector, or map".to_owned(),
            )),
        }
    }
}

/// Builds an `auto/vehicle/by-plate` expression.
pub fn vehicle_by_plate_expr(source: VehicleSource, plate: &str, market: &str) -> Expr {
    keyed_expr(
        Symbol::qualified("auto", "vehicle/by-plate"),
        &[
            ("source", source.as_str()),
            ("market", market),
            ("plate", plate),
        ],
    )
}

/// Builds an `auto/vehicle/by-vin` expression.
pub fn vehicle_by_vin_expr(source: VehicleSource, vin: &str, market: &str) -> Expr {
    keyed_expr(
        Symbol::qualified("auto", "vehicle/by-vin"),
        &[
            ("source", source.as_str()),
            ("market", market),
            ("vin", vin),
        ],
    )
}

fn parse_parts(operator: String, args: &[Expr]) -> Result<VehicleLookupRequest> {
    let fields = keyed_args(args)?;
    let source = VehicleSource::parse(
        fields
            .get("source")
            .map(String::as_str)
            .unwrap_or(VehicleSource::Modeled.as_str()),
    )?;
    let market = fields
        .get("market")
        .cloned()
        .unwrap_or_else(|| "SE".to_owned());
    match operator.as_str() {
        "vehicle/by-plate" | "auto/vehicle/by-plate" => {
            VehicleLookupRequest::by_plate(source, required_field(&fields, "plate")?, &market)
        }
        "vehicle/by-vin" | "auto/vehicle/by-vin" => {
            VehicleLookupRequest::by_vin(source, required_field(&fields, "vin")?, &market)
        }
        _ => Err(Error::Eval(format!(
            "unsupported vehicle lookup operation {operator}"
        ))),
    }
}

fn parse_map_request(entries: &[(Expr, Expr)]) -> Result<VehicleLookupRequest> {
    let mut fields = BTreeMap::new();
    for (key, value) in entries {
        fields.insert(field_key(key)?, string_value(value)?);
    }
    let operation = required_field(&fields, "op")?;
    parse_parts(operation.to_owned(), &map_to_args(&fields))
}

fn keyed_args(args: &[Expr]) -> Result<BTreeMap<String, String>> {
    if !args.len().is_multiple_of(2) {
        return Err(Error::Eval(
            "vehicle lookup arguments must be key/value pairs".to_owned(),
        ));
    }
    let mut fields = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        fields.insert(field_key(&pair[0])?, string_value(&pair[1])?);
    }
    Ok(fields)
}

fn map_to_args(fields: &BTreeMap<String, String>) -> Vec<Expr> {
    fields
        .iter()
        .filter(|(key, _)| key.as_str() != "op")
        .flat_map(|(key, value)| {
            [
                Expr::Symbol(Symbol::new(format!(":{key}"))),
                Expr::String(value.clone()),
            ]
        })
        .collect()
}

fn keyed_expr(operator: Symbol, fields: &[(&str, &str)]) -> Expr {
    let mut items = vec![Expr::Symbol(operator)];
    for (key, value) in fields {
        items.push(Expr::Symbol(Symbol::new(format!(":{key}"))));
        items.push(Expr::String((*value).to_owned()));
    }
    Expr::List(items)
}

fn operator_symbol(expr: &Expr) -> Result<String> {
    match expr {
        Expr::Symbol(symbol) => Ok(symbol.as_qualified_str()),
        _ => Err(Error::Eval(
            "vehicle lookup operator must be a symbol".to_owned(),
        )),
    }
}

fn field_key(expr: &Expr) -> Result<String> {
    match expr {
        Expr::Symbol(symbol) => Ok(symbol.name.trim_start_matches(':').to_owned()),
        Expr::String(value) => Ok(value.trim_start_matches(':').to_owned()),
        _ => Err(Error::Eval(
            "vehicle lookup field key must be a symbol or string".to_owned(),
        )),
    }
}

fn string_value(expr: &Expr) -> Result<String> {
    match expr {
        Expr::String(value) => Ok(value.clone()),
        Expr::Symbol(symbol) => Ok(symbol.as_qualified_str()),
        _ => Err(Error::Eval(
            "vehicle lookup field value must be a string or symbol".to_owned(),
        )),
    }
}

fn required_field<'a>(fields: &'a BTreeMap<String, String>, name: &'static str) -> Result<&'a str> {
    fields
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| Error::Eval(format!("missing vehicle lookup field {name}")))
}
