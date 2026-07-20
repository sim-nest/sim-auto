//! Scene projection for the automotive bay.

use sim_kernel::{Error, Expr, Result, Symbol};
use sim_lib_scene::{badge, box_, stack, text_node, validate_scene};

use crate::{BayDtc, BayPanelStatus, BayState, BayTimelineEntry, dtc_status_label};

/// Symbol naming the bay Scene surface.
pub fn bay_scene_symbol() -> Symbol {
    Symbol::qualified("auto", "bay")
}

/// Renders a complete bay Scene and validates it before returning.
pub fn bay_scene(state: &BayState) -> Result<Expr> {
    let scene = stack(
        "column",
        vec![
            vehicle_header_panel(state),
            site_selection_panel(state),
            dtc_list_panel(state),
            repair_document_panel(state),
            parts_cart_panel(state),
            status_panel(&state.coding_status),
            status_panel(&state.flash_gate_status),
            ledger_timeline_panel(state),
        ],
    );
    validate_scene(&scene).map_err(|err| Error::Eval(format!("auto bay Scene invalid: {err}")))?;
    Ok(scene)
}

fn vehicle_header_panel(state: &BayState) -> Expr {
    panel(
        "vehicle-header",
        vec![
            text_node(format!(
                "vehicle {}/{}",
                state.vehicle.namespace, state.vehicle.key
            )),
            badge("active", "modeled bay"),
            text_node(format!("surface {}", bay_scene_symbol().as_qualified_str())),
        ],
    )
}

fn site_selection_panel(state: &BayState) -> Expr {
    panel(
        "site-selection",
        vec![
            text_node(format!("site {} ({})", state.site.site, state.site.brand)),
            text_node(format!("lanes {}", state.site.lanes.join(", "))),
            text_node(format!("operations {}", state.site.operations.join(", "))),
        ],
    )
}

fn dtc_list_panel(state: &BayState) -> Expr {
    let mut children = vec![text_node("DTC list")];
    children.extend(state.dtcs.iter().map(dtc_row));
    panel("dtc-list", children)
}

fn dtc_row(row: &BayDtc) -> Expr {
    panel(
        "dtc",
        vec![
            text_node(format!(
                "{} {} {}",
                row.dtc.code,
                dtc_status_label(&row.dtc.status),
                row.ecu
            )),
            text_node(row.dtc.description.clone()),
            badge(dtc_status_label(&row.dtc.status), &row.dtc.system),
        ],
    )
}

fn repair_document_panel(state: &BayState) -> Expr {
    panel(
        "repair-document",
        vec![
            text_node("repair document"),
            text_node(state.repair_title.clone()),
            text_node(state.repair_summary.clone()),
        ],
    )
}

fn parts_cart_panel(state: &BayState) -> Expr {
    let mut children = vec![text_node("parts cart")];
    children.extend(state.parts_cart.iter().map(|part| {
        panel(
            "part-line",
            vec![
                text_node(format!("{} x{}", part.sku, part.qty)),
                text_node(part.description.clone()),
            ],
        )
    }));
    panel("parts-cart", children)
}

fn status_panel(status: &BayPanelStatus) -> Expr {
    panel(
        &format!("{}-status", status.label.replace(' ', "-")),
        vec![
            text_node(status.label.clone()),
            badge(&status.state, &status.state),
            text_node(status.detail.clone()),
        ],
    )
}

fn ledger_timeline_panel(state: &BayState) -> Expr {
    let mut children = vec![text_node("ledger timeline")];
    children.extend(state.ledger_timeline.iter().map(timeline_row));
    panel("ledger-timeline", children)
}

fn timeline_row(row: &BayTimelineEntry) -> Expr {
    panel(
        "ledger-event",
        vec![
            text_node(format!("{} {}", row.id, row.label)),
            badge(&row.status, &row.status),
        ],
    )
}

fn panel(role: &str, children: Vec<Expr>) -> Expr {
    box_(role, children)
}
