//! Expression projection for UDS frames.

use sim_kernel::{CodecId, Expr, NumberLiteral, Result, Symbol};

use crate::{
    DtcFrame, UdsFrame,
    frame::codec_error,
    status::{dtc_status_expr, status_byte_from_expr},
};

pub(crate) fn frame_to_expr(frame: &UdsFrame) -> Expr {
    match frame {
        UdsFrame::ReadDataByIdentifierRequest { did } => Expr::Map(vec![
            symbol_field("kind", "uds", "read-did-request"),
            number_field("service", 0x22),
            number_field("did", u64::from(*did)),
            string_field("name", did_name(*did)),
        ]),
        UdsFrame::ReadDataByIdentifierResponse { did, data } => Expr::Map(vec![
            symbol_field("kind", "uds", "read-did-response"),
            number_field("service", 0x62),
            number_field("did", u64::from(*did)),
            string_field("name", did_name(*did)),
            bytes_field("data", data.clone()),
        ]),
        UdsFrame::ObdModeRequest { mode, pid } => {
            let mut entries = vec![
                symbol_field("kind", "obd", "mode-request"),
                number_field("mode", u64::from(*mode)),
                string_field("mode-name", obd_mode_name(*mode)),
            ];
            if let Some(pid) = pid {
                entries.push(number_field("pid", u64::from(*pid)));
            }
            Expr::Map(entries)
        }
        UdsFrame::ReadDtcRequest {
            subfunction,
            status_mask,
        } => {
            let mut entries = vec![
                symbol_field("kind", "uds", "read-dtc-request"),
                number_field("service", 0x19),
                number_field("subfunction", u64::from(*subfunction)),
            ];
            if let Some(mask) = status_mask {
                entries.push(number_field("status-mask", u64::from(*mask)));
            }
            Expr::Map(entries)
        }
        UdsFrame::ReadDtcResponse {
            subfunction,
            status_availability_mask,
            dtcs,
        } => Expr::Map(vec![
            symbol_field("kind", "uds", "read-dtc-response"),
            number_field("service", 0x59),
            number_field("subfunction", u64::from(*subfunction)),
            number_field(
                "status-availability-mask",
                u64::from(*status_availability_mask),
            ),
            (
                key("dtcs"),
                Expr::List(dtcs.iter().map(dtc_frame_to_expr).collect()),
            ),
        ]),
    }
}

pub(crate) fn expr_to_frame(codec: CodecId, expr: &Expr) -> Result<UdsFrame> {
    let Expr::Map(entries) = expr else {
        return Err(codec_error(codec, "UDS expression must be a map"));
    };
    match required_symbol(entries, "kind", codec)?
        .as_qualified_str()
        .as_str()
    {
        "uds/read-did-request" => Ok(UdsFrame::ReadDataByIdentifierRequest {
            did: required_u16(entries, "did", codec)?,
        }),
        "uds/read-did-response" => Ok(UdsFrame::ReadDataByIdentifierResponse {
            did: required_u16(entries, "did", codec)?,
            data: required_bytes(entries, "data", codec)?.to_vec(),
        }),
        "obd/mode-request" => Ok(UdsFrame::ObdModeRequest {
            mode: required_u8(entries, "mode", codec)?,
            pid: optional_u8(entries, "pid", codec)?,
        }),
        "uds/read-dtc-request" => Ok(UdsFrame::ReadDtcRequest {
            subfunction: required_u8(entries, "subfunction", codec)?,
            status_mask: optional_u8(entries, "status-mask", codec)?,
        }),
        "uds/read-dtc-response" => Ok(UdsFrame::ReadDtcResponse {
            subfunction: required_u8(entries, "subfunction", codec)?,
            status_availability_mask: required_u8(entries, "status-availability-mask", codec)?,
            dtcs: required_dtc_list(entries, codec)?,
        }),
        other => Err(codec_error(
            codec,
            format!("unsupported UDS expression kind {other}"),
        )),
    }
}

fn dtc_frame_to_expr(frame: &DtcFrame) -> Expr {
    let code = dtc_code_text(frame.raw_code);
    Expr::Map(vec![
        symbol_field("class", "auto", "Dtc"),
        string_field("system", dtc_system_name(frame.raw_code)),
        string_field("code", code),
        string_field("description", "status-only diagnostic"),
        bytes_field("raw-code", frame.raw_code.to_vec()),
        number_field("status-byte", u64::from(frame.status)),
        (key("status"), dtc_status_expr(frame.status)),
    ])
}

fn required_dtc_list(entries: &[(Expr, Expr)], codec: CodecId) -> Result<Vec<DtcFrame>> {
    let Expr::List(items) = required_field(entries, "dtcs", codec)? else {
        return Err(codec_error(codec, "dtcs field must be a list"));
    };
    items
        .iter()
        .map(|item| dtc_frame_from_expr(codec, item))
        .collect()
}

fn dtc_frame_from_expr(codec: CodecId, expr: &Expr) -> Result<DtcFrame> {
    let Expr::Map(entries) = expr else {
        return Err(codec_error(codec, "DTC entry must be a map"));
    };
    let raw_code = required_raw_code(entries, codec)?;
    let status = match optional_field(entries, "status") {
        Some(status) => status_byte_from_expr(status)
            .ok_or_else(|| codec_error(codec, "status field must be a DTC status map"))?,
        None => required_u8(entries, "status-byte", codec)?,
    };
    Ok(DtcFrame { raw_code, status })
}

