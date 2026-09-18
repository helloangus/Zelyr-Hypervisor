# P8-W13 Guest Fault Diagnostics — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The fault-classification and diagnostic-context contract needed to
debug Linux integration without global failure, as required by
[P8-W13](../../plans/p8-w13-guest-fault-diagnostics.md).  
**Owner-change context:** P8-W13 implementation handoff; diagnostics extend the
evidenced P4-W06 boundary and the P0-W12/W14 semantic baselines. It owns no
fault handler, crash dump, or trace encoding.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P8-W13. The plan makes W13 the
package that ensures every failure observable during Linux integration is
classified, carries the minimum actionable context, and stays contained at the
correct boundary: kernel panic, synchronous exception, Stage-2 fault,
unsupported sysreg, invalid MMIO, PSCI, vGIC, timer, vCPU-state, and
Hypervisor-invariant distinctions. This design converts that into (a) a closed
classification taxonomy that extends the P4-W06 classification with
Linux-integration classes under the P0-W14 top-level failure model, (b) the
minimum actionable context and recent-trace requirement, (c) routing rules that
keep Guest faults VM-facing and never let them become EL2 panics, and (d) the
regression-observable mapping that W16 and W18 consume. It deliberately does
**not** design a crash-dump system, fault-handler APIs, or a final trace
encoding, and it never treats a Guest fault as permission to panic EL2.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then loads
only the linked supporting file needed for its assigned step:

- [Fault classification](01-fault-classification.md) — read before classifying
  or reviewing coverage; upstream baselines, the taxonomy, decision rules, and
  containment routing.
- [Diagnostic context contracts](02-diagnostic-context-contracts.md) — read
  before building or reviewing classification/context logic; normative
  interface obligations.
- [Implementation workflow and review](03-implementation-workflow-and-review.md)
  — read before executing; ordered steps, validation matrix, failure/security/
  observability model, and handoff checklist.

Before editing, the agent must also satisfy the Coding Guidelines preflight:
repository `AGENTS.md`, documentation index, ADR baseline, P8 task book, and
the P8-W13 plan. Nothing here claims that any fault class has been exercised
or that the upstream boundaries deliver as assumed.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P8 task book → evidenced P0/P4 handoff
contracts → P8-W13 plan → this design → Coding Guidelines. In particular:

