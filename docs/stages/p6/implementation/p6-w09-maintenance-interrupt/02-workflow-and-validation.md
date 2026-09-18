# P6-W09 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P6-W09 detailed design](README.md).  
**Companions:** [01-maintenance-contract.md](01-maintenance-contract.md)
(behavior authority for steps 2–4).

## 1. Preconditions and failure boundary

Before changing anything, the implementer verifies it has loaded the parent
README's document set and inspects the actual tree (`git ls-files`; the
repository is a P0 scaffold — no Rust sources exist yet). All P6-W01–W08
prerequisites are assumed contracts from their plans and handoffs
([P2-W10](../../../p2/plans/p2-w10-p3-p4-handoff-contract.md),
[P3-W14](../../../p3/plans/p3-w14-p4-smp-handoff.md),
[P4-W09](../../../p4/plans/p4-w09-closeout-p5-handoff.md),
[P5-W10](../../../p5/plans/p5-w10-closeout-p6-handoff.md)); their designs
and evidence, not their names, are what step 1 must find.

Stop and record a blocker (never repair a prerequisite, never guess) when any
of the following occurs:

- a prerequisite design (W03/W07/W08) delivers a contract that conflicts with
  [01-maintenance-contract.md](01-maintenance-contract.md) §2/§3 — raise an
  Architecture Change Request against the owning design;
- the specification reconciliation of step 2 cannot resolve the maintenance
  register semantics for the platform's GIC — record a Specification
  Investigation; do not code against a guessed bit layout;
- safe processing appears to require changing W08 allocation internals, W07
  queue discipline, or W03 dispatch — that is other packages' authority; or
- making the path work appears to require a scheduler hook or Guest vGIC MMIO
  behavior — P7/P8 scope; stop.

## 2. Ordered implementation steps

### Step 1 — prerequisite reconciliation

Target: implementation record (`../p6-w09-maintenance-interrupt-record.md`,
created in this step).

Work: locate and read the W03, W07, and W08 designs and their verification
records; confirm the three interfaces W09 consumes exist as designed: (a) W03
dispatch of the maintenance PPI class plus the safe physical-IRQ completion
rule; (b) W07 vIRQ completion entry point; (c) W08 per-vCPU in-flight LR
records, slot release, and IRQ-context admission. Record each finding as
confirmed, deviant, or absent.

**Acceptance:** the record names each prerequisite, its evidence location, and
a confirm/deviate/absent verdict.  
**Failure/blocker:** a deviate or absent verdict blocks step 3; record the
conflict per §1.

### Step 2 — specification reconciliation

Target: implementation record.

Work: against the W01 capability facts and the applicable authoritative
architecture/GIC specification revision, fix: the maintenance INTID value,
the status register(s) and field-to-class mapping for
`MaintenanceConditions`, and the required access ordering (read-before-EOI
sequencing, barriers). This resolves the Specification Investigation item
task book §8 assigns; until it lands, no register spelling from this design
is authoritative.

**Acceptance:** the record states the spec revision consulted, the mapped
fields, and the barrier/sequencing rules; `MaintenanceConditions`'s mapping
is now expressible without contradicting the type contract (§3.1).  
**Failure/blocker:** an unresolvable mapping blocks step 3; record the
investigation; do not proceed on partial bit knowledge.

### Step 3 — implement the maintenance transition

Target: the W08-established Host GIC virtualization module (logical names per
[§3](01-maintenance-contract.md)).

Work: implement `MaintenanceConditions`, `decode_maintenance_status`,
`reconcile_completed_slots`, `refill_freed_capacity`,
`report_unexpected_maintenance`, and `maintenance_irq_entry` per their
contracts. All MMIO/system-register access goes through the W08 access layer
with volatile semantics, required barriers, and reserved-bit handling; all
`unsafe` is minimal, audited, and inventoried per
[P0-W10](../../../p0/plans/p0-w10-unsafe-rust-governance.md) with nearby
SAFETY explanations. No allocation, no reachable panic, and no unbounded loop
in IRQ context (Coding Guidelines; contract §4 I4).

**Acceptance:** code review against contract §3 field-by-field finds no
deviation; new `unsafe` is inventoried; the E1/E2/E3/E4 behaviors are each
reachable in review.  
**Failure/blocker:** any contract deviation fails review; fix the code, not
the contract (contract changes need a new design decision).

### Step 4 — integrate telemetry and safe Host IRQ completion

Target: maintenance telemetry events; W03 completion sequencing.

Work: emit the six logical events of [§5](01-maintenance-contract.md) through
the P0-W13 namespace mechanism once that contract exists; fix the
decode-then-complete ordering from step 2 so the controller read precedes EOI
as the specification requires. Confirm completion of the physical interrupt
always happens exactly once per invocation on every return path, including
the unexpected and orphan paths.

**Acceptance:** every §3 entry/return path provably completes the physical
IRQ once; events carry the payload classes of contract §5.  
**Failure/blocker:** a path that can return without completion, or complete
twice, is a review failure and a P6-V17 risk; stop and fix.

### Step 5 — boundary review

Target: review findings in the implementation record.

Work: review the code against contract §4 invariants I1–I5 and the
state-leak/duplicate-completion/cross-vCPU/storm-boundary constraints named by
plan step 5; confirm the E1 (benign duplicate) vs E2 (orphan invariant)
separation; hand the orphan case to W12's FI-C matrix and the bounded-work
statement to W12's storm review.

