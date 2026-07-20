# sim-lib-auto-parts

In one line: a modeled parts catalog and ordering layer that keeps supplier effects reviewable.

## What it gives you

It gives automotive workflows a shared way to find safe, synthetic replacement parts and place modeled supplier orders without reaching into a live account. Catalog data behaves like ordinary SIM directory tables, while ordering uses the same effect gate as other shop operations.

## Why you will be glad

- Parts lookup can move through groups, keys, and entries instead of hardcoded lists.
- Supplier orders leave a fixture ledger trail that tests and agents can inspect.
- Live ordering is visible as a contract, but network access stays denied until the host grants it.

## Where it fits

This crate sits after vehicle identity, diagnostics, and repair information. It supplies the parts and order surface that bay, vendor, and agent layers can reuse before coding or flash work begins.
