use std::sync::Arc;

use sim_codec::{Input, Output, decode_with_codec, encode_with_codec};
use sim_kernel::{Cx, DefaultFactory, EagerPolicy, EncodeOptions, Expr, ReadPolicy, Symbol};

use crate::{decode_dtc_status, install_uds_codec_lib, uds_codec_symbol};

#[test]
fn read_did_request_round_trips() {
    let mut cx = test_context();
    let expr = decode(&mut cx, &[0x22, 0xF1, 0x90]);

    assert_eq!(
        symbol_value(&expr, "kind"),
        Some(Symbol::qualified("uds", "read-did-request"))
    );
    assert_eq!(string_value(&expr, "name"), Some("vin"));
    assert_eq!(encoded(&mut cx, &expr), vec![0x22, 0xF1, 0x90]);
}

#[test]
fn read_did_response_round_trips_with_bytes() {
    let mut cx = test_context();
    let frame = [0x62, 0xF1, 0x90, b'S', b'I', b'M'];
    let expr = decode(&mut cx, &frame);

    assert_eq!(bytes_value(&expr, "data"), Some(&b"SIM"[..]));
    assert_eq!(encoded(&mut cx, &expr), frame);
}

#[test]
fn obd_mode_request_round_trips() {
    let mut cx = test_context();
    let expr = decode(&mut cx, &[0x01, 0x0C]);

    assert_eq!(
        symbol_value(&expr, "kind"),
        Some(Symbol::qualified("obd", "mode-request"))
    );
    assert_eq!(string_value(&expr, "mode-name"), Some("current-data"));
    assert_eq!(encoded(&mut cx, &expr), vec![0x01, 0x0C]);
}

#[test]
fn dtc_response_decodes_status_without_fault_text() {
    let mut cx = test_context();
    let expr = decode(&mut cx, &[0x59, 0x02, 0xFF, 0x03, 0x01, 0x00, 0x89]);
    let dtc = first_dtc(&expr);

    assert_eq!(string_value(dtc, "code"), Some("P030100"));
    assert_eq!(
        string_value(dtc, "description"),
        Some("status-only diagnostic")
    );
    assert_eq!(number_value(dtc, "status-byte"), Some(0x89));
    let status = map_value(dtc, "status").expect("status field");
    assert_eq!(bool_value(status, "test_failed"), Some(true));
    assert_eq!(bool_value(status, "confirmed"), Some(true));
    assert_eq!(bool_value(status, "warning_indicator"), Some(true));
    assert_eq!(
        encoded(&mut cx, &expr),
        vec![0x59, 0x02, 0xFF, 0x03, 0x01, 0x00, 0x89]
    );
}

#[test]
fn status_byte_round_trips_through_core_status() {
    let status = decode_dtc_status(0xCA);

    assert!(!status.test_failed);
    assert!(status.test_failed_this_operation_cycle);
    assert!(!status.pending);
    assert!(status.confirmed);
    assert!(status.warning_indicator);
    assert_eq!(status.to_byte(), 0xCA);
}

#[test]
fn rejects_unknown_service() {
    let mut cx = test_context();
    let err = decode_with_codec(
        &mut cx,
        &uds_codec_symbol(),
        Input::Bytes(vec![0xAA]),
        ReadPolicy::default(),
    )
    .unwrap_err();

    assert!(
        err.to_string().contains("unsupported UDS/OBD-II service"),
        "{err}"
    );
}

fn test_context() -> Cx {
    let mut cx = Cx::new(Arc::new(EagerPolicy), Arc::new(DefaultFactory));
    install_uds_codec_lib(&mut cx).unwrap();
    cx
}

fn decode(cx: &mut Cx, bytes: &[u8]) -> Expr {
    decode_with_codec(
        cx,
        &uds_codec_symbol(),
        Input::Bytes(bytes.to_vec()),
        ReadPolicy::default(),
    )
    .unwrap()
}

fn encoded(cx: &mut Cx, expr: &Expr) -> Vec<u8> {
    match encode_with_codec(cx, &uds_codec_symbol(), expr, EncodeOptions::default()).unwrap() {
        Output::Bytes(bytes) => bytes,
        Output::Text(text) => text.into_bytes(),
    }
}

fn first_dtc(expr: &Expr) -> &Expr {
    match map_value(expr, "dtcs").expect("dtcs field") {
        Expr::List(items) => items.first().expect("first dtc"),
        other => panic!("expected dtcs list, got {other:?}"),
    }
}

fn map_value<'a>(expr: &'a Expr, name: &str) -> Option<&'a Expr> {
    let Expr::Map(entries) = expr else {
        return None;
    };
    entries
        .iter()
        .find_map(|(key, value)| (key == &Expr::Symbol(Symbol::new(name))).then_some(value))
}

fn symbol_value(expr: &Expr, name: &str) -> Option<Symbol> {
    match map_value(expr, name)? {
        Expr::Symbol(symbol) => Some(symbol.clone()),
        _ => None,
    }
}

fn string_value<'a>(expr: &'a Expr, name: &str) -> Option<&'a str> {
    match map_value(expr, name)? {
        Expr::String(value) => Some(value.as_str()),
        _ => None,
    }
}

fn bytes_value<'a>(expr: &'a Expr, name: &str) -> Option<&'a [u8]> {
    match map_value(expr, name)? {
        Expr::Bytes(value) => Some(value.as_slice()),
        _ => None,
    }
}

fn number_value(expr: &Expr, name: &str) -> Option<u64> {
    match map_value(expr, name)? {
        Expr::Number(value) => value.canonical.parse().ok(),
        _ => None,
    }
}

fn bool_value(expr: &Expr, name: &str) -> Option<bool> {
    match map_value(expr, name)? {
        Expr::Bool(value) => Some(*value),
        _ => None,
    }
}
