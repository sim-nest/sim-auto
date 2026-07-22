#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::sync::Arc;

use sim_kernel::{CapabilityName, Cx, DefaultFactory, Error, NoopEvalPolicy, Result};
use sim_lib_auto_core::{AUTO_DIAGNOSTICS_READ, AUTO_ORDER};
use sim_lib_auto_order::{ConformanceReport, expected_modeled_sites, run_modeled_conformance};

pub const EXPECTED_LANES: &[&str] = &["read", "info", "parts", "service", "order", "flash"];

pub fn assert_modeled_work_order() -> Result<String> {
    let report = modeled_report()?;
    require_report_passed(&report)?;
    let sites = report.work_order.ledger.sites();
    let lanes = lanes_in(&report);
    let missing_sites = expected_modeled_sites()
        .into_iter()
        .filter(|site| !sites.contains(*site))
        .collect::<Vec<_>>();
    if !missing_sites.is_empty() {
        return Err(Error::Eval(format!(
            "modeled work-order recipe missed sites {}",
            missing_sites.join(",")
        )));
    }
    let missing_lanes = EXPECTED_LANES
        .iter()
        .copied()
        .filter(|lane| !lanes.contains(*lane))
        .collect::<Vec<_>>();
    if !missing_lanes.is_empty() {
        return Err(Error::Eval(format!(
            "modeled work-order recipe missed lanes {}",
            missing_lanes.join(",")
        )));
    }
    Ok(coverage_summary(
        "work-order",
        "modeled-work-order",
        report.work_order.ledger.events.len(),
        report.accepted_count as usize,
        report.denied_count as usize,
    ))
}

pub fn assert_lane(lane: &str) -> Result<String> {
    let report = modeled_report()?;
    require_report_passed(&report)?;
    let events = report
        .work_order
        .ledger
        .events
        .iter()
        .filter(|event| event.lane == lane)
        .collect::<Vec<_>>();
    if events.is_empty() {
        return Err(Error::Eval(format!("modeled recipe missed lane {lane}")));
    }
    Ok(coverage_summary(
        "lane",
        lane,
        events.len(),
        events.iter().filter(|event| event.accepted()).count(),
        events.iter().filter(|event| event.denied()).count(),
    ))
}

pub fn assert_site(site: &str) -> Result<String> {
    let report = modeled_report()?;
    require_report_passed(&report)?;
    let events = report
        .work_order
        .ledger
        .events
        .iter()
        .filter(|event| event.site == site)
        .collect::<Vec<_>>();
    if events.is_empty() {
        return Err(Error::Eval(format!("modeled recipe missed site {site}")));
    }
    Ok(coverage_summary(
        "site",
        site,
        events.len(),
        events.iter().filter(|event| event.accepted()).count(),
        events.iter().filter(|event| event.denied()).count(),
    ))
}

fn modeled_report() -> Result<ConformanceReport> {
    let (mut cx, seat) = Cx::new_seated(Arc::new(NoopEvalPolicy), Arc::new(DefaultFactory));
    seat.grant(&mut cx, CapabilityName::new(AUTO_DIAGNOSTICS_READ))?;
    seat.grant(&mut cx, CapabilityName::new(AUTO_ORDER))?;
    run_modeled_conformance(&mut cx)
}

fn require_report_passed(report: &ConformanceReport) -> Result<()> {
    if report.passed {
        Ok(())
    } else {
        Err(Error::Eval(format!(
            "modeled conformance failed: issues={:?} delegation={:?}",
            report.issues, report.delegation_violations
        )))
    }
}

fn lanes_in(report: &ConformanceReport) -> BTreeSet<String> {
    report
        .work_order
        .ledger
        .events
        .iter()
        .map(|event| event.lane.clone())
        .collect()
}

fn coverage_summary(
    subject_kind: &str,
    subject: &str,
    event_count: usize,
    accepted_count: usize,
    denied_count: usize,
) -> String {
    format!(
        "auto recipe {subject_kind} {subject} events={event_count} accepted={accepted_count} denied={denied_count}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_order_recipe_covers_all_expected_sites_and_lanes() {
        assert_modeled_work_order().unwrap();
    }

    #[test]
    fn lane_and_site_checks_are_specific() {
        assert_lane("read").unwrap();
        assert_site("xentry").unwrap();
    }
}
