//! Helpers for explicit citizen read-construct expression forms.

use sim_kernel::{Expr, Symbol};

use crate::VehicleId;

/// Builds the explicit read-construct expression for a modeled vehicle id.
pub fn vehicle_read_construct(vehicle: &VehicleId) -> Expr {
    read_construct_expr(
        vehicle_class_symbol(),
        vec![
            Expr::Symbol(Symbol::new("v0")),
            Expr::String(vehicle.namespace.clone()),
            Expr::String(vehicle.key.clone()),
        ],
    )
}

/// Builds a `citizen/read-construct` extension from class text and arguments.
pub fn text_read_construct_expr(class: &str, args: Vec<Expr>) -> Expr {
    read_construct_expr(symbol_text(class), args)
}

/// Builds a `citizen/read-construct` extension from a class symbol and arguments.
pub fn read_construct_expr(class: Symbol, args: Vec<Expr>) -> Expr {
    Expr::Extension {
        tag: Symbol::qualified("citizen", "read-construct"),
        payload: Box::new(Expr::Vector(
            std::iter::once(Expr::Symbol(class)).chain(args).collect(),
        )),
    }
}

fn vehicle_class_symbol() -> Symbol {
    Symbol::qualified("auto", "VehicleId")
}

fn symbol_text(text: &str) -> Symbol {
    if let Some((namespace, name)) = text.split_once('/') {
        Symbol::qualified(namespace, name)
    } else {
        Symbol::new(text)
    }
}
