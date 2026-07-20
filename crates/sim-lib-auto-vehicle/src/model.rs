//! Modeled vehicle identity records and normalization helpers.

use sim_kernel::{Error, Expr, Result, Symbol};
use sim_lib_auto_core::{VehicleId, vehicle_read_construct};

/// Lookup source for a vehicle identity request.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VehicleSource {
    /// Public modeled data bundled with this crate.
    #[default]
    Modeled,
    /// Host-owned HaynesPro bridge contract.
    HaynesPro,
    /// Host-owned biluppgifter.se bridge contract.
    BiluppgifterSe,
}

impl VehicleSource {
    /// Parses a source label accepted by request expressions.
    pub fn parse(value: &str) -> Result<Self> {
        match normalize_source(value).as_str() {
            "modeled" | "fixture" => Ok(Self::Modeled),
            "haynespro" | "haynes-pro" => Ok(Self::HaynesPro),
            "biluppgifter.se" | "biluppgifter-se" | "biluppgifter" => Ok(Self::BiluppgifterSe),
            other => Err(Error::Eval(format!(
                "unknown vehicle lookup source {other}"
            ))),
        }
    }

    /// Stable source label used in responses and contracts.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Modeled => "modeled",
            Self::HaynesPro => "haynespro",
            Self::BiluppgifterSe => "biluppgifter.se",
        }
    }

    /// Returns whether this source is a live host-owned bridge contract.
    pub fn is_live(self) -> bool {
        !matches!(self, Self::Modeled)
    }
}

/// A resolved vehicle identity record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VehicleRecord {
    /// Shared vehicle identity.
    pub vehicle: VehicleId,
    /// Source that produced the identity.
    pub source: VehicleSource,
    /// Market for the identity labels.
    pub market: String,
    /// Normalized plate label, when present.
    pub plate: Option<String>,
    /// Normalized VIN label, when present.
    pub vin: Option<String>,
    /// Synthetic make label.
    pub make: String,
    /// Synthetic model label.
    pub model: String,
    /// Synthetic model-year label.
    pub model_year: String,
}

impl VehicleRecord {
    /// Builds a vehicle record without plate or VIN labels.
    pub fn new(
        vehicle: VehicleId,
        source: VehicleSource,
        market: impl Into<String>,
        make: impl Into<String>,
        model: impl Into<String>,
        model_year: impl Into<String>,
    ) -> Self {
        Self {
            vehicle,
            source,
            market: market.into(),
            plate: None,
            vin: None,
            make: make.into(),
            model: model.into(),
            model_year: model_year.into(),
        }
    }

    /// Attaches a normalized plate label.
    pub fn with_plate(mut self, plate: &str) -> Result<Self> {
        self.plate = Some(normalize_plate(plate, &self.market)?);
        Ok(self)
    }

    /// Attaches a normalized VIN label.
    pub fn with_vin(mut self, vin: &str) -> Result<Self> {
        self.vin = Some(normalize_vin(vin)?);
        Ok(self)
    }

    /// Encodes the record as a SIM expression.
    pub fn to_expr(&self) -> Expr {
        let mut fields = vec![
            (
                Expr::Symbol(Symbol::new("vehicle")),
                vehicle_read_construct(&self.vehicle),
            ),
            string_field("source", self.source.as_str()),
            string_field("market", &self.market),
            string_field("make", &self.make),
            string_field("model", &self.model),
            string_field("model-year", &self.model_year),
        ];
        if let Some(plate) = &self.plate {
            fields.push(string_field("plate", plate));
        }
        if let Some(vin) = &self.vin {
            fields.push(string_field("vin", vin));
        }
        Expr::Map(fields)
    }
}

/// Normalizes a Swedish plate label for modeled lookup.
///
/// The modeled source accepts synthetic labels that resemble Swedish workshop
/// input without requiring real registration-number shape in committed data.
pub fn normalize_plate(plate: &str, market: &str) -> Result<String> {
    if !market.trim().eq_ignore_ascii_case("SE") {
        return Err(Error::Eval(format!(
            "vehicle plate lookup only supports market SE, got {market}"
        )));
    }
    normalize_identifier(plate, "plate", 5, 8)
}

/// Normalizes a VIN label for lookup.
pub fn normalize_vin(vin: &str) -> Result<String> {
    normalize_identifier(vin, "vin", 8, 20)
}

pub(crate) fn string_field(name: &str, value: &str) -> (Expr, Expr) {
    (
        Expr::Symbol(Symbol::new(name.to_owned())),
        Expr::String(value.to_owned()),
    )
}

fn normalize_source(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace('_', "-")
}

fn normalize_identifier(
    value: &str,
    label: &'static str,
    min: usize,
    max: usize,
) -> Result<String> {
    let mut normalized = String::new();
    for ch in value.chars() {
        if ch.is_ascii_whitespace() || ch == '-' {
            continue;
        }
        if !ch.is_ascii_alphanumeric() {
            return Err(Error::Eval(format!(
                "vehicle {label} contains unsupported character {ch:?}"
            )));
        }
        normalized.push(ch.to_ascii_uppercase());
    }
    if normalized.len() < min || normalized.len() > max {
        return Err(Error::Eval(format!(
            "vehicle {label} must normalize to {min}..={max} ASCII alphanumeric characters"
        )));
    }
    Ok(normalized)
}
