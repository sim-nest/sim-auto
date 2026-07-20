//! Modeled `auto` command rendering.

use sim_kernel::{Error, Result};
use sim_lib_view_bay::{BayState, all_modeled_intents, bay_scene, dtc_status_label};

/// Renders the modeled automotive command output for bootloader handoff args.
pub fn render_auto_command(args: &[String]) -> Result<String> {
    let args = command_args(args);
    match args.first().map(String::as_str) {
        None | Some("bay") | Some("scene") => bay_output(),
        Some("diag") | Some("diagnostics") => diag_output(&args[1..]),
        Some("intents") => intents_output(),
        Some("help") | Some("--help") | Some("-h") => Ok(auto_help()),
        Some(other) => Err(Error::Eval(format!(
            "unknown auto command '{other}'; run 'auto help'"
        ))),
    }
}

/// Returns help text for the modeled automotive command surface.
pub fn auto_help() -> String {
    [
        "auto commands:",
        "  auto diag --vehicle MODELED-SE-1 --market SE --site modeled",
        "  auto bay",
        "  auto intents",
    ]
    .join("\n")
}

fn command_args(args: &[String]) -> Vec<String> {
    match args.first().map(String::as_str) {
        Some("auto") => args.iter().skip(1).cloned().collect(),
        _ => args.to_vec(),
    }
}

fn diag_output(args: &[String]) -> Result<String> {
    let opts = DiagOptions::parse(args)?;
    if opts.site != "modeled" {
        return Err(Error::Eval(
            "public auto command only serves the modeled site".to_owned(),
        ));
    }
    let state = BayState::modeled_mercedes()?;
    let primary = state
        .primary_dtc()
        .ok_or_else(|| Error::Eval("modeled bay state has no DTC".to_owned()))?;
    Ok(format!(
        "{} {} {}\nvehicle {}\nmarket {}\nsite {}\nscene auto/bay",
        primary.dtc.code,
        dtc_status_label(&primary.dtc.status),
        primary.ecu,
        opts.vehicle,
        opts.market,
        state.site.site
    ))
}

fn bay_output() -> Result<String> {
    let state = BayState::modeled_mercedes()?;
    let _scene = bay_scene(&state)?;
    let primary = state
        .primary_dtc()
        .ok_or_else(|| Error::Eval("modeled bay state has no DTC".to_owned()))?;
    Ok(format!(
        "auto/bay\nvehicle {}/{}\nsite {}\nprimary {} {} {}\nparts {}\ncoding {}\nflash gate {}\nledger events {}",
        state.vehicle.namespace,
        state.vehicle.key,
        state.site.site,
        primary.dtc.code,
        dtc_status_label(&primary.dtc.status),
        primary.ecu,
        state.parts_cart.len(),
        state.coding_status.state,
        state.flash_gate_status.state,
        state.ledger_timeline.len()
    ))
}

fn intents_output() -> Result<String> {
    let state = BayState::modeled_mercedes()?;
    let intents = all_modeled_intents(&state, 1)?;
    Ok(format!("auto intents {}", intents.len()))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DiagOptions {
    vehicle: String,
    market: String,
    site: String,
}

impl DiagOptions {
    fn parse(args: &[String]) -> Result<Self> {
        let mut vehicle = "MODELED-SE-1".to_owned();
        let mut market = "SE".to_owned();
        let mut site = "modeled".to_owned();
        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--vehicle" | "--plate" => vehicle = required_value(arg, iter.next())?,
                "--market" => market = required_value(arg, iter.next())?,
                "--site" => site = required_value(arg, iter.next())?,
                other => {
                    return Err(Error::Eval(format!(
                        "unknown auto diag option '{other}'; run 'auto help'"
                    )));
                }
            }
        }
        Ok(Self {
            vehicle,
            market,
            site,
        })
    }
}

fn required_value(flag: &str, value: Option<&String>) -> Result<String> {
    value
        .cloned()
        .ok_or_else(|| Error::Eval(format!("missing value for {flag}")))
}
