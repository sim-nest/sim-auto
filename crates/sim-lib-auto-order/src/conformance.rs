//! Modeled work-order conformance runner.

use std::{collections::BTreeSet, sync::Arc};

use sim_kernel::{
    CapabilityName, Consistency, Cx, Error, EvalFabric, EvalMode, EvalRequest, Expr, Result, Symbol,
};
use sim_lib_auto_core::{
    AUTO_DIAGNOSTICS_READ, AUTO_FLASH, AUTO_ORDER, AUTO_SERVICE_WRITE, SiteManifest, VehicleId,
};
use sim_lib_auto_parts::{OrderStatus, Supplier};
use sim_lib_auto_vendor::{
    ModeledVendorBridge, VendorEffectClass, VendorGateLedger, VendorSiteFabric, autotuner_manifest,
    biluppgifter_se_manifest, esitronic_manifest, flash_site_cassettes, haynespro_manifest,
    ista_manifest, manifest_operation, mekonomen_pro_manifest, odis_manifest, oem_site_cassettes,
    supplier_site_cassettes, vendor_irreversible_request_expr, vendor_request_expr, vida_manifest,
    xentry_manifest,
};

use crate::{
    ConformanceReport, LedgerInvoiceExport, WorkOrder, WorkOrderEvent, model::WorkOrderEventInput,
};

/// Runs the public modeled automotive work-order conformance story.
pub fn run_modeled_conformance(cx: &mut Cx) -> Result<ConformanceReport> {
    run_modeled_conformance_with_parent_grants(cx, default_parent_grants())
}

/// Runs conformance with an explicit parent grant set.
pub fn run_modeled_conformance_with_parent_grants(
    cx: &mut Cx,
    parent_grants: Vec<CapabilityName>,
) -> Result<ConformanceReport> {
    ModeledWorkOrderEngine::new(parent_grants).run(cx)
}

/// Returns the site labels covered by the modeled conformance story.
pub fn expected_modeled_sites() -> Vec<&'static str> {
    vec![
        "xentry",
        "ista",
        "vida",
        "odis",
        "esitronic",
        "haynespro",
        "biluppgifter-se",
        "mekonomen-pro",
        "autotuner",
    ]
}

/// Manifest-driven engine for a modeled work-order conformance story.
#[derive(Clone, Debug)]
pub struct ModeledWorkOrderEngine {
    parent_grants: Vec<CapabilityName>,
}

impl ModeledWorkOrderEngine {
    /// Builds an engine with the parent session grants.
    pub fn new(parent_grants: Vec<CapabilityName>) -> Self {
        Self { parent_grants }
    }

    /// Runs the conformance story.
    pub fn run(&self, cx: &mut Cx) -> Result<ConformanceReport> {
        cx.require_all(&self.parent_grants)?;
        let mut work_order = WorkOrder::new(
            "SIM-WO-MBTECH-1",
            VehicleId::new("modeled-se", "vehicle-alpha"),
            self.parent_grants.clone(),
        );
        let mut issues = Vec::new();
        let mut delegation_violations = Vec::new();
        let expected_sites = expected_modeled_sites();

        for story in site_stories() {
            self.run_story(
                cx,
                story,
                &mut work_order,
                &mut issues,
                &mut delegation_violations,
            )?;
        }

        if let Some(order) = &work_order.order_status {
            work_order.invoice = Some(LedgerInvoiceExport::for_work_order(
                &work_order.id,
                order,
                &work_order.parts,
                90,
            ));
        } else {
            issues.push("mekonomen-pro order/place did not produce an order status".to_owned());
        }

        let seen_sites = work_order.ledger.sites();
        for expected in &expected_sites {
            if !seen_sites.contains(*expected) {
                issues.push(format!("missing modeled site {expected}"));
            }
        }
        if !has_denial(&work_order, "xentry", "code/sca", AUTO_SERVICE_WRITE) {
            issues.push("missing denied xentry code/sca event".to_owned());
        }
        if !has_denial(&work_order, "autotuner", "flash/write", AUTO_FLASH) {
            issues.push("missing denied autotuner flash/write event".to_owned());
        }

        Ok(ConformanceReport::new(
            work_order,
            expected_sites.len(),
            delegation_violations,
            issues,
        ))
    }

