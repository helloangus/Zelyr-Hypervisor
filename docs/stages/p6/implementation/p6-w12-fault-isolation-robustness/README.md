# P6-W12 Fault Isolation and Robustness — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The P6 failure boundary — authorization and ownership rejection of
invalid virtual-IRQ operations, safe spurious/unknown Host IRQ handling,
diagnosed impossible internal states, and bounded high-rate smoke cases, per
[P6-W12](../../plans/p6-w12-fault-isolation-robustness.md).  
**Owner/change context:** P6-W12 robustness handoff; this design owns the
case matrix, containment classes, and fault-exercise semantics as
integrations over the W03/W07/W09 mechanisms and the P5 Guest-error boundary
— it owns no new recovery framework.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P6-W12. It is a validation
package: it defines the fault/robustness case matrix, each case's injection
point and containment class, pass conditions and proof boundaries, and
evidence destinations. It produces **no results** — results belong to the
verification record after real runs.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

- [01-robustness-case-matrix.md](01-robustness-case-matrix.md) — the case
  classes FI-A–FI-D, per-case input/precondition, expected observable, pass
  condition, proves/does-not-prove, repetition and injection semantics, and
  evidence destinations. Load before running or reviewing any case.
- [02-workflow-and-validation.md](02-workflow-and-validation.md) — ordered
  steps, validation matrix, the error/security/observability model, and the
  handoff checklist.

Before editing, follow the Coding Guidelines preflight: repository
[AGENTS.md](../../../../../AGENTS.md), [documentation
index](../../../../README.md), [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P6 task
book](../../task-book-v0.1.md), and the [P6-W12
plan](../../plans/p6-w12-fault-isolation-robustness.md). This document is
proposed design only; it contains no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P6 task book → P6-W12 plan → this
design → Coding Guidelines. In particular:

- ADR-007 makes Guest input untrusted by default; ADR-013 makes
  capability/handle + rights + generation the authorization model (never
  role or VM-ID shortcuts); the ADR §19 invariants require Guest-caused
  faults to stay VM-local and forbid degradation of failed capability checks
  into implicit allowance.
- The task book (§1 Required) demands invalid Guest interrupt requests,
  unknown/spurious Host IRQs, impossible state, and storm smoke cases be
  constrained to diagnosable safe outcomes (P6-V20–P6-V22), and explicitly
  says storm smoke "does not prove production DoS resistance."
