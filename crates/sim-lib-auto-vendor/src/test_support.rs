use sim_kernel::{
    CapabilityName, Consistency, Cx, EvalMode, EvalRequest, Expr, Symbol, Value,
    testing::bare_cx as cx,
};
use sim_lib_auto_core::{AutoLane, BrandNeed};

pub(crate) fn brand_need(make: &str, lanes: &[&str]) -> BrandNeed {
    BrandNeed::new(
        make,
        lanes.iter().map(|lane| AutoLane::new(*lane)).collect(),
    )
}

pub(crate) fn request(expr: Expr, capabilities: &[&'static str]) -> EvalRequest {
    EvalRequest {
        expr,
        result_shape: None,
        required_capabilities: capabilities
            .iter()
            .copied()
            .map(CapabilityName::new)
            .collect(),
        deadline: None,
        consistency: Consistency::LocalFirst,
        mode: EvalMode::Eval,
        answer_limit: None,
        stream_buffer: None,
        stream: false,
        trace: false,
    }
}

pub(crate) fn export_symbol(export: &sim_kernel::Export) -> String {
    match export {
        sim_kernel::Export::Class { symbol, .. }
        | sim_kernel::Export::Function { symbol, .. }
        | sim_kernel::Export::Macro { symbol, .. }
        | sim_kernel::Export::Shape { symbol, .. }
        | sim_kernel::Export::Codec { symbol, .. }
        | sim_kernel::Export::NumberDomain { symbol, .. }
        | sim_kernel::Export::Value { symbol }
        | sim_kernel::Export::Site { symbol, .. }
        | sim_kernel::Export::Open { symbol, .. } => symbol.to_string(),
    }
}

pub(crate) trait RequestEdit {
    fn with_expr(self, expr: Expr) -> Self;
}

impl RequestEdit for EvalRequest {
    fn with_expr(mut self, expr: Expr) -> Self {
        self.expr = expr;
        self
    }
}

pub(crate) trait WithoutHumanGate {
    fn without_human_gate(self) -> Expr;
}

impl WithoutHumanGate for Expr {
    fn without_human_gate(self) -> Expr {
        let Expr::Map(fields) = self else {
            return self;
        };
        Expr::Map(
            fields
                .into_iter()
                .map(|(key, value)| {
                    if key == Expr::Symbol(Symbol::new("human-approved")) {
                        (key, Expr::Bool(false))
                    } else {
                        (key, value)
                    }
                })
                .collect(),
        )
    }
}

pub(crate) fn cx_with(capabilities: &[&'static str]) -> Cx {
    let mut cx = cx();
    for capability in capabilities {
        cx.grant_named(capability);
    }
    cx
}

pub(crate) fn expr_text(cx: &mut Cx, value: &Value) -> String {
    format!("{:?}", value.object().as_expr(cx).unwrap())
}

pub(crate) fn reversal_artifact(content_key: &str) -> Expr {
    Expr::Map(vec![
        (
            Expr::Symbol(Symbol::new("content-key")),
            Expr::String(content_key.to_owned()),
        ),
        (
            Expr::Symbol(Symbol::new("bytes")),
            Expr::Bytes(content_key.as_bytes().to_vec()),
        ),
    ])
}
