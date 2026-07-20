# sim-lib-auto-parts

`sim-lib-auto-parts` provides a modeled automotive parts catalog and supplier
ordering layer. Mercedes EPC-shaped and aftermarket catalog fixtures are exposed
as SIM directory tables, so callers can use ordinary table operations to list
groups and fetch synthetic part lines.

Mekonomen Pro ordering is represented as a reversible vendor operation through
the shared automotive effect gate. Modeled orders write only to an in-memory
fixture ledger. Live supplier mode requires `auto/order` and `net/http`, then
fails closed unless the host supplies its own HTTP-backed placement.

## Validation

```bash
cargo test -p sim-lib-auto-parts
```
