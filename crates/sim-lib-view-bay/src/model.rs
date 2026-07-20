//! Bay-facing state reduced from modeled automotive domain values.

use sim_kernel::{Error, Result};
use sim_lib_auto_core::{Dtc, DtcStatus, SiteManifest, VehicleId};
use sim_lib_auto_diag::ModeledVehicle;
use sim_lib_auto_info::{
    InfoSource, RepairQuery, fixture_vehicle, rank_repair_docs, repair_catalog,
};
use sim_lib_auto_parts::PartLine;
use sim_lib_auto_vendor::xentry_manifest;
use sim_lib_view::SurfaceCaps;

/// Diagnostic trouble code plus the ECU that reported it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BayDtc {
    /// ECU identifier displayed in the bay.
    pub ecu: String,
    /// Decoded DTC citizen.
    pub dtc: Dtc,
}

impl BayDtc {
    /// Builds a bay DTC row.
    pub fn new(ecu: impl Into<String>, dtc: Dtc) -> Self {
        Self {
            ecu: ecu.into(),
            dtc,
        }
    }
}

/// Compact status row used by bay panels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BayPanelStatus {
    /// Short status label.
    pub label: String,
    /// Machine-readable state token.
    pub state: String,
    /// Human-facing status detail.
    pub detail: String,
}

impl BayPanelStatus {
    /// Builds a bay panel status.
    pub fn new(
        label: impl Into<String>,
        state: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            label: label.into(),
            state: state.into(),
            detail: detail.into(),
        }
    }
}

/// One event summarized in the bay ledger timeline panel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BayTimelineEntry {
    /// Stable event id.
    pub id: String,
    /// Event label.
    pub label: String,
    /// Event status token.
    pub status: String,
}

impl BayTimelineEntry {
    /// Builds a timeline entry.
    pub fn new(id: impl Into<String>, label: impl Into<String>, status: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            status: status.into(),
        }
    }
}

/// State needed to render an automotive bay Scene and its available Intents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BayState {
    /// Vehicle being worked on.
    pub vehicle: VehicleId,
    /// Active vendor site manifest.
    pub site: SiteManifest,
    /// Diagnostic trouble codes shown in the bay.
    pub dtcs: Vec<BayDtc>,
    /// Selected repair document title.
    pub repair_title: String,
    /// Selected repair document summary.
    pub repair_summary: String,
    /// Parts selected for the order cart.
    pub parts_cart: Vec<PartLine>,
    /// Coding lane status.
    pub coding_status: BayPanelStatus,
    /// Flash gate status.
    pub flash_gate_status: BayPanelStatus,
    /// Ledger timeline summary.
    pub ledger_timeline: Vec<BayTimelineEntry>,
}

impl BayState {
    /// Builds the public modeled Mercedes bay fixture.
    pub fn modeled_mercedes() -> Result<Self> {
        let vehicle = fixture_vehicle();
        let dtcs = modeled_engine_dtcs()?;
        let selected_dtc = dtcs
            .iter()
            .find(|row| row.dtc.code == "P0301")
            .or_else(|| dtcs.first())
            .ok_or_else(|| Error::Eval("modeled bay fixture has no DTCs".to_owned()))?;
        let repair = rank_repair_docs(
            &RepairQuery::new(vehicle.clone())
                .with_source(InfoSource::WisModeled)
                .with_dtc(&selected_dtc.dtc)
                .with_ecu(&selected_dtc.ecu)
                .with_symptom("rough idle"),
            &repair_catalog(),
        )?;

        Ok(Self {
            vehicle,
            site: xentry_manifest(),
            dtcs,
            repair_title: repair.title,
            repair_summary: repair.summary,
            parts_cart: vec![PartLine::new(
                "SIM-COIL-1",
                Some("A0001500180"),
                "modeled ignition coil for cylinder 1",
                1,
            )],
            coding_status: BayPanelStatus::new(
                "coding",
                "ready",
                "modeled service coding can be requested after review",
            ),
            flash_gate_status: BayPanelStatus::new(
                "flash gate",
                "blocked",
                "backup, warrant, ledger, and human approval are required",
            ),
            ledger_timeline: vec![
                BayTimelineEntry::new("scan", "modeled scan accepted", "done"),
                BayTimelineEntry::new("repair-doc", "repair document selected", "done"),
                BayTimelineEntry::new("parts", "parts cart prepared", "ready"),
                BayTimelineEntry::new("flash-gate", "irreversible flash gate held", "blocked"),
            ],
        })
    }

    /// Returns the first confirmed DTC, falling back to the first visible DTC.
    pub fn primary_dtc(&self) -> Option<&BayDtc> {
        self.dtcs
            .iter()
            .find(|row| row.dtc.status.confirmed)
            .or_else(|| self.dtcs.first())
    }
}

/// Returns a stable bay-oriented surface capability descriptor.
pub fn bay_surface_caps() -> SurfaceCaps {
    SurfaceCaps::from_preset("tui", "auto.bay.modeled")
        .expect("the shared VIEW surface preset catalog includes tui")
}

/// Renders a compact status label for DTC status bits.
pub fn dtc_status_label(status: &DtcStatus) -> &'static str {
    if status.confirmed {
        "confirmed"
    } else if status.pending {
        "pending"
    } else if status.test_failed {
        "failed"
    } else {
        "stored"
    }
}

fn modeled_engine_dtcs() -> Result<Vec<BayDtc>> {
    let vehicle = ModeledVehicle::fixture();
    let ecu = vehicle.ecu("ME97")?;
    let mut rows = ecu
        .dtcs
        .iter()
        .cloned()
        .map(|dtc| BayDtc::new(ecu.name.clone(), dtc))
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| {
        (
            row.dtc.code != "P0301",
            !row.dtc.status.confirmed,
            row.dtc.code.clone(),
        )
    });
    Ok(rows)
}
