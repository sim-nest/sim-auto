//! Document and Scene projection for selected repair procedures.

use sim_kernel::{Error, Expr, Result};
use sim_lib_view_doc::{article, article_formatted, prose, section, table};

use crate::{RepairProcedure, RepairQuery, rank_repair_docs, repair_catalog};

/// Selects and renders a repair document from the built-in synthetic catalog.
pub fn repair_document(query: &RepairQuery) -> Result<Expr> {
    repair_document_from_catalog(query, &repair_catalog())
}

/// Selects and renders a repair document from `catalog`.
pub fn repair_document_from_catalog(
    query: &RepairQuery,
    catalog: &[RepairProcedure],
) -> Result<Expr> {
    let procedure = rank_repair_docs(query, catalog)?;
    Ok(procedure_document(&procedure))
}

/// Projects the selected repair document to a valid Scene.
pub fn repair_scene(query: &RepairQuery) -> Result<Expr> {
    repair_scene_from_catalog(query, &repair_catalog())
}

/// Projects the selected repair document from `catalog` to a valid Scene.
pub fn repair_scene_from_catalog(query: &RepairQuery, catalog: &[RepairProcedure]) -> Result<Expr> {
    let document = repair_document_from_catalog(query, catalog)?;
    let scene = article_formatted(&document);
    sim_lib_scene::validate_scene(&scene)
        .map_err(|err| Error::Eval(format!("repair document scene is invalid: {err}")))?;
    Ok(scene)
}

/// Renders a procedure as a SIM document value.
pub fn procedure_document(procedure: &RepairProcedure) -> Expr {
    let mut blocks = vec![
        section(&procedure.title),
        prose(&format!(
            "{} synthetic procedure. {}",
            procedure.source.display_name(),
            procedure.summary
        )),
        section("Applicability"),
        applicability_table(procedure),
        section("Modeled procedure"),
    ];
    blocks.extend(
        procedure
            .steps
            .iter()
            .enumerate()
            .map(|(index, step)| prose(&format!("{}. {step}", index + 1))),
    );
    if !procedure.safety_notes.is_empty() {
        blocks.push(section("Review notes"));
        blocks.extend(procedure.safety_notes.iter().map(|note| prose(note)));
    }
    article(&procedure.title, blocks)
}

fn applicability_table(procedure: &RepairProcedure) -> Expr {
    table(vec![
        vec![
            Expr::String("field".to_owned()),
            Expr::String("value".to_owned()),
        ],
        vec![
            Expr::String("source".to_owned()),
            Expr::String(procedure.source.as_str().to_owned()),
        ],
        vec![
            Expr::String("vehicle".to_owned()),
            Expr::String(format!(
                "{}/{}",
                procedure.vehicle.namespace, procedure.vehicle.key
            )),
        ],
        vec![
            Expr::String("dtc".to_owned()),
            Expr::String(optional(&procedure.dtc)),
        ],
        vec![
            Expr::String("ecu".to_owned()),
            Expr::String(optional(&procedure.ecu)),
        ],
        vec![
            Expr::String("symptom".to_owned()),
            Expr::String(optional(&procedure.symptom)),
        ],
        vec![
            Expr::String("lane".to_owned()),
            Expr::String(procedure.lane.name.clone()),
        ],
    ])
}

fn optional(value: &Option<String>) -> String {
    value.clone().unwrap_or_else(|| "not specified".to_owned())
}
