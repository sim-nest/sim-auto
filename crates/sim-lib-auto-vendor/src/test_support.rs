use sim_kernel::{CapabilityName, Consistency, Cx, EvalMode, EvalRequest, Expr, testing::bare_cx};

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

pub(crate) fn cx_with(capabilities: &[&'static str]) -> Cx {
    let mut cx = bare_cx();
    for capability in capabilities {
        cx.grant_named(capability);
    }
    cx
}