fn required_raw_code(entries: &[(Expr, Expr)], codec: CodecId) -> Result<[u8; 3]> {
    let bytes = required_bytes(entries, "raw-code", codec)?;
    let raw_code: [u8; 3] = bytes
        .try_into()
        .map_err(|_| codec_error(codec, "DTC raw-code field must contain exactly three bytes"))?;
    Ok(raw_code)
}

fn required_field<'a>(entries: &'a [(Expr, Expr)], name: &str, codec: CodecId) -> Result<&'a Expr> {
    optional_field(entries, name)
        .ok_or_else(|| codec_error(codec, format!("missing UDS field {name}")))
}

fn optional_field<'a>(entries: &'a [(Expr, Expr)], name: &str) -> Option<&'a Expr> {
    entries
        .iter()
        .find_map(|(field, value)| (field == &key(name)).then_some(value))
}

fn required_symbol(entries: &[(Expr, Expr)], name: &str, codec: CodecId) -> Result<Symbol> {
    match required_field(entries, name, codec)? {
        Expr::Symbol(symbol) => Ok(symbol.clone()),
        _ => Err(codec_error(codec, format!("{name} field must be a symbol"))),
    }
}

fn required_u8(entries: &[(Expr, Expr)], name: &str, codec: CodecId) -> Result<u8> {
    let value = required_u64(entries, name, codec)?;
    u8::try_from(value).map_err(|_| codec_error(codec, format!("{name} field must fit in u8")))
}

fn optional_u8(entries: &[(Expr, Expr)], name: &str, codec: CodecId) -> Result<Option<u8>> {
    optional_field(entries, name)
        .map(|expr| u64_from_expr(expr, codec, name))
        .transpose()?
        .map(|value| {
            u8::try_from(value)
                .map_err(|_| codec_error(codec, format!("{name} field must fit in u8")))
        })
        .transpose()
}

fn required_u16(entries: &[(Expr, Expr)], name: &str, codec: CodecId) -> Result<u16> {
    let value = required_u64(entries, name, codec)?;
    u16::try_from(value).map_err(|_| codec_error(codec, format!("{name} field must fit in u16")))
}

fn required_u64(entries: &[(Expr, Expr)], name: &str, codec: CodecId) -> Result<u64> {
    u64_from_expr(required_field(entries, name, codec)?, codec, name)
}

fn u64_from_expr(expr: &Expr, codec: CodecId, name: &str) -> Result<u64> {
    let Expr::Number(number) = expr else {
        return Err(codec_error(codec, format!("{name} field must be a number")));
    };
    number
        .canonical
        .parse()
        .map_err(|_| codec_error(codec, format!("{name} field must be an unsigned integer")))
}

fn required_bytes<'a>(entries: &'a [(Expr, Expr)], name: &str, codec: CodecId) -> Result<&'a [u8]> {
    match required_field(entries, name, codec)? {
        Expr::Bytes(bytes) => Ok(bytes),
        _ => Err(codec_error(codec, format!("{name} field must be bytes"))),
    }
}

fn symbol_field(key_name: &str, namespace: &str, name: &str) -> (Expr, Expr) {
    (
        key(key_name),
        Expr::Symbol(Symbol::qualified(namespace, name)),
    )
}

fn number_field(name: &str, value: u64) -> (Expr, Expr) {
    (
        key(name),
        Expr::Number(NumberLiteral {
            domain: Symbol::qualified("numbers", "u64"),
            canonical: value.to_string(),
        }),
    )
}

fn string_field(name: &str, value: impl Into<String>) -> (Expr, Expr) {
    (key(name), Expr::String(value.into()))
}

fn bytes_field(name: &str, value: Vec<u8>) -> (Expr, Expr) {
    (key(name), Expr::Bytes(value))
}

fn key(name: &str) -> Expr {
    Expr::Symbol(Symbol::new(name))
}

fn did_name(did: u16) -> &'static str {
    match did {
        0xF190 => "vin",
        0xF187 => "part-number",
        0xF18C => "serial-number",
        _ => "unknown-did",
    }
}

fn obd_mode_name(mode: u8) -> &'static str {
    match mode {
        0x01 => "current-data",
        0x02 => "freeze-frame",
        0x03 => "stored-dtc",
        0x04 => "clear-dtc",
        0x07 => "pending-dtc",
        0x09 => "vehicle-information",
        _ => "obd-mode",
    }
}

fn dtc_system_name(raw_code: [u8; 3]) -> &'static str {
    match raw_code[0] >> 6 {
        0 => "powertrain",
        1 => "chassis",
        2 => "body",
        _ => "network",
    }
}

fn dtc_code_text(raw_code: [u8; 3]) -> String {
    let letter = match raw_code[0] >> 6 {
        0 => 'P',
        1 => 'C',
        2 => 'B',
        _ => 'U',
    };
    let first_digit = (raw_code[0] >> 4) & 0x03;
    format!(
        "{letter}{first_digit:X}{:X}{:02X}{:02X}",
        raw_code[0] & 0x0F,
        raw_code[1],
        raw_code[2]
    )
}
