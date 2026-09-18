# P8-W13 Implementation Workflow and Review Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W13 detailed design](README.md).

## 1. Preconditions and failure boundary

Before any step, the implementer verifies it has loaded the documents named in
the parent README and inspects the actual state of the prerequisites: the P0-W12/W14
records, the P4 stage material referenced by the P4-W09 handoff
(`../../../p4/plans/p4-w09-closeout-p5-handoff.md`), the W05–W12 designs and
records, and the W16 harness design. P8 consumes only evidenced predecessor
facts ([plans index](../../plans/README.md)).

Stop and obtain direction instead of improvising when any of the following
occurs:

- the evidenced P4-W06 boundary does not produce a field this design's context
  requires — record a `P4DependencyIssue`/diagnostic gap against the P4 owner
  and mark the affected class's coverage blocked; do not add a capture
  mechanism from W13;
- the delivered P0-W12/W14 semantics differ from the assumed classes — stop and
  record the conflict against those baselines' authorities; the taxonomy's top
  level is not renegotiable here;
- a W05–W08 sibling design does not declare the fault surface a class assumes —
  mark that class's coverage blocked; do not invent the surface;
- inducing a class appears to require a fault-injection API, crash-dump
  mechanism, new trace field, or fault-handler change — stop; that is an
  `Architecture Change Request` or a gap for the owning package, never a W13
  addition;
- W16's harness realizes the [02 §6](02-diagnostic-context-contracts.md)
  assertion semantics differently — raise the conflict; this design owns the
  semantics, W16 owns mechanics;
- a real event matches no rule — record it as `F13-0` with full context and
  carry it to closure review (step 6); do not force a class;
- a Guest-facing event produces an EL2-level outcome — classify
  `F13-9`, record as hypervisor-level failure per P0-W14, and hand the finding
  to the containment-contract owners; never absorb it as a VM-facing result.

## 2. Ordered implementation steps

### Step 1 — verify prerequisites and record the baseline

Target: implementation record (`../p8-w13-guest-fault-diagnostics-record.md`,
created in this step).

Work: for each row of [01 §1](01-fault-classification.md), locate the evidenced
fact and record its pointer, or mark the row blocked with the reason. Record
which fields the evidenced P4-W06 boundary actually produces and which context
fields are therefore available.

Suggested observation: read the P0/P4 records and the sibling designs' declared
fault surfaces; no runtime action.

**Acceptance:** every row has an evidence pointer or explicit blocked entry;
the available context-field set is named.
**Failure/blocker:** missing P4 fields block the affected classes' sufficiency
checks; they are blocked, not skipped.

### Step 2 — review taxonomy traceability and coverage declarations

Target: the taxonomy ([01 §3](01-fault-classification.md)) and coverage map
([01 §7](01-fault-classification.md)) as instantiated in the record.

Work: trace every class to its declaring upstream surface; for every
Guest-facing class, confirm a declared inducing scenario and its expected
contained outcome exist among the W05–W12 scenarios and fixture hooks. Confirm
F13-9/F13-10/F13-0 are non-induced by design.

**Acceptance:** W13-DV02 review passes: all plan-listed failure kinds
(panic, sync exception, Stage-2, unsupported sysreg, invalid MMIO, PSCI, vGIC,
timer, vCPU-state, Hypervisor-invariant) map to classes with declared sources.
**Failure/blocker:** a class without a declared source or scenario has its
coverage marked blocked with the gap named.

### Step 3 — check context sufficiency definition against real fields

Target: `FaultDiagnosticContext` instantiation and the sufficiency predicate
([02 §3](02-diagnostic-context-contracts.md), [02 §5](02-diagnostic-context-contracts.md)).

Work: map each required context field to its producing evidenced capture path;
record the trace-window bound. Verify with W16 that the predicate is checkable
from harness-observable data.

**Acceptance:** W13-DV03 review passes: every field has a producer or an
explicit gap entry; the bound is recorded; no fabricated fields.
**Failure/blocker:** a field without a producer is a diagnostic gap for the
owning package; the affected classes' "actionable" clause is blocked.

### Step 4 — execute fault scenarios through the W16 harness

Target: verification record
(`../../verification/p8-w13-guest-fault-diagnostics-verification.md`).

Work: run the [01 §7](01-fault-classification.md) inducing scenarios via W16
(using the W15 fixture hooks and W05–W12 scenarios). Record commands,
environment, console transcripts, captured context records, and verdicts.
Record what was not run and why.

**Acceptance:** each executed scenario yields its event's classification,
context record, containment outcome, and console observable.
**Failure/blocker:** an execution failure is evidence — record failed/blocked;
do not relax assertions.

### Step 5 — evaluate coverage and sufficiency

Target: coverage table in the verification record.

Work: per class, apply the [02 §6](02-diagnostic-context-contracts.md)
assertion set to every induced event; compile the coverage table (class →
scenarios → outcomes). A class counts as covered only when all five assertions
hold for at least one executed scenario.

