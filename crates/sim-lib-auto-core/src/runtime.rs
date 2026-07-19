//! Loadable library wiring for the automotive core exports.

use sim_citizen::CitizenInfo;
use sim_kernel::{
    AbiVersion, Cx, Export, Lib, LibManifest, LibTarget, Linker, LoadCx, Result, Symbol, Version,
};

use crate::auto_capability_texts;

const AUTO_CITIZENS: &[&str] = &[
    "auto/VehicleId",
    "auto/Dtc",
    "auto/DtcStatus",
    "auto/BrandCaps",
    "auto/AutoLane",
    "auto/EffectClass",
    "auto/OpCap",
    "auto/TransportSpec",
    "auto/SiteManifest",
];

const AUTO_LANES: &[&str] = &["diagnostics", "telemetry", "manifest", "service"];

/// Loadable library that contributes SIM automotive core citizens and values.
#[derive(Clone, Copy, Debug, Default)]
pub struct AutoCoreLib;

impl Lib for AutoCoreLib {
    fn manifest(&self) -> LibManifest {
        let mut exports = auto_citizen_symbols()
            .into_iter()
            .map(|symbol| Export::Class {
                symbol,
                class_id: None,
            })
            .collect::<Vec<_>>();
        exports.extend([
            Export::Value {
                symbol: auto_caps_symbol(),
            },
            Export::Value {
                symbol: auto_lanes_symbol(),
            },
            Export::Value {
                symbol: manifest_shape_symbol(),
            },
        ]);
        LibManifest {
            id: Symbol::qualified("auto", "core"),
            version: Version(env!("CARGO_PKG_VERSION").to_owned()),
            abi: AbiVersion { major: 0, minor: 1 },
            target: LibTarget::HostRegistered,
            requires: Vec::new(),
            capabilities: Vec::new(),
            exports,
        }
    }

    fn load(&self, cx: &mut LoadCx, linker: &mut Linker<'_>) -> Result<()> {
        sim_citizen::install_namespace(linker, "auto")?;
        linker.value(
            auto_caps_symbol(),
            string_list(cx, auto_capability_texts())?,
        )?;
        linker.value(auto_lanes_symbol(), string_list(cx, AUTO_LANES)?)?;
        linker.value(
            manifest_shape_symbol(),
            cx.factory().string(
                "auto/SiteManifest v0 site vehicle brand lanes transports operations".to_owned(),
            )?,
        )?;
        Ok(())
    }
}

/// Installs the automotive core library into a context once.
pub fn install_auto_core_lib(cx: &mut Cx) -> Result<()> {
    if cx
        .registry()
        .lib(&Symbol::qualified("auto", "core"))
        .is_some()
    {
        return Ok(());
    }
    cx.load_lib(&AutoCoreLib).map(|_| ())
}

/// Returns the inventory rows for automotive core citizens.
pub fn auto_citizen_registry() -> Vec<&'static CitizenInfo> {
    sim_citizen::registered_citizens()
        .filter(|info| AUTO_CITIZENS.contains(&info.symbol))
        .collect()
}

/// Returns the automotive citizen symbols exported by the core library.
pub fn auto_citizen_symbols() -> Vec<Symbol> {
    AUTO_CITIZENS
        .iter()
        .map(|symbol| parse_symbol(symbol))
        .collect()
}

/// Symbol for the exported automotive capability list.
pub fn auto_caps_symbol() -> Symbol {
    Symbol::qualified("auto", "caps")
}

/// Symbol for the exported automotive lane list.
pub fn auto_lanes_symbol() -> Symbol {
    Symbol::qualified("auto", "lanes")
}

/// Symbol for the exported automotive manifest shape descriptor.
pub fn manifest_shape_symbol() -> Symbol {
    Symbol::qualified("auto", "manifest-shape")
}

fn string_list(cx: &mut LoadCx, items: &[&str]) -> Result<sim_kernel::Value> {
    cx.factory().list(
        items
            .iter()
            .map(|item| cx.factory().string((*item).to_owned()))
            .collect::<Result<Vec<_>>>()?,
    )
}

fn parse_symbol(text: &str) -> Symbol {
    if let Some((namespace, name)) = text.split_once('/') {
        Symbol::qualified(namespace, name)
    } else {
        Symbol::new(text)
    }
}
