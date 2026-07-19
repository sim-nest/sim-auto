use std::sync::Arc;

use sim_kernel::{Cx, DefaultFactory, NoopEvalPolicy};
use sim_lib_auto_core::{auto_caps_symbol, auto_citizen_registry, install_auto_core_lib};

fn main() -> sim_kernel::Result<()> {
    let mut cx = Cx::new(Arc::new(NoopEvalPolicy), Arc::new(DefaultFactory));
    let registry = auto_citizen_registry();
    install_auto_core_lib(&mut cx)?;

    println!("registered {} auto citizens", registry.len());
    println!("exported {}", auto_caps_symbol());
    Ok(())
}
