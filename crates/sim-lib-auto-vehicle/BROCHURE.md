# sim-lib-auto-vehicle

In one line: a vehicle identity lookup layer that keeps workshop data sources behind explicit network contracts.

## What it gives you

It gives SIM a safe way to turn a plate label or VIN label into the shared vehicle identity used by diagnostics and vendor sites. Modeled records cover tests and examples, while host-owned bridges can stand in for HaynesPro or biluppgifter.se without putting live endpoints or private vehicle data in the crate.

## Why you will be glad

- Vehicle workflows can agree on one identity before they reach diagnostics, service records, or vendor tools.
- Public tests stay deterministic because modeled records use synthetic labels.
- Live data-source paths are visible and reviewable, but network access stays denied until the caller grants it.

## Where it fits

This crate sits above the automotive core citizens and below diagnostic, workshop, and agent-facing layers. It supplies the identity lookup surface those layers can share before they dispatch a brand or vehicle-specific operation.
