//! Candidate ranking for modeled repair documents.

use sim_kernel::{Error, Result};

use crate::{RepairProcedure, RepairQuery};

/// A ranked repair-information candidate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepairCandidate {
    /// Candidate score.
    pub score: i32,
    /// Procedure selected for this score.
    pub procedure: RepairProcedure,
}

/// Selects the best repair-information document for `query`.
pub fn rank_repair_docs(
    query: &RepairQuery,
    candidates: &[RepairProcedure],
) -> Result<RepairProcedure> {
    candidates
        .iter()
        .filter(|procedure| source_matches(query, procedure))
        .filter(|procedure| procedure.vehicle == query.vehicle)
        .map(|procedure| RepairCandidate {
            score: score(query, procedure),
            procedure: procedure.clone(),
        })
        .filter(|candidate| candidate.score > 0)
        .max_by(|left, right| {
            left.score
                .cmp(&right.score)
                .then_with(|| right.procedure.id.cmp(&left.procedure.id))
        })
        .map(|candidate| candidate.procedure)
        .ok_or_else(|| Error::Eval("no modeled repair information matched query".to_owned()))
}

fn source_matches(query: &RepairQuery, procedure: &RepairProcedure) -> bool {
    query
        .source
        .map(|source| source == procedure.source)
        .unwrap_or(true)
}

fn score(query: &RepairQuery, procedure: &RepairProcedure) -> i32 {
    let mut score = 20;
    score += string_match(query.dtc.as_deref(), procedure.dtc.as_deref(), 40);
    score += string_match(query.ecu.as_deref(), procedure.ecu.as_deref(), 20);
    score += symptom_score(query.symptom.as_deref(), procedure);
    if query.lane.name.eq_ignore_ascii_case(&procedure.lane.name) {
        score += 10;
    }
    if query.source == Some(procedure.source) {
        score += 5;
    }
    score
}

fn string_match(query: Option<&str>, candidate: Option<&str>, points: i32) -> i32 {
    match (query, candidate) {
        (Some(query), Some(candidate)) if query.eq_ignore_ascii_case(candidate) => points,
        (Some(_), Some(_)) => -points,
        (Some(_), None) => -points / 2,
        _ => 0,
    }
}

fn symptom_score(query: Option<&str>, procedure: &RepairProcedure) -> i32 {
    let Some(query) = query else {
        return 0;
    };
    let query = normalize(query);
    let mut text = procedure
        .symptom
        .as_deref()
        .map(normalize)
        .unwrap_or_default();
    for tag in &procedure.tags {
        text.push(' ');
        text.push_str(&normalize(tag));
    }
    if text.contains(&query) || query.contains(&text) {
        15
    } else {
        -5
    }
}

fn normalize(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace('-', " ")
}
