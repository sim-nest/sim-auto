//! Runtime codec registration.

use std::sync::Arc;

use sim_codec::{DecodeBudget, Decoder, DomainCodecLib, Encoder, Input, Output, ReadCx};
use sim_kernel::{CodecId, Cx, Expr, Lib, LibManifest, Linker, LoadCx, Result, Symbol, WriteCx};

use crate::{
    expr::{expr_to_frame, frame_to_expr},
    frame::{decode_frame, encode_frame},
};

/// Decoder and encoder for the `codec/uds` byte surface.
#[derive(Clone, Copy, Debug, Default)]
pub struct UdsCodec;

impl Decoder for UdsCodec {
    fn decode(&self, cx: &mut ReadCx<'_>, input: Input) -> Result<Expr> {
        let bytes = match input {
            Input::Bytes(bytes) => bytes,
            Input::Text(text) => text.into_bytes(),
        };
        let budget = DecodeBudget::new(cx.limits);
        budget.check_input_bytes(cx.codec, bytes.len())?;
        decode_frame(cx.codec, &bytes).map(|frame| frame_to_expr(&frame))
    }
}

impl Encoder for UdsCodec {
    fn encode(&self, cx: &mut WriteCx<'_>, expr: &Expr) -> Result<Output> {
        let frame = expr_to_frame(cx.codec, expr)?;
        Ok(Output::Bytes(encode_frame(&frame)))
    }
}

/// Host-registered library that installs `codec/uds`.
pub struct UdsCodecLib {
    symbol: Symbol,
    codec_id: CodecId,
}

impl UdsCodecLib {
    /// Creates a UDS codec lib for the given runtime codec id.
    pub fn new(id: CodecId) -> Self {
        Self {
            symbol: uds_codec_symbol(),
            codec_id: id,
        }
    }

    fn domain_lib(&self) -> DomainCodecLib {
        DomainCodecLib::new(
            self.symbol.clone(),
            self.codec_id,
            Arc::new(UdsCodec),
            Arc::new(UdsCodec),
            Symbol::qualified("codec", "UdsFrame"),
        )
    }
}

impl Lib for UdsCodecLib {
    fn manifest(&self) -> LibManifest {
        self.domain_lib().manifest()
    }

    fn load(&self, cx: &mut LoadCx, linker: &mut Linker<'_>) -> Result<()> {
        self.domain_lib().load(cx, linker)
    }
}

/// Returns the runtime symbol for the UDS codec.
pub fn uds_codec_symbol() -> Symbol {
    Symbol::qualified("codec", "uds")
}

/// Installs `codec/uds` into a context once.
pub fn install_uds_codec_lib(cx: &mut Cx) -> Result<()> {
    if cx.registry().lib(&uds_codec_symbol()).is_some() {
        return Ok(());
    }
    let id = cx.registry_mut().fresh_codec_id();
    cx.load_lib(&UdsCodecLib::new(id)).map(|_| ())
}
