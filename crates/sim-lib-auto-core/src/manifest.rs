//! Automotive citizens used by core manifests and transport descriptors.

use sim_citizen_derive::Citizen;
use sim_kernel::CapabilityName;

use crate::{
    AUTO_CONTROL_EXEC, AUTO_DIAGNOSTICS_READ, AUTO_MANIFEST_READ, AUTO_SERVICE_WRITE,
    AUTO_TELEMETRY_READ, AUTO_TRANSPORT_CONNECT,
};

/// A modeled vehicle identity safe for committed fixtures and manifests.
#[derive(Clone, Debug, PartialEq, Eq, Citizen)]
#[citizen(symbol = "auto/VehicleId", version = 0)]
pub struct VehicleId {
    /// Namespace that owns the modeled key, such as a shop or fixture set.
    pub namespace: String,
    /// Synthetic key for the vehicle inside the namespace.
    pub key: String,
}

impl Default for VehicleId {
    fn default() -> Self {
        vehicle_id_example()
    }
}

impl VehicleId {
    /// Builds a modeled vehicle identity.
    pub fn new(namespace: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            key: key.into(),
        }
    }
}

/// A decoded diagnostic trouble code with a modeled description.
#[derive(Clone, Debug, PartialEq, Eq, Citizen)]
#[citizen(symbol = "auto/Dtc", version = 0)]
pub struct Dtc {
    /// Diagnostic family or subsystem.
    pub system: String,
    /// Diagnostic code text.
    pub code: String,
    /// Human-facing description for the modeled code.
    pub description: String,
}

impl Default for Dtc {
    fn default() -> Self {
        dtc_example()
    }
}

impl Dtc {
    /// Builds a diagnostic trouble code descriptor.
    pub fn new(
        system: impl Into<String>,
        code: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            system: system.into(),
            code: code.into(),
            description: description.into(),
        }
    }
}

/// Brand or workshop capability set.
#[derive(Clone, Debug, PartialEq, Eq, Citizen)]
#[citizen(symbol = "auto/BrandCaps", version = 0)]
pub struct BrandCaps {
    /// Brand, workshop, or fleet label.
    pub brand: String,
    /// Capabilities granted by this brand profile.
    pub capabilities: Vec<CapabilityName>,
}

impl Default for BrandCaps {
    fn default() -> Self {
        brand_caps_example()
    }
}

impl BrandCaps {
    /// Builds a brand capability set.
    pub fn new(brand: impl Into<String>, capabilities: Vec<CapabilityName>) -> Self {
        Self {
            brand: brand.into(),
            capabilities,
        }
    }
}

/// An open automotive lane name, such as diagnostics or telemetry.
#[derive(Clone, Debug, PartialEq, Eq, Citizen)]
#[citizen(symbol = "auto/AutoLane", version = 0)]
pub struct AutoLane {
    /// Lane name.
    pub name: String,
}

impl Default for AutoLane {
    fn default() -> Self {
        auto_lane_example()
    }
}

