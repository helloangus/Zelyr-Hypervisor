# P2-W09 — QEMU platform integration regression

Chinese readers can use the [Chinese edition](p2-w09-qemu-integration-regression.zh-CN.md).

**Status:** Planned work package; implementation not claimed
**Parent:** [P2 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P2 plan index](README.md)

## Goal

Establish the reference-platform integration evidence required to show P2 facts,
memory accounting, and diagnostics remain stable across supported QEMU `virt`
configurations.

## Scope

P2-K01–K05: standard AArch64 `virt` configuration, CPU-count variants,
RAM-size variants, memory-map accounting, and repeated-boot stability.

## Out of scope

QEMU behavior as architectural authority, real-hardware proof, AP startup,
GIC initialization, Guest execution, or a new QEMU runner.

## Work sequence

1. Confirm P0's one QEMU runner entry and W01–W08 contracts/evidence are
   available without recreating P0 automation.
2. Define the required configuration matrix for the reference platform,
   including CPU-count and RAM-size variation.
3. Establish expected discovery, protected-map accounting, allocation, and W06
   inspection observations for each configuration.
4. Define repeated-boot evidence that detects uninitialized-state dependence
   while separating QEMU results from hardware-semantic proof.
5. Review integration evidence against the hard protected-page safety gate and
   host-side negative-regression coverage.
6. Record required evidence/limitations and hand off the reference result to
   W10, P3, and P4 planners.

## Acceptance and closure

P2-V11 requires actual automated QEMU evidence across the stated matrix and
repeated boots, with map accounting in its documented domain and active-state
inspection consistency. P2-V01–V10 remain separately required; this plan does
not claim their execution.

## Handoff

W10 may cite the evidence location and distinguish run/not-run results. P3/P4
receive a reference-platform input baseline, not a proof of real hardware.

## P2-ACR-02 planning amendment

W11 is an additional prerequisite. Include actual allocated-frame Host access and cleanup/reuse for representative stack, table and image-buffer fixtures in the supported reference matrix under P2-V11/P2-V14. Preserve P1 boot continuity and distinguish these fixtures from P3/P4 execution; do not infer SMP or hardware evidence.

Owner-selected ADR-062 follow-up (2026-10-02): consume the
[W12 common ownership/view foundation](p2-w12-minimal-memory-objects.md) and
P2-V15 evidence at the corresponding lifetime/acceptance boundary. W08 owns
Host negative/lifetime scenarios, W09 owns actual integrated Host access, W10
owns condition-specific handoff, and W11 owns the translation adapter. W12
Host modeling precedes W11 and does not require its hardware; W04-MAP remains
independent. Formal ADR integration and approved detailed designs are still
required before affected coding; no object or runtime delivery is inferred.
