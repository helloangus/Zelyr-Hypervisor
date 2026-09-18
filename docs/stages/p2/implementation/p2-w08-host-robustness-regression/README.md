# P2-W08 Host Robustness and Negative Regression — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The reproducible host-side regression design for P2's
untrusted-input, memory-map, allocator, and determinism safety properties
(P2-J01–J05) — scenario matrices, pass conditions, automation contract, and
evidence locations — required by
[P2-W08](../../plans/p2-w08-host-robustness-regression.md).  
**Owner/change context:** P2-W08 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P2-W08. It is a
validation-design package, not a code-bearing mechanism package: its
deliverables are (a) the scenario matrices that turn W01–W05's typed
contracts and W07's fixture format into concrete, individually assertable
cases, (b) the automation contract that makes those cases one reproducible
suite (scenario IDs, seeding, pass/fail aggregation), and (c) the evidence
destinations and record format for P2-V10. It deliberately does **not**
execute anything (no results exist until the implementing agent runs the
suite), does not change any W01–W05/W07 contract or production module, does
not define a fuzzing architecture (property-based generation with fixed
seeds is the bounded scope), does not create QEMU automation (W09), does
not benchmark performance, and does not fabricate acceptance evidence —
"planning this package provides neither runtime output nor completion
evidence" (plan acceptance wording).

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step. Before
editing it must also follow the Coding Guidelines preflight, including the
repository `AGENTS.md`, documentation index,
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P2 task book](../../task-book-v0.1.md), and the
[P2-W08 plan](../../plans/p2-w08-host-robustness-regression.md). This
document is a proposed design; it contains no implementation or validation
claim.

| Supporting file | Load it for |
|---|---|
| [01-scope-and-foundations.md](01-scope-and-foundations.md) | Requirement enumeration, scope classification, assumed contracts (W01–W05/W07 outcomes, P0 test baseline), harness placement, and the host-only proof boundary. |
| [02-matrices-input-and-map.md](02-matrices-input-and-map.md) | Scenario matrices W08-S1xx (malformed DTB) and W08-S2xx (map conflicts/overflow) with per-scenario pass conditions and proof boundaries. |
| [03-matrices-allocator-stress-determinism.md](03-matrices-allocator-stress-determinism.md) | Scenario matrices W08-S3xx (allocator lifecycle/protected-page safety), W08-S4xx (stress), W08-S5xx (determinism). |
| [04-automation-and-evidence.md](04-automation-and-evidence.md) | The automation contract: suite organization, scenario ID scheme, seeding, aggregation, evidence record format, and rerun rules. |
| [05-validation-and-handoff.md](05-validation-and-handoff.md) | The package's own validation matrix (reviewing the regression design), evidence mapping to P2-V10, and the handoff checklist. |

## Authority, constraints, and scope classification

Governing order: [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md)
→ [P2 task book](../../task-book-v0.1.md) →
[P2-W08 plan](../../plans/p2-w08-host-robustness-regression.md) → this
design → Coding Guidelines. Binding constraints:

- The task book's hard gate — **every protected physical range is excluded
  from allocation for every valid allocation/free sequence** — is exercised
  here as the soak/property scenarios (S3xx) at host strength; W09 adds
  integration evidence; neither substitutes for the other.
- ADR-049 layers validation; the plan bounds W08 to host-side logic
  properties. Every scenario row therefore carries an explicit
  proves/does-not-prove boundary, and host results never substitute for
  QEMU (W09) or hardware semantics.
- The plan's out-of-scope list forbids "implementation test commands" as
  package content: the automation contract fixes organization, IDs,
  seeding, and evidence, while exact command spelling follows the P0-W08
  baseline's entry points once they exist (assumed contract; recorded
  boundary).
- W08 owns no production code. Test-support code it adds must obey the
  Coding Guidelines like any other code and must not alter tested modules.

Classification. **Required:** the five scenario groups with concrete
inputs, expected observables, and pass conditions; fixed-seed pseudo-random
generation for stress/soak scenarios; the aggregation and evidence format;
reproducibility of the whole suite. **Reserved** (recorded triggers, no P2
implementation): coverage-guided fuzzing, formal/model-checking backends,
continuous-fuzz CI infrastructure, cross-platform harness porting,
performance/regression timing of any kind. **Out of Scope:** QEMU
integration execution (W09), real-hardware validation (P15+), contract
changes to tested packages, guest/device scenarios (no guest exists in
P2), and any completion claim.

