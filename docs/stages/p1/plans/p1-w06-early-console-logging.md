# P1-W06 — Early console and bring-up logging

Status: Planned work package; implementation not claimed  
Parent: [P1 task book](../task-book-v0.1.md)  
Prerequisites and consumers: [P1 plan index](README.md); requires W02/W05 and feeds W07–W12.

## Goal

Keep a reliable, diagnosable early output channel available from entry through
stable EL2 state without defining the future console subsystem.

## Scope

Startup markers, capability output, exception/panic output, MMU transition
markers, reference-console assumptions and pass/stable markers.

## Out of scope

Generic console framework, platform probing, device drivers, runtime tracing
architecture and any permanent QEMU-specific Core contract.

## Work sequence

1. Identify P0 logging and panic contracts usable before discovery.
2. Define the early channel's supported message categories and phase markers.
3. Integrate output requirements with W05 exception and W08 MMU transitions.
4. Review fixed reference assumptions and future replacement boundary.
5. Define evidence that output remains available through stable state.
6. Hand off marker and diagnostic-channel expectations to automation and docs.

## Acceptance and closure

P1-V10: diagnostics identify each required stage from entry to stable state and
remain available across the Host Stage-1 transition.

## Handoff

W07 can use the channel for crash reports and W10 can use bounded markers for
automated verdicts. The full console subsystem remains later work.
