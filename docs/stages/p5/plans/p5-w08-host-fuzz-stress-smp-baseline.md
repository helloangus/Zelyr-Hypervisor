# P5-W08 — Host fuzz, lifecycle stress, SMP, and performance baseline

**Status:** Planned work package; implementation not claimed
**Parent:** [P5 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P5 plan index](README.md)

## Goal

Establish P5 host-side robustness evidence for invalid inputs and repeated
lifecycles, plus bounded multi-pCPU safety and performance-baseline evidence.

## Scope

Cover P5-T16, T17, T19 and §32: fuzz/property-ready parsing, range, handle,
rights, and state checks; create/lookup/destroy/recreate stress; declared
two-pCPU concurrent foundation; and baseline measurements for minimum call,
handle lookup, authority check, and Guest-data validation.

## Out of scope

A whole-Hypervisor fuzz framework, exact random generators, final lock/index
strategy, a scheduler, scalability claims, performance KPI, real-hardware
proof, or a replacement for Guest-side integration evidence.

## Work sequence

1. Inspect W03–W06 boundaries together with P0 host-test and P3 SMP/
   synchronization contracts.
2. Produce an approved detailed design for testable validation seams, lifecycle
   stress authority, multi-pCPU state protection, failure detection, and the
   baseline measurement method.
3. Define randomized/boundary malformed-input and repeated object/authority
   lifecycle scenarios with explicit invariant and non-success criteria.
4. Define declared multi-pCPU/concurrent operations that expose object lookup,
   validation, grant/revoke, and destruction races without assuming final locks.
5. Define how baseline data is recorded without using it as an unreviewed
   performance target or treating QEMU timing as universal hardware behavior.
6. Record implementation/evidence status and hand robustness/limit facts to
   W09–W10.

## Acceptance and closure

P5-V13 and V14 require reproducible evidence that declared randomized/stress
and multi-pCPU scenarios preserve required invariants and report their limits;
the four baseline categories are recorded. Passing does not prove no bug, final
scalability, or real-hardware timing.

## Handoff

W09 consumes test outcomes and limits for regression telemetry; W10 consumes
the evidence, stress duration/configuration, and performance record.
