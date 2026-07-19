# sim-lib-auto-core

`sim-lib-auto-core` defines the automotive citizens and capability names shared
by SIM automotive libraries. It describes modeled vehicle identities, diagnostic
codes and status bits, brand capability sets, service lanes, effect classes, operation
capabilities, transport descriptors, site manifests, and session placement
descriptors.

The crate also provides a loadable library that exports the core automotive
capability list, lane list, manifest shape descriptor, and citizen classes.

## Validation

```bash
cargo test -p sim-lib-auto-core
```
