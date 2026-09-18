# P5-W10 Closeout Workflow and Validation

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W10 detailed design](README.md).

## 1. Preconditions and failure boundary

Before writing any closeout artifact, the closeout agent verifies it has
loaded the documents named in the parent README and inspects the actual
records:

- W01–W09 implementation records and verification records exist and are
  reachable at their governed paths;
- W01's routing deliverable names the governed locations for ABI, security,
  handle/capability, and input-safety artifacts;
- the P5 task book's exit criteria (EC-P5-01–EC-P5-12) and validation rows
  (P5-V01–V17) are at hand as the closure checklist.

Stop and obtain direction instead of guessing when: a package's record is
missing (that package's rows are recorded as not implemented — never
summarized from its plan); a record contradicts another (the conflict is
recorded and routed to the owning packages; the affected closeout row is
parked); W01's routing is absent for a publishable artifact (parked pending
routing); or a closure claim is requested for anything without linked
evidence (refuse — this package cannot manufacture evidence).

## 2. Ordered implementation steps

### Step 1 — inventory the P5 records

Target: implementation record
(`../p5-w10-closeout-p6-handoff-record.md`, created in this step).

Work: list every W01–W09 record and verification artifact with its path and
status; map each EC-P5 and P5-V item to its owning package(s) and evidence
rows; identify the gaps.

**Acceptance:** a complete inventory in which every exit criterion has an
owner, evidence references, or an explicit absence.  
**Failure/blocker:** a missing record is recorded as absence for its
package — the inventory is complete precisely because absences are named.

### Step 2 — compile the factual records

Target: the closeout record's sections per
[01 §2](01-closeout-contract.md) and [01 §5](01-closeout-contract.md).

Work: compile environment/execution facts, the unsafe and dependency
deltas, known limitations, the regression inventory, and open decisions
from the packages' records. Compile only; flag any content that would
require a technical decision for the owning packages.

**Acceptance:** every statement carries a linked source record; compile-only
status is preserved.  
**Failure/blocker:** a statement without evidence is dropped or parked —
never smoothed into plausibility.

### Step 3 — operate the publication gate

Target: `docs/abi/` and `docs/security/` artifacts and the link index, per
[01 §3](01-closeout-contract.md) and W01's routing.

Work: for each candidate artifact, run the four gate checks; publish,
park with reason, or record pending status; build the link index.

**Acceptance:** every published artifact passes all four checks with the
evidence linked; every parked artifact has its reason recorded.  
**Failure/blocker:** gate failure is recorded as parking — publication
never proceeds on a failed check.

### Step 4 — require fresh verification where closure depends on it

Target: the verification record
(`../../verification/p5-w10-closeout-p6-handoff-verification.md`).

Work: if any closure-relevant evidence is stale relative to the final P5
tree (later changes touched a verified boundary), require the owning
package to re-run the affected validation rows and record the fresh status
before closure. W10 does not run tests itself.

**Acceptance:** every closure-relevant evidence row is either fresh or has
a recorded re-run requirement with owner.  
**Failure/blocker:** an owner unable to re-run records the block; closure
cannot proceed on that item.

### Step 5 — reconcile the invariant inventory

Target: the reconciliation of
[01 §4](01-closeout-contract.md).

Work: complete one row per INV-P5-nn with enforcers, evidence, status, and
deferrals; cross-check the regression rows against W09's frozen list.

**Acceptance:** ten rows, each evidence-linked; no row's status exceeds its
evidence.  
**Failure/blocker:** a status/evidence mismatch is corrected downward, not
upward.

### Step 6 — evaluate closure and produce the handoff

Target: the verification record (closure review) and the handoff section of
the closeout record.

Work: evaluate P5-V17 and, from it, the closure status of EC-P5-01–12 and
P5-V01–V17 strictly against the inventory: every claim is evidence-backed;
every gap is explicit; nothing is manufactured. Produce the P6 handoff per
[01 §6](01-closeout-contract.md): the consumer map with record references
and the non-deliverables list.

