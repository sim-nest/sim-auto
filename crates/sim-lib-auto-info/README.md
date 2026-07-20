# sim-lib-auto-info

`sim-lib-auto-info` provides modeled automotive repair information as SIM
documents. It covers synthetic public fixtures for WIS, ISTA, VIDA, ESI[tronic],
HaynesPro, and shop-authored procedures, then ranks candidate procedures by
vehicle identity, DTC, ECU, symptom, and lane.

The selected procedure renders through the existing document view surface as a
Scene. Public fixtures contain only synthetic procedure text and carry no vendor
manual excerpts, screenshots, wiring images, credentials, customer identifiers,
or licensed repair content.

## Validation

```bash
cargo test -p sim-lib-auto-info
```
