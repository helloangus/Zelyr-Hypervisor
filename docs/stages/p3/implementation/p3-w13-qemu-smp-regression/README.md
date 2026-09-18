# P3-W13 QEMU SMP Regression Matrix — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The repeatable QEMU `virt` Host SMP regression matrix for 1, 2,
4, and 8 CPUs with repeated cold boots required by
[P3-W13](../../plans/p3-w13-qemu-smp-regression.md).  
**Owner/change context:** P3-W13 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P3-W13. It defines the matrix
rows, repetition policy, per-row assertions, diagnostic-capture
requirements, and result classification that make P3-V13 assessable, and
it fixes the boundary every row inherits: QEMU `virt` is the ADR-003
reference environment; passing here proves reference-platform
repeatability, never Orange Pi or other hardware behavior (P15 owns that),
never performance, and never guest or Stage-2 semantics. It deliberately
does **not** implement the runner or CI (the P0-W09 automation entry and
P0-W20 CI own those; this design consumes their contracts), does not
execute the matrix (execution happens when the P3 implementation exists,
and results live only in verification records), and does not restate
mechanism designs.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

| Supporting file | Load it for |
|---|---|
| [01-scope-and-foundations.md](01-scope-and-foundations.md) | goal-to-baseline ledger, prerequisite failure boundaries, resolved decisions |
| [02-regression-matrix.md](02-regression-matrix.md) | the matrix rows (R-series), per-row assertions, repetition policy, scenario mapping |
| [03-result-classification-and-evidence.md](03-result-classification-and-evidence.md) | result classes, diagnostic capture, environment recording, evidence destinations, hardware gap |
| [04-workflow-validation-and-handoff.md](04-workflow-validation-and-handoff.md) | ordered workflow, validation matrix, error model, handoff checklist |

Before editing it must also follow the Coding Guidelines preflight. This
document defines a regression contract; it contains no results, and no
matrix row has run.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P3 task book → P3-W13 plan → this
design → Coding Guidelines. Binding constraints:

- The task book requires QEMU 1/2/4/8-CPU regression and repeated cold
  boots with declared boot/online/event/stress/allocator criteria
  (P3-V13), and its exit criteria require that unsupported hardware
  behavior is not inferred from QEMU results.
- ADR-003 names QEMU `virt` the reference/CI platform and Orange Pi 3B
  the later hardware target; ADR-049 layers validation; the guides require
  every validation to state what it does not prove. ADR-044/ADR-052 keep
  platform selection out of Core: the matrix is *evidence tooling*, and
  nothing it consumes may force a board-name branch into hypervisor code.
- The P0 QEMU automation entry
  ([P0-W09](../../../p0/plans/p0-w09-qemu-automation-entry-baseline.md))
  and CI baseline
  ([P0-W20](../../../p0/plans/p0-w20-ci-baseline.md)) are assumed
  contracts (boundaries in [01](01-scope-and-foundations.md) §1.2): the
  matrix defines rows and classifications that their machinery executes;
  it defines no script, workflow, or runner.
- Stress and failure content is consumed, not redefined: scenarios
  S1–S7 are owned by [P3-W12](../p3-w12-smp-stress-failure-tests/README.md);
  this design fixes at which CPU counts and repetition depths each runs.
- Boot/bring-up assertion content comes from
  [P3-W02](../p3-w02-secondary-cpu-bring-up/README.md) (outcomes,
  induced-failure input) and
  [P3-W05](../p3-w05-smp-boot-synchronization/README.md) (phase
  sequence, `SmpReadyState`, degraded accounting); observability surfaces
  from [P3-W11](../p3-w11-smp-observability/README.md).

Classification:

- **Required** for W13 closure: the R-series matrix rows with per-row
  assertions and repetition policy for 1/2/4/8 CPUs; the cold-boot
  repetition policy; the result-classification vocabulary and per-row
  capture requirements; the scenario mapping to W12; the hardware-gap
  statement; and the review evidence that the contract is coherent.
