# sim-auto

`sim-auto` holds SIM's automotive domain library. The core crate contributes
automotive citizens, capability names, lane metadata, transport descriptors, and
a loadable manifest library that other SIM crates can register without adding
automotive policy to the kernel.
The UDS codec crate turns diagnostic bytes into these same open records without
shipping vendor fault text or captured shop traces.

The repository keeps vehicle identities modeled and synthetic by default. Tests
guard committed fixtures against values shaped like VINs, Swedish plates, dealer
cookies, and vendor tokens.

## Crates

- `sim-lib-auto-core`: automotive citizens, capability names, site manifests,
  transport descriptors, and loadable runtime exports.
- `sim-codec-uds`: UDS and OBD-II byte-frame codec with DTC status-bit decode.

## Validation

```bash
cargo fmt --all --check
cargo run -p xtask -- check-file-sizes
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
cargo run -p xtask -- simdoc --check
```
