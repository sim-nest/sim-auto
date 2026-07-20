//! Public modeled OEM vendor site manifests and cassettes.

use sim_kernel::{CapabilityName, Expr, Symbol};
use sim_lib_auto_core::{
    AUTO_DIAGNOSTICS_READ, AUTO_FLASH, AUTO_ORDER, AUTO_SERVICE_WRITE, OpCap, SiteManifest,
    TransportSpec,
};

use crate::ModeledVendorCassette;

const PURE: &str = "pure";
const REVERSIBLE: &str = "reversible";
const IRREVERSIBLE: &str = "irreversible";

/// Returns the modeled OEM diagnostic site manifests.
pub fn oem_site_manifests() -> Vec<SiteManifest> {
    vec![
        xentry_manifest(),
        ista_manifest(),
        odis_manifest(),
        vida_manifest(),
        esitronic_manifest(),
    ]
}

/// Returns synthetic modeled replies for the OEM diagnostic sites.
pub fn oem_site_cassettes() -> Vec<ModeledVendorCassette> {
    vec![
        cassette("xentry", "read/dtc", "mercedes modeled DTC read"),
        cassette(
            "xentry",
            "info/wis-procedure",
            "mercedes modeled WIS repair information",
        ),
        cassette(
            "xentry",
            "parts/epc-lookup",
            "mercedes modeled EPC parts lookup",
        ),
        cassette("xentry", "code/sca", "mercedes modeled service coding"),
        cassette("ista", "read/dtc", "bmw modeled DTC read"),
        cassette("ista", "info/test-plan", "bmw modeled ISTA test plan"),
        cassette(
            "ista",
            "code/coding",
            "bmw coding denied without service write",
        ),
        cassette(
            "odis",
            "service/guided-function",
            "vag modeled guided function",
        ),
        cassette("vida", "read/dtc", "volvo modeled DTC read"),
        cassette("vida", "info/procedure", "volvo modeled repair information"),
        cassette(
            "esitronic",
            "read/dtc",
            "bosch modeled multi-brand DTC read",
        ),
        cassette(
            "esitronic",
            "info/procedure",
            "bosch modeled repair information",
        ),
    ]
}

/// Returns the modeled public data and supplier site manifests.
pub fn supplier_site_manifests() -> Vec<SiteManifest> {
    vec![
        haynespro_manifest(),
        biluppgifter_se_manifest(),
        mekonomen_pro_manifest(),
    ]
}

/// Returns synthetic modeled replies for public data and supplier sites.
pub fn supplier_site_cassettes() -> Vec<ModeledVendorCassette> {
    vec![
        cassette(
            "haynespro",
            "read/identity",
            "haynespro modeled vehicle identity",
        ),
        cassette(
            "haynespro",
            "info/procedure",
            "haynespro modeled repair information",
        ),
        cassette(
            "biluppgifter-se",
            "read/plate-lookup",
            "biluppgifter.se modeled Swedish plate lookup",
        ),
        cassette(
            "mekonomen-pro",
            "parts/catalog-lookup",
            "mekonomen pro modeled parts catalog",
        ),
        cassette(
            "mekonomen-pro",
            "order/status",
            "mekonomen pro modeled order status",
        ),
        cassette(
            "mekonomen-pro",
            "order/place",
            "mekonomen pro modeled order accepted",
        ),
    ]
}

/// Returns the modeled ECU flash site manifests.
pub fn flash_site_manifests() -> Vec<SiteManifest> {
    vec![autotuner_manifest()]
}

/// Returns synthetic modeled replies for ECU flash sites.
pub fn flash_site_cassettes() -> Vec<ModeledVendorCassette> {
    vec![
        cassette("autotuner", "flash/read-ecu", "autotuner modeled ECU read"),
        cassette(
            "autotuner",
            "flash/backup-stock",
            "autotuner modeled stock backup",
        ),
        cassette(
            "autotuner",
            "flash/write",
            "autotuner modeled flash accepted",
        ),
        cassette(
            "autotuner",
            "flash/restore-stock",
            "autotuner modeled stock restore",
        ),
    ]
}

/// Autotuner modeled manifest for ECU stock backup, flash, and restore.
pub fn autotuner_manifest() -> SiteManifest {
    manifest(
        "autotuner",
        "vehicle-flash-fixture",
        "autotuner",
        &["*"],
        &["read", "flash"],
        &[
            ("flash/read-ecu", AUTO_FLASH, PURE),
            ("flash/backup-stock", AUTO_FLASH, REVERSIBLE),
            ("flash/restore-stock", AUTO_FLASH, REVERSIBLE),
            ("flash/write", AUTO_FLASH, IRREVERSIBLE),
        ],
    )
}

/// HaynesPro modeled manifest for vehicle data and repair information.
pub fn haynespro_manifest() -> SiteManifest {
    manifest(
        "haynespro",
        "vehicle-multibrand",
        "haynespro",
        &["*"],
        &["read", "info"],
        &[
            ("read/identity", AUTO_DIAGNOSTICS_READ, PURE),
            ("info/procedure", AUTO_DIAGNOSTICS_READ, PURE),
        ],
    )
}

/// biluppgifter.se modeled manifest for Swedish registration lookup.
pub fn biluppgifter_se_manifest() -> SiteManifest {
    manifest(
        "biluppgifter-se",
        "vehicle-se-registration",
        "biluppgifter.se",
        &["*"],
        &["read", "identity"],
        &[("read/plate-lookup", AUTO_DIAGNOSTICS_READ, PURE)],
    )
}