- **Reserved** with recorded triggers: CPU counts beyond the declared
  1/2/4/8 (trigger: an approved topology change revisiting W01's
  inventory bound); CI integration of matrix rows as required checks
  (trigger: P0-W20's wiring consuming this contract); additional machine
  types or board emulation rows (trigger: an approved platform addition
  per ADR-045 support tiers); performance measurement rows (trigger: a
  performance-baseline design — P7-W13/P8-W17 territory, not P3).
- **Out of Scope:** runner/CI implementation; hardware validation (P15);
  QEMU behavior as an architectural specification (the ADR rules keep
  Core semantics platform-independent); performance certification; guest
  and Stage-2 regression content (P4+); any pass/fail claim in this
  design; editing W12's scenarios to fit matrix convenience.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Matrix rows for 1/2/4/8 CPUs: boot, online-set, stress, cross-CPU event, allocator-stress expectations | [matrix](02-regression-matrix.md) §2–§3 | P3-V13 (W13-DV01) |
| Repeated cold boots to expose timing/rendezvous/allocator races | [repetition policy](02-regression-matrix.md) §4 | P3-V13 (W13-DV01, DV02) |
| P0 QEMU/CI contract retained | [foundations](01-scope-and-foundations.md) §1.2; consumption rules in [evidence](03-result-classification-and-evidence.md) §2 | W13 closure review (W13-DV03) |
| Diagnostic capture and result classification | [classification](03-result-classification-and-evidence.md) §3 | P3-V13 (W13-DV04) |
| Integrate stress/failure cases with the matrix | [scenario mapping](02-regression-matrix.md) §5 | W13 closure review (W13-DV05) |
| Review QEMU limits and platform independence | [workflow](04-workflow-validation-and-handoff.md) step 5 | W13 closure review (W13-DV06) |
| Hand off regression contract, evidence location, hardware gap | [handoff](04-workflow-validation-and-handoff.md) §5, [gap](03-result-classification-and-evidence.md) §5 | W13 closure review (W13-DV07); execution deferred until implementation exists |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p3-implementation-designs`
at `4e631ee`): P0 documentation scaffold only — no workspace, no sources,
no QEMU runner, no CI, and no P3 implementation or verification evidence.
P0-W09/P0-W20 exist as plans only; the P0 QEMU smoke placeholder named in
the P0 task book is unimplemented. Sibling P3 designs W01–W05 are present
as proposed designs; W06–W10 are parallel. Each ledger row below states
the missing foundation the plan outcome necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| A repeatable 1/2/4/8 regression matrix exists | No matrix, no runner, no boots | The R-series row contract: per-count rows with assertions derived from the owning designs | Without fixed rows and assertions, "regression" degenerates into ad-hoc boot watching | W13 (this design); assertions from W02/W03/W04/W05/W11/W12 | W13-DV01 |
| Repeated cold boots expose races | No repetition policy exists anywhere | Cold-boot repetition policy per row with declared depth and honesty about what repetition can and cannot catch | Race exposure is statistical; the declared depth bounds the claim | W13 (policy, recorded) | W13-DV01/DV02 |
| Stress and allocator expectations per count | Scenarios exist only as the W12 contract | The scenario mapping: which S-scenarios run at which counts and depths in matrix rows | W12 owns scenarios; W13 owns deployment across counts — without the mapping the two contracts would drift | W13; scenarios W12 | W13-DV05 |
| Diagnostic capture per run | No capture convention exists | The per-run capture set (boot log, SMP-ready dump, counter snapshots, phase/rendezvous events, environment block) | P3-V13's criteria are checkable only from captured evidence, not from a passing exit code alone | W13; surfaces W11/W03/W05 | W13-DV04 |
| Result classification | None | run-passed / run-failed / blocked / not-run with divergence analysis and destination rules | The task book requires honest result recording ("recording each result honestly", plan step 5) | W13 | W13-DV04 |
| P0 automation contract consumption | P0-W09/P0-W20 are plans; no runner exists | Declared consumption rules: matrix rows expressed against the automation entry's inputs/outputs once it exists; blocked status until then | A matrix no machinery can execute is not a regression; the dependency must be explicit, not assumed | P0-W09/W20 (machinery); W13 (consumption rules) | W13-DV03 |
| Hardware-validation gap recorded | Nothing records it in matrix terms | The per-row does-not-prove statement and the P15 inheritance note | Exit criterion 4 of the task book requires it | W13 (statement); P15 (future evidence) | W13-DV07 |

No ledger row requires inventing a runner, a CI system, or new mechanisms;
the assumed-contract boundaries are in [01](01-scope-and-foundations.md)
§1.2.

## Resolved design decisions and their authority

1. **Row = CPU count × fixed assertion set; the count axis is complete.**
   The matrix is exactly the four declared counts, each getting the same
   R-series rows (boot/readiness, online-set, isolation, observability,
   stress, cross-CPU event, allocator stress, failure path). No other
   count is authorized (W01's inventory bound); a fifth count is a
   Reserved trigger, not a row. Rationale: the task book and plan declare
   1/2/4/8; completeness of the axis is what makes the matrix a
   regression rather than a sample.
2. **Assertions come from the owning designs, verbatim.** Each row cites
   the contracts it asserts (W05 phase sequence and `SmpReadyState`, W03
   online-set rules, W04 isolation diagnostics, W11 dump/attribution,
   W02 outcome accounting); W13 never re-derives an assertion with
   different semantics. Rationale: one fact, one owner — a matrix that
   redefines readiness or accounting would create a second authority.
3. **Cold-boot repetition is the race-exposure instrument, with a bounded
   honest claim.** Every count boots R13 times per campaign; depth is a
   recorded constant with rationale and revisit trigger
   ([02](02-regression-matrix.md) §4). Repetition increases exposure to
   timing/rendezvous/allocator races; it cannot prove their absence —
   stated per row. Rationale: the plan names this purpose; overclaiming
   repetition would violate the prove/does-not-prove discipline.
4. **QEMU variance is recorded, never asserted away.** TCG/KVM mode,
   QEMU version, host environment, and smp count are mandatory
   environment-block fields; results compare within one declared
   environment class; timing values never appear in pass conditions
   (consistent with W11's informative-only timing rule and W12's
   determinism rules). Rationale: P3-V13 compares across repeated boots;
   without environment discipline, comparisons are meaningless.
5. **The failure-path row reuses W12's S5/S6 injections.** The matrix
   includes one failure row per count (absent-CPU start, invalid/offline
   targets) so the regression proves failure paths stay diagnosable at
   every supported count, not only at the default count. Rationale:
   P3-V13's declared criteria include failure behavior ("expected CPUs
   either become online or yield precise failure diagnostics" — task book
   exit criterion 1).
6. **Blocked is a first-class result.** Until the P0-W09 automation entry
   exists, matrix execution is blocked, not skipped; a row blocked by a
   missing mechanism cites the blocking contract. Rationale: the
   plan's "run or collect matrix evidence when implementation exists,
   recording each result honestly".
7. **Platform independence is a review check, not a hope.** The matrix
   consumes capability-declared boot inputs only; its review step
   verifies that no row requires a board-name branch or QEMU-only
   constant in hypervisor code, and that QEMU-specific expectations (e.g.
   deterministic absent-CPU rejection) are recorded as QEMU observations,
   not Core semantics. Rationale: ADR-044/ADR-052 and the plan's
   "Review QEMU limits and platform-independence constraints".

## Work breakdown and loading order

1. Read [01-scope-and-foundations.md](01-scope-and-foundations.md) for
   the ledger and the P0-W09/W20 assumed-contract boundaries.
2. Read [02-regression-matrix.md](02-regression-matrix.md) for the
   R-series rows, per-row assertions, repetition policy, and the W12
   scenario mapping.
3. Read [03-result-classification-and-evidence.md](03-result-classification-and-evidence.md)
   for result classes, capture sets, environment blocks, destinations,
   and the hardware-gap statement.
4. Execute per [04-workflow-validation-and-handoff.md](04-workflow-validation-and-handoff.md):
   contract reviews first, execution only when the P3 implementation and
   the automation entry exist, results only in verification records.
5. Record implementation decisions in
   `../p3-w13-qemu-smp-regression-record.md` and evidence in
   `../../verification/p3-w13-qemu-smp-regression-verification.md` only
   when produced. Until then, P3-V13 is *planned*; this design claims no
   result and no coverage.

## Explicitly excluded interfaces

No runner binary, CI workflow, QEMU invocation script, or expect-style
harness is designed here; matrix rows are expressed as consumption rules
against the P0-W09 automation contract once it exists. No hypervisor
mechanism, assertion hook, or diagnostic output beyond the owning designs'
published surfaces is added. No guest image, vCPU workload, or Stage-2
row is defined (P4's regression is P4-W08's). No performance metric or
timing threshold exists. A row that cannot be expressed without changing
a mechanism or adding a QEMU-specific branch to Core is a design conflict
to stop at review.

## Downstream handoff

- **W14/P4** receives the matrix contract, the result classification, and
  the explicit hardware gap as P4-entry review inputs (P4-W01 reconciles
  P3 evidence; P4-W08 builds its own regression on the P0 automation
  contract, not on this matrix's rows).
- **W15** receives the matrix-to-P3-V13 traceability and evidence
  locations for stage documentation.
- **P15** inherits the hardware-validation responsibility with the
  recorded statement of everything the QEMU matrix does not establish.
- **P0-W20** receives the matrix as the candidate content for CI gating
  (its wiring decision), per the Reserved trigger.
