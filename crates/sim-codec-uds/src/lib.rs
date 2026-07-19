//! UDS and OBD-II byte-frame codec for SIM automotive diagnostics.
//!
//! The runtime codec symbol is `codec/uds`. It reads diagnostic bytes and emits
//! inert `Expr::Map` records for read-DID requests/responses, OBD-II mode
//! requests, and DTC responses. Encoding those records produces the original
//! bytes. DTC status bytes are decoded into the shared `auto/DtcStatus` shape
//! without adding fault descriptions or proprietary traces.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod codec;
mod expr;
mod frame;
mod status;

#[cfg(test)]
mod tests;

pub use codec::{UdsCodec, UdsCodecLib, install_uds_codec_lib, uds_codec_symbol};
pub use frame::{DtcFrame, UdsFrame, decode_frame, encode_frame};
pub use status::{decode_dtc_status, dtc_status_expr};

/// Cookbook recipes for this codec, embedded at build time.
pub static RECIPES: sim_cookbook::EmbeddedDir =
    include!(concat!(env!("OUT_DIR"), "/cookbook_recipes.rs"));
