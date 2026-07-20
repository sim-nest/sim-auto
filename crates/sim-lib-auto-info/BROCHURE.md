# sim-lib-auto-info

In one line: a modeled repair-information layer that turns safe workshop fixtures into ranked SIM documents.

## What it gives you

It gives automotive workflows a shared way to ask for repair guidance without copying vendor manuals into the public tree. Modeled WIS, ISTA, VIDA, ESI[tronic], HaynesPro, and shop-authored entries are ordinary SIM documents, so they can be selected, inspected, and rendered like other document values.

## Why you will be glad

- Repair guidance can be matched to the vehicle, fault code, ECU, symptom, and lane instead of handed around as loose notes.
- Public examples stay safe because every procedure is synthetic fixture text.
- The selected procedure becomes a document Scene that workshop and agent surfaces can show without adding an automotive view system.

## Where it fits

This crate sits above vehicle identity and diagnostic data and below vendor, bay, and agent-facing surfaces. It supplies the repair-information document source those layers can reuse before parts, ordering, coding, or flash workflows run.