/// Mekonomen Pro modeled manifest for parts lookup and supplier ordering.
pub fn mekonomen_pro_manifest() -> SiteManifest {
    manifest(
        "mekonomen-pro",
        "vehicle-multibrand",
        "mekonomen-pro",
        &["*"],
        &["parts", "order"],
        &[
            ("parts/catalog-lookup", AUTO_DIAGNOSTICS_READ, PURE),
            ("order/status", AUTO_ORDER, PURE),
            ("order/place", AUTO_ORDER, REVERSIBLE),
        ],
    )
}

/// Mercedes-Benz XENTRY, WIS, and EPC modeled manifest.
pub fn xentry_manifest() -> SiteManifest {
    manifest(
        "xentry",
        "vehicle-mercedes",
        "mercedes-benz",
        &["mercedes-benz", "smart"],
        &["read", "info", "parts", "service"],
        &[
            ("read/dtc", AUTO_DIAGNOSTICS_READ, PURE),
            ("info/wis-procedure", AUTO_DIAGNOSTICS_READ, PURE),
            ("parts/epc-lookup", AUTO_DIAGNOSTICS_READ, PURE),
            ("code/sca", AUTO_SERVICE_WRITE, REVERSIBLE),
        ],
    )
}

/// BMW ISTA modeled manifest.
pub fn ista_manifest() -> SiteManifest {
    manifest(
        "ista",
        "vehicle-bmw",
        "bmw",
        &["bmw", "mini", "rolls-royce"],
        &["read", "info", "service"],
        &[
            ("read/dtc", AUTO_DIAGNOSTICS_READ, PURE),
            ("info/test-plan", AUTO_DIAGNOSTICS_READ, PURE),
            ("code/coding", AUTO_SERVICE_WRITE, REVERSIBLE),
        ],
    )
}

/// VAG ODIS modeled manifest.
pub fn odis_manifest() -> SiteManifest {
    manifest(
        "odis",
        "vehicle-vag",
        "vag",
        &["volkswagen", "audi", "seat", "skoda", "cupra"],
        &["read", "service"],
        &[
            ("read/dtc", AUTO_DIAGNOSTICS_READ, PURE),
            ("service/guided-function", AUTO_SERVICE_WRITE, REVERSIBLE),
        ],
    )
}

/// Volvo VIDA modeled manifest.
pub fn vida_manifest() -> SiteManifest {
    manifest(
        "vida",
        "vehicle-volvo",
        "volvo",
        &["volvo"],
        &["read", "info"],
        &[
            ("read/dtc", AUTO_DIAGNOSTICS_READ, PURE),
            ("info/procedure", AUTO_DIAGNOSTICS_READ, PURE),
        ],
    )
}

/// Bosch ESI\[tronic\] modeled multi-brand manifest.
pub fn esitronic_manifest() -> SiteManifest {
    manifest(
        "esitronic",
        "vehicle-multibrand",
        "bosch-esitronic",
        &["*"],
        &["read", "info"],
        &[
            ("read/dtc", AUTO_DIAGNOSTICS_READ, PURE),
            ("info/procedure", AUTO_DIAGNOSTICS_READ, PURE),
        ],
    )
}

fn manifest(
    site: &str,
    vehicle: &str,
    brand: &str,
    makes: &[&str],
    lanes: &[&str],
    operations: &[(&str, &str, &str)],
) -> SiteManifest {
    let op_caps = operations
        .iter()
        .map(|(operation, capability, effect)| op_cap(operation, capability, effect))
        .collect::<Vec<_>>();
    let ceiling = op_caps
        .iter()
        .map(|op_cap| op_cap.capability.clone())
        .fold(Vec::<CapabilityName>::new(), push_unique_capability);
    SiteManifest::new(
        site,
        vehicle,
        brand,
        lanes.iter().map(|lane| (*lane).to_owned()).collect(),
        vec![
            TransportSpec::new(
                format!("{site}-modeled"),
                "modeled",
                "read",
                CapabilityName::new(AUTO_DIAGNOSTICS_READ),
                CapabilityName::new(AUTO_SERVICE_WRITE),
            )
            .name,
            "cassette".to_owned(),
        ],
        operations
            .iter()
            .map(|(operation, _, _)| (*operation).to_owned())
            .collect(),
    )
    .with_makes(makes.iter().map(|make| (*make).to_owned()).collect())
    .with_op_caps(op_caps)
    .with_ceiling(ceiling)
}

fn op_cap(operation: &str, capability: &str, effect: &str) -> OpCap {
    OpCap::new(operation, CapabilityName::new(capability), effect)
}

fn push_unique_capability(
    mut capabilities: Vec<CapabilityName>,
    capability: CapabilityName,
) -> Vec<CapabilityName> {
    if !capabilities.iter().any(|item| item == &capability) {
        capabilities.push(capability);
    }
    capabilities
}

fn cassette(site: &str, operation: &str, label: &str) -> ModeledVendorCassette {
    ModeledVendorCassette::new(
        site,
        operation,
        Expr::Map(vec![
            string_field("site", site),
            string_field("operation", operation),
            string_field("cassette", label),
            (Expr::Symbol(Symbol::new("accepted")), Expr::Bool(true)),
        ]),
    )
}

fn string_field(name: &str, value: &str) -> (Expr, Expr) {
    (
        Expr::Symbol(Symbol::new(name.to_owned())),
        Expr::String(value.to_owned()),
    )
}
