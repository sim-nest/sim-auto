use sim_kernel::testing::bare_cx as cx;
use sim_lib_auto_core::{AUTO_DIAGNOSTICS_READ, AUTO_ORDER};
use sim_lib_auto_order::{expected_modeled_sites, run_modeled_conformance};

#[test]
fn full_modeled_work_order_replays_all_vendor_sites() {
    let mut cx = cx();
    cx.grant_named(AUTO_DIAGNOSTICS_READ);
    cx.grant_named(AUTO_ORDER);

    let report = run_modeled_conformance(&mut cx).unwrap();

    assert!(report.passed, "{:?}", report.issues);
    assert_eq!(report.site_count, expected_modeled_sites().len() as u32);
    for site in expected_modeled_sites() {
        assert!(
            report
                .work_order
                .ledger
                .events
                .iter()
                .any(|event| event.site == site),
            "missing site {site}"
        );
    }
    assert!(
        report
            .work_order
            .ledger
            .events
            .iter()
            .any(|event| event.site == "xentry"
                && event.operation == "info/wis-procedure"
                && event.outcome == "accepted")
    );
    assert!(
        report
            .work_order
            .ledger
            .events
            .iter()
            .any(|event| event.site == "xentry"
                && event.operation == "parts/epc-lookup"
                && event.outcome == "accepted")
    );
    assert!(
        report
            .work_order
            .ledger
            .events
            .iter()
            .any(|event| event.site == "mekonomen-pro"
                && event.operation == "order/place"
                && event.outcome == "accepted")
    );
    assert!(
        report
            .work_order
            .ledger
            .events
            .iter()
            .any(|event| event.site == "autotuner"
                && event.operation == "flash/write"
                && event.outcome == "denied")
    );
    assert!(report.work_order.invoice.as_ref().unwrap().is_balanced());
    assert!(report.delegation_violations.is_empty());
}
