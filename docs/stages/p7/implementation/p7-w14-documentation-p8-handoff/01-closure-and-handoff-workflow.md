# P7-W14 Closure and Handoff Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W14 detailed design](README.md).

## 1. Preconditions and failure boundary

W14 runs only when the packages it closes over have produced records — or
have definitively not, which W14 then records as an outstanding item. Before
executing any step, the implementer verifies it has loaded the documents
named in the parent README.

Stop and record a blocker instead of improvising when:

- a required input (W12 matrix results, W13 baseline, any W01–W11 record) is
  absent — record it as an outstanding item with its owner; do not
  reconstruct its content from plans;
- a factual statement cannot be linked to an existing implementation or
  verification record — reclassify it as planned work or drop it;
- reconciling exit criteria exposes a contradiction between records or with
  a governing document — record the conflict as an issue for the governing
  owners; W14 never edits a task book, plan, ADR, or another package's
  record to resolve it;
- the P8 handoff review finds a mapping row that would transfer an
  unevidenced fact or a P8-owned decision — remove or void that row in the
  handoff record and note the gap; do not widen the boundary to make the
  handoff look complete.

## 2. Authoritative artifact groups and required content

### 2.1 Scheduler behavior record — `docs/stages/p7/implementation/p7-w14-scheduler-behavior.md`

Factual documentation of scheduler behavior as implemented, with one section
per behavior area: entry/admission and lifecycle (W02), placement and
configuration (W03), preemption and context switch (W04), M:N and multi-VM
scheduling (W05), block/wakeup (W06), pause/stop/fault (W07), SMP/idle
(W08), accounting/trace/diagnostics (W09), Validation Guest workloads (W10),
stress and invariant evidence summary (W11), QEMU regression (W12),
performance baseline (W13). Every section uses the three-way classification:

- **Implemented fact** — a statement of behavior with a link to the
  implementation record and the verification evidence that supports it;
- **Limitation** — a boundary on what the evidence shows (QEMU-only,
  non-coverage, bounded fairness, environment-specific timing);
- **Planned, not implemented** — behavior the stage intends but that has no
  evidence; stated without mechanism detail that no approved design owns.

The record carries a factual header (status, scope, owner/change context,
supersedes) per the documentation-index rules and states that it documents
P7 facts only, not P8 machine behavior.

### 2.2 Evidence index — `docs/stages/p7/verification/p7-w14-evidence-index.md`

One row per task-book validation ID P7-V01–P7-V30: the owning package, the
evidence record(s) and artifact locations, and the status passed / failed /
blocked / not run, with a reason for every non-passed row. The index is
complete over the ID range even where the status is not run; completeness of
coverage is what makes the closure review (step 3) possible. It links to,
and never duplicates or re-judges, the underlying verification records.

### 2.3 Closeout and handoff record — `docs/stages/p7/implementation/p7-w14-documentation-p8-handoff-record.md`

The package's implementation record and the P8-facing handoff in one
bounded document, containing:

- the changed-artifact list of W14 itself and links to all P7 implementation
  records;
- the unsafe-delta inventory: new `unsafe` introduced by P7 implementation,
  with links to the governing P0 unsafe-governance records — aggregated from
  implementation records, never estimated;
- the dependency-status inventory: dependencies added by P7 implementation
  with the same linkage;
- the limitations and unresolved-conflicts list compiled from step 3;
- the P8 handoff: the consumer-mapping table of the parent README,
  instantiated with per-row evidence links and current status, plus the
  boundary statements;
- the closure-readiness statement: either that the closure review passed
  with evidence for all gates, or the explicit list of outstanding, failed,
  or blocked items — never an unqualified completion claim.

## 3. Ordered implementation steps

### Step 1 — collect W12/W13 and all-package records

Target: closeout record (draft structure).

Work: per the plan's first work item, collect the W12 automation results and
evidence locations and the W13 baseline record and limitations, then the
implementation and verification records of W01–W11. Record what exists, what
is missing, and the location of each item.

**Acceptance:** an inventory of all P7 records exists in the closeout
record, with explicit missing entries.  
**Failure/blocker:** a missing W12/W13 input is an outstanding item with
owner; the affected closure sections state the gap.

### Step 2 — write the behavior record

Target: behavior record (§2.1).

Work: write each behavior-area section strictly from the collected records,
applying the three-way classification; keep mechanism descriptions at the
level the implementation records support.

**Acceptance:** every fact statement carries an evidence link; every section
has its limitation and planned-work entries; no statement describes P8
mechanisms.  
**Failure/blocker:** an unlinkable statement is reclassified or dropped;
the removal is noted.

### Step 3 — build the evidence index and reconcile exit criteria