impl AutoLane {
    /// Builds an automotive lane descriptor.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// An open effect classification used by operation capabilities.
#[derive(Clone, Debug, PartialEq, Eq, Citizen)]
#[citizen(symbol = "auto/EffectClass", version = 0)]
pub struct EffectClass {
    /// Effect class name.
    pub name: String,
}

impl Default for EffectClass {
    fn default() -> Self {
        effect_class_example()
    }
}

impl EffectClass {
    /// Builds an automotive effect class.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Capability required to run one automotive operation.
#[derive(Clone, Debug, PartialEq, Eq, Citizen)]
#[citizen(symbol = "auto/OpCap", version = 0)]
pub struct OpCap {
    /// Operation symbol text.
    pub operation: String,
    /// Capability required by the operation.
    pub capability: CapabilityName,
    /// Effect class applied by the operation.
    pub effect_class: String,
}

impl Default for OpCap {
    fn default() -> Self {
        op_cap_example()
    }
}

impl OpCap {
    /// Builds an operation capability descriptor.
    pub fn new(
        operation: impl Into<String>,
        capability: CapabilityName,
        effect_class: impl Into<String>,
    ) -> Self {
        Self {
            operation: operation.into(),
            capability,
            effect_class: effect_class.into(),
        }
    }
}

/// Transport endpoint descriptor for an automotive site.
#[derive(Clone, Debug, PartialEq, Eq, Citizen)]
#[citizen(symbol = "auto/TransportSpec", version = 0)]
pub struct TransportSpec {
    /// Transport name.
    pub name: String,
    /// Protocol or codec family.
    pub protocol: String,
    /// Lane this transport serves.
    pub lane: String,
    /// Capability required to read through the transport.
    pub read_capability: CapabilityName,
    /// Capability required to write through the transport.
    pub write_capability: CapabilityName,
}

impl Default for TransportSpec {
    fn default() -> Self {
        transport_spec_example()
    }
}

impl TransportSpec {
    /// Builds an automotive transport descriptor.
    pub fn new(
        name: impl Into<String>,
        protocol: impl Into<String>,
        lane: impl Into<String>,
        read_capability: CapabilityName,
        write_capability: CapabilityName,
    ) -> Self {
        Self {
            name: name.into(),
            protocol: protocol.into(),
            lane: lane.into(),
            read_capability,
            write_capability,
        }
    }
}

/// Site-level automotive manifest.
#[derive(Clone, Debug, PartialEq, Eq, Citizen)]
#[citizen(symbol = "auto/SiteManifest", version = 0)]
pub struct SiteManifest {
    /// Site label.
    pub site: String,
    /// Modeled vehicle key the site describes.
    pub vehicle: String,
    /// Brand or workshop label.
    pub brand: String,
    /// Lane names exposed by the site.
    pub lanes: Vec<String>,
    /// Transport names exposed by the site.
    pub transports: Vec<String>,
    /// Operation names exposed by the site.
    pub operations: Vec<String>,
}

impl Default for SiteManifest {
    fn default() -> Self {
        site_manifest_example()
    }
}

impl SiteManifest {
    /// Builds an automotive site manifest.
    pub fn new(
        site: impl Into<String>,
        vehicle: impl Into<String>,
        brand: impl Into<String>,
        lanes: Vec<String>,
        transports: Vec<String>,
        operations: Vec<String>,
    ) -> Self {
        Self {
            site: site.into(),
            vehicle: vehicle.into(),
            brand: brand.into(),
            lanes,
            transports,
            operations,
        }
    }
}

/// Standard diagnostics lane.
pub fn diagnostic_lane() -> AutoLane {
    auto_lane("diagnostics")
}

/// Standard telemetry lane.
pub fn telemetry_lane() -> AutoLane {
    auto_lane("telemetry")
}

/// Standard manifest lane.
pub fn manifest_lane() -> AutoLane {
    auto_lane("manifest")
}

/// Builds an open automotive lane.
pub fn auto_lane(name: impl Into<String>) -> AutoLane {
    AutoLane::new(name)
}

/// Standard diagnostic effect class.
pub fn diagnostic_effect() -> EffectClass {
    EffectClass::new("diagnostic-read")
}

/// Standard control effect class.
pub fn control_effect() -> EffectClass {
    EffectClass::new("control-write")
}

fn vehicle_id_example() -> VehicleId {
    VehicleId::new("fixture", "vehicle-alpha")
}

fn dtc_example() -> Dtc {
    Dtc::new("body", "B0000", "modeled diagnostic")
}

fn brand_caps_example() -> BrandCaps {
    BrandCaps::new(
        "fixture-brand",
        vec![
            CapabilityName::new(AUTO_DIAGNOSTICS_READ),
            CapabilityName::new(AUTO_TELEMETRY_READ),
        ],
    )
}

fn auto_lane_example() -> AutoLane {
    diagnostic_lane()
}

fn effect_class_example() -> EffectClass {
    diagnostic_effect()
}

fn op_cap_example() -> OpCap {
    OpCap::new(
        "diagnostics/read-dtc",
        CapabilityName::new(AUTO_DIAGNOSTICS_READ),
        "diagnostic-read",
    )
}

fn transport_spec_example() -> TransportSpec {
    TransportSpec::new(
        "fixture-transport",
        "modeled-bus",
        "diagnostics",
        CapabilityName::new(AUTO_TRANSPORT_CONNECT),
        CapabilityName::new(AUTO_SERVICE_WRITE),
    )
}

fn site_manifest_example() -> SiteManifest {
    SiteManifest::new(
        "fixture-site",
        "vehicle-alpha",
        "fixture-brand",
        vec!["diagnostics".to_owned(), "telemetry".to_owned()],
        vec!["fixture-transport".to_owned()],
        vec!["diagnostics/read-dtc".to_owned()],
    )
}

#[allow(dead_code)]
fn capability_examples() -> [CapabilityName; 4] {
    [
        CapabilityName::new(AUTO_CONTROL_EXEC),
        CapabilityName::new(AUTO_MANIFEST_READ),
        CapabilityName::new(AUTO_SERVICE_WRITE),
        CapabilityName::new(AUTO_TELEMETRY_READ),
    ]
}
