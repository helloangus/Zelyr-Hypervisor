# P5-W08 Host Fuzz, Lifecycle Stress, SMP, and Performance Baseline — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The host-side robustness evidence for P5 — fuzz/property seams,
lifecycle stress, declared two-pCPU concurrency scenarios, and the four
performance-baseline categories — required by
[P5-W08](../../plans/p5-w08-host-fuzz-stress-smp-baseline.md).  
**Owner/change context:** P5-W08 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P5-W08. It defines the
host-testable validation seams at the W02–W06 boundaries, the fuzz and
property matrices with explicit failure oracles, the object/authority
lifecycle stress scenarios, the declared two-pCPU race-exposure scenarios,
and the baseline measurement method for the four required categories. It is
a validation-package design: it defines scenarios, oracles, pass conditions,
repetition rules, and evidence destinations — **not results**. No run,
finding, or completion is claimed here, and no scenario may be executed
before its dependent mechanisms (W03–W06) are implemented.

A coding agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

| Assigned step | Load |
|---|---|
| Fuzz and property seams, input models, oracles, seed/repetition rules | [01 — fuzz and property matrix](01-fuzz-and-property-matrix.md) |
| Lifecycle stress and two-pCPU concurrency scenarios, failure detection | [02 — stress and SMP scenarios](02-stress-smp-scenarios.md) |
| Performance-baseline method; ordered implementation; validation matrix | [03 — performance baseline and workflow](03-performance-baseline-and-workflow.md) |

Before editing, the agent must also follow the Coding Guidelines preflight,
including the repository `AGENTS.md`, documentation index, ADR baseline, P5
task book, and the P5-W08 plan.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P5 task book → frozen contracts →
P5-W08 plan → this design → Coding Guidelines. In particular:

- The plan covers P5-T16, T17, T19, and the source task book §32: fuzz- and
  property-ready parsing, range, handle, rights, and state checks;
  create/lookup/destroy/recreate stress; a declared two-pCPU concurrent
  foundation; and baseline measurements for minimum call, handle lookup,
  authority check, and Guest-data validation — without a performance KPI.
- Host-side evidence is **complementary** to W07's Guest-side evidence; it
  does not replace it. Neither proves the other's boundary.
- The P0 host-test basis
  ([P0-W08](../../../p0/plans/p0-w08-host-side-testing-baseline.md)) and the
  P3 SMP handoff ([P3-W14](../../../p3/plans/p3-w14-p4-smp-handoff.md)) are
  assigned prerequisites: the host-test harness conventions and the
  evidenced multi-pCPU QEMU foundation. New test dependencies are selected
  only under [P0-W18](../../../p0/plans/p0-w18-dependency-governance.md)
  governance; this design fixes requirements, not crates.
- QEMU timing is not hardware timing: the performance baseline is recorded
  with method and environment and carries no KPI claim (task book
  out-of-scope; Plan-Agent detailed reference §66–§68).

Classification:

- **Required:** the four seam families with input models and oracles; the
  lifecycle stress scenarios with invariant audits; the declared two-pCPU
  scenarios; the four-category baseline method; deterministic seed and
  repetition rules; the smoke configuration handed to W09; evidence
  destinations.
- **Reserved:** a whole-Hypervisor fuzz framework; persistent corpus
  policy; additional fuzz tooling beyond the P0-governed choice; scaling
  beyond the declared two-pCPU scenarios; any performance target or KPI.
- **Out of Scope:** exact random-generator selection as a contract, final
  lock/index strategy, a scheduler, scalability claims, real-hardware
  timing or proof, Guest-side integration evidence (W07), telemetry
  integration (W09), and any P6+ mechanism.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Fuzz/property-ready parsing, range, handle, rights, state checks (P5-T16) | [fuzz matrix](01-fuzz-and-property-matrix.md) §2–§3 | P5-V13 (W08-DV02–DV05) |
