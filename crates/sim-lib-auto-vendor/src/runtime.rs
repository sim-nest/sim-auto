//! Loadable library wiring for manifest-backed vendor sites.

use std::sync::Arc;

use sim_kernel::{
    AbiVersion, Cx, Export, Lib, LibManifest, LibTarget, Linker, LoadCx, Result, Symbol, Version,
};
use sim_lib_auto_core::SiteManifest;

use crate::{ModeledVendorBridge, VendorBridge, VendorSiteFabric};

/// Loadable library that contributes manifest-backed automotive vendor sites.
pub struct AutoVendorLib {
    manifests: Vec<SiteManifest>,
    bridge: Arc<dyn VendorBridge>,
}

impl AutoVendorLib {
    /// Builds a vendor library over explicit manifests and bridge.
    pub fn new(manifests: Vec<SiteManifest>, bridge: Arc<dyn VendorBridge>) -> Self {
        Self { manifests, bridge }
    }

    /// Builds a vendor library backed by the public modeled bridge.
    pub fn modeled(manifests: Vec<SiteManifest>) -> Self {
        Self::new(manifests, Arc::new(ModeledVendorBridge::new()))
    }

    /// Returns the manifests served by this library.
    pub fn manifests(&self) -> &[SiteManifest] {
        &self.manifests
    }
}

impl Lib for AutoVendorLib {
    fn manifest(&self) -> LibManifest {
        LibManifest {
            id: Symbol::qualified("auto", "vendor"),
            version: Version(env!("CARGO_PKG_VERSION").to_owned()),
            abi: AbiVersion { major: 0, minor: 1 },
            target: LibTarget::HostRegistered,
            requires: Vec::new(),
            capabilities: Vec::new(),
            exports: self
                .manifests
                .iter()
                .map(|manifest| Export::Site {
                    symbol: auto_vendor_site_symbol(manifest),
                    runtime_id: None,
                })
                .collect(),
        }
    }

    fn load(&self, cx: &mut LoadCx, linker: &mut Linker<'_>) -> Result<()> {
        for manifest in &self.manifests {
            let fabric = VendorSiteFabric::new(manifest.clone(), Arc::clone(&self.bridge));
            let value = cx.factory().opaque(Arc::new(fabric))?;
            linker.site_value(auto_vendor_site_symbol(manifest), value)?;
        }
        Ok(())
    }
}

/// Installs modeled vendor sites into a context.
pub fn install_auto_vendor_lib(cx: &mut Cx, manifests: Vec<SiteManifest>) -> Result<()> {
    if cx
        .registry()
        .lib(&Symbol::qualified("auto", "vendor"))
        .is_some()
    {
        return Ok(());
    }
    cx.load_lib(&AutoVendorLib::modeled(manifests)).map(|_| ())
}

/// Symbol for a manifest-backed vendor site export.
pub fn auto_vendor_site_symbol(manifest: &SiteManifest) -> Symbol {
    Symbol::qualified(
        "auto",
        format!("vendor-{}", sanitize_symbol(&manifest.site)),
    )
}

fn sanitize_symbol(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}
