//! Bootloader-backed automotive command surface.
//!
//! The package exports `cli/main/auto` from a host-registered library and ships
//! a `sim-auto` binary that boots through `sim-run-core::Bootloader`. The command
//! output is modeled and hardware-free; real vendor placements stay host-owned.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod command;
mod entrypoint;

#[cfg(test)]
mod tests;

pub use command::{auto_help, render_auto_command};
pub use entrypoint::{
    AUTO_HOST_LIB, AUTO_VERB, AutoCliLib, auto_boot_args, auto_bootloader, auto_entrypoint_symbol,
};
