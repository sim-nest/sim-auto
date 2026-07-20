//! Custom citizen field codecs for nested automotive work-order values.

use sim_kernel::{CapabilityName, Error, Expr, Result, Symbol};
use sim_lib_auto_core::{BrandCaps, VehicleId, vehicle_read_construct};
use sim_lib_auto_parts::{OrderStatus, PartLine, Supplier};

use crate::invoice::{bool_arg, read_construct_args, string_arg, string_list_arg, u32_arg};

pub(crate) mod vehicle_field {
    use super::*;

    pub(crate) fn encode(vehicle: &VehicleId) -> Expr {
        vehicle_read_construct(vehicle)
    }

    pub(crate) fn decode(expr: &Expr) -> Result<VehicleId> {
        let args = read_construct_args(expr, "auto/VehicleId", "vehicle", 2)?;
        Ok(VehicleId::new(
            string_arg(&args[0], "vehicle")?,
            string_arg(&args[1], "vehicle")?,
        ))
    }
}

pub(crate) mod brand_caps_field {
    use super::*;

    pub(crate) fn encode(site: &BrandCaps) -> Expr {
        Expr::Extension {
            tag: Symbol::qualified("citizen", "read-construct"),
            payload: Box::new(Expr::Vector(vec![
                symbol_expr("auto/BrandCaps"),
                Expr::Symbol(Symbol::new("v0")),
                Expr::String(site.brand.clone()),
                Expr::List(
                    site.capabilities
                        .iter()
                        .map(|capability| Expr::String(capability.as_str().to_owned()))
                        .collect(),
                ),
            ])),
        }
    }

    pub(crate) fn decode(expr: &Expr) -> Result<BrandCaps> {
        let args = read_construct_args(expr, "auto/BrandCaps", "site", 2)?;
        Ok(BrandCaps::new(
            string_arg(&args[0], "site")?,
            string_list_arg(&args[1], "site")?
                .into_iter()
                .map(CapabilityName::new)
                .collect(),
        ))
    }
}

pub(crate) mod part_lines_field {
    use super::*;

    pub(crate) fn encode(parts: &[PartLine]) -> Expr {
        Expr::List(parts.iter().map(PartLine::to_expr).collect())
    }

    pub(crate) fn decode(expr: &Expr) -> Result<Vec<PartLine>> {
        match expr {
            Expr::List(items) => items.iter().map(part_line).collect(),
            other => Err(Error::Eval(format!(
                "citizen field parts: expected list, found {other:?}"
            ))),
        }
    }

    fn part_line(expr: &Expr) -> Result<PartLine> {
        let args = read_construct_args(expr, "auto/PartLine", "parts", 4)?;
        let oem = match &args[1] {
            Expr::Nil => None,
            other => Some(string_arg(other, "parts")?),
        };
        Ok(PartLine::new(
            string_arg(&args[0], "parts")?,
            oem,
            string_arg(&args[2], "parts")?,
            u32_arg(&args[3], "parts")?,
        ))
    }
}

pub(crate) mod order_status_option_field {
    use super::*;

    pub(crate) fn encode(status: &Option<OrderStatus>) -> Expr {
        status
            .as_ref()
            .map(OrderStatus::to_expr)
            .unwrap_or(Expr::Nil)
    }

    pub(crate) fn decode(expr: &Expr) -> Result<Option<OrderStatus>> {
        match expr {
            Expr::Nil => Ok(None),
            other => order_status(other).map(Some),
        }
    }

    fn order_status(expr: &Expr) -> Result<OrderStatus> {
        let args = read_construct_args(expr, "auto/OrderStatus", "order_status", 6)?;
        let supplier = string_arg(&args[1], "order_status")?;
        if Supplier::parse(&supplier).is_none() {
            return Err(Error::Eval(format!(
                "citizen field order_status: unknown supplier {supplier}"
            )));
        }
        Ok(OrderStatus {
            id: string_arg(&args[0], "order_status")?,
            supplier,
            accepted: bool_arg(&args[2], "order_status")?,
            reversible: bool_arg(&args[3], "order_status")?,
            line_count: u32_arg(&args[4], "order_status")?,
            ledger_ref: string_arg(&args[5], "order_status")?,
        })
    }
}

fn symbol_expr(text: &str) -> Expr {
    if let Some((namespace, name)) = text.split_once('/') {
        Expr::Symbol(Symbol::qualified(namespace, name))
    } else {
        Expr::Symbol(Symbol::new(text))
    }
}
