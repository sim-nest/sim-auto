//! Automotive capability names exported by the core library.

use sim_kernel::CapabilityName;

/// Read access to modeled diagnostic values and decoded trouble codes.
pub const AUTO_DIAGNOSTICS_READ: &str = "auto/diagnostics/read";
/// Read access to modeled vehicle telemetry.
pub const AUTO_TELEMETRY_READ: &str = "auto/telemetry/read";
/// Execute access for vehicle-control effects.
pub const AUTO_CONTROL_EXEC: &str = "auto/control/exec";
/// Write access for shop-side service records and procedures.
pub const AUTO_SERVICE_WRITE: &str = "auto/service/write";
/// Read access to automotive site manifests.
pub const AUTO_MANIFEST_READ: &str = "auto/manifest/read";
/// Connect access for automotive transport endpoints.
pub const AUTO_TRANSPORT_CONNECT: &str = "auto/transport/connect";

const AUTO_CAPABILITIES: &[&str] = &[
    AUTO_DIAGNOSTICS_READ,
    AUTO_TELEMETRY_READ,
    AUTO_CONTROL_EXEC,
    AUTO_SERVICE_WRITE,
    AUTO_MANIFEST_READ,
    AUTO_TRANSPORT_CONNECT,
];

/// Returns the stable automotive capability strings.
pub fn auto_capability_texts() -> &'static [&'static str] {
    AUTO_CAPABILITIES
}

/// Returns the stable automotive capability names.
pub fn auto_capability_names() -> Vec<CapabilityName> {
    AUTO_CAPABILITIES
        .iter()
        .map(|capability| CapabilityName::new(*capability))
        .collect()
}
