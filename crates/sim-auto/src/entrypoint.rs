//! Loadable library and Bootloader wiring for the auto command.

use std::{ffi::OsString, io::Write, sync::Arc};

use sim_kernel::{
    AbiVersion, Args, Callable, CodecId, Cx, Error, Export, Expr, Lib, LibManifest, LibTarget,
    Linker, LoadCx, Object, ObjectCompat, Result, Symbol, Value, Version,
};
use sim_run_core::{Bootloader, cli_main_entrypoint_symbol};

use crate::render_auto_command;

/// Bootloader verb served by this package.
pub const AUTO_VERB: &str = "auto";

/// Host library id registered for the auto verb.
pub const AUTO_HOST_LIB: &str = "lib/auto";

/// Returns the function symbol exported for the bootloader handoff.
pub fn auto_entrypoint_symbol() -> Symbol {
    cli_main_entrypoint_symbol(AUTO_VERB)
}

/// Builds the Bootloader used by the `sim-auto` binary.
pub fn auto_bootloader() -> Bootloader {
    Bootloader::standard()
        .host_lib("codec/lisp", lisp_boot_codec)
        .host_verb(AUTO_VERB, AUTO_HOST_LIB, || Box::new(AutoCliLib::new()))
}

/// Normalizes process args so `sim-auto diag` boots the same `auto` verb.
pub fn auto_boot_args<I, S>(args: I) -> Vec<OsString>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let mut args = args.into_iter().map(Into::into).collect::<Vec<_>>();
    match args.get(1).and_then(|arg| arg.to_str()) {
        Some(AUTO_VERB) => args,
        Some("--help" | "-h") | None => {
            args.insert(1, OsString::from(AUTO_VERB));
            args
        }
        Some(_) => {
            args.insert(1, OsString::from(AUTO_VERB));
            args
        }
    }
}

fn lisp_boot_codec() -> Box<dyn Lib> {
    Box::new(sim_codec_lisp::LispCodecLib::new(CodecId(1)).expect("lisp boot codec"))
}

/// Loadable automotive command library.
#[derive(Clone, Debug, Default)]
pub struct AutoCliLib;

impl AutoCliLib {
    /// Creates an auto command library instance.
    pub fn new() -> Self {
        Self
    }
}

impl Lib for AutoCliLib {
    fn manifest(&self) -> LibManifest {
        LibManifest {
            id: Symbol::qualified("lib", "auto"),
            version: Version(env!("CARGO_PKG_VERSION").to_owned()),
            abi: AbiVersion { major: 0, minor: 1 },
            target: LibTarget::HostRegistered,
            requires: Vec::new(),
            capabilities: Vec::new(),
            exports: vec![Export::Function {
                symbol: auto_entrypoint_symbol(),
                function_id: None,
            }],
        }
    }

    fn load(&self, cx: &mut LoadCx, linker: &mut Linker<'_>) -> Result<()> {
        let entrypoint = cx.factory().opaque(Arc::new(AutoCliEntrypoint))?;
        linker.function_value(auto_entrypoint_symbol(), entrypoint)?;
        Ok(())
    }
}

#[derive(Clone)]
struct AutoCliEntrypoint;

impl Object for AutoCliEntrypoint {
    fn display(&self, _cx: &mut Cx) -> Result<String> {
        Ok("cli/main/auto".to_owned())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ObjectCompat for AutoCliEntrypoint {
    fn as_callable(&self) -> Option<&dyn Callable> {
        Some(self)
    }
}

impl Callable for AutoCliEntrypoint {
    fn call(&self, cx: &mut Cx, args: Args) -> Result<Value> {
        let envelope = match args.values().first() {
            Some(envelope) => AutoEnvelope::from_value(cx, envelope)?,
            None => AutoEnvelope::default(),
        };
        let output = render_auto_command(&envelope.args)?;
        writeln!(std::io::stdout(), "{output}")
            .map_err(|err| Error::Eval(format!("write stdout: {err}")))?;
        cx.factory().bool(true)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct AutoEnvelope {
    args: Vec<String>,
}

impl AutoEnvelope {
    fn from_value(cx: &mut Cx, envelope: &Value) -> Result<Self> {
        Ok(Self {
            args: envelope_args(cx, envelope)?,
        })
    }
}

fn envelope_args(cx: &mut Cx, envelope: &Value) -> Result<Vec<String>> {
    let Some(table) = envelope.object().as_table_impl() else {
        return Err(Error::Eval("CLI envelope is not a table".to_owned()));
    };
    let value = table.get(cx, Symbol::new("args"))?;
    let Expr::List(items) = value.object().as_expr(cx)? else {
        return Err(Error::TypeMismatch {
            expected: "argument list",
            found: "non-list",
        });
    };
    items
        .into_iter()
        .map(|item| match item {
            Expr::String(value) => Ok(value),
            other => Err(Error::TypeMismatch {
                expected: "string argument",
                found: sim_value::kind::expr_kind(&other),
            }),
        })
        .collect()
}
