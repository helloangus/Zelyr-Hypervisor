# P5-W10 — P5 factual closeout and P6 handoff

**Status:** Planned work package; implementation not claimed
**Parent:** [P5 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P5 plan index](README.md)

## Goal

Record P5's actually implemented and verified authority-boundary facts,
documentation, limitations, and P6 inputs without turning a plan or temporary
test convention into a completed public contract.

## Scope

Cover P5-D01–D09, EC-P5-01–EC-P5-12, and P5-V17: factual HVC ABI,
handle/capability/Guest-data/security documentation links, invariant and
compatibility review, Validation Guest and host/QEMU evidence, unsafe delta,
performance baseline, limitations, regression inventory, and P6 handoff.

## Out of scope

Writing a specification, security claim, test report, or completion assertion
without supporting implementation/evidence; freezing a management/machine ABI;
creating P6 interrupt semantics; or changing an accepted ADR.

## Work sequence

1. Inspect W01–W09 implementation records, evidence, and each P5 exit
   criterion for completeness and consistency.
2. Publish ABI/security/input-safety/handle/capability artifacts only for
   choices actually approved, implemented, compatibility-reviewed, and
   evidenced; link their governed locations.
3. Record factual P5 test environment, CPU count, negative/stress duration,
   performance method/results, unsafe delta, dependencies, and known limits.
4. Reconcile the permanent invariant and regression inventory with all actual
   validation outcomes, deferrals, failures, and architecture-change records.
5. Evaluate closure without manufacturing evidence and preserve every absent
   criterion or later-stage item as unimplemented.
6. Hand P6 only the evidenced HVC, authority, Guest-data, telemetry, and
   regression facts required to plan interrupt-object extension.

## Acceptance and closure

P5-V17 passes only when all stated facts have linked supporting evidence,
required documentation is factual and compatibility-reviewed, limitations are
explicit, and P6 inputs are constrained to proven behavior. This review cannot
manufacture successful runtime evidence.

## Handoff

P6 can consume documented and evidenced P5 object-reference/right, controlled
Guest-data, structured-result, telemetry, and regression facts. Formal machine
and management ABIs, IPC, Control Domain, scheduler, and final concurrent
mechanisms remain unimplemented.