**Acceptance:** the closure evaluation names exactly what is evidenced,
what is partially evidenced, and what is unimplemented; the handoff is
usable by P6-W01's entry review without further compilation.  
**Failure/blocker:** any pressure to convert a gap into a claim stops the
review — the outcome is a recorded partial closure or continued-open
status, both legitimate outcomes of an honest evaluation.

### Step 7 — final link and consistency review

Target: all closeout artifacts.

Work: resolve every relative link from a fresh checkout; confirm the stage
implementation index reflects truthful status; confirm no closeout artifact
claims completion beyond the verification record; confirm the task book's
non-handoffs are restated in the handoff section.

**Acceptance:** links resolve; statuses truthful; no claim inflation.  
**Failure/blocker:** an inconsistent artifact is fixed or parked before
review.

## 3. Validation matrix

Record each validation as **passed**, **failed**, **blocked**, or **not
run** with artifact, check, and timestamp. These are documentation and
gate reviews, not runtime tests; their evidence is the reviewed artifacts
themselves.

| ID | Review | Passing condition | Proves / does not prove |
|---|---|---|---|
| W10-DV01 → P5-V17 | record inventory review | Step 1 inventory complete with absences named | traceability of closeout inputs; not that the inputs are true (their evidence backs that) |
| W10-DV02 → P5-V17 | factual-document review | Step 2/3: every published artifact passes the §3 gate; minimums of [01 §2](01-closeout-contract.md) met | published facts are implemented-and-evidenced; not that unpublished work exists silently (parked items are listed) |
| W10-DV03 → P5-V17 | invariant reconciliation review | Step 5: ten rows, statuses ≤ evidence | the permanent invariant set's true status; not closure of gaps |
| W10-DV04 → P5-V17 | evidence and regression index review | Step 4 freshness rule applied; W09 inventory referenced with version | evidence currency; not re-execution by W10 |
| W10-DV05 → P5-V17 | deltas and limitations review | [01 §5](01-closeout-contract.md) records complete, compiled-only | factual reporting per AGENTS.md rules; not resolution of open items |
| W10-DV06 → P5-V17 | P6 consumer-map review | [01 §6](01-closeout-contract.md) map complete; each row record-referenced; non-deliverables restated | P6 can consume evidenced facts only; not P6 design adequacy |
| W10-DV07 → P5-V17 | closure evaluation review | Step 6: claims strictly evidence-backed; gaps explicit | an honest closure status; this review cannot manufacture runtime evidence |
| W10-DV08 → W10 closure | link and consistency review | Step 7 checks pass | closeout usability; not downstream stage work |

## 4. Error, security, and observability model

The closeout's failure mode is overstatement, and its control is the
evidence-backing rule plus the downward-correction rule (statuses never
exceed their evidence). Its security duty is faithful compilation of the
redaction and containment facts into the security documentation route —
including the commitment-level marking that keeps the experimental
internal boundary from reading as a public contract. Its observability is
the closeout record itself: inventory, gates, reconciliations, and the
handoff map, all link-resolvable from a fresh checkout. No code, no
`unsafe`, and no runtime surface is created by this package.

## 5. Handoff checklist

Before handing W10 to a reviewer, provide:

- the complete W01–W09 record inventory with per-item status and the named
  absences;
- the publication-gate table: published / parked / pending per artifact,
  with reasons and governing routes;
- the invariant reconciliation (INV-P5-01..10) with statuses and deferrals;
- the environment, deltas (unsafe, dependencies), limitations, and
  regression-inventory records with their source links;
- the P6 consumer map with record references and the non-deliverables
  list;
- the closure evaluation with its evidence-backed claims and explicit
  gaps, and every open `Architecture Change Request` / `ADR Required` /
  blocked-prerequisite record carried forward with owner;
- confirmation that no runtime code, ABI values, P6 semantics, upstream
  edits, or unevidenced claims were introduced by the closeout.
