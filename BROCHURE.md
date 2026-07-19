# sim-auto

In one line: a safe automotive domain layer for describing vehicles, service lanes, and shop capabilities inside SIM.

## What it gives you

Automotive work needs names for vehicles, diagnostic lanes, byte-level fault frames, service operations, shop-side access, and replayable diagnostic sessions without leaking private shop data into the runtime. This repository gives SIM a shared vocabulary for those pieces, with synthetic fixture discipline baked into the tests.

## Why you will be glad

- Vehicle-facing experiments can use a common language instead of ad hoc labels.
- Diagnostic frames and modeled ECU replies become inspectable records while shop capabilities stay explicit, so access and effects are easy to review.
- Fixture checks catch accidental private-looking automotive values before they leave the tree.

## Where it fits

The kernel carries open runtime contracts; this repository supplies the automotive domain vocabulary that loads into those contracts. It sits beside the codec, runtime, storage, and agent libraries as the automotive layer they can all name consistently.
