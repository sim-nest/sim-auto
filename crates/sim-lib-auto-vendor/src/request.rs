//! Request expression helpers and parser.

use std::collections::BTreeMap;

use sim_kernel::{Error, Expr, Result, Symbol};
use sim_lib_auto_core::{SiteManifest, VehicleId};

use crate::{VendorWarrant, manifest_operation};

/// Request sent from a manifest site to a vendor bridge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VendorBridgeRequest {
    /// Manifest site label.
    pub site: String,
    /// Lane selected for the operation.
    pub lane: sim_lib_auto_core::AutoLane,
    /// Operation symbol text.
    pub op: String,
    /// Vehicle identity attached to the request.
    pub vehicle: VehicleId,
    /// Operation arguments.
    pub args: Expr,
    /// Reversal artifact required for irreversible operations.
    pub reversal_artifact: Option<Expr>,
    /// Warrant required for irreversible operations.
    pub warrant: Option<VendorWarrant>,
    /// Human gate for irreversible operations.
    pub human_approved: bool,
}

impl VendorBridgeRequest {
    /// Builds a request with no irreversible-effect gate artifacts.
    pub fn new(
        site: impl Into<String>,
        lane: sim_lib_auto_core::AutoLane,
        op: impl Into<String>,
        vehicle: VehicleId,
        args: Expr,
    ) -> Self {
        Self {
            site: site.into(),
            lane,
            op: op.into(),
            vehicle,
            args,
            reversal_artifact: None,
            warrant: None,
            human_approved: false,
        }
    }

    /// Attaches a reversal artifact.
    pub fn with_reversal_artifact(mut self, artifact: Expr) -> Self {
        self.reversal_artifact = Some(artifact);
        self
    }

    /// Attaches a warrant.
    pub fn with_warrant(mut self, warrant: VendorWarrant) -> Self {
        self.warrant = Some(warrant);
        self
    }

    /// Opens or closes the human gate.
    pub fn with_human_approval(mut self, approved: bool) -> Self {
        self.human_approved = approved;
        self
    }
}

/// Builds a vendor request expression with explicit arguments.
pub fn vendor_request_expr(operation: &str, args: Expr) -> Expr {
    Expr::Map(vec![
        string_field("op", operation),
        (Expr::Symbol(Symbol::new("args")), args),
    ])
}

/// Builds an irreversible vendor request expression with gate artifacts.
pub fn vendor_irreversible_request_expr(
    operation: &str,
    args: Expr,
    reversal_artifact: Expr,
    warrant_id: &str,
) -> Expr {
    Expr::Map(vec![
        string_field("op", operation),
        (Expr::Symbol(Symbol::new("args")), args),
        (Expr::Symbol(Symbol::new("reversal")), reversal_artifact),
        string_field("warrant", warrant_id),
        (
            Expr::Symbol(Symbol::new("human-approved")),
            Expr::Bool(true),
        ),
    ])
}

pub(crate) fn parse_vendor_request(
    manifest: &SiteManifest,
    expr: &Expr,
) -> Result<VendorBridgeRequest> {
    let mut fields = match expr {
        Expr::Map(entries) => map_fields(entries)?,
        Expr::List(items) | Expr::Vector(items) => list_fields(items)?,
        _ => {
            return Err(Error::Eval(
                "auto vendor request must be a map, list, or vector".to_owned(),
            ));
        }
    };
    let op = string_field_value(&mut fields, "op")?;
    let operation = manifest_operation(manifest, &op)?;
    let vehicle = match fields.remove("vehicle") {
        Some(expr) => VehicleId::new("manifest", string_value(&expr)?),
        None => VehicleId::new("manifest", manifest.vehicle.clone()),
    };
    let args = fields
        .remove("args")
        .unwrap_or_else(|| Expr::Map(Vec::new()));
    let reversal_artifact = fields.remove("reversal");
    let warrant = match fields.remove("warrant") {
        Some(expr) => Some(VendorWarrant::new(string_value(&expr)?, "manifest request")),
        None => None,
    };
    let human_approved = fields
        .remove("human-approved")
        .map(|expr| bool_value(&expr))
        .transpose()?
        .unwrap_or(false);

    Ok(
        VendorBridgeRequest::new(manifest.site.clone(), operation.lane, op, vehicle, args)
            .with_human_approval(human_approved)
            .with_optional_reversal(reversal_artifact)
            .with_optional_warrant(warrant),
    )
}

trait OptionalGate {
    fn with_optional_reversal(self, artifact: Option<Expr>) -> Self;
    fn with_optional_warrant(self, warrant: Option<VendorWarrant>) -> Self;
}

impl OptionalGate for VendorBridgeRequest {
    fn with_optional_reversal(mut self, artifact: Option<Expr>) -> Self {
        self.reversal_artifact = artifact;
        self
    }

    fn with_optional_warrant(mut self, warrant: Option<VendorWarrant>) -> Self {
        self.warrant = warrant;
        self
    }
}

fn map_fields(entries: &[(Expr, Expr)]) -> Result<BTreeMap<String, Expr>> {
    entries
        .iter()
        .map(|(key, value)| Ok((field_key(key)?, value.clone())))
        .collect()
}

fn list_fields(items: &[Expr]) -> Result<BTreeMap<String, Expr>> {
    let Some((head, tail)) = items.split_first() else {
        return Err(Error::Eval("empty auto vendor request".to_owned()));
    };
    if !tail.len().is_multiple_of(2) {
        return Err(Error::Eval(
            "auto vendor request arguments must be key/value pairs".to_owned(),
        ));
    }
    let mut fields = BTreeMap::new();
    fields.insert("op".to_owned(), head.clone());
    for pair in tail.chunks_exact(2) {
        fields.insert(field_key(&pair[0])?, pair[1].clone());
    }
    Ok(fields)
}

fn string_field_value(fields: &mut BTreeMap<String, Expr>, name: &'static str) -> Result<String> {
    fields
        .remove(name)
        .ok_or_else(|| Error::Eval(format!("missing auto vendor request field {name}")))
        .and_then(|expr| string_value(&expr))
}

fn string_value(expr: &Expr) -> Result<String> {
    match expr {
        Expr::String(value) => Ok(value.clone()),
        Expr::Symbol(symbol) => Ok(symbol.as_qualified_str()),
        _ => Err(Error::Eval(
            "auto vendor request field must be string or symbol".to_owned(),
        )),
    }
}

fn bool_value(expr: &Expr) -> Result<bool> {
    match expr {
        Expr::Bool(value) => Ok(*value),
        _ => Err(Error::Eval(
            "auto vendor human-approved field must be bool".to_owned(),
        )),
    }
}

fn field_key(expr: &Expr) -> Result<String> {
    match expr {
        Expr::Symbol(symbol) => Ok(symbol.name.trim_start_matches(':').to_owned()),
        Expr::String(value) => Ok(value.trim_start_matches(':').to_owned()),
        _ => Err(Error::Eval(
            "auto vendor request key must be a symbol or string".to_owned(),
        )),
    }
}

fn string_field(name: &str, value: &str) -> (Expr, Expr) {
    (
        Expr::Symbol(Symbol::new(name.to_owned())),
        Expr::String(value.to_owned()),
    )
}
