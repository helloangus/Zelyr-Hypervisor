# P3-W13 Workflow, Validation, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W13 detailed design](README.md).

## 1. Preconditions and failure boundary

Before any work, the implementer verifies it has loaded the parent README
and the supporting files its step needs, and inspects the current tree
(`git ls-files`; which P3 records, which P0 automation contracts exist).
Stop and record instead of improvising when:

- the P0-W09 automation entry does not exist — all QEMU rows are
  **blocked**; authoring a runner, script, or CI file here is out of
  scope;
- a mechanism an assertion cites is missing or changed semantics — the
  row is blocked on that contract; assertions are never re-derived
  locally;
- a row appears to require a QEMU-specific branch or constant in
  hypervisor code — design conflict; the expectation is recorded as a
  QEMU observation instead;
- repetition depth or scenario depth proves impractical — change via a
  recorded decision with rationale, never mid-campaign;
- a row fails — it is recorded `run-failed` with captures; silently
  re-running until green is forbidden (a divergent boot is evidence).

## 2. Ordered workflow

### Step 1 — verify assertion sources and automation contract

Target: the R-series matrix.

Work: check each cited design surface exists in the current
implementation; check the P0-W09 entry contract status for rows'
execution path.

**Acceptance:** per-row source status recorded; execution path status
(executable / blocked-on-P0-W09) recorded.  
**Failure/blocker:** blocked rows stay in the matrix with their blocker
cited — removal would hide the gap.

### Step 2 — fix repetition and depth constants

Target: implementation record (created in this step).

Work: fix R13, the full-depth session subset policy, and the deployment
mapping against W12's constants, with rationale per
[02-regression-matrix.md](02-regression-matrix.md) §4–§5.

**Acceptance:** every row has a complete repetition/depth statement
before any campaign.  
**Failure/blocker:** a constant without rationale fails review.

### Step 3 — confirm capture plumbing against the automation entry

Target: the capture set of
[03-result-classification-and-evidence.md](03-result-classification-and-evidence.md)
§4.

Work: verify each capture item is obtainable from the automation entry's
declared outputs (or via the hypervisor's own diagnostic outputs on the
boot log); name the mapping per item.

**Acceptance:** every capture item has a source; items with no source are
resolved at design level or the affected rows are blocked.  
**Failure/blocker:** a capture gap is never closed by weakening an
assertion.

### Step 4 — dry-run classification rules

Target: classification vocabulary.

Work: against one available boot (or a fixture capture), classify it per
§3 to prove the vocabulary is decidable; record the exercise as the
classification check.

**Acceptance:** the classification produces one unambiguous status from a
real capture.  
**Failure/blocker:** ambiguity is fixed in the rules here, not per-run.

### Step 5 — limits and platform-independence review

Target: the whole matrix contract.

Work: review that (a) no row asserts guest/Stage-2/GIC behavior, (b) no
pass condition uses timing, (c) no row needs a board/QEMU-name branch or
QEMU-only constant in Core, (d) QEMU-specific expectations are recorded
as observations, (e) the hardware-gap statement stands per row.

**Acceptance:** W13-DV06 evidence recorded.  
**Failure/blocker:** a violation is fixed in the row; never absorbed by
rewording an ADR constraint.

### Step 6 — execute campaigns and record results (deferred phase)

Target: verification record
`docs/stages/p3/verification/p3-w13-qemu-smp-regression-verification.md`
(created when evidence exists).

Work: for each count c ∈ {1, 2, 4, 8}: run the campaign per the
repetition policy; capture per session; classify every row honestly per
§3; record blocked/not-run rows with reasons; keep superseded campaigns.

**Acceptance:** P3-V13 is satisfied only by a complete campaign at all
four counts with `run-passed` on every applicable row within declared
depths — and the record says exactly that and nothing stronger.  
**Failure/blocker:** a failing row is a result; the campaign records it
and stops claiming coverage for the affected row.

### Step 7 — closure review

Work: run the validation matrix below and the handoff checklist; confirm
row-to-P3-V13 traceability is complete and the hardware gap is carried.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W13-DV01 → P3-V13 | matrix completeness review | inspect R1–R8 against the plan scope (boot, online-set, stress, cross-CPU event, allocator) and the 1/2/4/8 axis | every declared criterion has row(s) at every applicable count; assertions cite owning designs; no orphan criterion | the contract is complete; not that anything passes |
| W13-DV02 → P3-V13 | repetition policy review | inspect depth constants, subset policy, and drift rules | every row's repetition statement is complete, bounded, and rationale-backed; race-exposure purpose stated honestly | repeatability is well-defined; not race freedom |
| W13-DV03 → P3-V13 | automation-contract consumption review | inspect §2 consumption rules against the delivered P0-W09 contract (or its absence) | every row executable or explicitly blocked with the blocker cited; no local scripts/workflows authored | the P0 contract is retained, per the plan |
| W13-DV04 → P3-V13 | classification and capture review | step 4 dry run + inspect §3–§4 | classification is decidable on a real capture; capture set complete per session | honest result recording is possible; not any campaign outcome |
| W13-DV05 → W13 closure | scenario-mapping review | cross-check §5 against W12's matrix design | mapping consistent, no duplication/contradiction; vacuous cases declared | integration coherence |
| W13-DV06 → W13 closure | limits and platform-independence review | step 5 checklist | no timing criteria, no guest/Stage-2 assertions, no board-name branch requirement; hardware gap present | boundary integrity |
| W13-DV07 → P3-V13 | campaign execution evidence | step 6 records | all four counts' campaigns complete with honest statuses and captures; P3-V13 satisfied only by `run-passed` rows at all applicable counts within declared depths | exactly the declared reference-platform criteria over the declared repetitions; never hardware, performance, or guest behavior |

Record each validation as **passed**, **failed**, **blocked**, or
**not run** with command, input, environment, timestamp, and reason.
The design phase can honestly complete DV01–DV06; DV07 waits for the
implementation and the automation entry.

## 4. Error, security, and observability model

- **Failure handling.** A bounded hang or a broken assertion ends the
  session with captures; the matrix records `run-failed` and any
  dependent-row status. No retry-until-green, no environment change
  mid-campaign.
- **Security boundary.** The matrix adds no guest-reachable surface and
  no privileged bypass; stimulus is W12's declared entry points; QEMU
  invocation stays inside the P0 automation contract.
- **Observability.** The matrix is the largest W11 consumer; its captures
  double as P3-V11 evidence. Where W11 seams are gapped, affected row
  checks degrade and cite the gap — recorded, never papered over.

## 5. Handoff checklist

Before handing W13 to a reviewer, provide:

- the exact changed/created file list;
- the matrix with per-row assertion sources and repetition/depth
  constants and rationale;
- campaign statuses (or explicit blocked/not-run statements with
  blockers) and evidence links, per row and count;
- confirmation that no runner, script, CI file, guest asset, mechanism
  change, timing criterion, board-name branch, or new `unsafe` was added;
- the standing hardware-gap statement and its P15 inheritance;
- open items for W14 (P4-entry review inputs), W15 (traceability), and
  P0-W20 (gating decision on the matrix).

## 6. Future record paths

Implementation record: `../p3-w13-qemu-smp-regression-record.md`
(created only when implementation begins). Verification record:
`../../verification/p3-w13-qemu-smp-regression-verification.md` (created
only when evidence exists). Neither exists today; this design claims no
result.
