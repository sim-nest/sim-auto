# sim-lib-view-bay

`sim-lib-view-bay` projects the modeled automotive bay data into a validated
Scene plus validated Intent values. The Scene includes the vehicle header, active
site, DTC list, repair document summary, parts cart, coding status, flash gate,
and ledger timeline. The Intent builders expose the bay actions as ordinary
`intent/select` and `intent/invoke` values over the shared VIEW contracts.

The crate composes the existing automotive citizens, diagnostic fixture, repair
documents, parts catalog values, vendor manifests, and SurfaceCaps metadata. It
does not add a kernel device type or a second editor protocol.

## Validation

```bash
cargo test -p sim-lib-view-bay
cargo clippy -p sim-lib-view-bay --all-targets -- -D warnings
```
