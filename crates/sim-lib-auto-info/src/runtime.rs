//! Loadable library wiring for modeled repair information.

use std::sync::Arc;

use sim_kernel::{
    AbiVersion, Cx, Export, Lib, LibManifest, LibTarget, Linker, LoadCx, Result, Symbol, Version,
};

use crate::{AutoInfoFabric, InfoSource};

/// Loadable library that contributes the modeled automotive information site.
#[derive(Clone, Copy, Debug, Default)]
pub struct AutoInfoLib;

impl Lib for AutoInfoLib {
    fn manifest(&self) -> LibManifest {
        LibManifest {
            id: Symbol::qualified("auto", "info"),
            version: Version(env!("CARGO_PKG_VERSION").to_owned()),
            abi: AbiVersion { major: 0, minor: 1 },
            target: LibTarget::HostRegistered,
            requires: Vec::new(),
            capabilities: Vec::new(),
            exports: vec![
                Export::Site {
                    symbol: auto_info_site_symbol(),
                    runtime_id: None,
                },
                Export::Value {
                    symbol: auto_info_sources_symbol(),
                },
                Export::Value {
                    symbol: auto_info_shape_symbol(),
                },
            ],
        }
    }

    fn load(&self, cx: &mut LoadCx, linker: &mut Linker<'_>) -> Result<()> {
        let fabric = cx.factory().opaque(Arc::new(AutoInfoFabric::fixture()))?;
        linker.site_value(auto_info_site_symbol(), fabric)?;
        linker.value(auto_info_sources_symbol(), source_list(cx)?)?;
        linker.value(
            auto_info_shape_symbol(),
            cx.factory().string(
                "auto/info request: vehicle namespace source dtc ecu symptom lane".to_owned(),
            )?,
        )?;
        Ok(())
    }
}

/// Installs the modeled automotive information library into a context once.
pub fn install_auto_info_lib(cx: &mut Cx) -> Result<()> {
    if cx
        .registry()
        .lib(&Symbol::qualified("auto", "info"))
        .is_some()
    {
        return Ok(());
    }
    cx.load_lib(&AutoInfoLib).map(|_| ())
}

/// Symbol for the modeled information site export.
pub fn auto_info_site_symbol() -> Symbol {
    Symbol::qualified("auto", "info-site")
}

/// Symbol for the supported modeled information source list.
pub fn auto_info_sources_symbol() -> Symbol {
    Symbol::qualified("auto", "info-sources")
}

/// Symbol for the information request shape descriptor.
pub fn auto_info_shape_symbol() -> Symbol {
    Symbol::qualified("auto", "info-shape")
}

fn source_list(cx: &mut LoadCx) -> Result<sim_kernel::Value> {
    cx.factory().list(
        InfoSource::all()
            .iter()
            .map(|source| cx.factory().string(source.as_str().to_owned()))
            .collect::<Result<Vec<_>>>()?,
    )
}
