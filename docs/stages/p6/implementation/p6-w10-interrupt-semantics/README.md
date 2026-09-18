# P6-W10 Interrupt Semantics — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The minimum P6 Guest-visible interrupt semantics — masking,
priority, pending accumulation, repeated events, and concurrent
timer-plus-vIRQ delivery — their boundaries with the vCPU timer and
virtualization-interface contracts, and the authoritative P6 interrupt-semantics
artifact (task-book deliverable P6-DOC-02), per
[P6-W10](../../plans/p6-w10-interrupt-semantics.md).  
**Owner/change context:** P6-W10 semantics consolidation and exercise handoff;
this design owns the P6-level semantic decisions the task book reserves to
detailed design and owns the Interrupt Semantics v0 record.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P6-W10. It is a
semantics/contract consolidation package: its primary deliverable is one
authoritative, bounded statement of P6 Guest-visible interrupt behavior,
exercised through the scenarios it defines. It deliberately does **not**
implement Host mechanisms (those belong to W03/W05–W09 designs), does not
maintain the Validation Guest asset (W11), and does not collect stage
evidence (W13).

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

- [01-interrupt-semantics-contract.md](01-interrupt-semantics-contract.md) —
  the semantic decisions themselves (masking layers, priority bands,
  pending/repeated-event rules, timer-plus-vIRQ relation), the schema of the
  Interrupt Semantics v0 record, and the scenario-semantics definitions W11
  consumes. Load for design, implementation, and review steps.
- [02-workflow-and-validation.md](02-workflow-and-validation.md) — ordered
  steps, validation matrix (P6-V13–P6-V15), observability model, and handoff
  checklist.

Before editing, follow the Coding Guidelines preflight: repository
[AGENTS.md](../../../../../AGENTS.md), [documentation
index](../../../../README.md), [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), [P6 task
book](../../task-book-v0.1.md), and the [P6-W10
plan](../../plans/p6-w10-interrupt-semantics.md). This document is proposed
design only; it contains no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P6 task book → P6-W10 plan → this
design → Coding Guidelines. In particular:

- The task book §1 **Reserved** clause explicitly delegates to detailed
  design: "a P6 virtual-IRQ namespace, temporary Validation Guest interrupt
  layout, timer pause/resume behavior, and basic priority semantics may be
  selected in detailed design; none freezes the P8 rusthv-arm-virt-v1 machine
  ABI." Every semantic decision below cites that clause; none of them is a
  machine-ABI, vGIC, or scheduler decision.
- The task book §1 Required clause P6-J demands masking, priority,
  pending, and concurrent timer/vIRQ semantics be defined and exercised
  (P6-V13–P6-V15), including the rule that Guest masking does not imply
  completion.
- The plan excludes: complete GIC priority/preemption specification, Linux
  vGIC compatibility, scheduler run-state policy, detailed priority encoding,
  and a final event-ordering algorithm. The semantics below are deliberately
  minimal and bounded.
- ADR-007 (Guest untrusted), ADR-032 (GICv3 baseline), and ADR-048
  (structured telemetry) bound how the semantics may be realized; Coding
  Guidelines bind any code-bearing step (integration touches W07/W08/W09
  code paths).

Classification:

| Class | Items |
|---|---|
| **Required** | Masking semantics (all layers) with no-false-completion rule; two-band priority relation; pending-accumulation and repeated-event policy; timer-plus-vIRQ progression rules; semantics-across-exit statement; the Interrupt Semantics v0 record; scenario-semantics definitions for P6-V13–P6-V15 consumed by W11. |
| **Reserved** | Priority-mask (ICC_PMR-class) semantics beyond the conditional rule S-M2 (depends on the W08-presented virtual CPU interface); exact numeric priority encoding (Specification Investigation, task book §8); any richer enable/mask model (P8 vGIC owns the Linux-visible form); timer pause/resume behavior beyond the W06 record reference. |
| **Out of Scope** | Full GIC priority/preemption specification coverage; Linux vGIC Distributor/Redistributor MMIO semantics (P8); scheduler run-state, blocking, or wakeup policy (P7); final event-ordering algorithm; machine ABI or Guest DTB; new Host mechanisms; real-hardware semantics claims. |

## Requirement → design-location → acceptance mapping

