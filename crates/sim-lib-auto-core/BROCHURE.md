# sim-lib-auto-core

In one line: the automotive vocabulary SIM uses to name vehicles, shop lanes, effects, and access rules.

## What it gives you

It gives automotive work a shared set of names for modeled vehicles, diagnostics, service channels, shop capabilities, and transport endpoints. That shared vocabulary makes experiments easier to compare and easier to review because every piece describes the same kind of thing in the same way.

## Why you will be glad

- Vehicle identities stay modeled instead of becoming private customer data.
- Access rules travel beside the operation they authorize.
- Service lanes and transport endpoints can be described without adding automotive behavior to the kernel.

## Where it fits

This crate is the first automotive layer loaded above the SIM kernel. It names the domain objects that codec, service, agent, and workshop libraries use when they talk about cars, diagnostics, and controlled effects.
