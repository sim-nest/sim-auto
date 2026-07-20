use std::sync::Arc;

use sim_kernel::{Error, EvalFabric, Expr, Lib};
use sim_lib_auto_core::{AUTO_FLASH, ModeledFlashSession};

use crate::test_support::{cx_with, export_symbol, expr_text, request};
use crate::{
    AutoVendorLib, ModeledVendorBridge, VendorEffectClass, VendorSiteFabric,
    auto_vendor_site_symbol, autotuner_manifest, flash_site_cassettes, flash_site_manifests,
    manifest_operation, vendor_irreversible_request_expr, vendor_request_expr,
};

#[test]
fn autotuner_manifest_declares_flash_ops_and_modeled_transport() {
    let manifests = flash_site_manifests();
    assert_eq!(manifests.len(), 1);
    let manifest = autotuner_manifest();
    assert_eq!(manifest.site, "autotuner");
    assert_eq!(manifest.lanes, vec!["read", "flash"]);
    assert!(
        manifest
            .transports
            .iter()
            .all(|transport| transport.ends_with("-modeled") || transport == "cassette")
    );

    let read = manifest_operation(&manifest, "flash/read-ecu").unwrap();
    assert_eq!(read.effect, VendorEffectClass::Pure);
    assert_eq!(read.capability.as_str(), AUTO_FLASH);
    let backup = manifest_operation(&manifest, "flash/backup-stock").unwrap();
    assert_eq!(backup.effect, VendorEffectClass::Reversible);
    let restore = manifest_operation(&manifest, "flash/restore-stock").unwrap();
    assert_eq!(restore.effect, VendorEffectClass::Reversible);
    let write = manifest_operation(&manifest, "flash/write").unwrap();
    assert_eq!(write.effect, VendorEffectClass::Irreversible);
    assert_eq!(write.capability.as_str(), AUTO_FLASH);

    let lib = AutoVendorLib::modeled(manifests);
    let export = auto_vendor_site_symbol(&manifest).to_string();
    assert!(
        lib.manifest()
            .exports
            .iter()
            .map(export_symbol)
            .any(|item| item == export)
    );
}

#[test]
fn autotuner_flash_write_requires_content_key_warrant_and_human_gate() {
    let bridge = Arc::new(ModeledVendorBridge::with_cassettes(flash_site_cassettes()));
    let mut cx = cx_with(&[AUTO_FLASH]);
    let fabric = VendorSiteFabric::new(autotuner_manifest(), bridge.clone());

    let missing_artifact = fabric.realize(
        &mut cx,
        request(
            vendor_request_expr("flash/write", Expr::Bytes(vec![0xaa])),
            &[AUTO_FLASH],
        ),
    );
    assert!(matches!(missing_artifact, Err(Error::Eval(message)) if message.contains("reversal")));

    let missing_key = fabric.realize(
        &mut cx,
        request(
            vendor_irreversible_request_expr(
                "flash/write",
                Expr::Bytes(vec![0xaa]),
                Expr::String("stock-map".to_owned()),
                "warrant-1",
            ),
            &[AUTO_FLASH],
        ),
    );
    assert!(matches!(missing_key, Err(Error::Eval(message)) if message.contains("content-key")));
    assert!(fabric.gate_ledger().records().unwrap().is_empty());
    assert_eq!(cx.effect_ledger().records().len(), 0);
    assert!(bridge.calls().unwrap().is_empty());
}

#[test]
fn modeled_autotuner_backup_flash_restore_returns_stock_bytes() {
    let bridge = Arc::new(ModeledVendorBridge::with_cassettes(flash_site_cassettes()));
    let mut cx = cx_with(&[AUTO_FLASH]);
    let fabric = VendorSiteFabric::new(autotuner_manifest(), bridge.clone());
    let stock = vec![0x01, 0x02, 0x03, 0x04];
    let tuned = vec![0x10, 0x20, 0x30, 0x40];
    let mut session = ModeledFlashSession::new("DME", stock.clone());

    let read = fabric
        .realize(
            &mut cx,
            request(
                vendor_request_expr("flash/read-ecu", Expr::Bytes(session.read_ecu().to_vec())),
                &[AUTO_FLASH],
            ),
        )
        .unwrap();
    assert!(expr_text(&mut cx, &read.value).contains("modeled ECU read"));

    let backup = session.backup_stock();
    let backup_reply = fabric
        .realize(
            &mut cx,
            request(
                vendor_request_expr("flash/backup-stock", backup.reversal_artifact()),
                &[AUTO_FLASH],
            ),
        )
        .unwrap();
    assert!(expr_text(&mut cx, &backup_reply.value).contains("stock backup"));

    fabric
        .realize(
            &mut cx,
            request(
                vendor_irreversible_request_expr(
                    "flash/write",
                    Expr::Bytes(tuned.clone()),
                    backup.reversal_artifact(),
                    "warrant-flash-1",
                ),
                &[AUTO_FLASH],
            ),
        )
        .unwrap();
    session.flash(tuned.clone(), &backup).unwrap();
    assert_eq!(session.read_ecu(), tuned.as_slice());

    let restore_reply = fabric
        .realize(
            &mut cx,
            request(
                vendor_request_expr("flash/restore-stock", backup.reversal_artifact()),
                &[AUTO_FLASH],
            ),
        )
        .unwrap();
    assert!(expr_text(&mut cx, &restore_reply.value).contains("stock restore"));
    let restored = session.restore(&backup).unwrap();
    assert_eq!(restored, stock);
    assert_eq!(session.read_ecu(), stock.as_slice());

    let records = fabric.gate_ledger().records().unwrap();
    assert_eq!(records.len(), 3);
    assert_eq!(records[0].operation, "flash/backup-stock");
    assert_eq!(records[1].operation, "flash/write");
    assert_eq!(records[1].effect, VendorEffectClass::Irreversible);
    assert_eq!(
        records[1].reversal_content_key.as_deref(),
        Some(backup.content_key.as_str())
    );
    assert_eq!(records[2].operation, "flash/restore-stock");
    assert_eq!(cx.effect_ledger().records().len(), 3);
    assert_eq!(bridge.calls().unwrap().len(), 4);
}
