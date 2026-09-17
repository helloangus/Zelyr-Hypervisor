# P4-W04 — Single-vCPU Guest-EL1 entry and recovery

**Status:** Planned work package; implementation not claimed
**Parent:** [P4 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P4 plan index](README.md)

## Goal

Establish a reproducible single-vCPU path from normal EL2 control flow into
Guest EL1 and back to a live, diagnosable EL2 state, including one re-entry and
defined stop result.

## Scope

P4-C01–C04 and P4-E01, E03–E05 are covered: initial Guest PC/SP/processor
state, Guest EL1 rather than virtual EL2, repeatable construction, real `ERET`,
EL2 recovery, re-entry, and stop. This package consumes the Stage-2 activation
and validated Guest-memory inputs.

## Out of scope

Final generic exit-reason software design, HVC ABI, Guest SMP, scheduler,
vGIC/timer, VM lifecycle policy, assembly/Rust implementation details, and
formal VM/vCPU API or crate boundaries.

## Work sequence

1. Inspect W02/W03 contracts and P1 exception evidence needed for safe EL2 and
   Guest context transitions.
2. Produce an approved detailed design for one vCPU's initialization,
   entry/return authority, recovery boundary, re-entry, and stop result.
3. Integrate the transition boundary with active Stage-2 state and validated
   Guest memory/image inputs.
4. Define acceptance scenarios for Guest EL1 identity, PC/SP/state, EL2
   recovery, re-entry, and completion stop.
5. Review that Guest execution cannot continue as ordinary EL2 flow and that no
   permanent single-pCPU or single-VM architecture is asserted.
6. Record implementation/evidence status and hand the execution boundary to
   W05–W08.

## Acceptance and closure

P4-V04 and P4-V09 require evidence of correct EL1 entry, preserved EL2 control,
one re-entry, a defined stop path, and diagnosable WFI/WFE or controlled Guest
fault behavior. Passing does not demonstrate scheduler semantics or full VM
destruction.

## Handoff

W05 can exercise a real Guest-EL1 path; W06 can classify returned conditions;
W07/W08 can evaluate repeatability and automation. P5 receives only evidenced
entry/exit facts.