Target: evidence index (§2.2); reconciliation content in the closeout
record.

Work: build the P7-V01–V30 index from the verification records; reconcile
against the task-book exit criteria (§7): CPU-bound preemption, M:N
stability without permanent starvation, blocked-vCPU release and wakeup,
no double-run, affinity/pinning/pause/stop/fault/SMP semantics, Validation
Guest/stress/QEMU evidence, telemetry/diagnostics/baseline reviewability,
and no board-specific Core contract. Compile the limitations, unsafe-delta,
and dependency inventories, and list unresolved conflicts.

**Acceptance:** the index is complete over P7-V01–V30; every exit criterion
is mapped to its evidence or an explicit gap; inventories are link-complete.  
**Failure/blocker:** a contradiction between records is recorded as an issue
with the governing owners named; it is not silently harmonized.

### Step 4 — review the P8 handoff against the boundary

Target: closeout record (handoff section, §2.3).

Work: instantiate the consumer-mapping table with evidence links; check each
row against the task-book §7 boundary and ADR constraints: no machine ABI,
no Linux boot, no guest SMP presentation, no management policy transfers;
every row is evidenced or voided; QEMU evidence does not transfer as
hardware semantics. Read the resulting handoff as a P8-W01 entry reviewer
would and record that review.

**Acceptance:** every mapping row carries evidence or is explicitly voided;
the boundary statements are present; the P8-consumer reading found the
handoff consumable.  
**Failure/blocker:** a row that cannot be evidenced is voided with the gap
named, not papered over.

### Step 5 — record closure readiness

Target: closeout record (closure statement); stage implementation index
status rows if maintained.

Work: write the closure-readiness statement per decision 6 of the parent
README — either closure-review-passed with the evidence map, or the explicit
outstanding list — and set truthful status rows in any stage index. Update
nothing else; make no completion claim beyond the recorded evidence.

**Acceptance:** the statement is supported by the index and inventories; a
reader can determine P7's true state from the three artifacts alone.  
**Failure/blocker:** any pressure to declare completion without full
P7-V01–V30 evidence resolves to the outstanding-items form.

## 4. Validation matrix

Every row is planned review evidence; none asserts that the review has run.
Each is recorded as passed / failed / blocked / not run with inputs,
timestamp, and reason.

| ID | Review | Technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W14-DV01 | Input-collection review | Workflow step 1 against §2.3's inventory requirement | All P7 records inventoried; missing items explicit with owners | Collection completeness; not that the inputs are correct |
| W14-DV02 → P7-V30 | Behavior-record review | Workflow step 2; sample facts back to evidence | Every fact linked; classification discipline holds; no P8 content | Factual behavior documentation exists and is traceable; not that documented behavior is correct (that is the linked evidence's claim) |
| W14-DV03 → P7-V30 | Index and exit reconciliation review | Workflow step 3 | P7-V01–V30 fully indexed with statuses; every exit criterion mapped; inventories link-complete | Closure readiness is reviewable; not that outstanding items are acceptable |
| W14-DV04 → P7-V30 | P8-handoff boundary review | Workflow step 4, read as P8-W01 would | Mapping rows evidenced or voided; boundary statements present; nothing P8-owned pre-empted | The handoff is bounded and consumable; not that P8 accepts it (P8-W01/P8-W20 own that judgment) |
| W14-DV05 | Closure-claim discipline review | Workflow step 5 | Closure statement matches the index exactly; no unsupported completion claim anywhere in W14 artifacts | Discipline holds; not that P7 is complete |

No validation here creates P7-V01–V29 evidence or substitutes for it, and
none may be reported as doing so.

## 5. Error, security, and observability model

W14 adds no runtime error path. Its failure mode is documentation dishonesty
— facts without links, plans dressed as facts, limitations omitted, or a
handoff that quietly widens its boundary. Each defect fails the associated
review and is recorded as such; the remedy is reclassification or voiding,
never embellishment. Security-relevant closure content (unsafe delta,
containment limits, isolation evidence status) is inventoried factually so
the P8 security/isolation work (P8-W18) starts from the true state.

Observability is the three-artifact set itself: a reviewer starting from the
stage implementation index must reach the behavior record, the evidence
index, and the closeout/handoff record, and from them determine P7's actual
state without reading any package record in full.

## 6. Handoff checklist

Before closing W14, provide:

- the three artifacts of §2, complete per their content requirements;
- the record inventory including explicit missing items with owners;
- the unsafe-delta and dependency inventories with governing-record links;
- the instantiated P8 mapping with voided rows named;
- the closure-readiness statement and truthful index status rows;
- confirmation that W14 modified only its own artifacts and index status
  rows — no task book, plan, ADR, other-stage document, source, or evidence
  file was altered.
