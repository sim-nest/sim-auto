//! Diagnostic request expression helpers and parser.

use std::collections::BTreeMap;

use sim_kernel::{Error, Expr, Result, Symbol};
use sim_lib_auto_core::{AUTO_CONTROL_EXEC, AUTO_DIAGNOSTICS_READ};

/// Diagnostic operation accepted by the automotive fabric.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiagnosticRequest {
    /// Read all DTCs for an ECU.
    ReadDtcs {
        /// ECU identifier.
        ecu: String,
    },
    /// Read one PID value from an ECU.
    ReadPid {
        /// ECU identifier.
        ecu: String,
        /// PID identifier.
        pid: String,
    },
    /// Read freeze-frame records for an ECU.
    FreezeFrame {
        /// ECU identifier.
        ecu: String,
    },
    /// Controlled diagnostic operation.
    Control {
        /// Operation symbol text.
        operation: String,
        /// ECU identifier.
        ecu: String,
    },
}

impl DiagnosticRequest {
    /// Parses a diagnostic request expression.
    pub fn parse(expr: &Expr) -> Result<Self> {
        match expr {
            Expr::Call { operator, args } => {
                let operator = operator_symbol(operator)?;
                parse_parts(operator, args)
            }
            Expr::List(items) | Expr::Vector(items) => {
                let Some((head, args)) = items.split_first() else {
                    return Err(Error::Eval("empty diagnostic request".to_owned()));
                };
                parse_parts(operator_symbol(head)?, args)
            }
            Expr::Map(entries) => parse_map_request(entries),
            _ => Err(Error::Eval(
                "diagnostic request must be a call, list, vector, or map".to_owned(),
            )),
        }
    }

    /// Returns the capability required by this operation.
    pub fn required_capability(&self) -> &'static str {
        match self {
            Self::ReadDtcs { .. } | Self::ReadPid { .. } | Self::FreezeFrame { .. } => {
                AUTO_DIAGNOSTICS_READ
            }
            Self::Control { .. } => AUTO_CONTROL_EXEC,
        }
    }

    /// Returns the ECU identifier.
    pub fn ecu(&self) -> &str {
        match self {
            Self::ReadDtcs { ecu }
            | Self::ReadPid { ecu, .. }
            | Self::FreezeFrame { ecu }
            | Self::Control { ecu, .. } => ecu,
        }
    }
}

/// Builds an `auto/read-dtc` expression.
pub fn read_dtcs_expr(ecu: &str) -> Expr {
    keyed_expr(Symbol::qualified("auto", "read-dtc"), &[("ecu", ecu)])
}

/// Builds an `auto/read-pid` expression.
pub fn read_pid_expr(ecu: &str, pid: &str) -> Expr {
    keyed_expr(
        Symbol::qualified("auto", "read-pid"),
        &[("ecu", ecu), ("pid", pid)],
    )
}

/// Builds an `auto/freeze-frame` expression.
pub fn freeze_frame_expr(ecu: &str) -> Expr {
    keyed_expr(Symbol::qualified("auto", "freeze-frame"), &[("ecu", ecu)])
}

/// Builds an `auto/code` expression.
pub fn code_expr(ecu: &str) -> Expr {
    keyed_expr(Symbol::qualified("auto", "code"), &[("ecu", ecu)])
}

fn parse_parts(operator: String, args: &[Expr]) -> Result<DiagnosticRequest> {
    let fields = keyed_args(args)?;
    let ecu = required_field(&fields, "ecu")?;
    match operator.as_str() {
        "auto/read-dtc" | "auto/read-dtcs" => Ok(DiagnosticRequest::ReadDtcs { ecu }),
        "auto/read-pid" => Ok(DiagnosticRequest::ReadPid {
            ecu,
            pid: required_field(&fields, "pid")?,
        }),
        "auto/freeze-frame" => Ok(DiagnosticRequest::FreezeFrame { ecu }),
        "auto/code"
        | "auto/coding"
        | "auto/service"
        | "auto/service-function"
        | "auto/actuate"
        | "auto/actuation" => Ok(DiagnosticRequest::Control {
            operation: operator,
            ecu,
        }),
        _ => Err(Error::Eval(format!(
            "unsupported diagnostic operation {operator}"
        ))),
    }
}

fn parse_map_request(entries: &[(Expr, Expr)]) -> Result<DiagnosticRequest> {
    let mut fields = BTreeMap::new();
    for (key, value) in entries {
        fields.insert(field_key(key)?, string_value(value)?);
    }
    let operation = required_field(&fields, "op")?;
    parse_parts(operation, &map_to_args(&fields))
}

fn keyed_args(args: &[Expr]) -> Result<BTreeMap<String, String>> {
    if !args.len().is_multiple_of(2) {
        return Err(Error::Eval(
            "diagnostic request arguments must be key/value pairs".to_owned(),
        ));
    }
    let mut fields = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        fields.insert(field_key(&pair[0])?, string_value(&pair[1])?);
    }
    Ok(fields)
}

fn map_to_args(fields: &BTreeMap<String, String>) -> Vec<Expr> {
    fields
        .iter()
        .filter(|(key, _)| key.as_str() != "op")
        .flat_map(|(key, value)| {
            [
                Expr::Symbol(Symbol::new(format!(":{key}"))),
                Expr::String(value.clone()),
            ]
        })
        .collect()
}

fn keyed_expr(operator: Symbol, fields: &[(&str, &str)]) -> Expr {
    let mut items = vec![Expr::Symbol(operator)];
    for (key, value) in fields {
        items.push(Expr::Symbol(Symbol::new(format!(":{key}"))));
        items.push(Expr::String((*value).to_owned()));
    }
    Expr::List(items)
}

fn operator_symbol(expr: &Expr) -> Result<String> {
    match expr {
        Expr::Symbol(symbol) => Ok(symbol.as_qualified_str()),
        _ => Err(Error::Eval(
            "diagnostic request operator must be a symbol".to_owned(),
        )),
    }
}

fn field_key(expr: &Expr) -> Result<String> {
    match expr {
        Expr::Symbol(symbol) => Ok(symbol.name.trim_start_matches(':').to_owned()),
        Expr::String(value) => Ok(value.trim_start_matches(':').to_owned()),
        _ => Err(Error::Eval(
            "diagnostic request field key must be a symbol or string".to_owned(),
        )),
    }
}

fn string_value(expr: &Expr) -> Result<String> {
    match expr {
        Expr::String(value) => Ok(value.clone()),
        Expr::Symbol(symbol) => Ok(symbol.as_qualified_str()),
        _ => Err(Error::Eval(
            "diagnostic request field value must be a string or symbol".to_owned(),
        )),
    }
}

fn required_field(fields: &BTreeMap<String, String>, name: &'static str) -> Result<String> {
    fields
        .get(name)
        .cloned()
        .ok_or_else(|| Error::Eval(format!("missing diagnostic request field {name}")))
}
