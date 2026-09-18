# P1-W10 QEMU Boot Regression — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The automated, repeatable QEMU `virt` boot verdict, its evidence
retention, and the 100-cycle clean-boot acceptance required by
[P1-W10](../../plans/p1-w10-qemu-boot-regression.md).  
**Owner/change context:** P1-W10 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P1-W10. It defines the automation
entry contract, the verdict rules, the outcome classification, the evidence
layout, and the 100-cycle procedure. It defines **what the regression must
establish and how evidence is retained**; it is not a validation result, and
nothing here claims that a run has happened. It deliberately does **not**
build the hypervisor image (W01–W09), implement the QEMU runner itself
(P0-W09), configure CI providers (P0-W20 boundary), benchmark performance, or
prove anything about real hardware.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

- [01-automation-contract.md](01-automation-contract.md) — the regression
  entry's behavioral contract: invocation, runner integration, marker
  protocol, timeout/exit semantics, panic detection, and evidence layout.
  Load first for any step.
- [02-scenarios-and-verdict.md](02-scenarios-and-verdict.md) — the scenario
  matrix with pass conditions, proof boundaries, and the 100-cycle procedure
  and evidence set.
- [03-implementation-and-review.md](03-implementation-and-review.md) — ordered
  workflow, validation matrix, observability model, and handoff checklist.

Before editing, follow the Coding Guidelines preflight: repository
[AGENTS.md](../../../../../AGENTS.md), [documentation
index](../../../../README.md), [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P1 task
book](../../task-book-v0.1.md), and the [P1-W10
plan](../../plans/p1-w10-qemu-boot-regression.md). This design contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P1 task book → P1-W10 plan → this design
→ Coding Guidelines. In particular:

- P1-V16 requires the test to **independently determine pass/fail** with
  bounded markers, timeout/exit behavior, and preserved failure evidence;
  P1-V17 requires 100 consecutive clean boots reaching the same stable marker
  with no random startup failure.
- The plan requires reusing the W01/W09 canonical path and integrating
  evidence preservation with the P0 QEMU runner baseline. P0-W09, W01, and W09
  deliverables are **assumed contracts** known at plan level (P0-W09: a single
  documented QEMU runner entry with serial capture, timeout, exit status, and
  evidence collection; W01: the canonical invocation; W09: the stable state
  and marker vocabulary). Missing or contradicting prerequisites are upstream
  defects, recorded per
  [03-implementation-and-review.md](03-implementation-and-review.md) §1.
- The task book assigns CI wiring to P0-W20; this design's output is a locally
  and CI-invocable entry whose contract CI can consume, not a workflow file.

