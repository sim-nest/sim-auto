//! Custom citizen field codecs for manifest records.

pub(crate) mod dtc_status_field {
    use sim_kernel::{Error, Expr, Result, Symbol};

    use crate::DtcStatus;

    pub(crate) fn encode(status: &DtcStatus) -> Expr {
        Expr::Map(vec![
            field("test_failed", status.test_failed),
            field(
                "test_failed_this_operation_cycle",
                status.test_failed_this_operation_cycle,
            ),
            field("pending", status.pending),
            field("confirmed", status.confirmed),
            field(
                "test_not_completed_since_clear",
                status.test_not_completed_since_clear,
            ),
            field("test_failed_since_clear", status.test_failed_since_clear),
            field(
                "test_not_completed_this_operation_cycle",
                status.test_not_completed_this_operation_cycle,
            ),
            field("warning_indicator", status.warning_indicator),
        ])
    }

    pub(crate) fn decode(expr: &Expr) -> Result<DtcStatus> {
        let Expr::Map(entries) = expr else {
            return Err(Error::Eval(
                "DTC status citizen field must be a map".to_owned(),
            ));
        };
        Ok(DtcStatus {
            test_failed: bool_field(entries, "test_failed")?,
            test_failed_this_operation_cycle: bool_field(
                entries,
                "test_failed_this_operation_cycle",
            )?,
            pending: bool_field(entries, "pending")?,
            confirmed: bool_field(entries, "confirmed")?,
            test_not_completed_since_clear: bool_field(entries, "test_not_completed_since_clear")?,
            test_failed_since_clear: bool_field(entries, "test_failed_since_clear")?,
            test_not_completed_this_operation_cycle: bool_field(
                entries,
                "test_not_completed_this_operation_cycle",
            )?,
            warning_indicator: bool_field(entries, "warning_indicator")?,
        })
    }

    fn field(name: &str, value: bool) -> (Expr, Expr) {
        (Expr::Symbol(Symbol::new(name)), Expr::Bool(value))
    }

    fn bool_field(entries: &[(Expr, Expr)], name: &'static str) -> Result<bool> {
        entries
            .iter()
            .find_map(|(key, value)| {
                if key == &Expr::Symbol(Symbol::new(name)) {
                    match value {
                        Expr::Bool(value) => Some(Ok(*value)),
                        _ => Some(Err(Error::Eval(format!(
                            "DTC status field {name} must be bool"
                        )))),
                    }
                } else {
                    None
                }
            })
            .unwrap_or_else(|| Err(Error::Eval(format!("missing DTC status field {name}"))))
    }
}

pub(crate) mod op_caps_field {
    use sim_kernel::{CapabilityName, Error, Expr, Result, Symbol};

    use crate::OpCap;

    pub(crate) fn encode(op_caps: &[OpCap]) -> Expr {
        Expr::List(
            op_caps
                .iter()
                .map(|op_cap| {
                    Expr::Map(vec![
                        string_field("operation", &op_cap.operation),
                        string_field("capability", op_cap.capability.as_str()),
                        string_field("effect_class", &op_cap.effect_class),
                    ])
                })
                .collect(),
        )
    }

    pub(crate) fn decode(expr: &Expr) -> Result<Vec<OpCap>> {
        let (Expr::List(items) | Expr::Vector(items)) = expr else {
            return Err(Error::Eval(
                "site manifest op_caps field must be a list".to_owned(),
            ));
        };
        items.iter().map(decode_one).collect()
    }

    fn decode_one(expr: &Expr) -> Result<OpCap> {
        let Expr::Map(entries) = expr else {
            return Err(Error::Eval(
                "site manifest op_cap entry must be a map".to_owned(),
            ));
        };
        Ok(OpCap::new(
            string_value(entries, "operation")?,
            CapabilityName::new(string_value(entries, "capability")?),
            string_value(entries, "effect_class")?,
        ))
    }

    fn string_value(entries: &[(Expr, Expr)], name: &'static str) -> Result<String> {
        entries
            .iter()
            .find_map(|(key, value)| {
                (field_name(key).as_deref() == Some(name)).then(|| match value {
                    Expr::String(value) => Ok(value.clone()),
                    Expr::Symbol(symbol) => Ok(symbol.as_qualified_str()),
                    _ => Err(Error::Eval(format!(
                        "site manifest op_cap field {name} must be string or symbol"
                    ))),
                })
            })
            .unwrap_or_else(|| {
                Err(Error::Eval(format!(
                    "missing site manifest op_cap field {name}"
                )))
            })
    }

    fn field_name(expr: &Expr) -> Option<String> {
        match expr {
            Expr::Symbol(symbol) => Some(symbol.name.trim_start_matches(':').to_owned()),
            Expr::String(value) => Some(value.trim_start_matches(':').to_owned()),
            _ => None,
        }
    }

    fn string_field(name: &str, value: &str) -> (Expr, Expr) {
        (
            Expr::Symbol(Symbol::new(name.to_owned())),
            Expr::String(value.to_owned()),
        )
    }
}
