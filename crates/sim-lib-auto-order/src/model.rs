//! Work-order and conformance report model values.

use std::collections::BTreeSet;

use sim_citizen::CitizenField;
use sim_kernel::{CapabilityName, Expr, Symbol};
use sim_lib_auto_core::{
    AUTO_DIAGNOSTICS_READ, AUTO_ORDER, BrandCaps, VehicleId, text_read_construct_expr,
    vehicle_read_construct,
};
use sim_lib_auto_parts::{OrderStatus, PartLine};

use crate::LedgerInvoiceExport;
use crate::invoice::{
    bool_arg, number_expr, read_construct_args, string_arg, string_list_arg, u32_arg,
};

mod fields;

/// One work-order ledger event.
#[derive(Clone, Debug, PartialEq, Eq, sim_citizen_derive::Citizen)]
#[citizen(symbol = "auto/WorkOrderEvent", version = 0)]
pub struct WorkOrderEvent {
    /// Manifest site label.
    pub site: String,
    /// Operation symbol text.
    pub operation: String,
    /// Manifest lane selected by policy.
    pub lane: String,
    /// Required operation capability.
    pub capability: String,
    /// Effect class applied by the vendor gate.
    pub effect: String,
    /// Stable outcome label.
    pub outcome: String,
    /// Operator-facing note.
    pub note: String,
    /// Capabilities delegated to the child site request.
    pub delegated_capabilities: Vec<String>,
}

impl Default for WorkOrderEvent {
    fn default() -> Self {
        Self::from_input(WorkOrderEventInput {
            site: "xentry".to_owned(),
            operation: "read/dtc".to_owned(),
            lane: "read".to_owned(),
            capability: CapabilityName::new(AUTO_DIAGNOSTICS_READ),
            effect: "pure".to_owned(),
            outcome: "accepted".to_owned(),
            note: "modeled diagnostic read".to_owned(),
            delegated_capabilities: vec![CapabilityName::new(AUTO_DIAGNOSTICS_READ)],
        })
    }
}

impl WorkOrderEvent {
    pub(crate) fn from_input(input: WorkOrderEventInput) -> Self {
        Self {
            site: input.site,
            operation: input.operation,
            lane: input.lane,
            capability: input.capability.as_str().to_owned(),
            effect: input.effect,
            outcome: input.outcome,
            note: input.note,
            delegated_capabilities: input
                .delegated_capabilities
                .into_iter()
                .map(|capability| capability.as_str().to_owned())
                .collect(),
        }
    }

    /// Returns whether this event was accepted.
    pub fn accepted(&self) -> bool {
        self.outcome == "accepted"
    }

    /// Returns whether this event was denied by capability policy.
    pub fn denied(&self) -> bool {
        self.outcome == "denied"
    }

    /// Encodes this event as explicit read-construct data.
    pub fn to_expr(&self) -> Expr {
        text_read_construct_expr(
            "auto/WorkOrderEvent",
            vec![
                Expr::Symbol(Symbol::new("v0")),
                Expr::String(self.site.clone()),
                Expr::String(self.operation.clone()),
                Expr::String(self.lane.clone()),
                Expr::String(self.capability.clone()),
                Expr::String(self.effect.clone()),
                Expr::String(self.outcome.clone()),
                Expr::String(self.note.clone()),
                Expr::List(
                    self.delegated_capabilities
                        .iter()
                        .map(|capability| Expr::String(capability.clone()))
                        .collect(),
                ),
            ],
        )
    }
}

pub(crate) struct WorkOrderEventInput {
    pub(crate) site: String,
    pub(crate) operation: String,
    pub(crate) lane: String,
    pub(crate) capability: CapabilityName,
    pub(crate) effect: String,
    pub(crate) outcome: String,
    pub(crate) note: String,
    pub(crate) delegated_capabilities: Vec<CapabilityName>,
}

impl CitizenField for WorkOrderEvent {
    fn encode_field(&self) -> Expr {
        self.to_expr()
    }

    fn decode_field_expr(expr: &Expr, field: &'static str) -> sim_kernel::Result<Self> {
        let args = read_construct_args(expr, "auto/WorkOrderEvent", field, 8)?;
        Ok(Self {
            site: string_arg(&args[0], field)?,
            operation: string_arg(&args[1], field)?,
            lane: string_arg(&args[2], field)?,
            capability: string_arg(&args[3], field)?,
            effect: string_arg(&args[4], field)?,
            outcome: string_arg(&args[5], field)?,
            note: string_arg(&args[6], field)?,
            delegated_capabilities: string_list_arg(&args[7], field)?,
        })
    }
}

