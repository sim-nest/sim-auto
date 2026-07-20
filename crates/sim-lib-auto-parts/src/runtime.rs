//! Loadable library wiring for modeled parts lookup and ordering.

use std::sync::Arc;

use sim_citizen::registered_citizens;
use sim_kernel::{
    AbiVersion, Cx, Error, Export, Lib, LibManifest, LibTarget, Linker, LoadCx, Result, Symbol,
    Version,
};

use crate::{AutoPartsFabric, Supplier, modeled_epc_dir};

const PARTS_CITIZENS: &[&str] = &["auto/PartLine", "auto/OrderStatus"];

/// Loadable library that contributes modeled automotive parts exports.
#[derive(Clone, Copy, Debug, Default)]
pub struct AutoPartsLib;

impl Lib for AutoPartsLib {
    fn manifest(&self) -> LibManifest {
        let mut exports = auto_parts_citizen_symbols()
            .into_iter()
            .map(|symbol| Export::Class {
                symbol,
                class_id: None,
            })
            .collect::<Vec<_>>();
        exports.extend([
            Export::Site {
                symbol: auto_parts_site_symbol(),
                runtime_id: None,
            },
            Export::Value {
                symbol: auto_parts_dir_symbol(),
            },
            Export::Value {
                symbol: auto_parts_suppliers_symbol(),
            },
            Export::Value {
                symbol: auto_parts_shape_symbol(),
            },
        ]);
        LibManifest {
            id: Symbol::qualified("auto", "parts"),
            version: Version(env!("CARGO_PKG_VERSION").to_owned()),
            abi: AbiVersion { major: 0, minor: 1 },
            target: LibTarget::HostRegistered,
            requires: Vec::new(),
            capabilities: Vec::new(),
            exports,
        }
    }

    fn load(&self, cx: &mut LoadCx, linker: &mut Linker<'_>) -> Result<()> {
        install_parts_citizens(linker)?;
        linker.site_value(
            auto_parts_site_symbol(),
            cx.factory().opaque(Arc::new(AutoPartsFabric::fixture()))?,
        )?;
        linker.value(
            auto_parts_dir_symbol(),
            cx.factory().opaque(Arc::new(modeled_epc_dir()))?,
        )?;
        linker.value(auto_parts_suppliers_symbol(), supplier_list(cx)?)?;
        linker.value(
            auto_parts_shape_symbol(),
            cx.factory().string(
                "auto parts request: catalog/get path | order/place supplier lines; live supplier requires auto/order and net/http"
                    .to_owned(),
            )?,
        )?;
        Ok(())
    }
}

/// Installs the parts library into a context once.
pub fn install_auto_parts_lib(cx: &mut Cx) -> Result<()> {
    if cx
        .registry()
        .lib(&Symbol::qualified("auto", "parts"))
        .is_some()
    {
        return Ok(());
    }
    cx.load_lib(&AutoPartsLib).map(|_| ())
}

/// Returns the citizen class symbols exported by the parts library.
pub fn auto_parts_citizen_symbols() -> Vec<Symbol> {
    PARTS_CITIZENS
        .iter()
        .map(|symbol| parse_symbol(symbol))
        .collect()
}

/// Symbol for the modeled parts site.
pub fn auto_parts_site_symbol() -> Symbol {
    Symbol::qualified("auto", "parts-site")
}

/// Symbol for the modeled EPC directory value.
pub fn auto_parts_dir_symbol() -> Symbol {
    Symbol::qualified("auto", "parts-dir")
}

/// Symbol for supported supplier labels.
pub fn auto_parts_suppliers_symbol() -> Symbol {
    Symbol::qualified("auto", "parts-suppliers")
}

/// Symbol for the parts request shape descriptor.
pub fn auto_parts_shape_symbol() -> Symbol {
    Symbol::qualified("auto", "parts-shape")
}

fn install_parts_citizens(linker: &mut Linker<'_>) -> Result<()> {
    for expected in PARTS_CITIZENS {
        let Some(info) = registered_citizens().find(|info| info.symbol == *expected) else {
            return Err(Error::HostError(format!(
                "parts citizen {expected} is not registered"
            )));
        };
        (info.install)(linker)?;
    }
    Ok(())
}

fn supplier_list(cx: &mut LoadCx) -> Result<sim_kernel::Value> {
    cx.factory().list(
        [
            Supplier::MekonomenProModeled.as_str(),
            Supplier::MekonomenProLive.as_str(),
        ]
        .into_iter()
        .map(|supplier| cx.factory().string(supplier.to_owned()))
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
