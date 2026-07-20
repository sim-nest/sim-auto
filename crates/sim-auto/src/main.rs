use std::process::ExitCode;

fn main() -> ExitCode {
    let args = sim_auto::auto_boot_args(std::env::args_os());
    match sim_auto::auto_bootloader().run(args) {
        Ok(0) => ExitCode::SUCCESS,
        Ok(code) => ExitCode::from(code as u8),
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}
