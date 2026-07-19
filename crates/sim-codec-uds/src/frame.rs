//! UDS and OBD-II byte-frame parsing.

use sim_kernel::{CodecId, Error, Result};

/// A decoded diagnostic frame supported by `codec/uds`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UdsFrame {
    /// UDS ReadDataByIdentifier request, service `0x22`.
    ReadDataByIdentifierRequest {
        /// Data identifier.
        did: u16,
    },
    /// UDS positive ReadDataByIdentifier response, service `0x62`.
    ReadDataByIdentifierResponse {
        /// Data identifier.
        did: u16,
        /// Opaque payload bytes.
        data: Vec<u8>,
    },
    /// OBD-II mode request with an optional PID.
    ObdModeRequest {
        /// OBD-II mode byte.
        mode: u8,
        /// Optional PID byte.
        pid: Option<u8>,
    },
    /// UDS ReadDTCInformation request, service `0x19`.
    ReadDtcRequest {
        /// Subfunction byte.
        subfunction: u8,
        /// Optional status mask byte.
        status_mask: Option<u8>,
    },
    /// UDS positive ReadDTCInformation response, service `0x59`.
    ReadDtcResponse {
        /// Response subfunction byte.
        subfunction: u8,
        /// Status availability mask byte.
        status_availability_mask: u8,
        /// Decoded DTC entries.
        dtcs: Vec<DtcFrame>,
    },
}

/// A raw DTC plus its status byte.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DtcFrame {
    /// Three raw UDS DTC bytes.
    pub raw_code: [u8; 3],
    /// Standard UDS DTC status byte.
    pub status: u8,
}

/// Decodes supported diagnostic bytes into a frame.
pub fn decode_frame(codec: CodecId, bytes: &[u8]) -> Result<UdsFrame> {
    let Some((&service, rest)) = bytes.split_first() else {
        return Err(codec_error(codec, "empty UDS frame"));
    };
    match service {
        0x22 => read_did_request(codec, rest),
        0x62 => read_did_response(codec, rest),
        0x19 => read_dtc_request(codec, rest),
        0x59 => read_dtc_response(codec, rest),
        mode if is_obd_mode(mode) => obd_mode_request(codec, mode, rest),
        _ => Err(codec_error(
            codec,
            format!("unsupported UDS/OBD-II service 0x{service:02X}"),
        )),
    }
}

/// Encodes a supported diagnostic frame to bytes.
pub fn encode_frame(frame: &UdsFrame) -> Vec<u8> {
    match frame {
        UdsFrame::ReadDataByIdentifierRequest { did } => {
            let [high, low] = did.to_be_bytes();
            vec![0x22, high, low]
        }
        UdsFrame::ReadDataByIdentifierResponse { did, data } => {
            let [high, low] = did.to_be_bytes();
            let mut bytes = vec![0x62, high, low];
            bytes.extend(data);
            bytes
        }
        UdsFrame::ObdModeRequest { mode, pid } => {
            let mut bytes = vec![*mode];
            if let Some(pid) = pid {
                bytes.push(*pid);
            }
            bytes
        }
        UdsFrame::ReadDtcRequest {
            subfunction,
            status_mask,
        } => {
            let mut bytes = vec![0x19, *subfunction];
            if let Some(mask) = status_mask {
                bytes.push(*mask);
            }
            bytes
        }
        UdsFrame::ReadDtcResponse {
            subfunction,
            status_availability_mask,
            dtcs,
        } => {
            let mut bytes = vec![0x59, *subfunction, *status_availability_mask];
            for dtc in dtcs {
                bytes.extend(dtc.raw_code);
                bytes.push(dtc.status);
            }
            bytes
        }
    }
}

fn read_did_request(codec: CodecId, rest: &[u8]) -> Result<UdsFrame> {
    if rest.len() != 2 {
        return Err(codec_error(
            codec,
            "read-DID request must be exactly three bytes",
        ));
    }
    Ok(UdsFrame::ReadDataByIdentifierRequest {
        did: u16::from_be_bytes([rest[0], rest[1]]),
    })
}

fn read_did_response(codec: CodecId, rest: &[u8]) -> Result<UdsFrame> {
    if rest.len() < 2 {
        return Err(codec_error(
            codec,
            "read-DID response must include a two-byte DID",
        ));
    }
    Ok(UdsFrame::ReadDataByIdentifierResponse {
        did: u16::from_be_bytes([rest[0], rest[1]]),
        data: rest[2..].to_vec(),
    })
}

fn obd_mode_request(codec: CodecId, mode: u8, rest: &[u8]) -> Result<UdsFrame> {
    match rest {
        [] => Ok(UdsFrame::ObdModeRequest { mode, pid: None }),
        [pid] => Ok(UdsFrame::ObdModeRequest {
            mode,
            pid: Some(*pid),
        }),
        _ => Err(codec_error(
            codec,
            "OBD-II mode request supports at most one PID byte",
        )),
    }
}

fn read_dtc_request(codec: CodecId, rest: &[u8]) -> Result<UdsFrame> {
    match rest {
        [subfunction] => Ok(UdsFrame::ReadDtcRequest {
            subfunction: *subfunction,
            status_mask: None,
        }),
        [subfunction, mask] => Ok(UdsFrame::ReadDtcRequest {
            subfunction: *subfunction,
            status_mask: Some(*mask),
        }),
        _ => Err(codec_error(
            codec,
            "read-DTC request must be service, subfunction, and optional status mask",
        )),
    }
}

fn read_dtc_response(codec: CodecId, rest: &[u8]) -> Result<UdsFrame> {
    if rest.len() < 2 {
        return Err(codec_error(
            codec,
            "read-DTC response must include subfunction and availability mask",
        ));
    }
    let payload = &rest[2..];
    if !payload.len().is_multiple_of(4) {
        return Err(codec_error(
            codec,
            "read-DTC response DTC payload must be four-byte entries",
        ));
    }
    let dtcs = payload
        .chunks_exact(4)
        .map(|chunk| DtcFrame {
            raw_code: [chunk[0], chunk[1], chunk[2]],
            status: chunk[3],
        })
        .collect();
    Ok(UdsFrame::ReadDtcResponse {
        subfunction: rest[0],
        status_availability_mask: rest[1],
        dtcs,
    })
}

fn is_obd_mode(service: u8) -> bool {
    matches!(service, 0x01..=0x0A)
}

pub(crate) fn codec_error(codec: CodecId, message: impl Into<String>) -> Error {
    Error::CodecError {
        codec,
        message: message.into(),
    }
}
