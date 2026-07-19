# sim-codec-uds

In one line: a safe diagnostic byte-frame reader for automotive UDS and OBD-II data.

## What it gives you

It turns small diagnostic byte frames into SIM records that tools can inspect, compare, replay, and encode again. Read-DID requests, OBD-II mode requests, and trouble-code responses all become structured data with explicit status bits instead of opaque byte strings.

## Why you will be glad

- Diagnostic examples can stay synthetic while still matching real protocol shapes.
- Trouble-code status is visible without shipping vendor fault text or proprietary traces.
- Tests can prove that bytes and records round-trip before any workshop bridge exists.

## Where it fits

This crate sits above the automotive core vocabulary and beside SIM's other codecs. Diagnostic libraries use it when they need byte-level vehicle frames represented as ordinary SIM data.
