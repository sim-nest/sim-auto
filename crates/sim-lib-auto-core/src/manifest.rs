//! Automotive citizens used by core manifests and transport descriptors.

use sim_citizen_derive::Citizen;
use sim_kernel::CapabilityName;

use crate::{
    AUTO_CONTROL_EXEC, AUTO_DIAGNOSTICS_READ, AUTO_MANIFEST_READ, AUTO_ORDER, AUTO_SERVICE_WRITE,
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
    /// Standardized diagnostic status bits supplied by the transport.
    #[citizen(with = "dtc_status_field")]
    pub status: DtcStatus,
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
        Self::with_status(system, code, description, DtcStatus::default())
    }

    /// Builds a diagnostic trouble code descriptor with explicit status bits.
    pub fn with_status(
        system: impl Into<String>,
        code: impl Into<String>,
        description: impl Into<String>,
        status: DtcStatus,
    ) -> Self {
        Self {
            system: system.into(),
            code: code.into(),
            description: description.into(),
            status,
        }
    }
}

/// Standard UDS diagnostic status bits for a trouble code.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Citizen)]
#[citizen(symbol = "auto/DtcStatus", version = 0)]
pub struct DtcStatus {
    /// The DTC test is failed now.
    pub test_failed: bool,
    /// The DTC test failed during the current operation cycle.
    pub test_failed_this_operation_cycle: bool,
    /// The DTC is pending confirmation.
    pub pending: bool,
    /// The DTC is confirmed.
    pub confirmed: bool,
    /// The DTC has not completed since the last clear operation.
    pub test_not_completed_since_clear: bool,
    /// The DTC failed at least once since the last clear operation.
    pub test_failed_since_clear: bool,
    /// The DTC test has not completed in the current operation cycle.
    pub test_not_completed_this_operation_cycle: bool,
    /// The warning indicator is requested.
    pub warning_indicator: bool,
}

impl DtcStatus {
    /// Decodes a UDS DTC status byte.
    pub fn from_byte(byte: u8) -> Self {
        Self {
            test_failed: byte & 0x01 != 0,
            test_failed_this_operation_cycle: byte & 0x02 != 0,
            pending: byte & 0x04 != 0,
            confirmed: byte & 0x08 != 0,
            test_not_completed_since_clear: byte & 0x10 != 0,
            test_failed_since_clear: byte & 0x20 != 0,
            test_not_completed_this_operation_cycle: byte & 0x40 != 0,
            warning_indicator: byte & 0x80 != 0,
        }
    }

    /// Encodes the status bits back to a UDS status byte.
    pub fn to_byte(self) -> u8 {
        u8::from(self.test_failed)
            | (u8::from(self.test_failed_this_operation_cycle) << 1)
            | (u8::from(self.pending) << 2)
            | (u8::from(self.confirmed) << 3)
            | (u8::from(self.test_not_completed_since_clear) << 4)
            | (u8::from(self.test_failed_since_clear) << 5)
            | (u8::from(self.test_not_completed_this_operation_cycle) << 6)
            | (u8::from(self.warning_indicator) << 7)
    }
}

mod dtc_status_field {
    use sim_kernel::{Error, Expr, Result, Symbol};

    use super::DtcStatus;

    pub fn encode(status: &DtcStatus) -> Expr {
        Expr::Map(vec![
            field("test_failed", status.test_failed),
            field(
                "test_failed_this_operation_cycle",
                status.test_failed_this_operation_cycle,
            ),
            field("pending", status.pending),
            field("confirmed", status.confirmed),
            field(
                "test_not_completed_since_clear",
                status.test_not_completed_since_clear,
            ),
            field("test_failed_since_clear", status.test_failed_since_clear),
            field(
                "test_not_completed_this_operation_cycle",
                status.test_not_completed_this_operation_cycle,
            ),
            field("warning_indicator", status.warning_indicator),
        ])
    }

    pub fn decode(expr: &Expr) -> Result<DtcStatus> {
        let Expr::Map(entries) = expr else {
            return Err(Error::Eval(
                "DTC status citizen field must be a map".to_owned(),
            ));
        };
        Ok(DtcStatus {
            test_failed: bool_field(entries, "test_failed")?,
            test_failed_this_operation_cycle: bool_field(
                entries,
                "test_failed_this_operation_cycle",
            )?,
            pending: bool_field(entries, "pending")?,
            confirmed: bool_field(entries, "confirmed")?,
            test_not_completed_since_clear: bool_field(entries, "test_not_completed_since_clear")?,
            test_failed_since_clear: bool_field(entries, "test_failed_since_clear")?,
            test_not_completed_this_operation_cycle: bool_field(
                entries,
                "test_not_completed_this_operation_cycle",
            )?,
            warning_indicator: bool_field(entries, "warning_indicator")?,
        })
    }

    fn field(name: &str, value: bool) -> (Expr, Expr) {
        (Expr::Symbol(Symbol::new(name)), Expr::Bool(value))
    }

    fn bool_field(entries: &[(Expr, Expr)], name: &'static str) -> Result<bool> {
        entries
            .iter()
            .find_map(|(key, value)| {
                if key == &Expr::Symbol(Symbol::new(name)) {
                    match value {
                        Expr::Bool(value) => Some(Ok(*value)),
                        _ => Some(Err(Error::Eval(format!(
                            "DTC status field {name} must be bool"
                        )))),
                    }
                } else {
                    None
                }
            })
            .unwrap_or_else(|| Err(Error::Eval(format!("missing DTC status field {name}"))))
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
    Dtc::with_status(
        "body",
        "B0000",
        "modeled diagnostic",
        DtcStatus::from_byte(0x08),
    )
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
fn capability_examples() -> [CapabilityName; 5] {
    [
        CapabilityName::new(AUTO_CONTROL_EXEC),
        CapabilityName::new(AUTO_MANIFEST_READ),
        CapabilityName::new(AUTO_SERVICE_WRITE),
        CapabilityName::new(AUTO_ORDER),
        CapabilityName::new(AUTO_TELEMETRY_READ),
    ]
}
