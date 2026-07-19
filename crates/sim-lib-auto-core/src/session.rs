//! Automotive session descriptors shared by diagnostic fabric libraries.

use sim_kernel::{CapabilityName, CapabilitySet, diminish};

use crate::{BrandCaps, VehicleId};

/// Placement selected for one automotive diagnostic session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransportPlacement {
    /// Synthetic in-process ECU data.
    Modeled,
    /// Replayable diagnostic cassette identified by a stable label.
    Cassette {
        /// Cassette label selected by the caller.
        id: String,
    },
    /// Host-provided bridge selected by name.
    LocalBridge {
        /// Bridge label configured by the host.
        name: String,
    },
}

impl TransportPlacement {
    /// Builds a modeled placement.
    pub fn modeled() -> Self {
        Self::Modeled
    }

    /// Builds a cassette-backed placement.
    pub fn cassette(id: impl Into<String>) -> Self {
        Self::Cassette { id: id.into() }
    }

    /// Builds a local bridge placement.
    pub fn local_bridge(name: impl Into<String>) -> Self {
        Self::LocalBridge { name: name.into() }
    }

    /// Returns the placement family name.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Modeled => "modeled",
            Self::Cassette { .. } => "cassette",
            Self::LocalBridge { .. } => "local-bridge",
        }
    }
}

/// Session state carried by automotive diagnostic fabrics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutoSession {
    /// Modeled vehicle identity.
    pub vehicle: VehicleId,
    /// Brand or workshop capability profile.
    pub brand: BrandCaps,
    /// Placement that answers diagnostic requests.
    pub transport: TransportPlacement,
    /// Capabilities granted to this session before caller-side narrowing.
    pub grants: Vec<CapabilityName>,
}

impl AutoSession {
    /// Builds a session with explicit placement and grants.
    pub fn new(
        vehicle: VehicleId,
        brand: BrandCaps,
        transport: TransportPlacement,
        grants: Vec<CapabilityName>,
    ) -> Self {
        Self {
            vehicle,
            brand,
            transport,
            grants,
        }
    }

    /// Builds a modeled session for a synthetic vehicle.
    pub fn modeled(vehicle: VehicleId, brand: BrandCaps, grants: Vec<CapabilityName>) -> Self {
        Self::new(vehicle, brand, TransportPlacement::Modeled, grants)
    }

    /// Returns a copy of this session with a different placement.
    pub fn with_transport(mut self, transport: TransportPlacement) -> Self {
        self.transport = transport;
        self
    }

    /// Returns whether the session holds `capability` before caller narrowing.
    pub fn has_grant(&self, capability: &CapabilityName) -> bool {
        self.grants.iter().any(|grant| grant == capability)
    }

    /// Intersects the session grants with caller-allowed capabilities.
    pub fn diminished_grants(&self, allowed: &[CapabilityName]) -> CapabilitySet {
        let current = capability_set(self.grants.iter().cloned());
        let allowed = capability_set(allowed.iter().cloned());
        diminish(&current, &allowed)
    }
}

fn capability_set(capabilities: impl IntoIterator<Item = CapabilityName>) -> CapabilitySet {
    capabilities
        .into_iter()
        .fold(CapabilitySet::new(), CapabilitySet::grant)
}
