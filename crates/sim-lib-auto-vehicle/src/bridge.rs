//! Vehicle lookup bridge traits and default modeled routing.

use sim_kernel::{CapabilityName, Cx, Error, Result};
use sim_lib_auto_core::VehicleId;

use crate::{
    NET_HTTP_CAPABILITY, VehicleLookupKind, VehicleLookupRequest, VehicleRecord, VehicleSource,
};

/// Bridge boundary used by vehicle lookup sources.
pub trait VehicleLookupBridge: Send + Sync {
    /// Resolves one vehicle lookup request.
    fn lookup(&self, cx: &mut Cx, request: &VehicleLookupRequest) -> Result<VehicleRecord>;
}

/// Public modeled vehicle identity bridge.
#[derive(Clone, Debug)]
pub struct ModeledVehicleBridge {
    records: Vec<VehicleRecord>,
}

impl ModeledVehicleBridge {
    /// Builds a modeled bridge over explicit records.
    pub fn new(records: Vec<VehicleRecord>) -> Self {
        Self { records }
    }

    /// Builds the default synthetic Swedish fixture set.
    pub fn fixture() -> Self {
        Self::new(vec![
            VehicleRecord::new(
                VehicleId::new("modeled-se", "vehicle-alpha"),
                VehicleSource::Modeled,
                "SE",
                "modeled make",
                "alpha estate",
                "2024",
            )
            .with_plate("SIM 123")
            .expect("fixture plate is valid")
            .with_vin("SIM VIN ALPHA 001")
            .expect("fixture VIN is valid"),
            VehicleRecord::new(
                VehicleId::new("modeled-se", "vehicle-beta"),
                VehicleSource::Modeled,
                "SE",
                "modeled make",
                "beta van",
                "2025",
            )
            .with_plate("SIM 45 X")
            .expect("fixture plate is valid")
            .with_vin("SIM VIN BETA 002")
            .expect("fixture VIN is valid"),
        ])
    }

    /// Returns the modeled records served by the bridge.
    pub fn records(&self) -> &[VehicleRecord] {
        &self.records
    }
}

impl Default for ModeledVehicleBridge {
    fn default() -> Self {
        Self::fixture()
    }
}

impl VehicleLookupBridge for ModeledVehicleBridge {
    fn lookup(&self, _cx: &mut Cx, request: &VehicleLookupRequest) -> Result<VehicleRecord> {
        if request.source != VehicleSource::Modeled {
            return Err(Error::Eval(format!(
                "modeled vehicle bridge cannot answer source {}",
                request.source.as_str()
            )));
        }
        self.records
            .iter()
            .find(|record| record_matches(record, request))
            .cloned()
            .ok_or_else(|| Error::Eval(format!("no modeled vehicle for {}", request.key)))
    }
}

/// Contract bridge for live host-owned vehicle identity sources.
#[derive(Clone, Debug, Default)]
pub struct LiveVehicleBridge;

impl VehicleLookupBridge for LiveVehicleBridge {
    fn lookup(&self, cx: &mut Cx, request: &VehicleLookupRequest) -> Result<VehicleRecord> {
        if !request.source.is_live() {
            return Err(Error::Eval(format!(
                "live vehicle bridge cannot answer source {}",
                request.source.as_str()
            )));
        }
        cx.require(&CapabilityName::new(NET_HTTP_CAPABILITY))?;
        Err(Error::Eval(format!(
            "vehicle source {} requires a host-owned HttpDir under net/http; this crate commits no endpoint, key, plate, VIN, or owner data",
            request.source.as_str()
        )))
    }
}

/// Default router for modeled and live vehicle lookup sources.
#[derive(Clone, Debug, Default)]
pub struct VehicleLookupRouter {
    modeled: ModeledVehicleBridge,
    live: LiveVehicleBridge,
}

impl VehicleLookupRouter {
    /// Builds a router from explicit modeled and live bridges.
    pub fn new(modeled: ModeledVehicleBridge, live: LiveVehicleBridge) -> Self {
        Self { modeled, live }
    }

    /// Returns the modeled bridge.
    pub fn modeled(&self) -> &ModeledVehicleBridge {
        &self.modeled
    }
}

impl VehicleLookupBridge for VehicleLookupRouter {
    fn lookup(&self, cx: &mut Cx, request: &VehicleLookupRequest) -> Result<VehicleRecord> {
        match request.source {
            VehicleSource::Modeled => self.modeled.lookup(cx, request),
            VehicleSource::HaynesPro | VehicleSource::BiluppgifterSe => {
                self.live.lookup(cx, request)
            }
        }
    }
}

/// Looks up a vehicle record by plate using the default router.
pub fn vehicle_record_by_plate(
    cx: &mut Cx,
    source: VehicleSource,
    plate: &str,
    market: &str,
) -> Result<VehicleRecord> {
    let request = VehicleLookupRequest::by_plate(source, plate, market)?;
    VehicleLookupRouter::default().lookup(cx, &request)
}

/// Looks up a vehicle record by VIN using the default router.
pub fn vehicle_record_by_vin(
    cx: &mut Cx,
    source: VehicleSource,
    vin: &str,
    market: &str,
) -> Result<VehicleRecord> {
    let request = VehicleLookupRequest::by_vin(source, vin, market)?;
    VehicleLookupRouter::default().lookup(cx, &request)
}

/// Looks up a vehicle identity by plate using the default router.
pub fn vehicle_by_plate(
    cx: &mut Cx,
    source: VehicleSource,
    plate: &str,
    market: &str,
) -> Result<VehicleId> {
    vehicle_record_by_plate(cx, source, plate, market).map(|record| record.vehicle)
}

/// Looks up a vehicle identity by VIN using the default router.
pub fn vehicle_by_vin(
    cx: &mut Cx,
    source: VehicleSource,
    vin: &str,
    market: &str,
) -> Result<VehicleId> {
    vehicle_record_by_vin(cx, source, vin, market).map(|record| record.vehicle)
}

fn record_matches(record: &VehicleRecord, request: &VehicleLookupRequest) -> bool {
    if record.market != request.market {
        return false;
    }
    match request.kind {
        VehicleLookupKind::Plate => record.plate.as_deref() == Some(request.key.as_str()),
        VehicleLookupKind::Vin => record.vin.as_deref() == Some(request.key.as_str()),
    }
}
