# P7-W10 Validation Guest Scheduler Suite — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The Validation-Guest scenario suite, pass/fail conditions, and
evidence locations for scheduler behavior required by
[P7-W10](../../plans/p7-w10-validation-guest-suite.md).  
**Owner/change context:** P7-W10 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P7-W10. It defines the maintained
Validation-Guest workload suite that exercises P7 scheduler behavior: the
single-vCPU regression duty (existing P4/P5/P6 scenarios remain stable under
scheduler-controlled entry), and the declared multi-vCPU workloads — CPU-bound,
periodic WFI, virtual timer, SGI/IRQ, shared-memory, and HVC-heavy — each with
topology, guest-observable markers, hypervisor-telemetry correlates, objective
pass/fail conditions, and explicit proves/does-not-prove boundaries. This is a
validation-design package: it defines evidence, never results. QEMU success
never proves hardware behavior, and no row below may be recorded as passed
anywhere but the verification record.

It deliberately does **not** design Linux boot, a Guest machine ABI, Guest
module/function internals beyond the workload contracts, or a permanent Guest
SMP policy.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md), then loads
only the supporting file needed for its assigned step:

| Assigned step | Load |
|---|---|
| The scenario catalog, pass/fail conditions, proof boundaries | [01-scenario-suite.md](01-scenario-suite.md) |
| Guest-asset extension, marker protocol, harness seams | [02-guest-test-asset-contract.md](02-guest-test-asset-contract.md) |
| Execute the ordered workflow | [03-implementation-workflow.md](03-implementation-workflow.md) |
| Plan or review validation and closure | [04-validation-and-handoff.md](04-validation-and-handoff.md) |

Before editing, the agent must also follow the Coding Guidelines preflight:
repository `AGENTS.md`, documentation index,
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P7 task book](../../task-book-v0.1.md), the
[P7-W10 plan](../../plans/p7-w10-validation-guest-suite.md), and
[P7-W01](../p7-w01-entry-contract-reconciliation/README.md)'s recorded input
boundary. This document is a proposed design; it contains no implementation or
validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P7 task book → P7-W10 plan → this
design → Coding Guidelines. Binding constraints:

- ADR-008/009: the Validation Guest is the first EL1 Guest and stays a
  minimal, controlled `no_std` asset; ADR-007 keeps it untrusted — the suite
  observes the hypervisor, it must never grant it authority.
