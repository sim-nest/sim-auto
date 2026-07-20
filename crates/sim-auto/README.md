# sim-auto

`sim-auto` is the Bootloader-backed command surface for the modeled automotive
bay. It registers a host library that exports `cli/main/auto`, so the command
runs through the same loaded-library handoff as other SIM product binaries.

The public command serves synthetic data. `auto diag` prints the modeled primary
DTC, `auto bay` summarizes the bay Scene, and `auto intents` checks the available
bay actions. Live vendor tools, vehicle interfaces, accounts, and bridges stay
outside this package.

## Validation

```bash
cargo test -p sim-auto
cargo clippy -p sim-auto --all-targets -- -D warnings
```
