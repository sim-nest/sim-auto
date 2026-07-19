//! Loadable library wiring for the diagnostic fabric.

use std::sync::Arc;

use sim_kernel::{
    AbiVersion, Cx, Export, Lib, LibManifest, LibTarget, Linker, LoadCx, Result, Symbol, Version,
};

use crate::fabric::AutoDiagFabric;

/// Loadable library that contributes the modeled automotive diagnostic site.
#[derive(Clone, Copy, Debug, Default)]
pub struct AutoDiagLib;

impl Lib for AutoDiagLib {
    fn manifest(&self) -> LibManifest {
        LibManifest {
            id: Symbol::qualified("auto", "diag"),
            version: Version(env!("CARGO_PKG_VERSION").to_owned()),
            abi: AbiVersion { major: 0, minor: 1 },
            target: LibTarget::HostRegistered,
            requires: Vec::new(),
            capabilities: Vec::new(),
            exports: vec![Export::Site {
                symbol: auto_diag_site_symbol(),
                runtime_id: None,
            }],
        }
    }

    fn load(&self, cx: &mut LoadCx, linker: &mut Linker<'_>) -> Result<()> {
        let value = cx.factory().opaque(Arc::new(AutoDiagFabric::fixture()))?;
        linker.site_value(auto_diag_site_symbol(), value)?;
        Ok(())
    }
}

/// Installs the diagnostic library into a context once.
pub fn install_auto_diag_lib(cx: &mut Cx) -> Result<()> {
    if cx
        .registry()
        .lib(&Symbol::qualified("auto", "diag"))
        .is_some()
    {
        return Ok(());
    }
    cx.load_lib(&AutoDiagLib).map(|_| ())
}

/// Symbol for the modeled diagnostic site export.
pub fn auto_diag_site_symbol() -> Symbol {
    Symbol::qualified("auto", "diag-site")
}