- The task book reserves P8's territory: P8 owns Generic ARM64 machine/guest-
  CPU presentation, virtual PSCI/SGI, Linux boot, and the Linux-ready vGIC.
  Nothing here commits a machine ABI; scenario markers are test conventions,
  per the P4-W05 review rule ("distinguish P4 test conventions from a frozen
  Guest machine ABI").
- The Guest-asset upstream contracts —
  [P4-W05](../../../p4/plans/p4-w05-validation-guest.md) (asset and
  VG-001–VG-012),
  [P5-W07](../../../p5/plans/p5-w07-validation-guest-isolation-suite.md)
  (HVC/security scenarios, two-context isolation),
  [P6-W11](../../../p6/plans/p6-w11-validation-guest-interrupt-suite.md)
  (VG-TIMER-01–04, VG-IRQ-01–06, Host-SGI support) — are consumed and
  extended, never redefined.
- The suite must not depend on Hypervisor logs alone: P6-W11's rule
  ("independently observes ... rather than relying only on Hypervisor logs")
  is inherited — each scenario pairs Guest-observable markers with
  hypervisor-telemetry correlates, with a declared authority per scenario.

Classification:

- **Required:** the single-vCPU regression declaration (P7-V22) mapping
  inherited scenario sets onto scheduler-controlled entry; the P7 workload
  scenarios (CPU-bound, periodic WFI, virtual timer, SGI/IRQ, shared-memory,
  HVC-heavy) with topologies, markers, pass/fail conditions, and proof
  boundaries (P7-V23); the marker protocol and asset-extension contract;
  scenario-to-telemetry correlation rules; evidence locations.
- **Reserved:** performance or fairness measurements (P7-W13/W11 own those
  methods; W10 supplies workloads, not verdicts), guest-side self-scheduling
  observations beyond declared markers, and any automated-matrix execution
  contract (P7-W12 owns automation; W10 supplies scenario definitions).
- **Out of Scope:** Linux boot, Guest machine ABI/DTB, Guest production
  drivers, permanent Guest SMP policy, hypervisor module/function design,
  stress amplification (P7-W11 consumes these scenarios), and all P8+
  mechanisms including PSCI virtualization behavior.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect P4/P6 Validation Guest assets and W04–W07 behavior | [asset contracts](02-guest-test-asset-contract.md) §2, scenario prerequisites in [01-scenario-suite.md](01-scenario-suite.md) | [workflow](03-implementation-workflow.md) step 1 review |
| Define P7 workload scenarios and expected observable outcomes | [scenario suite](01-scenario-suite.md) §3–§4 | P7-V23 |
| Single-vCPU regression remains stable | [scenario suite](01-scenario-suite.md) §2 | P7-V22 |
| Integrate scenario needs with scheduler telemetry and failure boundaries | [telemetry correlation](02-guest-test-asset-contract.md) §4 | P7-V23; W09 contract |
| Review that guest workloads do not redefine platform contracts | [asset contract](02-guest-test-asset-contract.md) §5 review rules | [workflow](03-implementation-workflow.md) step 4 review |
| Plan regression evidence and hand off to stress automation | [validation and handoff](04-validation-and-handoff.md) | P7-V22–V23 evidence locations; handoff checklist |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p7-implementation-designs`): the repository is a P0 documentation
scaffold — no Cargo workspace, no Rust sources, no Validation Guest crate
(`guests/validation-aarch64/` holds only a `.gitkeep`), no implemented
crates. P0–P6 and P7-W04–W07 are planned, not implemented; everything the
suite consumes is an **assumed contract** cited by plan path, each with a
failure boundary in [02-guest-test-asset-contract.md](02-guest-test-asset-contract.md) §2.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| 1VM/1vCPU/1pCPU Validation Guest remains stable (P7-V22) | No guest asset exists yet; P4-W05 plans it | Assumed maintained P4 asset with VG-001–VG-012 markers running unchanged under scheduler-controlled entry (W02 seam) | Stability means the inherited scenarios pass without modification under the new admission path | P4-W05 (asset); P7-W02 (entry); P7-W10 (regression declaration) | P7-V22 regression evidence |
| Declared multi-vCPU workloads exercise P7 (P7-V23) | Absent | The scenario catalog of [01-scenario-suite.md](01-scenario-suite.md) implemented on the extended asset | Scenarios must exist and be runnable to exercise W04–W07 behavior | P7-W10 (catalog); asset owner (runtime) | P7-V23 per-scenario evidence |
| Workloads match W04–W07 behavior under test | W04–W07 designs proposed, not implemented | Assumed behavioral contracts: preemption (W04), M:N progress (W05), block/wake (W06), pause/stop/fault (W07) | A scenario can only prove behavior its named contract defines | W04–W07 designs (`../p7-w0X-<slug>/README.md`) | Per-scenario proof rows |
| Guest observation is independent of Host logs | Absent | Marker protocol on the P4 debug channel + telemetry correlates with declared authority | Host logs alone cannot witness Guest-visible behavior | P4-W05 (channel); P7-W10 (protocol) | Per-scenario marker evidence |
| Suite does not redefine platform contracts | N/A (nothing exists to check) | §5 review rules of the asset contract | Machine-ABI drift must be structurally preventable, not caught late | P7-W10 (rules); review | Review step 4 |
| Evidence handoff to W11 | Verification directory empty (`.gitkeep` only) | Validation matrix, scenario IDs, and record paths defined here | W11 consumes only declared, locatable scenarios | P7-W10 | `../../verification/p7-w10-validation-guest-suite-verification.md` |

No row invents an upstream mechanism. If P7-W01's reconciliation (P7-V01)
marks any assumed input missing or contradictory, the affected scenario is
blocked and recorded; it is not improvised here.

## Resolved design decisions and their authority

1. **Inherited scenarios are the regression suite.** P7-V22 is discharged by
   the existing P4/P5/P6 scenario sets (VG-001–VG-012; P5 HVC/security set;
   VG-TIMER/VG-IRQ sets) passing unchanged on 1VM/1vCPU/1pCPU under
   scheduler-controlled entry — not by new scenarios. Rationale: "remains
   stable" is only meaningful if the scenarios did not change. Authority:
   plan scope item 1; P4-W05/P5-W07/P6-W11 handoffs.
2. **A distinct `VG-SCHED` scenario namespace.** P7 adds `VG-SCHED-01..NN`,
   disjoint from the inherited families. Rationale: avoids collision with
   the P4/P5/P6 namespaces and makes W11 stress references unambiguous;
   mirrors P0-W13's discipline of declared namespaces for a test context.
   Authority: stage-local design freedom owned here; no machine-ABI content.
3. **Two-sided observation with declared authority.** Each scenario declares
   Guest markers (what the Guest observed) and telemetry correlates (what the
   scheduler recorded, per [P7-W09](../p7-w09-accounting-diagnostics/README.md)),
   and states which side is authoritative for its pass condition. Rationale:
   the Guest is untrusted; telemetry is authoritative for scheduler-internal
   claims, markers for Guest-visible claims; both must agree or the
   discrepancy is itself a finding. Authority: ADR-007; P6-W11's
   independence rule.
4. **Scenarios prove behavior, not performance.** Pass conditions are
   qualitative and bounded (progress, ordering, wake delivery, exclusion);
   no latency, fairness-ratio, or throughput conditions appear here — those
   are W11/W13 verdicts over W10 workloads. Rationale: plan out-of-scope
   (performance conclusions) and the W10 handoff ("W11 receives maintained
   scenarios"). Authority: plan scope/handoff.
5. **Scheduler-visible workloads are self-declaring.** Each workload module
   emits deterministic, sequence-numbered markers at defined points
   (iteration boundaries, WFI entry/exit, SGI receipt, HVC results) so an
   automated checker can verify ordering without timing assumptions.
   Rationale: timing-based pass conditions would import performance
   sensitivity and QEMU-vs-hardware variance into correctness evidence.
   Authority: stage-local design freedom owned here.
6. **No scenario requires trusting the Guest.** Every scenario's Guest side
   is written so a misbehaving Guest can only fail itself — no scenario
   passes because the hypervisor believed a Guest report. Rationale: ADR-007
   and the P5-W07 marker-safety review. Authority: ADR-007; stage-local rule
   owned here.
7. **QEMU is the only evidenced environment in P7.** All scenario evidence in
   P7 is QEMU-based (P1/P4 baselines); every scenario row records that
   hardware RK3566 behavior is not proven. Rationale: task book §2
   ("QEMU is a reference test environment, never the architecture
   definition") and P6-W13's limits discipline. Authority: task book; ADR-003.

## Work breakdown and loading order

1. Read [01-scenario-suite.md](01-scenario-suite.md) — the regression
   declaration and the `VG-SCHED` catalog with pass/fail and proof
   boundaries; identify which scenario you are implementing.
2. Read [02-guest-test-asset-contract.md](02-guest-test-asset-contract.md)
   for the asset-extension contract, marker protocol, harness seams, and
   platform-contract review rules.
3. Follow [03-implementation-workflow.md](03-implementation-workflow.md).
4. Record planned and actual evidence per
   [04-validation-and-handoff.md](04-validation-and-handoff.md).
   Implementation notes go to `../p7-w10-validation-guest-suite-record.md`
   (created when work starts); evidence to
   `../../verification/p7-w10-validation-guest-suite-verification.md`.
   Neither this design nor the record may claim W10 complete.

## Explicitly excluded interfaces

No Guest machine ABI, DTB contract, production Guest API, Linux artifact,
hypervisor module/function design, management surface, public API, or wire
format is designed or authorized by W10. Marker strings are test conventions
with declared stability ("stable within P7 for automation"), not machine
interfaces; their format changes are ordinary suite changes, not ABI events.
The scenario definitions constrain the Guest asset, not the hypervisor's
interfaces. Adding any excluded surface is a scope conflict stopped at
review.

## Downstream handoff

Per the [P7 plan index](../../plans/README.md), W11 consumes this package:

- **[P7-W11](../p7-w11-stress-invariants/README.md)** receives the maintained
  scenario inventory with IDs, markers, topologies, and proof boundaries as
  the base material for its stress amplification (P7-V24–V27) — including
  which scenarios tolerate parameter scaling (iteration counts, vCPU counts)
  and which have fixed shapes.
- **[P7-W12](../p7-w12-qemu-regression/README.md)** (indirect, via W11)
  receives determinate scenario expectations its automated QEMU matrix
  (P7-V28) can assert.
- **P8** (via P7-W14's handoff, not via this design) receives no machine-ABI
  commitment: scenario markers and the Guest asset remain test conventions
  (plan handoff: "P8 does not receive a machine-ABI commitment").
- **[P7-W13](../p7-w13-performance-baseline/README.md)** (indirect) may reuse
  workload shapes for its static-versus-scheduled comparison, drawing verdicts
  only from its own method and [P7-W09](../p7-w09-accounting-diagnostics/README.md)
  records.