## Requirement-to-design mapping

The tracked sources define P2-J01–J05 at group granularity only; the rows
below are this design's reviewable enumeration from the plan's scope
wording.

| Requirement group | Concrete requirement (this design) | Design location | Acceptance |
|---|---|---|---|
| P2-J01 | Malformed-DTB scenario set demonstrating bounded rejection with W01's distinct diagnostic classes: structural, encoding, truncation, and overflow input | [02 §2](02-matrices-input-and-map.md) | P2-V10 (S1xx rows) |
| P2-J02 | Map conflict and range-overflow scenarios: RAM/protected conflicts, clipping, overflow, capacity, and seal violations with explicit outcomes | [02 §3](02-matrices-input-and-map.md) | P2-V10 (S2xx rows) |
| P2-J03 | Allocator exhaustion, reuse after release, invalid-free classes, and the protected-page-never-returned property including under invalid sequences | [03 §2](03-matrices-allocator-stress-determinism.md) | P2-V10 (S3xx rows) |
| P2-J04 | Allocation stress (page + heap, combined) with accounting/invariant checks at checkpoints under fixed-seed pseudo-random sequences | [03 §3](03-matrices-allocator-stress-determinism.md) | P2-V10 (S4xx rows) |
| P2-J05 | Deterministic repeated discovery: identical inputs produce identical facts, map, and inspection render | [03 §4](03-matrices-allocator-stress-determinism.md) | P2-V10 (S5xx rows) |
| Reproducibility (plan step 6) | The suite re-runs to the same outcomes from the same seeds and fixtures; evidence format records run/not-run per scenario | [04](04-automation-and-evidence.md) | P2-V10 (W08-DV04) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p2-implementation-designs`):
documentation scaffold only — no workspace, no Rust sources, no test
infrastructure, and no W01–W07 implementation or fixtures. W08 is designed
against the siblings' published contracts (diagnostics, error enums,
invariants, fixture format) as assumed prerequisites; the P0-W08 host-test
baseline is likewise an assumed contract.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Reproducible evidence for every P2-J group (P2-V10) | No suite exists | Scenario matrices + automation contract + evidence format | Without per-scenario pass conditions, "reproducible" is unfalsifiable | W08 (this design); harness implemented against W01–W07 code | W08 evidence record rows when run |
| Assertions under test (plan step 1) | Contracts exist only as designs | Each matrix row names the exact typed outcome from the owning design's contract file | Assertions must reference published behavior, not reimplemented logic | W01–W05/W07 own the contracts; W08 references them | DV review of assertion fidelity |
| Malformed-input cases incl. structural/encoding/truncation/overflow (plan step 2) | Nothing | S1xx matrix over W07 fixture format | W01's taxonomy already names the classes; scenarios instantiate each | W08 | S1xx runs |
| Map-conflict and allocator-lifecycle cases (plan step 3) | Nothing | S2xx/S3xx matrices over W03/W04 typed errors | Conflict policy and free-validation are contract tables; scenarios instantiate them | W08 | S2xx/S3xx runs |
| Protected-page safety (hard gate, plan step 3) | Nothing | S3xx soak with per-op audit | The gate is a universal property; it needs a randomized soak, not one example | W08 (host strength); W09 (integration) | S3xx soak run |
| Deterministic repeated-input evidence (plan step 4) | Nothing | S5xx double-run comparisons incl. W06 render | Determinism is a designed property of W02/W03/W06; it must be exercised end-to-end | W08 | S5xx runs |
| Host test entry points (P0-W08) | Planned, unimplemented | Assumed contract with blocked-boundary | Suite must hang off the baseline entry, not invent one | P0-W08 owner | Entry verification when it lands |

No ledger row invents fuzzing infrastructure, CI, QEMU, or hardware scope.

## Resolved design decisions and their authority

1. **Scenarios assert published contracts only.** Every expected observable
   is a typed error/variant or documented property from a sibling design's
   contract file; W08 adds no wrapper that re-derives semantics. Rationale:
   a regression suite that re-implements behavior would pass while the
   product diverges — the exact anti-pattern the single-semantics rule
   (W07 Decision 1) forbids.
2. **Fixed seeds, recorded per run.** Pseudo-random generators in S3xx/S4xx
   are seeded from constants recorded in the evidence row (default seeds
   fixed in [04 §3](04-automation-and-evidence.md)); a run without its
   seed is invalid evidence. Rationale: P2-V10 demands reproducibility;
   unseeded randomness makes failures unrepeatable.
3. **One scenario = one matrix row = one evidence row.** Scenarios are
   individually named (`W08-S<group><nn>`) so a failure is diagnosed and
   re-run per row, and partial runs are recorded as not-run per row rather
   than blurring into a suite-level blur. Rationale: P2-V10's acceptance is
   per J-group; per-row evidence makes group completion checkable.
4. **The hard gate gets a soak scenario, not an example.** S3xx-P1 runs a
   seeded randomized sequence of valid and invalid operations with a
   protected-frame audit after every operation. Rationale: "never returns a
   protected page for every valid sequence" is universal; examples cannot
   carry it, and one long audit-per-op soak at host strength is the
   strongest evidence available before QEMU (W09) — while still not
   proving hardware behavior.
5. **Failure injection is part of the matrices, not an afterthought.**
   Divergence cases (accounting mismatch, seal mismatch) are included
   because W04/W05/W06 ship detectors whose correctness is itself
   safety-relevant. Rationale: a detector that silently passes would undo
   the typed-error guarantees the whole stage rests on.
6. **No performance measurement anywhere.** Stress scenarios check
   invariants and completion, never durations. Rationale: the plan
   separates correctness stress from performance claims (same split as
   W05-DV06/DV07); timing on shared CI hosts is noise, not evidence.
7. **The suite must itself be deterministic and hermetic.** No wall-clock,
   no network, no filesystem outside the declared fixture inputs, no
   parallel-order dependence; scenario execution order is fixed.
   Rationale: a flaky safety suite destroys the evidence chain P2-V10
   requires.

## Work breakdown and loading order

1. Read [01-scope-and-foundations.md](01-scope-and-foundations.md), then
   the two matrix files ([02](02-matrices-input-and-map.md),
   [03](03-matrices-allocator-stress-determinism.md)) — these define the
   product.
2. Implement the harness per
   [04-automation-and-evidence.md](04-automation-and-evidence.md) in its
   order; matrices are the requirement source, the automation contract is
   the organization source.
3. Validate the *regression design* per
   [05-validation-and-handoff.md](05-validation-and-handoff.md), record
   decisions/deviations in
   `../p2-w08-host-robustness-regression-record.md` and actual scenario
   evidence in
   `../../verification/p2-w08-host-robustness-regression-verification.md`
   when that work starts; nothing here claims W08 complete or records any
   result.

## Explicitly excluded interfaces

No production-code changes to W01–W06 modules (a scenario needing new
product observability is a sibling-design conflict to raise, not a local
instrumentation patch); no QEMU invocation, guest, or hardware access; no
coverage-guided fuzzer, CI workflow, or scheduler integration; no
performance assertions; no new dependencies beyond the P0 baseline's
test-support provisions; no board/platform names in harness code (fixtures
remain data). The suite's only published surface is the scenario IDs, the
evidence-row schema, and the summary format fixed in
[04](04-automation-and-evidence.md).

## Downstream handoff

- **W09** ([../p2-w09-qemu-integration-regression/README.md](../p2-w09-qemu-integration-regression/README.md))
  receives the host-side safety baseline: W09 may combine it with
  reference-platform integration and must treat S3xx/S5xx as the host
  strength of the properties it observes at integration strength.
- **W10** ([../p2-w10-p3-p4-handoff-contract/README.md](../p2-w10-p3-p4-handoff-contract/README.md))
  receives the scenario-ID scheme, evidence locations, and the explicit
  list of what host regression does not prove, for the stage-gate evidence
  map and the known-limitations record.
- **W01–W06 owners** receive their typed contracts back as exercised
  surfaces: any scenario failure is evidence against the owning package's
  verification record, traceable by scenario ID.
- **P3/P4 planners** (via W10) receive the seeded-property-test pattern as
  the established host-regression idiom for later stages' own robustness
  packages (e.g. p3-w12, p4-w08 style work), without any P3/P4 mechanism
  being designed here.
