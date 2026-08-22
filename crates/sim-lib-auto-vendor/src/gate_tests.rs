use std::sync::Arc;

use sim_kernel::{CapabilityName, Expr};
use sim_lib_operation_gate::ExecutionMode;

use crate::test_support::{cx_with, request};
use crate::{ModeledVendorBridge, cassette_vendor_fabric, mekonomen_pro_manifest, vendor_cassette};

#[test]
fn manifest_policy_is_explicit_and_public_replay_dispatches_once() {
    let manifest = mekonomen_pro_manifest();
    let operation = crate::manifest_operation(&manifest, "order/place").unwrap();
    assert_eq!(operation.declaration.mode, ExecutionMode::Recorded);
    assert_eq!(
        operation.declaration.capability,
        CapabilityName::new("auto/order")
    );

    let bridge = Arc::new(ModeledVendorBridge::new());
    let fabric = cassette_vendor_fabric(manifest, bridge.clone(), vendor_cassette());
    let request = request(
        Expr::Map(vec![
            (
                Expr::String("op".into()),
                Expr::String("order/place".into()),
            ),
            (Expr::String("args".into()), Expr::Map(Vec::new())),
        ]),
        &["auto/order"],
    );
    let mut cx = cx_with(&["auto/order"]);
    let first = sim_kernel::EvalFabric::realize(&fabric, &mut cx, request.clone()).unwrap();
    let second = sim_kernel::EvalFabric::realize(&fabric, &mut cx, request).unwrap();
    assert_eq!(first.value, second.value);
    assert_eq!(bridge.calls().unwrap().len(), 1);
}
