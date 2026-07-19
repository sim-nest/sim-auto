# sim-codec-uds

`sim-codec-uds` registers `codec/uds`, a byte codec for safe automotive
diagnostic fixtures. It decodes UDS read-data-by-identifier requests and
responses, OBD-II mode requests, and UDS DTC responses into `Expr::Map` records,
then encodes those records back to the original bytes.

DTC responses expose standardized status bits from `sim-lib-auto-core` and keep
fault text out of the frame model. The codec carries raw DTC bytes and synthetic
status-only records only.

## Validation

```bash
cargo test -p sim-codec-uds
```