- ADR §12/§19 make Guest-caused faults VM-scoped by default (`MUST`: "Guest
  errors … must not panic the hypervisor"); P0-W14
  (`../../../p0/plans/p0-w14-panic-failure-classification.md`) fixes the
  top-level failure classes and the rule that guest-caused faults affect only
  the corresponding Guest. W13's taxonomy is an extension of those classes, not
  a new failure model.
- P4-W06 (`../../../p4/plans/p4-w06-fault-isolation-diagnostics.md`) owns exit
  categorization, Guest-vs-Hypervisor distinction, and the diagnostic context
  at the Guest boundary. W13 labels and extends coverage for Linux-facing
  classes; it does not redefine the P4 contract. A mismatch with the delivered
  P4 boundary is a `P4DependencyIssue`.
- P0-W12 (`../../../p0/plans/p0-w12-logging-diagnostic-baseline.md`) owns log
  levels, structured-trace semantics, minimum panic/crash information, and
  version identity. W13 requires context fields and a bounded recent-trace
  window within those semantics; the final trace encoding stays out of scope.
- The plan's out-of-scope list is binding: no full crash-dump system, no
  fault-handler APIs, no final trace encoding, no Guest-fault-to-panic path.

Classification:

- **Required:** the Linux-facing fault taxonomy with an explicit
  Hypervisor-invariant family and an unclassified route; the minimum actionable
  context set (VM/vCPU/PC/PSTATE/syndrome/address/exit reason/recent trace);
  the containment and routing rules per class; the console and regression
  observable mapping; diagnostic coverage evidence for the plan-listed fault
  classes (P8-V18).
- **Reserved:** crash-dump enrichment beyond the minimum context (trigger: an
  approved later-stage diagnostics design); per-class host-side tooling;
  management-ABI error reporting (P5+ lane).
- **Out of Scope:** crash-dump framework; fault-handler APIs; final trace
  encoding; EL2 panic policy (P0-W14/P1-W07 lane); W16 harness mechanics;
  W18's adversarial scenario design; any new machine ABI surface.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect W05–W10 and existing diagnostic boundaries | [ledger](#current-state-findings-and-goal-to-baseline-ledger); [classification](01-fault-classification.md) §1–§2 | W13-DV01 upstream review |
| Classify Guest-facing and Hypervisor-invariant failure classes | [classification](01-fault-classification.md) §3–§5 | W13-DV02 taxonomy review |
| Define minimum actionable context and recent-trace requirement | [context contracts](02-diagnostic-context-contracts.md) §2–§3 | W13-DV03 context review |
| Relate diagnostics to console, VM containment, automated regression observables | [classification](01-fault-classification.md) §6; [context contracts](02-diagnostic-context-contracts.md) §5 | W13-DV06 observable-mapping review |
| Review unclassified failures as blocks or design investigations | [classification](01-fault-classification.md) §7; [workflow](03-implementation-workflow-and-review.md) §1 | W13-DV07 block review |
| P8-V18 declared diagnostic coverage for listed fault classes | [workflow](03-implementation-workflow-and-review.md) §2 step 5, §3 | W13-DV04 (executed via W16) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p8-implementation-designs`):
documentation scaffold only — no implementation or verification records for
P1–P7, no diagnostics subsystem, no Linux, and no approved P8 machine facts.
Every fault source W13 must distinguish is planned, not observable. Each ledger
row states the missing foundation the plan outcome requires and who owns it.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Listed fault classes distinguishable (P8-V18) | No fault paths exist at all | Evidenced P4-W06 exit categorization and diagnostic context at the Guest boundary | W13's classes label events the P4 boundary must already capture | P4-W06; P4-W09 handoff (`../../../p4/plans/p4-w09-closeout-p5-handoff.md`) | P4-W06 isolation/diagnostic evidence |
| Guest vs Hypervisor-invariant distinction | Only planned (P0-W14) | Evidenced P0-W14 failure-class semantics and their handling rules | The taxonomy's top-level split inherits these semantics verbatim | P0-W14 (`../../../p0/plans/p0-w14-panic-failure-classification.md`) | P0-W14 review evidence; W13-DV02 |
| Minimum context: VM/vCPU/PC/PSTATE/syndrome/address/exit/trace | No context fields exist; P0-W12 semantics planned | Evidenced P4-W06 context fields plus P0-W12 version identity and trace semantics | "Actionable" is defined over delivered fields, not invented ones | P4-W06; P0-W12 (`../../../p0/plans/p0-w12-logging-diagnostic-baseline.md`) | W13-DV03 sufficiency checks on real events |
| W05–W10 fault sources classified | W05–W10 are plans | Their approved designs' declared fault surfaces (sysreg classification, PSCI violations, vGIC/timer contract violations, vCPU-state anomalies) | Each taxonomy class must trace to a declared upstream fault surface | Sibling designs `../p8-w05-linux-cpu-virtualization/README.md` … `../p8-w10-linux-smp-bringup/README.md` | W13-DV01 traceability review |
| Faults contained without global failure | No containment exists | Evidenced P7-W07 pause/stop/fault containment and W03/W06 shutdown path | Every VM-facing class needs a defined contained outcome | P7-W07 (`../../../p7/plans/p7-w07-pause-stop-fault.md`); W06 | W13-DV05 containment review |
| Classes exercised under Linux and observable in regression | No Linux, no harness | W09–W12 evidence plus W16 harness realization of the §5 observable mapping | P8-V18 evidence comes from executed scenarios, not the taxonomy alone | W09–W12 records; [W16](../p8-w16-automated-linux-regression/README.md) | W13-DV04 |

No row authorizes W13 to build a diagnostics subsystem; where an upstream
boundary delivers differently, [the workflow](03-implementation-workflow-and-review.md)
§1 failure boundary applies.

## Resolved design decisions and their authority

1. **Two-family taxonomy under P0-W14.** All classes split into the
   Guest-facing family (VM-facing, recoverable, contained per P7-W07) and the
   Hypervisor-invariant family (hypervisor-level per P0-W14), plus an
   `Unclassified` route that is always a block. Rationale: the plan demands the
   Hypervisor-invariant distinction, and P0-W14 already owns the top-level
   semantics; re-deriving them would create a competing failure model.
2. **W13 labels; P4-W06 captures.** The classification step consumes the
   exit/syndrome facts the P4-W06 boundary already produces and assigns a
   W13 class; W13 adds no capture mechanism or field. A class that needs an
   unproduced field is a diagnostic gap handed back to the owning package.
   Rationale: the task book's P4 boundary ("no Stage-2 or VM foundation
   redesign") and the plan's no-new-API exclusion.
3. **Minimum context is a closed, checked set.** The required context is the
   plan's list plus build/version identity and a bounded recent-trace window
   from evidenced trace fields; `diagnostic_context_sufficient` makes it
   checkable per event. Rationale: P8-V18's wording and P0-W12's
   version-identity alignment rule.
4. **Recent trace is bounded and class-scoped.** The window carries the last
   declared number of scheduler/interrupt/timer/exit events (P7-W09/P6-W13
   fields) preceding the fault — enough to correlate cause, not a dump.
   Rationale: P8-V18 lists "recent trace" while the plan excludes crash dumps.
5. **Console is a capture path, not a trust boundary.** Guest-produced text
   (Linux panic output) is captured as console bytes and recorded; it is never
   parsed into control flow or taken as an EL2-side factual claim. Rationale:
   Guest-untrusted rule (ADR-007) applied to diagnostics.
6. **Coverage is declared per class with its inducing scenario.** P8-V18
   evidence requires, for every Guest-facing class, at least one declared
   scenario that induces it and the expected contained outcome — the class list
   alone is not coverage. Rationale: the plan's "declared diagnostic coverage"
   wording.

## Work breakdown and loading order

1. Read [the fault classification](01-fault-classification.md) to understand
   the upstream baselines, the class taxonomy, per-class decision rules,
   containment routing, and the scenario-to-class coverage map.
2. Read [the diagnostic context contracts](02-diagnostic-context-contracts.md)
   when building or reviewing classification/context logic or regression
   assertions; it fixes the obligations the implementing designs realize.
3. Execute in the order given in [the implementation workflow](03-implementation-workflow-and-review.md):
   verify prerequisites, review taxonomy traceability, check context
   sufficiency on real events, execute the fault scenarios through W16, review
   containment, and carry blocks to closure.
4. Store actual commands, observations, and results in
   `../../verification/p8-w13-guest-fault-diagnostics-verification.md`, and
   record scenario parameters, changed artifacts, and deviations in
   `../p8-w13-guest-fault-diagnostics-record.md` only when implementation
   begins. Neither this design nor a written record may claim W13 complete.

## Explicitly excluded interfaces

W13 designs no crash-dump format, fault-handler API, panic handler, trace
encoding, new diagnostic field, new hypercall, or management-ABI error surface;
no EL2 control-flow change; no W16 harness internals; no machine ABI value; no
security-regression scenario (W18's lane). The only interfaces fixed here are
the classification and context contracts in
[02-diagnostic-context-contracts.md](02-diagnostic-context-contracts.md),
obligations on the implementing designs, not implementations. A diagnostic
need that cannot be met within the excluded set is a gap or `Architecture
Change Request` to record — never a new surface designed here.

## Downstream handoff

- **W16** ([design](../p8-w16-automated-linux-regression/README.md)) receives
  the taxonomy, per-class expected outcomes, sufficiency predicate, and
  console/regression observables as normative content for its expected-
  diagnostic assertions (P8-V21/V22 rows referencing F13-*).
- **W18** ([design](../p8-w18-security-isolation-regression/README.md)) receives
  the taxonomy and containment expectations for its isolation regression
  (P8-V24): every induced Guest fault must resolve to a VM-facing class with a
  contained outcome, and any `HypervisorInvariantViolation` is a regression
  failure, not an acceptable observation.
- **W20** ([design](../p8-w20-documentation-closure-handoff/README.md)) receives
  the diagnostic coverage table, limitation statements (no full crash-dump
  claim), and the unclassified-failure block list for closeout.
- **W11/W12** dependency: their failure routing and diagnostic sufficiency
  checks consume this taxonomy; mismatches found there are W13 review inputs.
- **P4/P0, via issue routing:** `P4DependencyIssue` records go to the P4-W09
  owner; semantic conflicts with P0-W12/W14 are recorded against those
  baselines' authorities.
