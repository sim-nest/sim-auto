use sim_citizen::check_fixture;
use sim_kernel::{Args, Expr, Lib, Symbol, testing::bare_cx as cx};
use sim_lib_auto_core::{AUTO_DIAGNOSTICS_READ, AUTO_FLASH, AUTO_ORDER, AUTO_SERVICE_WRITE};

use crate::{
    AutoOrderLib, ConformanceReport, LedgerInvoiceEvidence, LedgerInvoiceExport,
    LedgerInvoicePosting, WorkOrder, WorkOrderEvent, WorkOrderLedger, auto_order_citizen_symbols,
    auto_order_function_symbol, auto_order_shape_symbol, expected_modeled_sites,
    install_auto_order_lib, run_modeled_conformance,
};

#[test]
fn work_order_citizens_round_trip() {
    let mut cx = cx_with(&[AUTO_DIAGNOSTICS_READ, AUTO_ORDER]);
    install_auto_order_lib(&mut cx).unwrap();

    check_fixture(&mut cx, WorkOrderEvent::default()).unwrap();
    check_fixture(&mut cx, WorkOrderLedger::default()).unwrap();
    check_fixture(&mut cx, LedgerInvoiceEvidence::default()).unwrap();
    check_fixture(&mut cx, LedgerInvoicePosting::default()).unwrap();
    check_fixture(&mut cx, LedgerInvoiceExport::default()).unwrap();
    check_fixture(&mut cx, WorkOrder::default()).unwrap();
    check_fixture(&mut cx, ConformanceReport::default()).unwrap();
}

#[test]
fn modeled_conformance_covers_sites_and_denies_unsafe_steps() {
    let mut cx = cx_with(&[AUTO_DIAGNOSTICS_READ, AUTO_ORDER]);

    let report = run_modeled_conformance(&mut cx).unwrap();

    assert!(report.passed, "{:?}", report.issues);
    assert_eq!(report.site_count, expected_modeled_sites().len() as u32);
    assert!(report.accepted_count > report.denied_count);
    assert!(report.delegation_violations.is_empty());
    assert!(
        report
            .work_order
            .order_status
            .as_ref()
            .is_some_and(|status| status.accepted)
    );
    let invoice = report.work_order.invoice.as_ref().unwrap();
    assert!(invoice.is_balanced());
    assert_eq!(invoice.minor_sum(), 0);
    assert_event(&report, "xentry", "code/sca", AUTO_SERVICE_WRITE, "denied");
    assert_event(&report, "autotuner", "flash/write", AUTO_FLASH, "denied");
    assert_event(
        &report,
        "mekonomen-pro",
        "order/place",
        AUTO_ORDER,
        "accepted",
    );
}

#[test]
fn missing_parent_order_grant_fails_before_delegation() {
    let mut cx = cx_with(&[AUTO_DIAGNOSTICS_READ]);

    let denied = run_modeled_conformance(&mut cx).unwrap_err();

    assert!(matches!(
        denied,
        sim_kernel::Error::CapabilityDenied { capability } if capability.as_str() == AUTO_ORDER
    ));
}

#[test]
fn order_library_exports_function_shape_and_citizens() {
    let manifest = AutoOrderLib.manifest();
    let exports = manifest
        .exports
        .iter()
        .map(export_symbol)
        .collect::<Vec<_>>();
    assert!(exports.contains(&"auto/work-order".to_owned()));
    assert!(exports.contains(&"auto/work-order-shape".to_owned()));
    for symbol in auto_order_citizen_symbols() {
        assert!(exports.contains(&symbol.to_string()));
    }

    let mut cx = cx_with(&[AUTO_DIAGNOSTICS_READ, AUTO_ORDER]);
    install_auto_order_lib(&mut cx).unwrap();
    assert!(
        cx.registry()
            .value_by_symbol(&auto_order_shape_symbol())
            .is_some()
    );
    let function = cx
        .registry()
        .function_by_symbol(&auto_order_function_symbol())
        .cloned()
        .unwrap();
    let value = function
        .object()
        .as_callable()
        .unwrap()
        .call(&mut cx, Args::default())
        .unwrap();
    let expr = value.object().as_expr(&mut cx).unwrap();
    assert!(matches!(
        expr,
        Expr::Extension { tag, payload }
            if tag == Symbol::qualified("citizen", "read-construct")
                && matches!(
                    payload.as_ref(),
                    Expr::Vector(items)
                        if items.first() == Some(&Expr::Symbol(Symbol::qualified("auto", "ConformanceReport")))
                )
    ));
}

fn cx_with(capabilities: &[&'static str]) -> sim_kernel::Cx {
    let mut cx = cx();
    for capability in capabilities {
        cx.grant_named(capability);
    }
    cx
}

fn assert_event(
    report: &ConformanceReport,
    site: &str,
    operation: &str,
    capability: &str,
    outcome: &str,
) {
    assert!(report.work_order.ledger.events.iter().any(|event| {
        event.site == site
            && event.operation == operation
            && event.capability == capability
            && event.outcome == outcome
            && event
                .delegated_capabilities
                .iter()
                .all(|capability| parent_capabilities().contains(capability))
    }));
}

fn parent_capabilities() -> Vec<String> {
    [AUTO_DIAGNOSTICS_READ, AUTO_ORDER]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

fn export_symbol(export: &sim_kernel::Export) -> String {
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