Classification: the entry contract, verdict rules, outcome classification,
evidence layout, and 100-cycle procedure are **Required**. Parameterized
variations beyond the canonical boot (RAM/CPU sizes, SMP, future-stage
machines) are **Reserved** with recorded triggers. Guest boot, performance
benchmarking, multi-platform matrices, CI provider configuration, real-hardware
evidence, and any change to the P0 runner's own contract are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Reuse W01/W09 canonical path; identify objective pass/fail markers (work seq 1) | [Automation contract](01-automation-contract.md) §2–§3 | P1-V16 (W10-DV01, DV02) |
| Define timeout, abnormal-exit, missing-marker, panic outcomes (work seq 2) | [Automation contract](01-automation-contract.md) §4; [scenarios](02-scenarios-and-verdict.md) R3–R5 | P1-V16 (W10-DV03) |
| Integrate evidence preservation with the P0 QEMU runner baseline (work seq 3) | [Automation contract](01-automation-contract.md) §6 | P1-V16 (W10-DV04) |
| Review repeatability; avoid output-order-only success criteria (work seq 4) | [Automation contract](01-automation-contract.md) §3.3; [scenarios](02-scenarios-and-verdict.md) R6 | P1-V16 (W10-DV06) |
| Define the 100-cycle evidence set and acceptance review (work seq 5) | [Scenarios](02-scenarios-and-verdict.md) §3–§4 | P1-V17 (W10-DV07, DV08) |
| Test independently determines verdict (P1-V16) | [Automation contract](01-automation-contract.md) §5 | P1-V16 (W10-DV01, DV05) |
| Hand off to fault validation and stage closure (work seq 6) | Downstream handoff below; [workflow](03-implementation-and-review.md) handoff checklist | W10 closure review (W10-DV08) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p1-implementation-designs` at
`4e631ee`): `git ls-files` shows no QEMU runner, script, or automation entry
(`scripts/` contains only `.gitkeep`), no CI workflow
(`.github/workflows/.gitkeep` only), no built or buildable hypervisor image
(no Cargo workspace, no Rust sources, no target definition), and no P1
verification evidence (`docs/stages/p1/verification/.gitkeep` only). The
P0-W09 plan defines the runner entry as a P0 deliverable; no P0 implementation
of it is observable. P1-W10's execution prerequisites are therefore planned,
not present — the ledger below states what must exist for the package outcome
to be true and who owns it.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Automated, repeatable verdict exists (P1-V16) | No runner, no driver, no image | Regression entry implementing this design's contract, over the P0-W09 runner and a W01-canonical image | A verdict requires an executable that boots, observes, decides, and exits without human judgment | W10 (driver + rules); runner per P0-W09; image per P0 build baseline + W01 | W10-DV01/DV05 execution evidence (future) |
| Objective pass/fail markers | None exist; W09 defines emission, no token selected | Stable-marker token and fatal/panic marker classes fixed as matching rules | "Objective" means the rules are fixed before the run and independent of the operator | W10 owns the token and rules; emission per W09/W06/W07 contracts | W10-DV02 rule review |
| Bounded timeout/exit semantics | Nothing bounded exists | Outcome classification table incl. timeout and abnormal exits, with a hard per-boot bound | Unbounded or ambiguous outcomes make the verdict non-repeatable | W10 | W10-DV03 |
| Preserved failure evidence | No evidence layout exists | Per-cycle evidence layout and retention policy integrated with the P0-W09 collection contract | A verdict without retained evidence cannot be reviewed or replayed | W10 layout; collection per P0-W09 | W10-DV04 |
| 100 consecutive clean boots, same stable marker, no random failure (P1-V17) | No repetition procedure exists | Cycle procedure, stop-on-failure rule, evidence set, acceptance review | A repetition claim needs a defined, non-cherry-picked evidence set | W10 | W10-DV07/DV08 (execution later) |
| Repeatability independent of output order (work seq 4) | N/A yet | Matching rules that never depend on line order, timing, or full-text equality | Order-only criteria break on benign output changes and hide real failures | W10 | W10-DV06 |

No row requires selecting CI tooling, guest images, or performance targets; no
decision blocker is outstanding for the design itself. The prerequisite rows
are failure boundaries, not work W10 may absorb.

## Resolved design decisions and their authority

1. **Verdict inputs are content classes, bounded runtime, and exit status —
   never output order, timing, or full-text equality.** Rationale: the plan
   forbids order-only success criteria and requires an independent verdict;
   content classes survive benign output changes and are checkable. Authority:
   P1-W10 plan work seq 4, P1-V16.
2. **Stable-marker token owned by W10.** The stable state's emission point is
   W09's ([../p1-w09-initialization-sequencing/README.md](../p1-w09-initialization-sequencing/README.md));
   the exact token text is selected by this design at implementation time
   (stage-local design freedom, recorded in the implementation record) because
   it is a matching rule of the verdict, and must be distinctive enough to
   never collide with other marker classes or QEMU's own output.
3. **Driver form follows the P0 runner; the command contract is normative.**
   The canonical regression entry is named `p1-boot-regression` (stage-local
   design freedom; rationale: the scaffold reserves `scripts/` for tooling and
   a single name prevents runner drift, the problem P0-W09 exists to prevent).
   If the P0 artifact-naming baseline (P0-W17) or the P0-W09 runner design
   fixes a conflicting name or form, that authority wins and the deviation is
   recorded; the behavioral contract of
   [01-automation-contract.md](01-automation-contract.md) is unchanged by such
   a rename.
4. **Evidence split: committed review documents versus retained raw captures.**
   `docs/stages/p1/verification/` receives documents with verdicts, counts,
   configurations, and representative excerpts; raw per-cycle serial captures
   are artifacts retained per the P0-W09 collection contract and referenced
   from the verification document by path and identity. Rationale: task-book
   separation of evidence documents from machine artifacts; keeps the
   repository reviewable. The verification document must state where raw
   evidence lives; "not retained" is a recorded limitation, never silence.
5. **100-cycle acceptance is strict: 100 of 100 must pass; the first failure
   stops the run.** No re-run-until-pass, no exclusion of failed cycles, no
   partial evidence sets. Rationale: P1-V17's purpose is detecting random
   startup failure; selective evidence would defeat it. A stopped run is
   valid failure evidence.
6. **The timeout bound exists by contract; its value is selected at
   implementation.** The default must be recorded with the measured boot time
   of the canonical image and headroom rationale. Rationale: the *existence*
   and *boundedness* of the bound is the contract; a hardcoded value here
   would pre-empt measured reality.
7. **QEMU verdicts are reference-platform evidence only.** They do not prove
   hardware behavior, cache/TLB semantics, or board bring-up correctness
   (ADR-003's division of roles; plan handoff).

## Work breakdown and loading order

1. Load [01-automation-contract.md](01-automation-contract.md) for the entry
   contract, marker protocol, and outcome classification.
2. Load [02-scenarios-and-verdict.md](02-scenarios-and-verdict.md) and execute
   the workflow in [03-implementation-and-review.md](03-implementation-and-review.md):
   confirm prerequisites, implement the driver, run the scenario matrix, then
   the 100-cycle procedure.
3. Record implementation decisions in
   `../p1-w10-qemu-boot-regression-record.md` when implementation begins;
   commands, environments, per-cycle results, and the 100-cycle evidence set
   in `../../verification/p1-w10-qemu-boot-regression-verification.md` when
   evidence exists. Neither file may exist yet; neither this design nor a
   record may claim W10 complete.

## Explicitly excluded interfaces

No hypervisor runtime API, boot-path code change, guest image, device model,
or virtual hardware configuration is designed here beyond invoking the W01
canonical boot. No CI provider workflow, secret, or hosted-runner
configuration is authorized (P0-W20 boundary). No performance metric,
benchmark harness, or statistical criterion exists in this design. The P0-W09
runner's own contract is consumed, not modified; if the runner lacks a
capability this design requires, that is an upstream defect to record, not a
reason to fork a second runner. No SMP, multi-platform, or future-stage
scenario matrix is designed; those are Reserved for their owning stages.

## Downstream handoff

- **[P1-W11](../p1-w11-negative-fault-validation/README.md)** receives the
  bounded execution and evidence conventions: per-run invocation, outcome
  classification, capture retention, and verdict recording, reused for every
  fault scenario.
- **[P1-W12](../p1-w12-p1-documentation-handoff/README.md)** receives the
  reference-QEMU-environment facts (invocation, runner entry, marker rules,
  evidence locations) for the reference environment contract and the evidence
  map.
- **P2** (named consumer: P2-W09 QEMU integration regression per the P2 plan
  index) receives the precedent that integration regressions run through the
  single P0 runner entry with content-class verdicts and retained evidence;
  P2 designs its own scenarios and does not inherit this package's tokens.
- The 100-cycle result, once run, is stage-gate evidence for the P1 completion
  review (P1-V17); its location is recorded by W12's evidence map.
