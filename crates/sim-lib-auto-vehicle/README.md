# sim-lib-auto-vehicle

`sim-lib-auto-vehicle` resolves synthetic vehicle identities by plate or VIN
label and publishes bridge contracts for host-owned automotive data sources. The
modeled source is safe for committed tests. HaynesPro and biluppgifter.se paths
are contract surfaces only: a caller must grant `net/http`, and no public
endpoint, key, owner data, plate, or VIN is carried by the crate.

The crate installs a loadable vehicle identity site and a contract catalog value
so workshop, diagnostic, and vendor layers can ask for a `VehicleId` without
embedding a provider API.

## Validation

```bash
cargo test -p sim-lib-auto-vehicle
```
