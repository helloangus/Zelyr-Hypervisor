# P8-W20 — Documentation, closure, and P9 handoff

**Status:** Planned work package; implementation not claimed
**Parent:** [P8 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P8 plan index](README.md)

## Goal

Plan factual P8 publication, evidence review, and a bounded handoff to P9 once work is complete.

## Scope

Cover source P8.20 and delivery/exit gates: machine specification, Linux boot specification, compatibility policy, validation report, evidence index, known limits, unsafe/dependency deltas, and P9 consumer statement.

## Out of scope

Writing factual specifications before approvals/evidence, claiming P8 closure, defining Virtio, or changing accepted ADRs.

## Work sequence

1. Inspect W14 and W16–W19 evidence requirements and implementation records.
2. Define the factual document set and distinction between plan, design, implementation, and verification.
3. Cross-check every exit gate against real validation IDs and evidence locations.
4. Record limitations, unresolved decisions, unsafe/API/dependency changes, and architecture conflicts.
5. Define the P9 handoff as evidenced facts only.

## Acceptance and closure

P8-V26 passes only when factual documents and evidence exist and every exit condition is reviewed. This plan itself does not close P8.

## Handoff

P9 receives only evidenced machine, boot, DTB, console, SMP, compatibility, fixture, and regression facts; Virtio remains separate design work.
