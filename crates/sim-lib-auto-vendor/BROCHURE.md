# sim-lib-auto-vendor

In one line: a manifest-driven vendor engine that keeps vehicle-side actions behind explicit proof gates.

## What it gives you

It turns an automotive site manifest into a runnable SIM site without linking a proprietary SDK. A modeled bridge answers requests for tests and fixtures, while the same request shape can point at a host bridge outside the crate.

## Why you will be glad

- Brand-specific surfaces can be registered from data instead of hardcoded runtimes.
- Read-only requests stay light, while write and control requests leave clear records.
- Risky actions require reversal material, a warrant, and a human gate before they run.

## Where it fits

This crate sits above the automotive core vocabulary and below later brand integrations. It supplies the manifest engine and effect gate that workshop, agent, and runtime layers can reuse for vendor-specific sites.