**Acceptance:** each invariant has a named code-level justification and a
corresponding validation row below.  
**Failure/blocker:** an unjustified invariant is a design gap; stop and
amend the design, not the review.

### Step 6 — acceptance scenarios and evidence

Target: `../../verification/p6-w09-maintenance-interrupt-verification.md`
(created when evidence exists).

Work: execute the validation matrix below in the declared QEMU environment;
record command, environment, output, timestamp, and run/failed/blocked/not-run
status per row; scenario definitions consumed by W11 (Guest-observable
progression) and W12 (storm) are cross-referenced, not duplicated.

### Step 7 — closure review and handoff

Work: run the handoff checklist (§5) and verify the package against the plan
goal, prerequisite compatibility, and the parent README's requirement map.
Completion is claimed only in the verification record, only for what ran.
P6-V16/P6-V17 statements must carry their proof boundaries verbatim: passing
does not prove fairness or Linux vGIC behavior.

## 3. Validation matrix

All rows are planned evidence with objective success conditions; none claims
an implementation exists. Evidence destination for every row:
`../../verification/p6-w09-maintenance-interrupt-verification.md`.

| ID | Test or review | Technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W09-DV01 → P6-V16 | over-capacity presentation | inject more pending vIRQs for one vCPU than the platform's presentable slots; run maintenance cycles | no vIRQ lost or overwritten; excess remains pending (I1); freed slots are reused (I3) | no-loss under capacity pressure in QEMU; not fairness, not real-hardware LR timing, not production IRQ-pressure tolerance |
| W09-DV02 → P6-V17 | reusable-slot progression | present, complete, reconcile, refill in repeated cycles | each cycle reconciles each slot once (I2); slots return FREE (I3); pending work advances (I4) | maintenance advances remaining work without duplicate completion or leak; not that every controller implementation behaves identically |
| W09-DV03 → P6-V17 | Guest-completion path | Guest acknowledges and EOIs a presented vIRQ; maintenance follows | completion correlates to the right vIRQ exactly once; no false completion of a different vIRQ | Guest-EOI-driven correlation correctness in QEMU; not P8 Linux vGIC EOI semantics |
| W09-DV04 | unexpected-maintenance injection | debug/forced-decode hook feeds unrecognized and contradictory status patterns (not synthesized hardware faults) | contained per §3.6; counters increment; no lifecycle mutation; no panic | containment behavior of the written code; not coverage of real hardware misbehavior modes |
| W09-DV05 → P6-V16/V17 | repeat-progress | N successive pressure cycles (N declared at run time) | determinate completion outcome every cycle; no drift across cycles | stable repetition; not a latency or throughput KPI |
| W09-DV06 | cross-vCPU isolation | maintenance on one pCPU while another vCPU carries pending/presented state | other vCPU's state unmodified (I5) | per-vCPU containment; not multi-vCPU scheduling behavior (P7) |
| W09-DV07 | bounded-work review + instrumented check | review §3.2 loop bounds; assert bound invariant in test runs | no loop without a controller-derived bound; no allocation on path | bounded IRQ-context work as designed; not worst-case timing on hardware |
| W09-DV08 → P6-V24 (with W13) | telemetry correlation | run DV02/DV03 with telemetry enabled | all six events observable and correlated per contract §5 | observability of the implemented path; not event-name stability (P0-W13 owns it) |

## 4. Error, security, and observability model

Errors are partitioned by the contract §4 cases E1–E4 with fixed containment
classes: benign counters (E1), contained hardware/specification diagnostics
(E3, E4), and fatal-invariant escalation for state-integrity loss (E2) per
[P0-W14](../../../p0/plans/p0-w14-panic-failure-classification.md). The
security position: the path is not Guest-reachable; Guest influence arrives
only through architectural controller state; the E1/E2 distinction keeps
Guest-influenceable anomalies from being misread as Host integrity failures
and vice versa. Observability is the six-event telemetry set plus the two
diagnostic counters; raw log dumps are never the contract surface
(P0-W12/P0-W13 govern mechanisms). QEMU success does not prove real-hardware
behavior; register sequencing evidence must carry its environment statement
(Coding Guidelines; task book §1).

## 5. Handoff checklist

Before handing W09 to a reviewer, provide:

- the exact changed-file list and the final Rust module binding recorded per
  parent README decision 6;
- step 1 prerequisite verdicts and step 2 specification-reconciliation facts
  (spec revision, field map, barrier rules);
- the new-`unsafe` inventory delta (expected: only the W08-access-mediated
  register reads, if any new);
- W09-DV01–DV08 evidence paths with run/failed/blocked/not-run status,
  including explicit not-run entries (for example real-hardware behavior);
- confirmation that no vGIC MMIO handler, LR allocation policy change, queue
  discipline change, scheduler hook, or Guest-callable interface was added;
- open items for W10 (completed-state meaning), W11 (VG-IRQ-06 observables),
  W12 (FI-C orphan case, storm boundary), W13 (telemetry coverage,
  maintenance-frequency data class), and P8 (evidence-limited consumption) —
  without resolving their contracts here;
- any recorded Architecture Change Request or Specification Investigation
  with its owner.