/// Append-only modeled work-order ledger.
#[derive(Clone, Debug, Default, PartialEq, Eq, sim_citizen_derive::Citizen)]
#[citizen(symbol = "auto/WorkOrderLedger", version = 0)]
pub struct WorkOrderLedger {
    /// Recorded work-order events.
    pub events: Vec<WorkOrderEvent>,
}

impl WorkOrderLedger {
    /// Builds an empty work-order ledger.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends an event.
    pub fn push(&mut self, event: WorkOrderEvent) {
        self.events.push(event);
    }

    /// Number of accepted events.
    pub fn accepted_count(&self) -> usize {
        self.events.iter().filter(|event| event.accepted()).count()
    }

    /// Number of denied events.
    pub fn denied_count(&self) -> usize {
        self.events.iter().filter(|event| event.denied()).count()
    }

    /// Returns the unique site labels in the ledger.
    pub fn sites(&self) -> BTreeSet<String> {
        self.events.iter().map(|event| event.site.clone()).collect()
    }

    /// Encodes this ledger as explicit read-construct data.
    pub fn to_expr(&self) -> Expr {
        text_read_construct_expr(
            "auto/WorkOrderLedger",
            vec![
                Expr::Symbol(Symbol::new("v0")),
                Expr::List(self.events.iter().map(WorkOrderEvent::to_expr).collect()),
            ],
        )
    }
}

impl CitizenField for WorkOrderLedger {
    fn encode_field(&self) -> Expr {
        self.to_expr()
    }

    fn decode_field_expr(expr: &Expr, field: &'static str) -> sim_kernel::Result<Self> {
        let args = read_construct_args(expr, "auto/WorkOrderLedger", field, 1)?;
        Ok(Self {
            events: Vec::<WorkOrderEvent>::decode_field_expr(&args[0], field)?,
        })
    }
}

/// One modeled automotive work-order session.
#[derive(Clone, Debug, PartialEq, Eq, sim_citizen_derive::Citizen)]
#[citizen(symbol = "auto/WorkOrder", version = 0)]
pub struct WorkOrder {
    /// Stable modeled work-order id.
    pub id: String,
    /// Vehicle bound to the work order.
    #[citizen(with = "fields::vehicle_field")]
    pub vehicle: VehicleId,
    /// Parent shop capability profile.
    #[citizen(with = "fields::brand_caps_field")]
    pub site: BrandCaps,
    /// Work-order ledger.
    pub ledger: WorkOrderLedger,
    /// Parts cart used by the order step.
    #[citizen(with = "fields::part_lines_field")]
    pub parts: Vec<PartLine>,
    /// Modeled supplier order status.
    #[citizen(with = "fields::order_status_option_field")]
    pub order_status: Option<OrderStatus>,
    /// Optional balanced invoice export.
    pub invoice: Option<LedgerInvoiceExport>,
}

impl Default for WorkOrder {
    fn default() -> Self {
        Self {
            id: "SIM-WO-1".to_owned(),
            vehicle: VehicleId::new("modeled-se", "vehicle-alpha"),
            site: BrandCaps::new(
                "modeled-shop",
                vec![
                    CapabilityName::new(AUTO_DIAGNOSTICS_READ),
                    CapabilityName::new(AUTO_ORDER),
                ],
            ),
            ledger: WorkOrderLedger::default(),
            parts: vec![PartLine::default()],
            order_status: Some(OrderStatus::default()),
            invoice: Some(LedgerInvoiceExport::default()),
        }
    }
}

impl WorkOrder {
    /// Builds a modeled work order.
    pub fn new(
        id: impl Into<String>,
        vehicle: VehicleId,
        parent_grants: Vec<CapabilityName>,
    ) -> Self {
        Self {
            id: id.into(),
            vehicle,
            site: BrandCaps::new("modeled-shop", parent_grants),
            ledger: WorkOrderLedger::new(),
            parts: vec![PartLine::default()],
            order_status: None,
            invoice: None,
        }
    }

    /// Encodes this work order as explicit read-construct data.
    pub fn to_expr(&self) -> Expr {
        text_read_construct_expr(
            "auto/WorkOrder",
            vec![
                Expr::Symbol(Symbol::new("v0")),
                Expr::String(self.id.clone()),
                vehicle_read_construct(&self.vehicle),
                brand_caps_expr(&self.site),
                self.ledger.to_expr(),
                Expr::List(self.parts.iter().map(PartLine::to_expr).collect()),
                self.order_status
                    .as_ref()
                    .map(OrderStatus::to_expr)
                    .unwrap_or(Expr::Nil),
                self.invoice
                    .as_ref()
                    .map(LedgerInvoiceExport::to_expr)
                    .unwrap_or(Expr::Nil),
            ],
        )
    }
}

