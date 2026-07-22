# sim-lib-auto-order

`sim-lib-auto-order` runs a modeled automotive work order across the public
manifest-backed vendor sites. It records each site operation in a work-order
ledger, carries the modeled vehicle and parts cart with the session, keeps
supplier ordering behind the existing reversible gate, and exports a balanced
invoice draft shape for ledger tooling.

The modeled conformance runner uses diagnostic-read and order grants as the
parent session. Read, information, parts, and modeled order steps are accepted.
Coding and flash steps are replayed as denied operations, proving that delegated
site sessions do not receive capabilities outside the parent grant set.

## Validation

```bash
cargo test -p sim-lib-auto-order
```
