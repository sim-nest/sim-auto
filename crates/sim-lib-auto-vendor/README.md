# sim-lib-auto-vendor

`sim-lib-auto-vendor` installs automotive vendor sites from open
`SiteManifest` records. A site is a kernel `EvalFabric` value exported through
the standard site registry, backed by one `VendorBridge` trait that can point at
modeled data, a cassette, or a host bridge.

Every operation reaches the bridge through `warranted_effect`. Pure reads use
only the diagnostic-read capability. Reversible operations also record a gate
and kernel effect-ledger entry. Irreversible operations additionally require a
reversal artifact, a warrant, and explicit human approval before dispatch.

## Validation

```bash
cargo test -p sim-lib-auto-vendor
```