impl CitizenField for WorkOrder {
    fn encode_field(&self) -> Expr {
        self.to_expr()
    }

    fn decode_field_expr(expr: &Expr, field: &'static str) -> sim_kernel::Result<Self> {
        let args = read_construct_args(expr, "auto/WorkOrder", field, 7)?;
        Ok(Self {
            id: string_arg(&args[0], field)?,
            vehicle: fields::vehicle_field::decode(&args[1])?,
            site: fields::brand_caps_field::decode(&args[2])?,
            ledger: WorkOrderLedger::decode_field_expr(&args[3], field)?,
            parts: fields::part_lines_field::decode(&args[4])?,
            order_status: fields::order_status_option_field::decode(&args[5])?,
            invoice: Option::<LedgerInvoiceExport>::decode_field_expr(&args[6], field)?,
        })
    }
}

/// Modeled work-order conformance result.
#[derive(Clone, Debug, Default, PartialEq, Eq, sim_citizen_derive::Citizen)]
#[citizen(symbol = "auto/ConformanceReport", version = 0)]
pub struct ConformanceReport {
    /// Whether the modeled conformance story passed.
    pub passed: bool,
    /// Work order produced by the story.
    pub work_order: WorkOrder,
    /// Number of distinct sites reached.
    pub site_count: u32,
    /// Number of expected sites.
    pub expected_site_count: u32,
    /// Number of accepted operation events.
    pub accepted_count: u32,
    /// Number of denied operation events.
    pub denied_count: u32,
    /// Delegation violations found while diminishing grants.
    pub delegation_violations: Vec<String>,
    /// Other conformance issues.
    pub issues: Vec<String>,
}

impl ConformanceReport {
    /// Builds a report.
    pub fn new(
        work_order: WorkOrder,
        expected_site_count: usize,
        delegation_violations: Vec<String>,
        issues: Vec<String>,
    ) -> Self {
        let site_count = work_order.ledger.sites().len();
        let accepted_count = work_order.ledger.accepted_count();
        let denied_count = work_order.ledger.denied_count();
        let passed = site_count == expected_site_count
            && delegation_violations.is_empty()
            && issues.is_empty()
            && work_order
                .invoice
                .as_ref()
                .is_some_and(LedgerInvoiceExport::is_balanced);
        Self {
            passed,
            work_order,
            site_count: site_count as u32,
            expected_site_count: expected_site_count as u32,
            accepted_count: accepted_count as u32,
            denied_count: denied_count as u32,
            delegation_violations,
            issues,
        }
    }

    /// Encodes this report as explicit read-construct data.
    pub fn to_expr(&self) -> Expr {
        text_read_construct_expr(
            "auto/ConformanceReport",
            vec![
                Expr::Symbol(Symbol::new("v0")),
                Expr::Bool(self.passed),
                self.work_order.to_expr(),
                number_expr(i64::from(self.site_count)),
                number_expr(i64::from(self.expected_site_count)),
                number_expr(i64::from(self.accepted_count)),
                number_expr(i64::from(self.denied_count)),
                string_list_expr(&self.delegation_violations),
                string_list_expr(&self.issues),
            ],
        )
    }
}

impl CitizenField for ConformanceReport {
    fn encode_field(&self) -> Expr {
        self.to_expr()
    }

    fn decode_field_expr(expr: &Expr, field: &'static str) -> sim_kernel::Result<Self> {
        let args = read_construct_args(expr, "auto/ConformanceReport", field, 8)?;
        Ok(Self {
            passed: bool_arg(&args[0], field)?,
            work_order: WorkOrder::decode_field_expr(&args[1], field)?,
            site_count: u32_arg(&args[2], field)?,
            expected_site_count: u32_arg(&args[3], field)?,
            accepted_count: u32_arg(&args[4], field)?,
            denied_count: u32_arg(&args[5], field)?,
            delegation_violations: string_list_arg(&args[6], field)?,
            issues: string_list_arg(&args[7], field)?,
        })
    }
}

fn brand_caps_expr(site: &BrandCaps) -> Expr {
    text_read_construct_expr(
        "auto/BrandCaps",
        vec![
            Expr::Symbol(Symbol::new("v0")),
            Expr::String(site.brand.clone()),
            Expr::List(
                site.capabilities
                    .iter()
                    .map(|capability| Expr::String(capability.as_str().to_owned()))
                    .collect(),
            ),
        ],
    )
}

fn string_list_expr(items: &[String]) -> Expr {
    Expr::List(
        items
            .iter()
            .map(|item| Expr::String(item.clone()))
            .collect(),
    )
}
