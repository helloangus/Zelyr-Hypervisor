# P1-W11 Negative and Fault Validation — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** Reproducible evidence for the P1 failure paths required by
[P1-W11](../../plans/p1-w11-negative-fault-validation.md): unsupported
environments and intentional faults that must be bounded and diagnosable.  
**Owner/change context:** P1-W11 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P1-W11. It defines the
negative/fault scenario matrix, the trigger containment mechanism, the
expected diagnostic classes and terminal outcomes, and the scope/security
review checklist. It defines **what the negative evidence must establish and
how each scenario is reproduced**; it is not a validation result, and nothing
here claims that a scenario has been run. It deliberately does **not** design
guest fault isolation, recoverable fault policy, hardware fault coverage,
GIC/IRQ testing, fuzzing, or production recovery — all out of scope per the
plan.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

- [01-fault-scenario-matrix.md](01-fault-scenario-matrix.md) — the scenario
  matrix: setup, trigger category and containment, expected diagnostic class,
  terminal outcome, pass condition, and proof boundary for every fault class.
  Load first for any step.
- [02-scope-security-review.md](02-scope-security-review.md) — the P1-V19
  review checklist: untrusted-input checks, no unintended RWX, unsafe
  inventory, and the P2–P4 stage boundary.
- [03-implementation-and-review.md](03-implementation-and-review.md) — ordered
  workflow, validation matrix, observability model, and handoff checklist.
- [04-trigger-reconciliation.md](04-trigger-reconciliation.md) — integrated
  W09 trigger placements, NC5 unsafe-language correction, and NC6 blocker.

Before editing, follow the Coding Guidelines preflight: repository
[AGENTS.md](../../../../../AGENTS.md), [documentation
index](../../../../README.md), [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P1 task
book](../../task-book-v0.1.md), and the [P1-W11
plan](../../plans/p1-w11-negative-fault-validation.md). This design contains
no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P1 task book → P1-W11 plan → this design
→ Coding Guidelines. In particular:

- P1-V18 requires panic, synchronous fault, post-MMU translation/access fault,
  and unexpected-vector classes to produce required diagnostics; P1-V02
  (via W01) requires unsupported entry to be rejected explicitly; the plan
  adds the missing-required-capability class from W03's fail-fast policy.
- W05 (vector entry and unexpected classification), W07 (diagnostic field
  contract), W10 (bounded execution and evidence conventions), and W09 (phase
  attribution and failure routes) are **assumed contracts** known at plan
  level. Their accepted designs fix the concrete fields and tokens W11's
  expectations reference; a missing or contradicting prerequisite is recorded
  per [03-implementation-and-review.md](03-implementation-and-review.md) §1.
