# sim-lib-auto-diag

`sim-lib-auto-diag` provides a modeled automotive diagnostic fabric. It exposes
synthetic ECU sites through the kernel `EvalFabric` contract, reads modeled DTCs,
PIDs, and freeze-frame data, and records successful replies through the shared
stream-fabric cassette layer.

The crate keeps diagnostic reads separate from coding, service, and actuation
effects. Read operations require the diagnostic-read capability, and controlled
operations require the automotive control capability after caller-side
diminishment.

## Validation

```bash
cargo test -p sim-lib-auto-diag
```
