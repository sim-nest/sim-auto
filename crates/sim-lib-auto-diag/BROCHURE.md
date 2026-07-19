# sim-lib-auto-diag

In one line: a safe diagnostic fabric that lets SIM read modeled vehicle data and replay it exactly.

## What it gives you

It gives automotive tools a vehicle-shaped eval target with synthetic ECUs, trouble codes, PID values, freeze frames, and replayable diagnostic answers. A session chooses modeled data, a cassette, or a named local bridge, while capability checks decide which operations can actually run.

## Why you will be glad

- Diagnostic tests can exercise vehicle workflows without private shop traces.
- Successful reads can be captured and replayed, so repeated checks stay deterministic.
- Coding and service actions stay behind an explicit control capability.

## Where it fits

This crate sits above the automotive core vocabulary and the UDS byte codec. It supplies the diagnostic site behavior that runtime, workshop, and agent-facing libraries can realize through the kernel fabric contract.
