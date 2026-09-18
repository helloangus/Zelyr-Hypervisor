# P8-W05 Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W05 detailed design](README.md).

This file contains the ordered steps only; the validation matrix,
error/security/observability model, and handoff checklist are in
[05-validation-and-handoff.md](05-validation-and-handoff.md), which closes
the package.

## 1. Preconditions and failure boundary

Before writing code or classification data, the implementer verifies it has
loaded the documents named in the parent README and confirms which assumed
contracts are evidenced at implementation time: the P4 exit path and
Stage-2 facts, the P5 error classes and hypercall rules, the P6 timer/
interrupt contracts, and the P7 run-state/block-wakeup contracts. Per the
[P8-W01](../p8-w01-entry-contract-reconciliation/README.md) reconciliation
rule, implementation may rely on a predecessor only when its evidence
supports the need; where a predecessor is still planned-only, the
corresponding step is designed against the assumed contract and marked
blocked until the class changes.

Stop and obtain direction instead of guessing when any of the following
occurs:

- realizing a classification row appears to require choosing a feature
  value, ID-presentation set, register encoding, or injected-fault encoding —
  that value is routed (W02 C2 `Specification Investigation`); record the
  route and leave the row at its category level;
- a consumed contract's shape differs from the assumed shape in
  [02](02-code-contracts-classification.md) (for example, a different exit
  disposition set or error-class enumeration) — stop and adapt the design
  delta explicitly; do not silently rename the mismatch away;
- an implementation need suggests reclassifying a behavior, adding a sixth
  class, widening the InvariantViolation path to Guest-caused conditions, or
  trapping WFE for policy reasons — these are design-change or
  architecture-change events (README decision 7 records the WFE trigger);
  raise them, do not code them;
- the code starts to implement an emulation body, a PSCI function, a vGIC
  or timer mechanism, or a console device — that belongs to W06/W07/W08/W09;
  stop and route.

## 2. Ordered implementation steps

### Step 1 — reconcile consumed contracts

Target: implementation record
(`../p8-w05-linux-cpu-virtualization-record.md`, created in this step).

Work: read the W01 reconciliation record and each consumed contract's
implemented facts; for each contract in
[02](02-code-contracts-classification.md) §0, record its evidence status
and its actual interface shape. Note every difference from the assumed
shape.

**Acceptance:** a per-contract status list with differences recorded;
blocked steps identified.  
**Failure/blocker:** a contradictory shape is a routed conflict; the
affected steps stop until resolved.

### Step 2 — realize the classification registry and types

Target: the classification types and registry of
[02](02-code-contracts-classification.md) §1–§3, in the layer-respecting
home the implementation assigns.

Work: implement `CpuBehaviorClass`, `ClassificationEntry`/`BehaviorRoute`,
and the registry with its initialization-time registration and
read-only-after-init invariant. Instantiate the per-area rows of
[01](01-classification-model.md) §4 at their approved granularity — rows
whose values are still routed stay at category level with their route
recorded. No `unsafe` here except where the Arch boundary requires it, with
`SAFETY` comments per the Coding Guidelines.

**Acceptance:** registry invariants hold (totality, immutability,
no-Guest-data keys); initialization fails closed on a malformed row;
routed rows are visibly routed.  
**Failure/blocker:** an invariant that cannot hold on the delivered
substrate is a routed conflict (§1).

### Step 3 — integrate the decision point into the exit path

Target: `classify_and_dispatch` ([02](02-code-contracts-classification.md)
§4) bound into the established P4 vCPU exit flow.

Work: implement the decision point and bind it at the exit-path point the
P4 contract provides; ensure Direct rows take no action and every other
resolution terminates in a disposition. Keep the exit-context bounds: no
allocation on Direct/Emulate paths, bounded work, no blocking.

**Acceptance:** every exit with a Guest-originated trapped operation
resolves through the decision point; no other resolution path exists;
dispositions match the P4 exit contract's vocabulary.  
**Failure/blocker:** a mismatch with the exit contract is a design delta to
record, then implement — never an ad-hoc second path.

### Step 4 — implement the controlled-rejection path and diagnostics

Target: `reject` and `GuestDiagnosticRecord`
([02](02-code-contracts-classification.md) §6–§7) plus the containment
rules of [03](03-controlled-failure-and-diagnostics.md) §3.

Work: implement record construction, fault injection via the established
exception/interrupt contracts, vCPU/VM containment per the P7/P4 lifecycle
contracts, and the telemetry event. Implement the InvariantViolation path
narrowly per [03](03-controlled-failure-and-diagnostics.md) §4.

**Acceptance:** every Reject produces a complete diagnostic, an injected
Guest-visible fault, one containment action, and one telemetry event; the
invariant path is reachable only from hypervisor-side failures.  
**Failure/blocker:** a containment rule that cannot be realized on the
delivered P5/P7 classes is routed per [03](03-controlled-failure-and-diagnostics.md)
§3's note; do not widen the invariant path to compensate.

### Step 5 — attach domain routes

Target: handler-route registrations for the domains with implemented
contracts at this time (PSCI: [W06](../p8-w06-psci-virtualization/README.md);
vGIC: [W07](../p8-w07-linux-vgicv3/README.md); timer:
[W08](../p8-w08-linux-timer-integration/README.md); console MMIO:
[W09](../p8-w09-virtual-console-single-cpu-linux/README.md)).

Work: register each implemented domain's handler against its route;
confirm missing handlers fail closed (registered-Emulate-with-no-handler ⇒
Reject).

**Acceptance:** every registered route resolves; every unimplemented route
fails closed with full diagnostics; no domain code was written inside W05.  
**Failure/blocker:** a domain contract that cannot meet the handler
obligations ([02](02-code-contracts-classification.md) §5) is a conflict to
route to that domain's package, not a W05 accommodation.

### Step 6 — classification and guardrail review

Target: verification record
(`../../verification/p8-w05-linux-cpu-virtualization-verification.md`);
implementation record.

Work: run the review items of [05-validation-and-handoff.md](05-validation-and-handoff.md)
§2 that are reviewable before runs (model completeness, decision-point
exclusivity, fail-closed audit, guardrails), then execute the V07/V08
scenarios through the W15/W16 fixture and regression route as available.
Record run/not-run precisely; the plan's boundary stands — passing these
validations does not prove trap/emulation implementation for every
register, only that the classified and negative scenarios behaved as
declared.

**Acceptance:** DV01–DV07 recorded with evidence or explicit not-run
entries.  
**Failure/blocker:** a failed scenario is recorded as failed with
diagnosis; the classification is not weakened to make a scenario pass.
