//! Modeled ECU inventory and diagnostic data.

use sim_codec_uds::{DtcFrame, UdsFrame, decode_frame, encode_frame};
use sim_kernel::{CodecId, Error, Expr, Result, Symbol};
use sim_lib_auto_core::{Dtc, DtcStatus, VehicleId};

/// One modeled PID value exposed by an ECU.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PidValue {
    /// PID identifier, such as `rpm`.
    pub pid: String,
    /// Human-facing PID label.
    pub name: String,
    /// Synthetic value text.
    pub value: String,
    /// Unit label.
    pub unit: String,
}

impl PidValue {
    /// Builds a modeled PID value.
    pub fn new(
        pid: impl Into<String>,
        name: impl Into<String>,
        value: impl Into<String>,
        unit: impl Into<String>,
    ) -> Self {
        Self {
            pid: pid.into(),
            name: name.into(),
            value: value.into(),
            unit: unit.into(),
        }
    }

    pub(crate) fn to_expr(&self) -> Expr {
        Expr::Map(vec![
            string_field("pid", &self.pid),
            string_field("name", &self.name),
            string_field("value", &self.value),
            string_field("unit", &self.unit),
        ])
    }
}

/// Freeze-frame values captured for one modeled DTC.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreezeFrame {
    /// DTC associated with the freeze frame.
    pub dtc: String,
    /// PID values captured with the DTC.
    pub values: Vec<PidValue>,
}

impl FreezeFrame {
    /// Builds a modeled freeze-frame record.
    pub fn new(dtc: impl Into<String>, values: Vec<PidValue>) -> Self {
        Self {
            dtc: dtc.into(),
            values,
        }
    }

    pub(crate) fn to_expr(&self) -> Expr {
        Expr::Map(vec![
            string_field("dtc", &self.dtc),
            (
                Expr::Symbol(Symbol::new("values")),
                Expr::List(self.values.iter().map(PidValue::to_expr).collect()),
            ),
        ])
    }
}

/// Synthetic ECU data served by the diagnostic fabric.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModeledEcu {
    /// ECU identifier used in diagnostic requests.
    pub name: String,
    /// Human-facing ECU family.
    pub family: String,
    /// Modeled DTCs exposed by this ECU.
    pub dtcs: Vec<Dtc>,
    /// Modeled PID values exposed by this ECU.
    pub pids: Vec<PidValue>,
    /// Modeled freeze-frame records exposed by this ECU.
    pub freeze_frames: Vec<FreezeFrame>,
}

impl ModeledEcu {
    /// Builds a modeled ECU.
    pub fn new(
        name: impl Into<String>,
        family: impl Into<String>,
        dtcs: Vec<Dtc>,
        pids: Vec<PidValue>,
        freeze_frames: Vec<FreezeFrame>,
    ) -> Self {
        Self {
            name: name.into(),
            family: family.into(),
            dtcs,
            pids,
            freeze_frames,
        }
    }

    /// Returns a PID by identifier.
    pub fn pid(&self, pid: &str) -> Result<&PidValue> {
        self.pids
            .iter()
            .find(|value| value.pid == pid)
            .ok_or_else(|| Error::Eval(format!("unknown PID {pid} on ECU {}", self.name)))
    }

    pub(crate) fn inventory_expr(&self) -> Expr {
        Expr::Map(vec![
            string_field("ecu", &self.name),
            string_field("family", &self.family),
        ])
    }
}

/// Synthetic vehicle data served by a diagnostic fabric.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModeledVehicle {
    /// Modeled vehicle identity.
    pub vehicle: VehicleId,
    /// Synthetic VIN label.
    pub vin: String,
    /// Modeled ECUs installed in the vehicle.
    pub ecus: Vec<ModeledEcu>,
}

impl ModeledVehicle {
    /// Builds a modeled vehicle.
    pub fn new(vehicle: VehicleId, vin: impl Into<String>, ecus: Vec<ModeledEcu>) -> Self {
        Self {
            vehicle,
            vin: vin.into(),
            ecus,
        }
    }

    /// Builds the default synthetic vehicle fixture.
    pub fn fixture() -> Self {
        Self::new(
            VehicleId::new("fixture", "vehicle-alpha"),
            "VIN-FIXTURE-ALPHA",
            vec![fixture_engine_ecu(), fixture_body_ecu()],
        )
    }

    /// Returns one modeled ECU by name.
    pub fn ecu(&self, name: &str) -> Result<&ModeledEcu> {
        self.ecus
            .iter()
            .find(|ecu| ecu.name == name)
            .ok_or_else(|| Error::Eval(format!("unknown ECU {name}")))
    }

    /// Returns inventory records for every modeled ECU.
    pub fn inventory(&self) -> Vec<Expr> {
        self.ecus.iter().map(ModeledEcu::inventory_expr).collect()
    }
}

pub(crate) fn dtc_expr(dtc: &Dtc) -> Expr {
    Expr::Map(vec![
        string_field("system", &dtc.system),
        string_field("code", &dtc.code),
        string_field("description", &dtc.description),
        string_field("status-byte", &format!("0x{:02X}", dtc.status.to_byte())),
    ])
}

pub(crate) fn string_field(name: &str, value: &str) -> (Expr, Expr) {
    (
        Expr::Symbol(Symbol::new(name.to_owned())),
        Expr::String(value.to_owned()),
    )
}

fn fixture_engine_ecu() -> ModeledEcu {
    let dtcs = fixture_dtc_statuses()
        .into_iter()
        .zip([
            ("powertrain", "P0100", "modeled air-flow range fault"),
            ("powertrain", "P0301", "modeled cylinder-one misfire"),
        ])
        .map(|(status, (system, code, description))| {
            Dtc::with_status(system, code, description, DtcStatus::from_byte(status))
        })
        .collect::<Vec<_>>();
    let pids = vec![
        PidValue::new("rpm", "engine speed", "1840", "rpm"),
        PidValue::new("coolant", "coolant temperature", "88", "celsius"),
    ];
    let freeze_frames = vec![FreezeFrame::new(
        "P0301",
        vec![
            PidValue::new("rpm", "engine speed", "1720", "rpm"),
            PidValue::new("load", "calculated load", "42", "percent"),
        ],
    )];
    ModeledEcu::new("ME97", "engine-control", dtcs, pids, freeze_frames)
}

fn fixture_body_ecu() -> ModeledEcu {
    ModeledEcu::new(
        "BCM1",
        "body-control",
        vec![Dtc::with_status(
            "body",
            "B1000",
            "modeled lamp circuit status",
            DtcStatus::from_byte(0x08),
        )],
        vec![PidValue::new("voltage", "system voltage", "12.4", "volt")],
        vec![FreezeFrame::new(
            "B1000",
            vec![PidValue::new("voltage", "system voltage", "11.9", "volt")],
        )],
    )
}

fn fixture_dtc_statuses() -> Vec<u8> {
    let frame = UdsFrame::ReadDtcResponse {
        subfunction: 0x02,
        status_availability_mask: 0xFF,
        dtcs: vec![
            DtcFrame {
                raw_code: [0x01, 0x00, 0x00],
                status: 0x89,
            },
            DtcFrame {
                raw_code: [0x03, 0x01, 0x00],
                status: 0x0D,
            },
        ],
    };
    let bytes = encode_frame(&frame);
    match decode_frame(CodecId(0), &bytes).expect("fixture UDS frame decodes") {
        UdsFrame::ReadDtcResponse { dtcs, .. } => dtcs.into_iter().map(|dtc| dtc.status).collect(),
        _ => unreachable!("fixture frame is a DTC response"),
    }
}
