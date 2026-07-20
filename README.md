# sim-auto

`sim-auto` holds SIM's automotive domain library. The core crate contributes
automotive citizens, capability names, lane metadata, transport descriptors, and
a loadable manifest library that other SIM crates can register without adding
automotive policy to the kernel.
The UDS codec crate turns diagnostic bytes into these same open records without
shipping vendor fault text or captured shop traces.
The diagnostic fabric crate serves modeled ECU sites through the kernel fabric
contract and records replayable synthetic diagnostic replies.
The vehicle identity crate resolves synthetic plate and VIN labels into shared
vehicle identities while advertising host-owned data-source contracts.
The repair information crate ranks modeled workshop procedures and projects them
through the shared document view surface.
The parts crate exposes modeled EPC and aftermarket catalog directories and runs
supplier ordering through the shared reversible-effect gate.
The vendor engine crate turns brand manifests into loadable placement sites and
keeps vendor-facing operations behind an effect gate.

The repository keeps vehicle identities modeled and synthetic by default. Tests
guard committed fixtures against values shaped like VINs, Swedish plates, dealer
cookies, and vendor tokens.

## Crates

- `sim-lib-auto-core`: automotive citizens, capability names, site manifests,
  transport descriptors, and loadable runtime exports.
- `sim-codec-uds`: UDS and OBD-II byte-frame codec with DTC status-bit decode.
- `sim-lib-auto-diag`: modeled diagnostic sites, session placement, and
  cassette-backed replay for synthetic ECU reads.
- `sim-lib-auto-info`: modeled WIS, ISTA, VIDA, ESI[tronic], HaynesPro, and
  shop-authored repair documents ranked by vehicle, DTC, ECU, symptom, and lane.
- `sim-lib-auto-parts`: modeled EPC and aftermarket parts directories plus
  Mekonomen Pro-style reversible ordering with a fixture ledger.
- `sim-lib-auto-vehicle`: modeled vehicle identity lookup by plate or VIN label,
  plus HaynesPro and biluppgifter.se bridge contracts gated by `net/http`.
- `sim-lib-auto-vendor`: manifest-driven vendor sites and warranted dispatch for
  read, reversible, and irreversible automotive operations.

## Validation

```bash
cargo fmt --all --check
cargo run -p xtask -- check-file-sizes
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
cargo run -p xtask -- simdoc --check
```
