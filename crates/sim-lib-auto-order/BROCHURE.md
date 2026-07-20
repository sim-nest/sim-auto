# sim-lib-auto-order

In one line: a safe modeled work order that shows a whole automotive job without private shop data.

## What it gives you

It gathers vehicle identity, diagnostics, repair information, parts lookup, supplier ordering, coding review, and flash review into one session record. Every step is modeled and every outcome is written into a small ledger that a tester or reviewer can inspect.

## Why you will be glad

- A complete workshop story can run in tests without credentials, customer cars, or vendor accounts.
- Read-only work and supplier ordering are separated from denied coding and flash attempts.
- The invoice export is balanced in exact minor units, so bookkeeping review starts from a clean draft.

## Where it fits

This crate sits above the automotive vendor, parts, and core crates. It is the conformance layer that proves the public modeled sites work together as one guarded service session.
