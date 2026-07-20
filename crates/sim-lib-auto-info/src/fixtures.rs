//! Synthetic repair-information fixtures.

use sim_lib_auto_core::VehicleId;

use crate::{InfoSource, RepairProcedure};

/// Shared modeled vehicle used by public repair-information fixtures.
pub fn fixture_vehicle() -> VehicleId {
    VehicleId::new("modeled-se", "vehicle-alpha")
}

/// Returns the synthetic public repair-information catalog.
pub fn repair_catalog() -> Vec<RepairProcedure> {
    let vehicle = fixture_vehicle();
    vec![
        RepairProcedure::new(
            "wis-misfire-modeled",
            InfoSource::WisModeled,
            vehicle.clone(),
            "Modeled misfire diagnosis",
            "Synthetic WIS-shaped workflow for isolating a single-cylinder misfire.",
        )
        .with_dtc("P0301")
        .with_ecu("ME97")
        .with_symptom("rough idle")
        .with_steps(&[
            "Confirm the modeled DTC and freeze-frame context.",
            "Compare synthetic ignition, fuel, and compression observations.",
            "Record the selected modeled repair path in the work order.",
        ])
        .with_safety_notes(&[
            "Use modeled data only.",
            "Require a separate warrant before any service write.",
        ])
        .with_tags(&["misfire", "engine", "diagnosis"]),
        RepairProcedure::new(
            "ista-air-meter-modeled",
            InfoSource::IstaModeled,
            vehicle.clone(),
            "Modeled air-meter plausibility path",
            "Synthetic ISTA-shaped workflow for an air-meter plausibility complaint.",
        )
        .with_dtc("P0100")
        .with_ecu("DME")
        .with_symptom("hesitation")
        .with_steps(&[
            "Read modeled intake and load observations.",
            "Compare the fixture sensor trend against the expected range.",
            "Select the modeled connector and smoke-test branch.",
        ])
        .with_safety_notes(&["Do not infer live sensor values from this fixture."])
        .with_tags(&["air", "meter", "plausibility"]),
        RepairProcedure::new(
            "vida-lamp-circuit-modeled",
            InfoSource::VidaModeled,
            vehicle.clone(),
            "Modeled lamp circuit check",
            "Synthetic VIDA-shaped workflow for a body-control lamp circuit status.",
        )
        .with_dtc("B1000")
        .with_ecu("CEM")
        .with_symptom("lamp warning")
        .with_steps(&[
            "Read the modeled body-control DTC.",
            "Inspect the synthetic lamp circuit observation set.",
            "Store the modeled service note for later customer review.",
        ])
        .with_safety_notes(&["Treat lighting output checks as modeled observations."])
        .with_tags(&["body", "lamp", "circuit"]),
        RepairProcedure::new(
            "esitronic-misfire-modeled",
            InfoSource::EsiTronicModeled,
            vehicle.clone(),
            "Modeled compression comparison",
            "Synthetic ESI[tronic]-shaped workflow for a misfire that survives ignition checks.",
        )
        .with_dtc("P0301")
        .with_ecu("ME97")
        .with_symptom("uneven compression")
        .with_steps(&[
            "Group the modeled DTC with the fixture compression note.",
            "Compare synthetic cylinder balance observations.",
            "Escalate to a shop-authored confirmation procedure when needed.",
        ])
        .with_safety_notes(&["This procedure contains no captured workshop trace."])
        .with_tags(&["compression", "engine", "misfire"]),
        RepairProcedure::new(
            "haynespro-no-start-modeled",
            InfoSource::HaynesProModeled,
            vehicle.clone(),
            "Modeled no-start triage",
            "Synthetic HaynesPro-shaped workflow for a no-start symptom without a confirmed DTC.",
        )
        .with_ecu("starter")
        .with_symptom("no start")
        .with_steps(&[
            "Separate modeled power, fuel, and immobilizer branches.",
            "Record which branch explains the synthetic observation.",
            "Leave live measurements to the host-owned bridge.",
        ])
        .with_safety_notes(&["No live wiring image or procedure excerpt is bundled."])
        .with_tags(&["no-start", "triage", "symptom"]),
        RepairProcedure::new(
            "shop-road-test-modeled",
            InfoSource::ShopAuthored,
            vehicle,
            "Modeled shop road-test checklist",
            "Shop-authored synthetic workflow for confirming a repair path after modeled diagnosis.",
        )
        .with_dtc("P0301")
        .with_ecu("ME97")
        .with_symptom("post repair confirmation")
        .with_steps(&[
            "Review the modeled work order and selected repair document.",
            "Run the synthetic idle, load, and restart observations.",
            "Attach the modeled confirmation note to the ledgered job.",
        ])
        .with_safety_notes(&["This is a public fixture, not a live road-test instruction."])
        .with_tags(&["confirmation", "shop", "work-order"]),
    ]
}