    fn run_story(
        &self,
        cx: &mut Cx,
        story: SiteStory,
        work_order: &mut WorkOrder,
        issues: &mut Vec<String>,
        delegation_violations: &mut Vec<String>,
    ) -> Result<()> {
        let bridge = Arc::new(ModeledVendorBridge::with_cassettes(all_cassettes()));
        let gate_ledger = Arc::new(VendorGateLedger::new());
        let fabric = VendorSiteFabric::with_gate_ledger(
            story.manifest.clone(),
            bridge,
            Arc::clone(&gate_ledger),
        );
        for operation in story.operations {
            let operation_policy = manifest_operation(&story.manifest, operation)?;
            let delegated = self.delegated_grants(&operation_policy.capability);
            let before = gate_ledger.records()?.len();
            let outcome = fabric.realize(
                cx,
                request(operation, &operation_policy.effect, delegated.clone()),
            );
            let event = match outcome {
                Ok(_) => {
                    if let Some(violation) =
                        delegated_violation(&delegated, &self.parent_grants, operation)
                    {
                        delegation_violations.push(violation);
                    }
                    if operation == "order/place" {
                        work_order.order_status = Some(OrderStatus::accepted(
                            "SIM-ORDER-MBTECH-1",
                            Supplier::MekonomenProModeled,
                            work_order.parts.len() as u32,
                        ));
                    }
                    WorkOrderEvent::from_input(WorkOrderEventInput {
                        site: story.manifest.site.clone(),
                        operation: operation.to_owned(),
                        lane: operation_policy.lane.name,
                        capability: operation_policy.capability,
                        effect: operation_policy.effect.as_str().to_owned(),
                        outcome: "accepted".to_owned(),
                        note: gate_note(&gate_ledger, before)?,
                        delegated_capabilities: delegated,
                    })
                }
                Err(Error::CapabilityDenied { capability }) => {
                    WorkOrderEvent::from_input(WorkOrderEventInput {
                        site: story.manifest.site.clone(),
                        operation: operation.to_owned(),
                        lane: operation_policy.lane.name,
                        capability: operation_policy.capability,
                        effect: operation_policy.effect.as_str().to_owned(),
                        outcome: "denied".to_owned(),
                        note: format!("denied missing {}", capability.as_str()),
                        delegated_capabilities: delegated,
                    })
                }
                Err(err) => {
                    issues.push(format!(
                        "{} {} failed: {err}",
                        story.manifest.site, operation
                    ));
                    WorkOrderEvent::from_input(WorkOrderEventInput {
                        site: story.manifest.site.clone(),
                        operation: operation.to_owned(),
                        lane: operation_policy.lane.name,
                        capability: operation_policy.capability,
                        effect: operation_policy.effect.as_str().to_owned(),
                        outcome: "failed".to_owned(),
                        note: err.to_string(),
                        delegated_capabilities: delegated,
                    })
                }
            };
            work_order.ledger.push(event);
        }
        Ok(())
    }

    fn delegated_grants(&self, required: &CapabilityName) -> Vec<CapabilityName> {
        if self.parent_grants.iter().any(|grant| grant == required) {
            vec![required.clone()]
        } else {
            Vec::new()
        }
    }
}

fn default_parent_grants() -> Vec<CapabilityName> {
    vec![
        CapabilityName::new(AUTO_DIAGNOSTICS_READ),
        CapabilityName::new(AUTO_ORDER),
    ]
}

#[derive(Clone, Debug)]
struct SiteStory {
    manifest: SiteManifest,
    operations: Vec<&'static str>,
}

