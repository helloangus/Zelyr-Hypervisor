# P3-W15 Acceptance Artifacts and Checks

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W15 detailed design](README.md).

## 1. Role

This file fixes the authoritative artifact groups, the checks C1–C8, the
traceability-matrix construction, the evidence rules, the open-issue
register, and the closure-review package template. It defines the
acceptance apparatus; it produces no results and opens no gate.

## 2. Artifact groups and ownership

| Artifact group | Authoritative owner | W15's relation to it |
|---|---|---|
| P3 task book (`task-book-v0.1.md`) | stage planning authority | read-only source for outcomes and validation IDs; findings reported, never edited |
| Plans index and plans (`plans/README.md`, `plans/p3-w*.md`) | stage planning authority | read-only source for coverage and dependency mapping; findings reported |
| Implementation designs (`implementation/p3-w*-*/` README + supporting files) | each package's design | checked for status truthfulness and link integrity; defects are findings to the owning package |
| Implementation records (`implementation/p3-w*-record.md`) | each implementing package | located and status-checked (exists/absent); never written by W15 |
| Verification records (`verification/p3-w*-verification.md` and attached captures) | each implementing package | located and status-checked; never created or edited by W15 |
| P4 handoff contract (`implementation/p3-w14-p4-smp-handoff-contract.md`) | P3-W14 | linked as the P4-facing summary; content owned by W14 |
| Stage README (`../README.md`), implementation index, verification directory | stage-level files; the implementation index is coordinator-owned | W15 keeps truthful only the rows that present W15's own artifacts, where those rows are W15's to write; all other rows are checked and reported |
| W15's own artifacts: the traceability matrix, evidence-location map, open-issue register, checks log, closure-review package | W15 | authored and maintained here; the only documents W15 writes |

Rule: every statement about P3 has exactly one home in this table; a
statement made in two homes is a C6 finding.

## 3. Traceability matrix construction

Construct a matrix with one row per task-book requirement and one per
validation ID (rows may share when the task book already groups them):

```text
| Task-book outcome (§3 ID) | Plan (exact path) | Design (path) |
| Impl. record (path/status) | Verification record (path/status) | Validation IDs (§5) |
```

Construction rules:

- Every §3 outcome row resolves to exactly one plan (the "exactly one"
  property); every §5 validation ID appears in ≥ 1 row and every row's
  IDs exist in §5.
- Every cell is a resolving relative path or an explicit `ABSENT`
  marker with date — `ABSENT` is information, not failure; a fabricated
  path is.
- The matrix includes its own P3-V15 row (evidence: W15's documentation
  review record), which does not exempt any other row.
- The matrix is regenerated (not incrementally patched) whenever an
  upstream document set changes; each regeneration is dated in the
  checks log.

## 4. Checks C1–C8

| ID | Object | Method | Pass condition | Failure meaning |
|---|---|---|---|---|
| C1 | Status truthfulness | scan every P3 document's status header | statuses are truthful ("Proposed detailed design; implementation and validation are not claimed" where nothing was implemented; records exist only where work happened); no completion wording before its evidence | a status lie is a finding with owner; blocks W15's own closure inputs |
| C2 | Link resolution | resolve every relative link in the P3 document set | all resolve from the repository root of the stage tree | a broken link is a finding with owner |
| C3 | Plan coverage | the §3 matrix | every outcome → exactly one plan; every validation ID mapped; no orphan plan/outcome | a duplicate or missing mapping is a finding for the planning authority |
| C4 | Dependency acyclicity | reconstruct the dependency graph from the plans index | acyclic; every cited prerequisite/consumer relationship matches the index | a cycle or mismatch is a finding for the planning authority |
| C5 | Evidence-location truthfulness | the evidence-location map (§5) | every map row names a real path or `ABSENT (dated)`; no record exists that its package's status does not support | an invented path or an orphan record is a finding |
| C6 | Layer separation | sample each document for foreign-layer content (evidence in designs, design decisions in verification records, closure claims anywhere) | no violation | a violation is a finding with owner; `docs/README.md` rules violated |
| C7 | Scope-boundary wording | scan for guest/Stage-2/vCPU/GIC commitments in P3 documents and for QEMU-as-semantics wording | P3 documents keep the reserved/out-of-scope boundaries; QEMU statements are labeled observations | a boundary breach is a finding, potentially `ADR Required` if architecture-level |
| C8 | Unperformed-work wording | scan plans/designs/records for claims implying run work | every run-work reference is conditional or status-marked (planned/blocked/not-run) | an implied claim is a finding; a false claim escalates |

Each check run is logged (date, scope, result, findings with owners) in
the W15 checks log — part of W15's implementation record area. Checks
are re-run when the document set changes; the log keeps history.

## 5. Evidence-location map

One row per P3 package W01–W15 (plus stage-level rows):

```text
| Package | Design path | Impl. record (path or ABSENT, date) |
| Verification record (path or ABSENT, date) | Validation IDs | Status (from the row's own header) |
```

The map is W15's single answer to "where is P3's evidence"; the closure
package and P4-W01 both consume it. Maintenance rule: statuses are
copied from the owning documents, never paraphrased into optimism.

## 6. Documentation-review evidence rules

- Every check run, matrix regeneration, and map update is evidence;
  log entries carry date, scope, result.
- W15's own verification record
  (`../../verification/p3-w15-stage-documentation-acceptance-verification.md`,
  created only when produced) holds the review evidence for P3-V15:
  which checks ran, what they found, what remains open.
- Absence is recorded as absence. The honest pre-implementation state is:
  designs present for W01–W15 (as they land), records `ABSENT`, closure
  package gated.

## 7. Open-issue register

Assembled from: upstream records' open items (P2-ACR-01 by reference),
the W14 contract's unresolved register, C7 boundary findings, and any
`ADR Required` / `Architecture Change Request` labels in the document
set. Row shape: issue, source (path), owner, status (open / resolved
where), downstream impact. The register is the stage's single visible
list of what P3 is still carrying; it lives with W15's artifacts and is
included in the closure package.

## 8. Closure-review package

Template (fixed now, assembled only when the gate opens):

1. Stage summary from the task book (what P3 is; no new claims).
2. The traceability matrix (§3) and evidence-location map (§5).
3. Per-validation evidence pointers P3-V01–V15 with each record's own
   status.
4. The open-issue register (§7).
5. The P4 handoff pointer (W14 contract) and the hardware-gap statement.
6. Known limits and non-guarantees (QEMU reference scope, no performance,
   no hardware proof).

**Gate:** the package is assembled only when every P3-V01–V15 row's
verification record exists with real evidence and all C1–C8 findings are
either resolved or registered with owners. Until then, the template and
the current check/matrix/map outputs exist, and W15's status states the
deferral plainly. The package recommends; it does not certify — closure
is decided by the stage reviewer against the task book §7 questions.