**Acceptance:** the P8-V18 substance: every Guest-facing class with an
unblocked source shows declared coverage; every `HypervisorInvariantViolation`
or forbidden outcome is recorded as a hypervisor-level finding, not normalized.
**Failure/blocker:** insufficient context or wrong containment fails the class
and is routed per §1; coverage is never claimed from taxonomy alone.

### Step 6 — closure review

Work: review every `F13-0` block and decide (recorded): block stands, bounded
design investigation in W13's lane, or gap routed to the owning package. Run
the validation matrix (§3), confirm the handoff checklist (§5), and verify the
package against the plan's acceptance wording and the task-book P8-V18 row,
including its explicit limitation: no full crash-dump claim. Completion is
claimed only in the verification record, with evidence, and only for what was
actually run.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W13-DV01 → prerequisite review | upstream diagnostic boundary review | inspect P0/P4/P8 evidence against 01 §1 rows | every row evidenced or explicitly blocked; available fields named | the baseline is real; not that classes are exercised |
| W13-DV02 → P8-V18 | taxonomy review | trace classes to sources; coverage-map completeness | all plan-listed failure kinds covered by classes with declared sources; F13-0 route defined; no new failure model | the classification is complete for declared surfaces; not that events occur |
| W13-DV03 → P8-V18 | context/trace review | map fields to producers; bound the trace window | every field has a producer or explicit gap; bound recorded; host-leak rule encoded | the context set is actionable and checkable; not that assembly works |
| W13-DV04 → P8-V18 | fault-class diagnostic run | execute 01 §7 scenarios via W16; apply 02 §6 assertions | every unblocked Guest-facing class shows ≥1 scenario with all five assertions true; forbidden outcomes absent or escalated as F13-9 findings | declared diagnostic coverage with VM/vCPU/PC/PSTATE/syndrome/address/exit/trace context; not a full crash-analysis capability |
| W13-DV05 → P8-V18 | containment review | inspect all executed fault events for VM-scoped outcomes | zero Guest-fault-to-EL2-event occurrences; F13-9 occurrences (if any) escalated per P0-W14 | Guest faults stay contained; not that all future fault kinds do |
| W13-DV06 → P8-V18 | observable-mapping review | review 01 §6/02 §6 against W16/W18 designs | every class's console observable and assertion set is realizable by W16 without new mechanisms | the regression mapping is consumable; not that W16 is implemented |
| W13-DV07 → closure | unclassified/block review | step-6 review of F13-0 entries | every block resolved to a recorded decision; none silently dropped | the escape route worked; not that no unknowns remain |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. A completed taxonomy
without W13-DV04 execution does not satisfy P8-V18. Nothing here contributes to
P8-V12–V17 (owned by W09–W12) or P8-V19–V26.

## 4. Error, security, and observability model

**Errors and failure guarantee.** W13 adds no runtime error path of its own.
Its failure model is evidential and conservative: `classify_guest_fault` is
total with an `Unclassified` escape that always blocks rather than guesses;
insufficient context fails the actionable-diagnostics clause rather than
passing; containment deviations escalate to `F13-9` rather than being absorbed.
The preserved guarantee: Guest-caused faults remain VM-scoped (ADR §19, P0-W14);
the only hypervisor-level path is genuine invariant failure handled by the
P0-W14/P1-W07 authorities, never defined here.

**Security.** Linux is untrusted (ADR-007). Classification and sufficiency
never trust Guest-writable contents: console text corroborates, it does not
decide; a Guest cannot select its own class. Diagnostic records exclude Host
physical addresses from any Guest-visible portion. No new attack surface is
created: no injection API, no fault handler, no new hypercall or trace field.
F13-9 is never deliberately induced. Coverage exercises declared W05–W08
surfaces and fixture hooks only.

**Observability.** The evidence surface is the verification record: per-event
classification + context records with the bounded recent-trace window, console
transcripts, the coverage table, the forbidden-outcome log, and the F13-0 block
list. The bounded trace window and per-event sufficiency checks make "minimum
actionable context" reviewable rather than rhetorical; crash-dump-class capture
is explicitly out of scope and its absence is a recorded limitation for W20.

## 5. Handoff checklist

Before handing W13 to a reviewer, provide:

- the exact changed-file list (expected: the implementation record; verification
  entries; no source, fault-handler, or trace-encoding changes);
- the baseline table with pointers or explicit blocked entries (W13-DV01);
- the taxonomy instantiation with per-class source traceability and the
  coverage map (W13-DV02);
- the context field-to-producer map and recorded trace-window bound
  (W13-DV03);
- W13-DV04/W13-DV05 outcomes: per-class coverage rows with assertion results,
  the forbidden-outcome log, and any F13-9 escalations, including explicit
  not-run entries;
- the F13-0 block decisions (W13-DV07);
- confirmation that no crash-dump, fault-handler API, trace encoding, panic
  policy, EL2 control-flow, or machine-ABI change was made or requested from
  W13;
- open items: diagnostic gaps for the P4/P0 owners, undeclared-surface gaps for
  W05–W08 owners, harness semantic conflicts for W16 — without resolving them
  here.