- The plan requires evidence collection **without changing normal P1 scope**:
  trigger code is validation-only, absent from the normal boot image
  ([Resolved decision 1](#resolved-design-decisions-and-their-authority)).

Classification: the scenario matrix, trigger containment mechanism, expected
diagnostic classes, terminal outcomes, and the scope/security review checklist
are **Required**. Additional fault classes beyond the plan's list, hardware
fault injection, and automated fuzz-style input generation are **Reserved**
with recorded triggers. Guest-caused fault isolation, recoverable VM faults,
production recovery policy, GIC/IRQ subsystem testing, and real-hardware fault
coverage are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Map each negative case to the W05/W07 diagnostic and W10 runner contracts (work seq 1) | [Scenario matrix](01-fault-scenario-matrix.md) §1–§2 | P1-V18 (W11-DV01) |
| Define setup, trigger category, expected bounded outcome per case (work seq 2) | [Scenario matrix](01-fault-scenario-matrix.md) §2 | P1-V18 (W11-DV01, DV03) |
| Integrate evidence collection without changing normal P1 scope (work seq 3) | [Scenario matrix](01-fault-scenario-matrix.md) §3; [workflow](03-implementation-and-review.md) step 3 | P1-V18 (W11-DV02) |
| Review input/range checks, no-RWX, unsafe-inventory expectations (work seq 4) | [Scope/security review](02-scope-security-review.md) | P1-V19 (W11-DV05) |
| Define reproducibility and objective acceptance per fault class (work seq 5) | [Scenario matrix](01-fault-scenario-matrix.md) §4; [workflow](03-implementation-and-review.md) step 5 | P1-V18 (W11-DV04) |
| Hand off the matrix and limitations to W12 (work seq 6) | Downstream handoff below; [workflow](03-implementation-and-review.md) handoff checklist | W11 closure review (W11-DV06) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p1-implementation-designs` at
`4e631ee`): no boot-path code, no exception vectors, no console, no fatal
reporting, no runner, and no P1 verification evidence exist — every executing
prerequisite of every scenario is still a plan. `docs/stages/p1/verification/`
contains only `.gitkeep`. No fault-injection mechanism exists anywhere in the
tracked tree. The ledger states what must exist for the package outcome to be
true and who owns it.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Unsupported environments are bounded and diagnosable (P1-V02 class) | No entry validation exists (W01 planned) | Scenario NC1 over the W01 rejection boundary via an EL2-less environment variation | The boundary can only be evidenced by entering it in a disallowed environment | Scenario by W11; boundary by W01 | W11-DV03 execution (future) |
| Intentional synchronous fault produces required diagnostics (P1-V18) | No vectors (W05), no report fields (W07) | Scenario NC3 with a validation-only trigger inside the post-vector window | An intentional fault needs a trigger the normal image never executes | Trigger by W11; vectors/diagnostics by W05/W07 | W11-DV03 |
| Panic path produces required diagnostics (P1-V18) | No panic route implementation (W02/P0 planned) | Scenario NC4 invoking the panic route with a recorded message | The panic path is only evidenced by taking it deliberately | Trigger by W11; route by W02/P0; fields by W07 | W11-DV03 |
| Post-MMU translation/access fault diagnosable (P1-V18) | No Stage-1 mapping (W08 planned) | Scenario NC5 accessing an address outside W08's mapped classes after `stage1` | Post-MMU diagnostics are only exercised by a mapped-state fault | Trigger by W11; mapping by W08 | W11-DV03 |
| Unexpected vector bounded (P1-V18) | No vector classification (W05 planned) | Scenario NC6 raising a category unexpected at the stable state | The unhandled path is only exercised by an unexpected event | Trigger by W11; classification by W05 | W11-DV03 |
| Missing required capability fails fast (W03 class) | No capability inventory (W03 planned) | Scenario NC2 under an environment lacking a W03-required capability | Fail-fast policy is only evidenced by an actual absence | Scenario by W11; required list by W03 | W11-DV03 |
| No security relaxation; boundaries intact (P1-V19) | Nothing to review yet | Static review checklist over the implemented P1 tree | The review must audit the tree that exists when W11 runs | W11 review procedure | W11-DV05 |
| Triggers absent from the normal image (plan work seq 3) | No image exists | Selection-gated trigger mechanism with a default build that contains none | "Without changing normal P1 scope" must be verifiable, not asserted | W11 (mechanism); verified with W10 regression | W11-DV02 |

No row requires designing guest isolation, recovery, or hardware injection;
no decision blocker is outstanding for the design itself.

## Resolved design decisions and their authority

### NC2 validation correction (2026-09-24)

NC2's preferred environment-only case remains unavailable on reference QEMU
8.2.2: model inspection exposes no granule-selection property, and the
[versioned model definitions](https://github.com/qemu/qemu/blob/v8.2.2/target/arm/tcg/cpu64.c)
report supported TGran4 for the AArch64 models, including `max`. The only
other W03 Required fact, EL2 execution, is rejected earlier by W01 when absent.
NC2 therefore permits the narrowly scoped validation-image variant in the
[scenario matrix](01-fault-scenario-matrix.md#nc2--missing-required-capability).
This supersedes the environment-only restriction for NC2 in decision 2 below
and the historical ledger. NC1 remains environment-only. The amendment
enables the package's required missing-capability policy check without
changing W03's production required set or claiming hardware absence.

The variant changes one sampled register field before the unchanged W03
decoder, classifier, required check and terminal diagnostic execute at EL2.
It proves fail-fast handling of an absent required fact (P1-V06) and its real
failure route. It does not prove that a physical or emulated CPU lacking that
capability was used. Retain the environment-only case as unavailable with
this reason; record injected execution separately. Default-image containment
review remains required.

1. **Validation-only triggers behind an explicit selection.** Every
   intentional trigger lives behind a distinct validation selection (a
   scenario identifier fixed at build time). The default build contains zero
   scenario triggers; the W10 regression passing on the default image is the
   standing guard. Rationale: the plan forbids changing normal P1 scope, and
   this makes compliance mechanically checkable rather than rhetorical.
   Authority: P1-W11 plan work seq 3; Coding Guidelines on minimal changes.
2. **Environment variations for environment-class scenarios.** NC1 and the
   preferred environment-only variant of NC2
   vary the *execution environment* (machine/CPU properties available from
   the reference platform), not the image. The technique category is fixed
   here; the exact property spelling follows the W01/P0-W09 contracts at
   implementation time. Rationale: these classes are about the environment,
   so those variants have no image trigger. The separately recorded NC2
   validation-image variant follows the correction above.
3. **Expected outcome per scenario is a diagnostic class plus a terminal
   outcome — never a recovery.** P1 has no recovery policy (plan out of
   scope); every scenario must end in the bounded terminal behavior its
   phase's W09 failure route prescribes. Authority: W09 routing matrix;
   P1-W11 plan scope.
4. **Objective acceptance is marker-class based.** A scenario passes when the
   capture contains the expected diagnostic class (per W07/W05 fields), the
   phase attribution is correct, the terminal behavior is the expected one,
   and nothing after the terminal marker suggests continuation. Rationale:
   consistent with W10's content-class verdicts; prose matching is excluded.
5. **Determinism requirement.** Every scenario is deterministic: fixed
   trigger, fixed phase, no timing or iteration dependence; each scenario is
   run twice with identical outcome class as reproducibility evidence.
   Rationale: P1-V18 says "reproducible"; a single observation would not.
6. **The unowned pre-vector window is not a scenario.** Phases before
   `exceptions.complete` have no owned vector table (W09 limitation); faults
   there are outside P1's owned failure surface, so W11 defines no scenario
   targeting that window and records the exclusion as a limitation for W12.
   Authority: W09 state machine §4; P1-W11 out-of-scope list (no hardware
   fault coverage).

## Work breakdown and loading order

1. Load [01-fault-scenario-matrix.md](01-fault-scenario-matrix.md) for the
   scenario definitions and the trigger containment contract.
2. Execute the workflow in
   [03-implementation-and-review.md](03-implementation-and-review.md):
   confirm prerequisite contracts, implement the selection-gated triggers,
   run each scenario twice through the W10 conventions, then perform the
   [02-scope-security-review.md](02-scope-security-review.md) checklist.
3. Record implementation decisions in
   `../p1-w11-negative-fault-validation-record.md` when implementation
   begins; commands, captures, per-scenario outcomes, and the review results
   in `../../verification/p1-w11-negative-fault-validation-verification.md`
   when evidence exists. Neither file may exist yet; neither this design nor
   a record may claim W11 complete.

## Explicitly excluded interfaces

No guest fault model, recoverable-fault classification, retry/recovery API,
fault-tolerance framework, or fuzzing harness is designed or authorized. No
GIC, timer, IRQ-subsystem, or hardware fault-injection mechanism is touched.
No production observability pipeline or persistent crash storage exists in
this package. W11 adds no runtime service: its only boot-path footprint is
the selection-gated trigger calls of
[01-fault-scenario-matrix.md](01-fault-scenario-matrix.md) §3, each returning
nothing and reachable only under the validation selection. No public ABI,
wire format, or persistent layout is introduced.

## Downstream handoff

- **[P1-W12](../p1-w12-p1-documentation-handoff/README.md)** receives the
  negative-evidence requirements (which classes must have evidence and where)
  for the evidence map, and the recorded limitations (unowned pre-vector
  window; environment-only coverage) for the known-limitations document.
- **P2** (named consumers: P2-W08 host robustness regression and P2-W09 QEMU
  integration regression per the P2 plan index) receives only the diagnostic
  and boundary contract — that faults in P1 are terminal, phase-attributed,
  and marker-observable — not a general fault framework, which remains
  undesigned.
- The P1 completion review consumes the per-class evidence through W12's map
  (P1-V18, P1-V19); W11 itself claims nothing.
