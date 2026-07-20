use sim_kernel::{Export, Lib};

use crate::{
    AutoCliLib, auto_boot_args, auto_bootloader, auto_entrypoint_symbol, render_auto_command,
};

#[test]
fn auto_cli_lib_exports_cli_main_auto() {
    let lib = AutoCliLib::new();
    let manifest = lib.manifest();

    assert!(manifest.exports.iter().any(|export| matches!(
        export,
        Export::Function { symbol, .. } if symbol == &auto_entrypoint_symbol()
    )));
}

#[test]
fn render_diag_contains_modeled_primary_dtc() {
    let output = render_auto_command(&[
        "auto".to_owned(),
        "diag".to_owned(),
        "--vehicle".to_owned(),
        "MODELED-SE-1".to_owned(),
        "--market".to_owned(),
        "SE".to_owned(),
        "--site".to_owned(),
        "modeled".to_owned(),
    ])
    .expect("diag renders");

    assert!(output.contains("P0301 confirmed ME97"));
}

#[test]
fn auto_bootloader_runs_modeled_diag() {
    let code = auto_bootloader()
        .run([
            "sim-auto",
            "auto",
            "diag",
            "--vehicle",
            "MODELED-SE-1",
            "--market",
            "SE",
            "--site",
            "modeled",
        ])
        .expect("bootloader runs");

    assert_eq!(code, 0);
}

#[test]
fn direct_binary_args_insert_auto_verb() {
    let args = auto_boot_args(["sim-auto", "diag"]);
    let text = args
        .iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    assert_eq!(text, ["sim-auto", "auto", "diag"]);
}
