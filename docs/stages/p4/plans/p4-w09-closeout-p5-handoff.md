# P4-W09 — P4 factual closeout and P5 handoff

**Status:** Planned work package; implementation not claimed
**Parent:** [P4 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P4 plan index](README.md)

## Goal

Record P4's actually implemented and verified facts, limitations, and P5 inputs
without mistaking planned work or temporary test conventions for completed
architecture.

## Scope

Cover P4-L01–L03 and P4-V16: Validation Guest boot contract, Stage-2 capability
matrix, known limitations, unsafe-inventory and ADR-deviation review, evidence
index, exit-criterion review, and the limited P5 handoff.

## Out of scope

Creating evidence before implementation, declaring P4 complete without all
matrix evidence, freezing the P8 machine ABI, defining P5 HVC/capability
contracts, or changing accepted ADR decisions.

## Work sequence

1. Inspect W01–W08 implementation records, verification evidence, and each P4
   exit criterion for completeness and consistency.
2. Produce factual boot, capability, limitation, unsafe, and evidence records
   only for work actually performed and verified.
3. Integrate links between the task book, package records, verification reports,
   and any required architecture-change record.
4. Review the closeout against P4 scope, the temporary-layout boundary, and the
   distinction between a Guest fault and a Hypervisor invariant failure.
5. Evaluate P4-V16 and record any missing evidence, failed criterion, or
   deferred scenario explicitly rather than making a completion claim.
6. Hand P5 the evidence-backed Stage-2/Guest-entry/fault facts and preserve all
   remaining scope as unimplemented.

## Acceptance and closure

P4-V16 passes only when all factual records have linked evidence, limitations
are explicit, and P5 inputs are constrained to proven P4 behavior. This is a
completion-review package; it cannot manufacture successful runtime evidence.

## Handoff

P5 can consume an evidenced single-Guest Stage-2 and EL1 execution foundation,
Validation Guest asset, fault diagnostics, and QEMU regression. Formal HVC,
capability, and management behavior remain P5 work.