| Explicit invariant and non-success criteria for randomized/boundary input | [fuzz matrix](01-fuzz-and-property-matrix.md) §4 | P5-V13 (W08-DV06) |
| Create/lookup/destroy/recreate stress (P5-T17) | [stress](02-stress-smp-scenarios.md) §2 | P5-V13 (W08-DV07) |
| Declared two-pCPU concurrent scenarios exposing lookup/validation/grant/revoke/destruction races | [stress](02-stress-smp-scenarios.md) §3 | P5-V14 (W08-DV08) |
| Multi-pCPU state protection without assuming final locks | [stress](02-stress-smp-scenarios.md) §4 | P5-V14 (W08-DV08) |
| Baseline measurements: minimum call, handle lookup, authority check, Guest-data validation (P5-T19, §32) | [baseline](03-performance-baseline-and-workflow.md) §2 | P5-V14 (W08-DV09) |
| Baseline recorded without unreviewed performance-target use; QEMU ≠ hardware | [baseline](03-performance-baseline-and-workflow.md) §2.4 | W08-DV09 review condition |
| Robustness/limit facts handed to W09–W10 | [workflow](03-performance-baseline-and-workflow.md) handoff checklist | W08 closure review (W08-DV10) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p5-implementation-designs`,
`git ls-files`): `tests/` holds only a `.gitkeep`; there is no host-test
harness, no fuzz target, no benchmark, and no Rust code at all in the
tracked tree. P0–P4 are planned but not implemented (no records in P1–P4
`implementation/` or `verification/`). The W03–W06 detailed designs are
parallel work on this branch (forward references by slug). No P5
stress/fuzz/baseline artifact exists.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Fuzz/property evidence for parsing, range, handle, rights, state checks (P5-V13) | No seams or harness exist | Host-testable seams at the W02–W06 boundaries (their designs own seam placement; W08 owns harness contracts) | Only decoupled, hardware-independent logic is host-fuzzable (Coding Guidelines §94) | W02–W06 designs (seams); W08 (matrix, oracles) | W08-DV01 prerequisite review; DV02–DV06 runs |
| Lifecycle stress with invariants preserved (P5-V13) | Nothing tracked | The delivered W04/W05 lifecycle contracts plus the stress scenarios of [02 §2](02-stress-smp-scenarios.md) | Stress can only exercise delivered create/destroy/revoke behavior | W04/W05 (contracts); W08 (scenarios) | W08-DV07 run record |
| Two-pCPU evidence (P5-V14) | No SMP runtime exists | The P3-evidenced multi-pCPU QEMU foundation | Concurrent scenarios need online pCPUs and the P3 synchronization rules | P3-W14 handoff (assigned prerequisite) | W08-DV08 run record |
| Four baseline categories recorded (P5-V14) | Nothing tracked | The measurement method of [03 §2](03-performance-baseline-and-workflow.md) and its raw records | A baseline exists only if method, environment, and raw data are recorded | W08 (method); implementing agent (records) | W08-DV09 evidence |
| Reproducibility of randomized evidence | No generator or seed policy exists | Deterministic seed and repetition rules of [01 §5](01-fuzz-and-property-matrix.md) | Unreproducible robustness evidence cannot be regression-checked by W09 | W08 (rules); P0-W18 (tool choice) | seed and config in each run record |
| Evidence lands in governed locations | `../../verification/` holds only `.gitkeep` | Run records under `../../verification/p5-w08-host-fuzz-stress-smp-baseline-verification.md` | Task book separates evidence from design | W08 | verification record |

No row requires inventing a crate, lock strategy, or performance target;
the fuzz-tool selection is explicitly routed to P0-W18 governance, and the
concurrency scenarios exercise the W04/W05 delivered protection without
prescribing it.

## Resolved design decisions and their authority

1. **Seam placement is owned by the boundary designs; harness contracts by
   W08.** Each W02–W06 validation boundary must expose its checks as
   host-callable, hardware-independent seams (parse/validate/resolve/check
   functions over plain inputs). If a boundary delivered without such a
   seam, that is a defect of that boundary's design or implementation —
   W08 records the blocked prerequisite rather than bypassing the boundary
   or duplicating its logic. Authority: plan prerequisite structure; Coding
   Guidelines parser/execution separation.
2. **Failure oracles are behavioral, not tool-specific.** A run fails on:
   panic or abort; hang beyond the declared watchpoint; an unclassified
   outcome (anything outside the delivered boundary's result vocabulary);
   an accepted-invalid (an invariant violation such as a stale handle
   resolving or an unauthorized success); or a post-run audit mismatch.
   This keeps evidence meaningful regardless of which P0-W18-governed
   fuzzing tool executes it. Authority: this design within plan scope;
   P5-V13's "without panic, corruption, or stale acceptance".
3. **Deterministic seeds and bounded runs.** Every randomized run records
   its seed, iteration bound, and configuration; smoke configurations are
   fixed and handed to W09 for regression; deep runs are local evidence.
   Persistent corpus policy is Reserved. Authority: this design;
   reproducibility requirement of P5-V13.
4. **Stress exercises delivered contracts, never a lock design.** The
   stress scenarios call the W04/W05 lifecycle and authority contracts and
   audit invariants at boundaries; they do not prescribe or measure lock
   internals. Authority: plan out-of-scope ("final lock/index strategy").
5. **Two-pCPU scope is declared, not scaled.** Exactly the declared
   scenarios of [02 §3](02-stress-smp-scenarios.md) run on the P3-evidenced
   two-pCPU QEMU configuration; outcomes must be consistent with some
   serial order of the operations. No 4/8-pCPU or scalability claim is
   made. Authority: plan out-of-scope; task book Reserved item (later
   scheduling may move callers, so no permanent binding is assumed).
6. **The baseline is a record, not a target.** The four categories are
   measured by the method of
   [03 §2](03-performance-baseline-and-workflow.md) in the declared QEMU
   environment; raw data and environment are recorded; no number is
   promoted to a requirement, and security checks are never disabled for
   measurement. Authority: plan ("without using it as an unreviewed
   performance target"); Coding Guidelines benchmark-path rule.
7. **Guest-side equivalence is not claimed.** Host-side seam evidence and
   QEMU Guest evidence (W07) prove different boundaries; a seam pass never
   substitutes for a Guest scenario and vice versa. Authority: plan
   out-of-scope ("a replacement for Guest-side integration evidence").

## Work breakdown and loading order

1. Load [01](01-fuzz-and-property-matrix.md) for seams, input models,
   oracles, and seed/repetition rules.
2. Load [02](02-stress-smp-scenarios.md) for stress and concurrency
   scenarios, invariant audits, and failure detection.
3. Load [03](03-performance-baseline-and-workflow.md) for the baseline
   method and the ordered implementation steps, validation matrix, and
   handoff checklist.
4. Record facts in
   `../p5-w08-host-fuzz-stress-smp-baseline-record.md` (created when work
   starts) and runs in
   `../../verification/p5-w08-host-fuzz-stress-smp-baseline-verification.md`.
   Neither this design nor any record may claim W08 complete; completion is
   claimed only in verification material, for what was actually run.

## Explicitly excluded interfaces

No whole-Hypervisor fuzz framework, no fuzz-target ABI, no persistent
corpus format, no benchmark API, no lock, index, or allocator strategy, no
scheduler, no Guest-side scenario (W07), no telemetry counter or trace
event (W09), and no P6+ mechanism. Seams consumed from W02–W06 keep their
owners' names and contracts; W08 defines only the harness-facing scenario
and oracle contracts above them. Any interface implied by prose but not
contracted in the supporting files must not be implemented; raise the gap
as a design conflict.

## Downstream handoff

- **W09** ([telemetry/regression](../p5-w09-telemetry-safe-logging-regression/README.md))
  receives the fixed fuzz-smoke configuration (seams, seeds, iteration
  bounds) as a regression row, the non-success/oracle classes, and the
  robustness limits observed in evidence.
- **W10** ([closeout](../p5-w10-closeout-p6-handoff/README.md)) receives the
  evidence, stress duration/configuration, and the performance record
  (method, environment, raw data, explicit non-claims) for the factual P5
  record.
- **P6** (through W10 only) may reuse the seam/oracle pattern for its own
  robustness package ([P6-W12](../../../p6/plans/p6-w12-fault-isolation-robustness.md));
  it designs its own scenarios and inherits no performance number and no
  P5 concurrency claim.