impl SiteStory {
    fn new(manifest: SiteManifest, operations: Vec<&'static str>) -> Self {
        Self {
            manifest,
            operations,
        }
    }
}

fn site_stories() -> Vec<SiteStory> {
    vec![
        SiteStory::new(
            xentry_manifest(),
            vec![
                "read/dtc",
                "info/wis-procedure",
                "parts/epc-lookup",
                "code/sca",
            ],
        ),
        SiteStory::new(
            ista_manifest(),
            vec!["read/dtc", "info/test-plan", "code/coding"],
        ),
        SiteStory::new(vida_manifest(), vec!["read/dtc", "info/procedure"]),
        SiteStory::new(odis_manifest(), vec!["service/guided-function"]),
        SiteStory::new(esitronic_manifest(), vec!["read/dtc", "info/procedure"]),
        SiteStory::new(
            haynespro_manifest(),
            vec!["read/identity", "info/procedure"],
        ),
        SiteStory::new(biluppgifter_se_manifest(), vec!["read/plate-lookup"]),
        SiteStory::new(
            mekonomen_pro_manifest(),
            vec!["parts/catalog-lookup", "order/status", "order/place"],
        ),
        SiteStory::new(autotuner_manifest(), vec!["flash/write"]),
    ]
}

fn all_cassettes() -> Vec<sim_lib_auto_vendor::ModeledVendorCassette> {
    oem_site_cassettes()
        .into_iter()
        .chain(supplier_site_cassettes())
        .chain(flash_site_cassettes())
        .collect()
}

fn request(
    operation: &str,
    effect: &VendorEffectClass,
    required_capabilities: Vec<CapabilityName>,
) -> EvalRequest {
    let expr = if *effect == VendorEffectClass::Irreversible {
        vendor_irreversible_request_expr(
            operation,
            story_args(),
            reversal_artifact(operation),
            "modeled-work-order-warrant",
        )
    } else {
        vendor_request_expr(operation, story_args())
    };
    EvalRequest {
        expr,
        result_shape: None,
        required_capabilities,
        deadline: None,
        consistency: Consistency::LocalFirst,
        mode: EvalMode::Eval,
        answer_limit: None,
        stream_buffer: None,
        stream: false,
        trace: false,
    }
}

fn story_args() -> Expr {
    Expr::Map(vec![
        string_field("work-order", "SIM-WO-MBTECH-1"),
        string_field("vehicle", "modeled-se/vehicle-alpha"),
    ])
}

fn reversal_artifact(operation: &str) -> Expr {
    Expr::Map(vec![
        (
            Expr::Symbol(Symbol::new("content-key")),
            Expr::String(format!("stock-map-before-{operation}")),
        ),
        (
            Expr::Symbol(Symbol::new("bytes")),
            Expr::Bytes(operation.as_bytes().to_vec()),
        ),
    ])
}

fn string_field(name: &str, value: &str) -> (Expr, Expr) {
    (
        Expr::Symbol(Symbol::new(name.to_owned())),
        Expr::String(value.to_owned()),
    )
}

fn gate_note(ledger: &VendorGateLedger, before: usize) -> Result<String> {
    let records = ledger.records()?;
    if records.len() > before {
        Ok(format!("vendor gate recorded {}", records.len() - before))
    } else {
        Ok("pure manifest reply".to_owned())
    }
}

fn delegated_violation(
    delegated: &[CapabilityName],
    parent_grants: &[CapabilityName],
    operation: &str,
) -> Option<String> {
    let parent = parent_grants.iter().collect::<BTreeSet<_>>();
    delegated
        .iter()
        .find(|capability| !parent.contains(*capability))
        .map(|capability| {
            format!(
                "{operation} delegated {} outside parent grant set",
                capability.as_str()
            )
        })
}

fn has_denial(work_order: &WorkOrder, site: &str, operation: &str, capability: &str) -> bool {
    work_order.ledger.events.iter().any(|event| {
        event.site == site
            && event.operation == operation
            && event.capability == capability
            && event.denied()
    })
}
