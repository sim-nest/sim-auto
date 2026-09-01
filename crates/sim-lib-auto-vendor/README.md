# sim-lib-auto-vendor

`sim-lib-auto-vendor` installs automotive vendor sites from open
`SiteManifest` records. A site is a kernel `EvalFabric` value exported through
the standard site registry, backed by one `VendorBridge` trait that can point at
modeled data, a cassette, or a host bridge.

Every operation reaches the domain-neutral `sim-lib-operation-gate` through an
explicit `OpCap` declaration. Observation and Recorded modes are distinct audit
labels; Reviewed operations require verified exact-subject approval whose
atomic use occurs inside the first-performance boundary. Kernel cassette replay
returns the recorded `Ref` directly without dispatching or consuming twice.

## Validation

```bash
cargo test -p sim-lib-auto-vendor
```
