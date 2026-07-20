//! Repair-information model and source labels.

use sim_lib_auto_core::{AutoLane, Dtc, VehicleId};

/// Source family for modeled repair information.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InfoSource {
    /// Mercedes-Benz WIS-shaped modeled procedure.
    WisModeled,
    /// BMW ISTA-shaped modeled procedure.
    IstaModeled,
    /// Volvo VIDA-shaped modeled procedure.
    VidaModeled,
    /// Bosch ESI\[tronic\]-shaped modeled procedure.
    EsiTronicModeled,
    /// HaynesPro-shaped modeled procedure.
    HaynesProModeled,
    /// Shop-authored modeled procedure.
    ShopAuthored,
}

impl InfoSource {
    /// Parses a request source label.
    pub fn parse(label: &str) -> Option<Self> {
        match normalize(label).as_str() {
            "wis" | "wis-modeled" => Some(Self::WisModeled),
            "ista" | "ista-modeled" => Some(Self::IstaModeled),
            "vida" | "vida-modeled" => Some(Self::VidaModeled),
            "esitronic" | "esi-tronic" | "esitronic-modeled" | "esi-tronic-modeled" => {
                Some(Self::EsiTronicModeled)
            }
            "haynespro" | "haynes-pro" | "haynespro-modeled" | "haynes-pro-modeled" => {
                Some(Self::HaynesProModeled)
            }
            "shop" | "shop-authored" | "shop-modeled" => Some(Self::ShopAuthored),
            _ => None,
        }
    }

    /// Stable source label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WisModeled => "wis-modeled",
            Self::IstaModeled => "ista-modeled",
            Self::VidaModeled => "vida-modeled",
            Self::EsiTronicModeled => "esitronic-modeled",
            Self::HaynesProModeled => "haynespro-modeled",
            Self::ShopAuthored => "shop-authored",
        }
    }

    /// Human-facing source label for modeled documents.
    pub fn display_name(self) -> &'static str {
        match self {
            Self::WisModeled => "Modeled WIS",
            Self::IstaModeled => "Modeled ISTA",
            Self::VidaModeled => "Modeled VIDA",
            Self::EsiTronicModeled => "Modeled ESI[tronic]",
            Self::HaynesProModeled => "Modeled HaynesPro",
            Self::ShopAuthored => "Shop-authored fixture",
        }
    }

    /// Returns all modeled source labels.
    pub fn all() -> &'static [Self] {
        &[
            Self::WisModeled,
            Self::IstaModeled,
            Self::VidaModeled,
            Self::EsiTronicModeled,
            Self::HaynesProModeled,
            Self::ShopAuthored,
        ]
    }
}

/// Query used to select one repair-information document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepairQuery {
    /// Vehicle identity to match.
    pub vehicle: VehicleId,
    /// Preferred information source. `None` ranks all modeled sources.
    pub source: Option<InfoSource>,
    /// Diagnostic trouble code text, when known.
    pub dtc: Option<String>,
    /// ECU label, when known.
    pub ecu: Option<String>,
    /// Symptom text, when known.
    pub symptom: Option<String>,
    /// Automotive lane, usually `info`.
    pub lane: AutoLane,
}

impl RepairQuery {
    /// Builds an information query for a vehicle.
    pub fn new(vehicle: VehicleId) -> Self {
        Self {
            vehicle,
            source: None,
            dtc: None,
            ecu: None,
            symptom: None,
            lane: AutoLane::new("info"),
        }
    }

    /// Restricts the query to one source.
    pub fn with_source(mut self, source: InfoSource) -> Self {
        self.source = Some(source);
        self
    }

    /// Adds a DTC filter from a decoded diagnostic record.
    pub fn with_dtc(mut self, dtc: &Dtc) -> Self {
        self.dtc = Some(dtc.code.clone());
        self
    }

    /// Adds a DTC code filter.
    pub fn with_dtc_code(mut self, code: impl Into<String>) -> Self {
        self.dtc = Some(code.into());
        self
    }

    /// Adds an ECU filter.
    pub fn with_ecu(mut self, ecu: impl Into<String>) -> Self {
        self.ecu = Some(ecu.into());
        self
    }

    /// Adds a symptom filter.
    pub fn with_symptom(mut self, symptom: impl Into<String>) -> Self {
        self.symptom = Some(symptom.into());
        self
    }

    /// Adds a lane filter.
    pub fn with_lane(mut self, lane: AutoLane) -> Self {
        self.lane = lane;
        self
    }
}

/// A synthetic repair procedure represented as a SIM document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepairProcedure {
    /// Stable procedure id.
    pub id: String,
    /// Modeled information source.
    pub source: InfoSource,
    /// Vehicle identity this procedure applies to.
    pub vehicle: VehicleId,
    /// Optional DTC code this procedure targets.
    pub dtc: Option<String>,
    /// Optional ECU label this procedure targets.
    pub ecu: Option<String>,
    /// Optional symptom label this procedure targets.
    pub symptom: Option<String>,
    /// Automotive lane.
    pub lane: AutoLane,
    /// Document title.
    pub title: String,
    /// Short synthetic summary.
    pub summary: String,
    /// Ordered synthetic procedure steps.
    pub steps: Vec<String>,
    /// Safety or review notes.
    pub safety_notes: Vec<String>,
    /// Search tags.
    pub tags: Vec<String>,
}

impl RepairProcedure {
    /// Builds a synthetic repair procedure.
    pub fn new(
        id: impl Into<String>,
        source: InfoSource,
        vehicle: VehicleId,
        title: impl Into<String>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            source,
            vehicle,
            dtc: None,
            ecu: None,
            symptom: None,
            lane: AutoLane::new("info"),
            title: title.into(),
            summary: summary.into(),
            steps: Vec::new(),
            safety_notes: Vec::new(),
            tags: Vec::new(),
        }
    }

    /// Adds DTC applicability.
    pub fn with_dtc(mut self, code: impl Into<String>) -> Self {
        self.dtc = Some(code.into());
        self
    }

    /// Adds ECU applicability.
    pub fn with_ecu(mut self, ecu: impl Into<String>) -> Self {
        self.ecu = Some(ecu.into());
        self
    }

    /// Adds symptom applicability.
    pub fn with_symptom(mut self, symptom: impl Into<String>) -> Self {
        self.symptom = Some(symptom.into());
        self
    }

    /// Adds procedure steps.
    pub fn with_steps(mut self, steps: &[&str]) -> Self {
        self.steps = steps.iter().map(|step| (*step).to_owned()).collect();
        self
    }

    /// Adds safety notes.
    pub fn with_safety_notes(mut self, notes: &[&str]) -> Self {
        self.safety_notes = notes.iter().map(|note| (*note).to_owned()).collect();
        self
    }

    /// Adds search tags.
    pub fn with_tags(mut self, tags: &[&str]) -> Self {
        self.tags = tags.iter().map(|tag| normalize(tag)).collect();
        self
    }
}

fn normalize(label: &str) -> String {
    label
        .trim()
        .to_ascii_lowercase()
        .chars()
        .filter(|ch| !matches!(ch, '[' | ']'))
        .map(|ch| if ch == '_' { '-' } else { ch })
        .collect()
}
