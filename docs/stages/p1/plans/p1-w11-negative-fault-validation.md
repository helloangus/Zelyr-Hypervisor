# P1-W11 — Negative and fault validation

Status: Planned work package; implementation not claimed  
Parent: [P1 task book](../task-book-v0.2.md)
Prerequisites and consumers: [P1 plan index](README.md); requires W05–W10 and feeds W12 and P2.

## Goal

Define reproducible evidence for P1 failure paths, proving that unsupported
environments and intentional faults are bounded and diagnosable.

## Scope

Unsupported execution/capability environment, intentional synchronous fault,
panic and post-MMU translation/access fault; expected diagnostic classes and
terminal outcomes. Preserve the unexecuted NC6 unexpected-vector risk and
transfer its execution gate to P6-W12/P6-V29 under ADR-061.

## Out of scope

Guest-caused fault isolation, recoverable VM faults, hardware fault coverage,
GIC/IRQ subsystem testing, fuzzing and production recovery policy.

## Work sequence

1. Map each negative case to the W05/W07 diagnostic and W10 runner contracts.
2. Define setup, trigger category and expected bounded outcome for each case.
3. Integrate evidence collection without changing normal P1 scope.
4. Review input/range checks, no-RWX and unsafe-inventory expectations.
5. Define reproducibility and objective P1 acceptance for NC1–NC5; record NC6
   as an unexecuted asynchronous case, not an accepted synchronous proxy.
6. Hand off the negative-validation matrix and limitations to W12 and the NC6
   execution obligation to P6-W12.

## Acceptance and closure

P1-V18 requires paired NC3 synchronous, NC4 panic and NC5 post-MMU fault
diagnostics; NC1 and NC2 support P1-V02 and P1-V06 separately. P1-V19 requires
the security review to find no unintended relaxation or later-stage mechanism.
NC6 does not pass P1-V18 by implication and remains required at P6-V29.

## Handoff

W12 can reference negative evidence requirements; P2 receives only the
diagnostic and boundary contract, not a general fault framework. P6-W12
receives NC6's genuine asynchronous-vector execution obligation.
