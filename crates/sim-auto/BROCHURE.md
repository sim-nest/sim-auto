# sim-auto

In one line: A modeled auto-bay command that boots through SIM's loader.

## What it gives you

It gives technicians and tests a simple way to open the public modeled bay,
read the primary fault, and inspect the available bay actions from a command
line. The command is backed by a loaded SIM library instead of a private runtime
path, so it follows the same contract as other product commands.

## Why you will be glad

The automotive demo is easy to run and hard to misuse. It shows useful modeled
diagnosis output without touching a vehicle, vendor account, bridge endpoint, or
licensed database. Runtime wiring stays uniform across the constellation.

## Where it fits

It sits at the edge of `sim-auto`: below it are the reusable bay Scene and Intent
values, diagnostics, parts, repair information, and manifests; above it are
shells or facades that launch the `auto` verb.
