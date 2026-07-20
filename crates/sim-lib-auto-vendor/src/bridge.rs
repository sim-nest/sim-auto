//! Vendor bridge trait and modeled implementation.

use std::sync::{Mutex, MutexGuard};

use sim_kernel::{Cx, Error, Expr, Result, Symbol};

use crate::VendorBridgeRequest;

/// Transport boundary used by manifest-backed automotive vendor sites.
pub trait VendorBridge: Send + Sync {
    /// Dispatches one manifest-approved request and returns the decoded reply.
    fn call(&self, cx: &mut Cx, request: &VendorBridgeRequest) -> Result<Expr>;
}

/// Modeled bridge used by public tests and cassettes.
#[derive(Default)]
pub struct ModeledVendorBridge {
    calls: Mutex<Vec<VendorBridgeRequest>>,
}

impl ModeledVendorBridge {
    /// Builds an empty modeled bridge.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the requests observed by the bridge.
    pub fn calls(&self) -> Result<Vec<VendorBridgeRequest>> {
        Ok(lock(&self.calls, "modeled vendor bridge calls")?.clone())
    }
}

impl VendorBridge for ModeledVendorBridge {
    fn call(&self, _cx: &mut Cx, request: &VendorBridgeRequest) -> Result<Expr> {
        lock(&self.calls, "modeled vendor bridge calls")?.push(request.clone());
        Ok(Expr::Map(vec![
            string_field("site", &request.site),
            string_field("operation", &request.op),
            string_field("lane", &request.lane.name),
            string_field("vehicle", &request.vehicle.key),
            (Expr::Symbol(Symbol::new("accepted")), Expr::Bool(true)),
        ]))
    }
}

fn string_field(name: &str, value: &str) -> (Expr, Expr) {
    (
        Expr::Symbol(Symbol::new(name.to_owned())),
        Expr::String(value.to_owned()),
    )
}

fn lock<'a, T>(mutex: &'a Mutex<T>, name: &str) -> Result<MutexGuard<'a, T>> {
    mutex
        .lock()
        .map_err(|_| Error::Eval(format!("{name} mutex poisoned")))
}