| Plan requirement (P6-W10) | Detailed-design location | Acceptance |
|---|---|---|
| Inspect W06 timer behavior, W09 maintenance behavior, W07 lifecycle semantics, applicable Validation Guest inputs | [workflow](02-workflow-and-validation.md) step 1 | prerequisite reconciliation verdicts in the implementation record |
| Approved design for bounded masking, priority, pending, repeated-event, timer-plus-vIRQ semantics | [contract](01-interrupt-semantics-contract.md) §2–§5 | design approved; each rule has an authority basis and a proof boundary |
| Integrate with deferred delivery and Guest entry/exit without assigning scheduler responsibility to P6 | [contract](01-interrupt-semantics-contract.md) §5–§6; [workflow](02-workflow-and-validation.md) step 3 | integration review finds no run-state or scheduling semantic introduced |
| Mask/pending/unmask, different-priority, multiple-pending, repeated, concurrent-event acceptance scenarios | [contract](01-interrupt-semantics-contract.md) §7; [workflow](02-workflow-and-validation.md) §3 (W10-DV01–DV06) | P6-V13, P6-V14, P6-V15 evidence via the W11 suite, cross-cited |
| Review Guest-untrusted controls, non-loss requirements, telemetry, P8 machine-ABI exclusion | [workflow](02-workflow-and-validation.md) step 5; [contract](01-interrupt-semantics-contract.md) §6, §8 | review findings recorded; no frozen machine-ABI statement exists |
| Record the semantic contract; hand to W11–W13 and P7/P8 | [workflow](02-workflow-and-validation.md) step 6; §5 handoff checklist | Interrupt Semantics v0 record exists with factual content only |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p6-implementation-designs`):
P0 documentation scaffold only — no Cargo workspace, no Rust sources, no P6
implementation records; no interrupt-semantics document exists anywhere under
`docs/stages/p6/`. P0–P5 are planned but unimplemented; the P6-W01–W09
detailed designs are being written in parallel and are referenced by slug and
ID only. Every prerequisite below is an assumed contract with an explicit
failure boundary in [02-workflow-and-validation.md](02-workflow-and-validation.md)
§1.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Minimum Guest-visible semantics are defined | No semantics document exists in any tracked file | The semantic rules of [01-interrupt-semantics-contract.md](01-interrupt-semantics-contract.md) §2–§5 as the P6-level authority | Without one authority, masking/priority/pending behavior would be inferred per-package and drift | W10 (this design; task book §1 Reserved delegation) | design review; then the v0 record |
| Semantics are exercised (P6-V13–P6-V15) | No evidence exists; no Guest suite exists yet | W11 scenario suite consuming §7 definitions; W07/W08/W09 implemented behavior beneath it | Exercise requires both a runner (W11) and implemented semantics (W07–W09) | W11 (`../p6-w11-validation-guest-interrupt-suite/README.md`); W07–W09 | W11 verification records citing VG-IRQ-02/03/04/05 |
| Masking does not imply completion | No rule exists anywhere | Rule S-M1 (§2.1) as the binding statement | The plan names this rule as the load-bearing masking property | W10 (this design) | VG-IRQ-03 evidence |
| Boundaries with vCPU timer and virtualization-interface contracts | W05/W06/W08 semantics not yet fixed in available documents | Cross-boundary rules §5, reconciled against W05/W06/W08 designs in step 1 | A semantics document that contradicts the timer or presentation contracts is worse than none | W05/W06/W08 designs | step 1 reconciliation verdicts |
| Semantic record handed to W11–W13 and P7/P8 | Absent | Interrupt Semantics v0 record (P6-DOC-02) at the §1 location, factual content only | Consumers need one citable, implemented-behavior statement | W10 (record owner); W13 (composition consumer) | record + handoff checklist |

No row requires inventing a machine ABI, a scheduler policy, or an
unapproved API; the P8-exclusion review is itself a required deliverable
(step 5), not a blocker.

## Resolved design decisions and their authority

1. **Authoritative artifact and location.** The P6 interrupt-semantics
   authority is the **Interrupt Semantics v0 record** (task-book deliverable
   P6-DOC-02) at `docs/stages/p6/implementation/p6-interrupt-semantics-v0.md`,
   owned by W10 and created when implementation begins. This design
   ([01-interrupt-semantics-contract.md](01-interrupt-semantics-contract.md))
   is the proposed semantic authority until then; the record transcribes the
   implemented facts and any deviations. Rationale: the task book requires a
   factual P6-DOC-02 in the documentation layers, and AGENTS.md separates
   design from implementation records. Stage-local design freedom owned here
   (location only; the deliverable itself is task-book-owned).
2. **Masking model — three layers, one no-completion rule.** Layer L1:
   architectural CPU masking (PSTATE DAIF) at Guest EL1. Layer L2: priority
   masking via the Guest-visible virtual CPU interface **only if** the W08
   design presents it; otherwise L2 is recorded as unavailable, not
   simulated. Layer L3: a hypervisor-side per-vIRQ `enabled` control in the
   P6 virtual-IRQ namespace (task book §1 Reserved delegation; distinct from
   any P8 vGIC enable semantic). Rule S-M1 binds all layers: masking never
   implies completion. Rationale: L1/L2 are architectural; L3 gives the
   validation suite a deterministic Host-side mask without pre-empting P8's
   machine model. Authority: task book §1 Reserved; plan goal.
3. **Priority — two bands, relation-only.** Band A = vCPU virtual-timer
   events (higher); band B = ordinary vIRQs (lower). The contract fixes only
   the band relation at presentation opportunities (A before B when both are
   ready) and its survival in pending state; intra-band order remains the
   W07 queue discipline; numeric priority encoding is a Specification
   Investigation resolved at implementation against the applicable
   specification. Rationale: plan excludes detailed priority encoding and the
   full GIC preemption model, while P6-V15 requires a documented, preserved
   relation. Authority: plan scope; task book §8.
4. **Pending and repeated events — bounded accumulation.** Pending state per
   vIRQ is a bounded accumulation (deduplicated flag semantics): P1 a repeat
   while pending collapses; P2 a repeat while presented adds at most one
   deferred re-pend; P3 distinct vIRQs are never merged; P4 capacity
   behavior is the W07 contract's, with same-ID collapse on overflow.
   Rationale: satisfies P6-V14 ("documented policy without state corruption")
   with a determinate, loss-bounded rule set and no queue growth. Authority:
   plan scope; ADR-007 (bounded Guest-influenceable state).
5. **Timer plus vIRQ — progression without ordering claims.** C1: both event
   classes progress within one Guest entry. C2: when both are ready at the
   same presentation opportunity, band A is presented first. C3: no further
   inter-class ordering is guaranteed. C4: a timer expiry while the vCPU is
   absent follows the W06 deferred-expiry contract and is presented at the
   next entry. Rationale: P6-V15 requires both classes to progress and the
   documented relation to hold — nothing more. Authority: plan scope.
6. **No scheduler responsibility.** No rule here depends on or assigns vCPU
   run-state, blocking, wakeup, or preemption semantics; where a rule touches
   vCPU presence ("while the vCPU is absent"), it uses only the W06/W08
   entry/exit contracts. Authority: plan step 3; task book §1 Out of scope
   (P7 boundary).
7. **Non-freeze statement.** None of decisions 2–5 freezes the P8
   rusthv-arm-virt-v1 machine ABI, a vGIC MMIO semantic, an interrupt
   layout, or a scheduler policy; §8 of
   [01-interrupt-semantics-contract.md](01-interrupt-semantics-contract.md)
   states the exclusions verbatim so downstream readers cannot mistake P6
   semantics for a machine contract. Authority: task book §1 Reserved and §7.

## Work breakdown and loading order

1. Read this README and the Coding Guidelines in full.
2. Load
   [01-interrupt-semantics-contract.md](01-interrupt-semantics-contract.md)
   — it is the sole authority for the semantic rules, the v0 record schema,
   and the §7 scenario definitions.
3. Execute [02-workflow-and-validation.md](02-workflow-and-validation.md) in
   order: prerequisite reconciliation, integration review of the implemented
   behavior (once W07–W09 code exists), record creation, scenario handoff to
   W11, boundary review, evidence cross-citation.
4. Evidence for P6-V13–P6-V15 is produced by the W11 suite runs and Host-side
   telemetry correlation; it lands in
   `../../verification/p6-w11-validation-guest-interrupt-suite-verification.md`
   and is cross-cited (never duplicated) in
   `../../verification/p6-w10-interrupt-semantics-verification.md`. Decisions
   and deviations are recorded in
   `../p6-w10-interrupt-semantics-record.md` when implementation begins.
   Neither file may claim W10 or stage completion; P6-DOC-02 carries factual
   content only.

## Explicitly excluded interfaces

No new Rust type, function, trait, module, crate, public API, ABI, wire
format, or persistent layout is authorized by W10. The L3 per-vIRQ
`enabled` control is realized through the W07 vIRQ lifecycle contract's
existing state surface — W10 defines its **semantics**, not a new interface;
if W07's design has no suitable surface, that is a step-1 deviate verdict
blocking integration, not a license to add one here. Specifically excluded:
vGIC MMIO register semantics; Guest DTB interrupt layout; machine ABI; any
scheduler-visible state; any Guest-callable control beyond the P5 boundary;
and any priority encoding beyond the band relation.

## Downstream handoff

- **P6-W11** (`../p6-w11-validation-guest-interrupt-suite/README.md`)
  receives the §7 scenario-semantics definitions (VG-IRQ-02/03/04/05 and the
  timer rows they interact with) as the normative expected behavior its
  scenarios assert.
- **P6-W12** (`../p6-w12-fault-isolation-robustness/README.md`) receives the
  Guest-untrusted control statement (§6): which inputs are validated, and
  that Guest-caused masking/EOI anomalies are GuestFault-class, never Host
  state corruption.
- **P6-W13** (`../p6-w13-telemetry-regression-handoff/README.md`) receives
  the Interrupt Semantics v0 record as the P6-DOC-02 deliverable for the
  documentation review (P6-V27) and the P7/P8 consumer review (P6-V28).
- **P7** receives, through W13's consumer review, the factual semantics
  needed for wakeup/block reasoning — with the explicit boundary that run
  state, switch, and wakeup policy are P7-owned and no P6 rule constrains
  them.
- **P8** receives the same factual record plus the §8 non-freeze statement;
  P8 owns the Linux-visible vGIC and machine ABI and may not treat P6
  masking/priority/pending rules as frozen controller behavior.
