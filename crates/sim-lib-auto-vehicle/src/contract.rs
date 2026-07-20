//! Contract catalog for host-owned vehicle identity sources.

use sim_kernel::{Expr, Symbol};

use crate::{VehicleSource, model::string_field};

/// Canonical capability for outbound HTTP bridge access.
pub const NET_HTTP_CAPABILITY: &str = "net/http";

/// A public bridge contract for a host-owned vehicle identity source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VehicleBridgeContract {
    /// Lookup source.
    pub source: VehicleSource,
    /// Human-facing provider label.
    pub provider: String,
    /// Operations supplied by the provider bridge.
    pub operations: Vec<String>,
    /// Capability required before live lookup dispatch.
    pub required_capability: String,
    /// Host boundary that owns endpoints and credentials.
    pub host_boundary: String,
}

impl VehicleBridgeContract {
    /// Builds a vehicle bridge contract.
    pub fn new(
        source: VehicleSource,
        provider: impl Into<String>,
        operations: Vec<String>,
        host_boundary: impl Into<String>,
    ) -> Self {
        Self {
            source,
            provider: provider.into(),
            operations,
            required_capability: NET_HTTP_CAPABILITY.to_owned(),
            host_boundary: host_boundary.into(),
        }
    }

    /// Encodes the contract as a SIM expression.
    pub fn to_expr(&self) -> Expr {
        Expr::Map(vec![
            string_field("source", self.source.as_str()),
            string_field("provider", &self.provider),
            (
                Expr::Symbol(Symbol::new("operations")),
                Expr::List(
                    self.operations
                        .iter()
                        .map(|operation| Expr::String(operation.clone()))
                        .collect(),
                ),
            ),
            string_field("required-capability", &self.required_capability),
            string_field("host-boundary", &self.host_boundary),
        ])
    }
}

/// Returns the live bridge contracts advertised by this crate.
pub fn vehicle_bridge_contracts() -> Vec<VehicleBridgeContract> {
    vec![
        VehicleBridgeContract::new(
            VehicleSource::HaynesPro,
            "HaynesPro",
            lookup_operations(),
            "host-owned HttpDir under net/http; no endpoint, key, plate, VIN, or owner data is committed",
        ),
        VehicleBridgeContract::new(
            VehicleSource::BiluppgifterSe,
            "biluppgifter.se",
            lookup_operations(),
            "host-owned HttpDir under net/http; no endpoint, key, plate, VIN, or owner data is committed",
        ),
    ]
}

fn lookup_operations() -> Vec<String> {
    vec!["vehicle/by-plate".to_owned(), "vehicle/by-vin".to_owned()]
}
