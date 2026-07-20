//! Loadable library wiring for modeled work orders.

use std::sync::Arc;

use sim_citizen::registered_citizens;
use sim_kernel::{
    AbiVersion, Args, Callable, Cx, Error, Export, Lib, LibManifest, LibTarget, Linker, LoadCx,
    Object, ObjectCompat, Result, Symbol, Value, Version,
};

use crate::run_modeled_conformance;

const ORDER_CITIZENS: &[&str] = &[
    "auto/WorkOrderEvent",
    "auto/WorkOrderLedger",
    "auto/LedgerInvoiceEvidence",
    "auto/LedgerInvoicePosting",
    "auto/LedgerInvoiceExport",
    "auto/WorkOrder",
    "auto/ConformanceReport",
];

/// Loadable library that contributes modeled work-order exports.
#[derive(Clone, Copy, Debug, Default)]
pub struct AutoOrderLib;

impl Lib for AutoOrderLib {
    fn manifest(&self) -> LibManifest {
        let mut exports = auto_order_citizen_symbols()
            .into_iter()
            .map(|symbol| Export::Class {
                symbol,
                class_id: None,
            })
            .collect::<Vec<_>>();
        exports.extend([
            Export::Function {
                symbol: auto_order_function_symbol(),
                function_id: None,
            },
            Export::Value {
                symbol: auto_order_shape_symbol(),
            },
        ]);
        LibManifest {
            id: Symbol::qualified("auto", "order"),
            version: Version(env!("CARGO_PKG_VERSION").to_owned()),
            abi: AbiVersion { major: 0, minor: 1 },
            target: LibTarget::HostRegistered,
            requires: Vec::new(),
            capabilities: Vec::new(),
            exports,
        }
    }

    fn load(&self, cx: &mut LoadCx, linker: &mut Linker<'_>) -> Result<()> {
        install_order_citizens(linker)?;
        linker.function_value(
            auto_order_function_symbol(),
            cx.factory().opaque(Arc::new(AutoWorkOrderFunction))?,
        )?;
        linker.value(
            auto_order_shape_symbol(),
            cx.factory().string(
                "auto/work-order: modeled MBTECH work order using diagnostic-read and order parent grants"
                    .to_owned(),
            )?,
        )?;
        Ok(())
    }
}

/// Installs the modeled work-order library into a context once.
pub fn install_auto_order_lib(cx: &mut Cx) -> Result<()> {
    if cx
        .registry()
        .lib(&Symbol::qualified("auto", "order"))
        .is_some()
    {
        return Ok(());
    }
    cx.load_lib(&AutoOrderLib).map(|_| ())
}

/// Returns the citizen class symbols exported by the work-order library.
pub fn auto_order_citizen_symbols() -> Vec<Symbol> {
    ORDER_CITIZENS
        .iter()
        .map(|symbol| parse_symbol(symbol))
        .collect()
}

/// Symbol for the modeled work-order callable.
pub fn auto_order_function_symbol() -> Symbol {
    Symbol::qualified("auto", "work-order")
}

/// Symbol for the work-order request shape descriptor.
pub fn auto_order_shape_symbol() -> Symbol {
    Symbol::qualified("auto", "work-order-shape")
}

#[derive(Clone)]
struct AutoWorkOrderFunction;

impl Object for AutoWorkOrderFunction {
    fn display(&self, _cx: &mut Cx) -> Result<String> {
        Ok("auto/work-order".to_owned())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ObjectCompat for AutoWorkOrderFunction {
    fn as_callable(&self) -> Option<&dyn Callable> {
        Some(self)
    }
}

impl Callable for AutoWorkOrderFunction {
    fn call(&self, cx: &mut Cx, args: Args) -> Result<Value> {
        if !args.values().is_empty() {
            return Err(Error::Eval(
                "auto/work-order currently accepts the modeled default story only".to_owned(),
            ));
        }
        let report = run_modeled_conformance(cx)?;
        cx.factory().expr(report.to_expr())
    }
}

fn install_order_citizens(linker: &mut Linker<'_>) -> Result<()> {
    for expected in ORDER_CITIZENS {
        let Some(info) = registered_citizens().find(|info| info.symbol == *expected) else {
            return Err(Error::HostError(format!(
                "work-order citizen {expected} is not registered"
            )));
        };
        (info.install)(linker)?;
    }
    Ok(())
}

fn parse_symbol(text: &str) -> Symbol {
    if let Some((namespace, name)) = text.split_once('/') {
        Symbol::qualified(namespace, name)
    } else {
        Symbol::new(text)
    }
}
