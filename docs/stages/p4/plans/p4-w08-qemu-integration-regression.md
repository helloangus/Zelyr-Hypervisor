# P4-W08 — QEMU integration regression

**Status:** Planned work package; implementation not claimed
**Parent:** [P4 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P4 plan index](README.md)

## Goal

Provide an automated QEMU `virt` integration regression that makes the P4
positive, fault, recovery, and repeat results objectively detectable.

## Scope

Cover P4-K01–K05: build Hypervisor and Validation Guest, prepare the selected
image, boot QEMU, collect expected serial/telemetry markers, determine positive
EL1 entry, translation and permission fault behavior, defined survival/stop,
and repeated-run stability.

## Out of scope

Treating QEMU as a complete hardware proof, Orange Pi runtime validation, CI
policy changes, Linux regression, performance benchmarking, or implementing
Guest mechanisms that its input packages have not established.

## Work sequence

1. Inspect the P0 QEMU automation contract and the W05–W07 scenario, diagnostic,
   repeatability, and event expectations.
2. Produce an approved detailed design for the automated P4 integration evidence
   path and its determinate success/failure conditions.
3. Integrate the declared build/image/boot/collection boundary with the existing
   QEMU runner without making QEMU behavior a Core contract.
4. Define positive, translation-fault, permission-fault, recovery/stop, and
   repeat acceptance cases with expected evidence markers.
5. Review timeout, incomplete-evidence, and unsupported-environment outcomes as
   diagnosable non-success results.
6. Record implementation/evidence status and hand automation outputs to W09 and
   P5 regression users.

## Acceptance and closure

P4-V13–P4-V15 require reproducible automated evidence that each specified test
returns a determinate result and repeated declared iterations remain stable.
Passing proves P4 behavior in the stated QEMU environment only; it does not
prove real-hardware correctness.

## Handoff

W09 receives the automation entry point, configuration facts, and evidence
references. P5 may reuse the regression boundary after preserving P4 scenarios.
