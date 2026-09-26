# P1-W10 — QEMU boot regression

Status: Planned work package; implementation not claimed  
Parent: [P1 task book](../task-book-v0.2.md)
Prerequisites and consumers: [P1 plan index](README.md); requires W01–W09 and feeds W11, W12 and P2.

## Goal

Turn successful P1 boot into an automated, repeatable verdict with preserved
failure evidence and a defined 100-cycle clean-boot acceptance target.

## Scope

QEMU reference invocation, bounded output markers, timeout/exit semantics,
panic detection, evidence retention, stable-state verdict and 100 consecutive
clean boots.

## Out of scope

SMP/multi-platform matrices, Guest boot, performance benchmarking, CI provider
configuration beyond the P0 runner boundary and real-hardware evidence.

## Work sequence

1. Reuse W01/W09 canonical path and identify objective pass/fail markers.
2. Define timeout, abnormal exit, missing-marker and panic outcomes.
3. Integrate evidence preservation with the P0 QEMU runner baseline.
4. Review repeatability and avoid output-order-only success criteria.
5. Define the 100-cycle evidence set and its acceptance review.
6. Hand off the automation contract to fault validation and stage closure.

## Acceptance and closure

P1-V16 and P1-V17: the test independently determines verdict and 100 clean
boots reach the same stable marker with no random startup failure. The plan
does not claim that this run has happened.

## Handoff

W11 uses the same bounded execution/evidence conventions; W12 records locations
for future verification evidence. QEMU remains reference evidence, not hardware
correctness proof.
