//! Vendor bridge trait and modeled implementation.

use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};

use sim_kernel::{Cx, Error, Expr, Result, Symbol};

use crate::VendorBridgeRequest;

/// Transport boundary used by manifest-backed automotive vendor sites.
pub trait VendorBridge: Send + Sync {
    /// Dispatches one manifest-approved request and returns the decoded reply.
    fn call(&self, cx: &mut Cx, request: &VendorBridgeRequest) -> Result<Expr>;
}

/// One modeled vendor reply keyed by site and operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModeledVendorCassette {
    /// Manifest site label.
    pub site: String,
    /// Operation symbol text.
    pub operation: String,
    /// Synthetic reply returned by the modeled bridge.
    pub reply: Expr,
}

impl ModeledVendorCassette {
    /// Builds a modeled vendor reply cassette.
    pub fn new(site: impl Into<String>, operation: impl Into<String>, reply: Expr) -> Self {
        Self {
            site: site.into(),
            operation: operation.into(),
            reply,
        }
    }
}

/// Modeled bridge used by public tests and cassettes.
#[derive(Default)]
pub struct ModeledVendorBridge {
    calls: Mutex<Vec<VendorBridgeRequest>>,
    cassettes: BTreeMap<(String, String), Expr>,
}

impl ModeledVendorBridge {
    /// Builds an empty modeled bridge.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a modeled bridge over explicit synthetic replies.
    pub fn with_cassettes(cassettes: impl IntoIterator<Item = ModeledVendorCassette>) -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            cassettes: cassettes
                .into_iter()
                .map(|cassette| ((cassette.site, cassette.operation), cassette.reply))
                .collect(),
        }
    }

    /// Returns the requests observed by the bridge.
    pub fn calls(&self) -> Result<Vec<VendorBridgeRequest>> {
        Ok(lock(&self.calls, "modeled vendor bridge calls")?.clone())
    }
}

impl VendorBridge for ModeledVendorBridge {
    fn call(&self, _cx: &mut Cx, request: &VendorBridgeRequest) -> Result<Expr> {
        lock(&self.calls, "modeled vendor bridge calls")?.push(request.clone());
        if let Some(reply) = self
            .cassettes
            .get(&(request.site.clone(), request.op.clone()))
        {
            return Ok(reply.clone());
        }
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
