# P3-W09 — CPU-local exception and interrupt foundations

**Status:** Planned work package; implementation not claimed
**Parent:** [P3 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P3 plan index](README.md)

## Goal

Make the P1 exception and interrupt diagnostic foundation correct for every online physical CPU.

## Scope

Legal per-CPU exception entry/local context, CPU identity in exceptional paths, non-overlapping exception state, CPU-attributed fatal diagnostics, and safe simultaneous diagnostic/logging behavior.

## Out of scope

Full host IRQ subsystem design, GICv3 virtualization, guest interrupt handling, and implementation-level vector/context layout.

## Work sequence

1. Inspect secondary bring-up, lifecycle, per-CPU state, and P1 exception contracts.
2. Define required local exceptional-path invariants and diagnostics.
3. Integrate exceptional-path requirements with audit, telemetry, stress, and regression consumers.
4. Review no-cross-CPU-scratch-state and shared-logging constraints.
5. Collect boot/secondary exceptional-path acceptance evidence.
6. Record limits and hand off the CPU-local diagnostic contract.

## Acceptance and closure

P3-V09 requires correct CPU identity and local diagnostic context on supported boot and secondary CPU exceptional paths.

## Handoff

W10–W15 and P4 receive diagnostic foundations only; later interrupt/guest work remains separately designed.
