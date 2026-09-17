# P1-W11 — Negative and fault validation

Status: Planned work package; implementation not claimed  
Parent: [P1 task book](../task-book-v0.1.md)  
Prerequisites and consumers: [P1 plan index](README.md); requires W05–W10 and feeds W12 and P2.

## Goal

Define reproducible evidence for P1 failure paths, proving that unsupported
environments and intentional faults are bounded and diagnosable.

## Scope

Unsupported execution/capability environment, intentional synchronous fault,
panic, post-MMU translation/access fault and unexpected vector; expected
diagnostic classes and terminal outcomes.

## Out of scope

Guest-caused fault isolation, recoverable VM faults, hardware fault coverage,
GIC/IRQ subsystem testing, fuzzing and production recovery policy.

## Work sequence

1. Map each negative case to the W05/W07 diagnostic and W10 runner contracts.
2. Define setup, trigger category and expected bounded outcome for each case.
3. Integrate evidence collection without changing normal P1 scope.
4. Review input/range checks, no-RWX and unsafe-inventory expectations.
5. Define reproducibility and objective acceptance for every fault class.
6. Hand off the negative-validation matrix and limitations to W12.

## Acceptance and closure

P1-V18 and P1-V19: all listed fault classes produce required diagnostics and
the review finds no unintended security relaxation or later-stage mechanism.

## Handoff

W12 can reference negative evidence requirements; P2 receives only the
diagnostic and boundary contract, not a general fault framework.