- The plan excludes: production-grade DoS policy, device passthrough
  isolation, a global recovery framework, a full fault-injection platform,
  and detailed panic/error implementation (those mechanisms are
  [P0-W14](../../../p0/plans/p0-w14-panic-failure-classification.md)'s and
  the P1 crash-diagnostics contract's).
- The upstream failure boundaries are assumed contracts:
  [P5-W10](../../../p5/plans/p5-w10-closeout-p6-handoff.md) (evidenced
  HVC/error/handle facts), [P4-W06](../../../p4/plans/p4-w06-fault-isolation-diagnostics.md)
  (exit/fault diagnostics), P1-W07 (fatal crash diagnostics).

Classification:

| Class | Items |
|---|---|
| **Required** | FI-A invalid vIRQ operation rejection (range, priority, target, owner, dead object, insufficient right, malformed input); FI-B spurious/unknown Host IRQ containment; FI-C impossible-state detection/diagnosis/escalation (including the W09 orphan-completion case); FI-D declared storm smoke; containment-class mapping per P0-W14; evidence destinations. |
| **Reserved** | Exact storm limits and durations (declared at run time from the environment, recorded in evidence); additional diagnostic detail in dumps; any future fault-injection platform (excluded by plan, revisitable only by a new design). |
| **Out of Scope** | Production DoS policy; device passthrough isolation; a global recovery framework; new panic/error machinery beyond P0-W14 classification use; scheduler or pause/fault lifecycle policy (P7); real-hardware fault behavior claims. |

## Requirement → design-location → acceptance mapping

| Plan requirement (P6-W12) | Detailed-design location | Acceptance |
|---|---|---|
| Inspect W03, W07, W09, P5 Guest-error/capability boundaries, unsafe/diagnostic governance | [workflow](02-workflow-and-validation.md) step 1 | prerequisite verdicts in the implementation record |
| Approved design for validation, rejection, containment, diagnostic, and escalation outcomes | [matrix](01-robustness-case-matrix.md) §1–§2; [workflow](02-workflow-and-validation.md) §4 | design approved; every case has containment class and proof boundary |
| Integrate expected failure behavior with physical-IRQ lifecycle, vIRQ/maintenance state, VM isolation assumptions | [matrix](01-robustness-case-matrix.md) §2 injection points; [workflow](02-workflow-and-validation.md) step 3 | each injection point exists in an owning contract or is recorded blocked |
| Invalid-range/priority/target/owner/dead-object, spurious/unknown, impossible-state, high-rate smoke acceptance cases | [matrix](01-robustness-case-matrix.md) §2; [workflow](02-workflow-and-validation.md) §3 (W12-DV01–DV08) | P6-V20, P6-V21, P6-V22 evidence classified |
| Guest-caused errors stay local; invariant failures are visible, not silent | [matrix](01-robustness-case-matrix.md) §3; [workflow](02-workflow-and-validation.md) step 5 | containment review recorded; no silent-repair path exists |
| Record factual results, limits, and handoff to W13 and downstream security review | [workflow](02-workflow-and-validation.md) step 6; §5 handoff checklist | verification record complete; limits explicit |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p6-implementation-designs`):
P0 documentation scaffold — no Rust sources, no hypervisor error machinery,
no P6 implementation records, no verification evidence. P0–P5 are planned but
unimplemented; their error/capability/diagnostic contracts are assumed from
plan text. The P6-W01–W11 designs are being written in parallel and are
referenced by slug and ID only. Each ledger row states the missing foundation
the plan outcome necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Invalid virtual-IRQ operations are rejected (P6-V21) | No vIRQ control path exists | W07 vIRQ lifecycle with the P5 authorization boundary (P5-W02 error semantics, P5-W05 rights) exercised through test inputs | Rejection is only defined where an authorization contract exists to reject against | W07 (`../p6-w07-virtual-interrupt-core/README.md`); P5 contracts | FI-A case evidence |
| Spurious/unknown Host IRQs handled safely (P6-V20) | No Host IRQ lifecycle exists | W03 physical-IRQ classification with spurious/unknown classes and safe completion | Unknown-IRQ behavior must already have a designed path for W12 to exercise and diagnose | W03 (`../p6-w03-physical-interrupt-lifecycle/README.md`) | FI-B case evidence |
| Impossible internal states are visible, not silent (P6-V20/V21 support) | No state machinery exists | W09 orphan-completion escalation contract; P0-W14 classification; P1-W07 crash diagnostics | Detection without a classification and dump path is a dead end | W09 (`../p6-w09-maintenance-interrupt/README.md`); P0-W14; P1-W07 | FI-C case evidence |
| Bounded storm smoke without corruption/hang (P6-V22) | Nothing exists to stress | Implemented W05/W07/W09 paths plus declared environment limits | A smoke case needs the real paths and a declared, bounded rate | W05/W07/W09; this design owns limits declaration | FI-D case evidence |
| No cross-VM or Host effect from Guest errors | No VM machinery exists | P4 VM/vCPU isolation boundary; ADR §19 invariants | Locality is only meaningful against an implemented isolation boundary | P4-W09 handoff; W11's isolation rows corroborate | FI-A cross-VM probes |
| Results, limits, and handoff recorded | No verification record exists | Evidence destinations and taxonomy (this design) | Downstream security review needs classified, located evidence | W12 (this design); W13 composition | verification record |

No row requires inventing a recovery framework or a DoS policy; both are
plan-excluded, and their absence is stated as a limit in every handoff.

## Resolved design decisions and their authority

1. **Four case classes.** FI-A (invalid vIRQ authorization/input), FI-B
   (spurious/unknown Host IRQ), FI-C (impossible internal state), FI-D
   (bounded storm smoke) — fixed in
   [01-robustness-case-matrix.md](01-robustness-case-matrix.md) §2 as the
   sole case authority. Rationale: maps one-to-one onto the plan's case list
   and the task book's P6-V20–P6-V22 rows, keeping W13 composition
   deterministic. Authority: plan scope; task book §5.
2. **Containment classes per P0-W14.** Every case outcome maps to exactly
   one containment class: **GuestFault-local** (Guest-caused; VM-local
   structured error; Host continues), **diagnostic-contain** (hardware/
   specification anomalies; counted, dumped, contained; Host continues), or
   **fatal-invariant** (hypervisor state-integrity loss; escalated with full
   diagnostics per the P0-W14 classification and P1-W07 crash-diagnostics
   contract). Rationale: the ADR §19 rule that Guest-caused faults stay
   VM-local, and the plan's visibility requirement for invariant failures.
   Authority: P0-W14; ADR §19; plan step 5.
3. **Authorization rejection reuses the P5 boundary verbatim.** FI-A cases
   exercise the P5-W02/P5-W05 error semantics through the vIRQ control
   surface; W12 invents no new error codes, rights, or fallback behavior,
   and a failed check may never degrade into an implicit allowance (ADR §19).
   Rationale: one authorization boundary for the whole Guest surface.
   Authority: ADR-013; P5-W10 handoff.
4. **Injection is by declared test hooks only.** Fault conditions are
   reached through test-only hooks (input fuzzing of the authorized control
   path, forced-decode hooks, induced-state hooks) that are compiled out or
   gated in non-test builds per the P0 feature/profile governance
   ([P0-W04](../../../p0/plans/p0-w04-build-profile-feature-governance.md));
   no production path is modified to fail, and no synthetic hardware fault is
   claimed. Rationale: bounded, honest injection semantics — what each case
   proves is limited to the exercised code, not to real hardware misbehavior.
   Authority: plan ("no full fault-injection platform"); Coding Guidelines
   (honest evidence).
5. **Storm smoke is bounded and declared, never a DoS claim.** FI-D runs
   declared rates/durations over the implemented timer/vIRQ/maintenance
   paths; success is the absence of observed corruption, unexplained loss,
   or hang within the declared limits, and every handoff statement carries
   the task book's "does not prove production DoS resistance" boundary.
   Rationale: P6-V22 wording; plan exclusion of DoS policy. Authority: task
   book §1 and validation matrix.
6. **No silent repair.** No case outcome may silently rewrite, drop, or
   "fix" inconsistent state: diagnostics are recorded, containment is
   explicit, and the only permitted automatic action is the pre-designed
   contained behavior of the owning contract (for example, W09's counted
   no-op for duplicate completion). Rationale: plan step 5 visibility
   requirement. Authority: plan; Coding Guidelines.
7. **Cross-VM probes ride the W11 isolation rows.** FI-A's cross-VM effects
   are probed with the same Guest observation machinery W11 maintains; W12
   owns the hostile-target case definitions, W11's asset provides
   observation. No duplicate observation layer is built. Authority: plan
   step 1 (inspect applicable boundaries); W11 asset reuse statement.

## Work breakdown and loading order

1. Read this README and the Coding Guidelines in full.
2. Load [01-robustness-case-matrix.md](01-robustness-case-matrix.md) before
   running or reviewing any case; it is the sole case authority.
3. Execute [02-workflow-and-validation.md](02-workflow-and-validation.md) in
   order: prerequisite reconciliation, hook integration review, case runs,
   containment review, evidence recording.
4. Evidence lands in
   `../../verification/p6-w12-fault-isolation-robustness-verification.md`
   (created when evidence exists); decisions, deviations, and the declared
   storm limits go to
   `../p6-w12-fault-isolation-robustness-record.md` when implementation
   begins. Neither file may claim W12 or stage completion; P6-V20–P6-V22
   closure wording carries its proof boundary: passing does not claim
   production DoS resistance.

## Explicitly excluded interfaces

No public/stable Rust API, ABI, wire format, persistent layout, crate
boundary, or Guest-callable surface is authorized by W12. Test hooks are
internal to their owning modules, gated for test builds, and add no
production interface. Specifically excluded: any new error type or panic
machinery (P0-W14 classification and P1-W07 diagnostics are consumed as
contracts); any recovery/rollback framework; any rate-limiter or DoS policy
mechanism; any passthrough/device isolation logic (ADR-030/031 territory);
any scheduler pause/fault policy (P7); and any modification of the P5
authorization semantics.

## Downstream handoff

- **P6-W13** (`../p6-w13-telemetry-regression-handoff/README.md`) receives
  per-case evidence references for the P6-DOC-04 validation matrix
  (P6-V20–P6-V22 rows), the declared storm limits, and every unresolved
  risk/investigation for the P7/P8 consumer review's "unresolved items."
- **P7** receives the isolation boundary statement (GuestFault-local
  containment; storm-smoke limits) as security-review input for its own
  pause/fault and stress designs — with the boundary that P7 owns run-state
  and fault lifecycle policy.
- **P8** receives the same boundary plus the explicit limit that nothing
  here proves passthrough-device isolation, production DoS resistance, or
  real-hardware fault behavior; P8's Linux-facing security review must not
  cite P6 FI results beyond their stated proof boundaries.
