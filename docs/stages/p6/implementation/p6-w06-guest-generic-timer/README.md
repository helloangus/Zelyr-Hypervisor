# P6-W06 Guest Generic Timer Virtualization — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Scope:** The vCPU-owned Guest generic-timer behavior required by
[P6-W06](../../plans/p6-w06-guest-generic-timer.md).
**Owner/change context:** P6-W06 implementation handoff.
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P6-W06. It converts the bounded
work-package plan into a designed, code-bearing mechanism: per-vCPU Guest
timer state that survives Guest entry/exit, direct Guest programming of the
EL1 virtual timer under the P1 trap baseline, typed physical/virtual counter
domains, expiry evaluation and deferred delivery while a vCPU is absent
(through the P6-W07 virtual-interrupt core), and the W06-owned handling of
the Guest virtual timer's physical PPI. It deliberately does **not** design
the List-Register presentation of the timer event (P6-W08), maintenance
processing (P6-W09), Guest-visible masking/priority semantics (P6-W10),
scheduler wakeup or vCPU selection (P7), the Linux time ABI or a Guest DTB
(P8), or any crate/file layout.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md), completes
the Coding-Guidelines preflight (repository `AGENTS.md`, documentation index,
ADR baseline, [P6 task book](../../task-book-v0.1.md), and the
[P6-W06 plan](../../plans/p6-w06-guest-generic-timer.md)), and then loads
only the linked supporting file needed for its assigned step:

| Assigned material | Supporting file |
|---|---|
| Baseline findings, scope classification, resolved decisions | [01-scope-and-foundations.md](01-scope-and-foundations.md) |
| Logical modules, vCPU timer ownership, lifecycle, concurrency | [02-architecture-and-state.md](02-architecture-and-state.md) |
| vCPU timer-state contracts (typed domains, save/restore, entry/exit) | [03-code-contracts-vcpu-timer-state.md](03-code-contracts-vcpu-timer-state.md) |
| Expiry-evaluation and delivery contracts (deferred expiry, PPI handling, W07 interface) | [04-code-contracts-expiry-and-delivery.md](04-code-contracts-expiry-and-delivery.md) |
| Ordered implementation steps | [05-implementation-workflow.md](05-implementation-workflow.md) |
| Validation matrix, error/security/observability model, handoff checklist | [06-validation-and-handoff.md](06-validation-and-handoff.md) |

Nothing in this design is an implementation or completion claim. Evidence is
recorded only in
`../../verification/p6-w06-guest-generic-timer-verification.md` (created when
evidence exists), and factual implementation traceability only in
`../p6-w06-guest-generic-timer-record.md` (created when work starts).

## Authority, constraints, and scope classification

The governing order is Architecture ADR → P6 task book → P6-W06 plan → this
design → Coding Guidelines. Binding ADR constraints: the Guest is untrusted
and Guest-caused faults stay VM-local (ADR-007, section 19); the virtual
timer is vCPU-owned state, never pCPU-bound (section 4, section 7; task book
§1); mechanism/policy separation (ADR-004/005); typed identities and checked
arithmetic (section 13); P7 owns scheduling semantics and P8 owns the
Linux-visible machine model, so neither may be pre-empted here. The task
book classifies "documented Guest timer behavior" as an Implementation
Choice this design must make and validate without introducing scheduler
policy ([01](01-scope-and-foundations.md) §4, decision D4).

Classification summary:

- **Required:** per-vCPU Guest timer state (saved virtual-timer registers,
  shadow state); Guest-visible virtual-timer access under the P1 baseline;
  entry/exit save/restore that leaves no timer state bound to a pCPU;
  expiry evaluation with honored Guest masking; deferred delivery of an
  expiry that occurred while the vCPU was absent, raised through the W07
  injection contract; W06-owned handling of the Guest virtual-timer physical
  PPI with a single-outstanding rule; Guest-caused error handling for
  trapped timer access; per-vCPU diagnostics and telemetry hooks.
- **Reserved:** timer-driven wakeups of an absent vCPU before its next entry
  (P7); pause/resume timer behavior (no evidenced facility at P6; task book
  §8); counter offset/scaling beyond `CNTVOFF_EL2 = 0`; any host-timer
  coupling for Guest deadlines beyond what W06 uses from W05 for its own
  assist paths.
- **Out of Scope:** List-Register programming and pressure (W08);
  maintenance processing (W09); Guest masking/priority semantics as
  Guest-visible contracts (W10); Validation Guest scenarios (W11);
  robustness/storm evidence (W12); telemetry collection and the latency
  baseline (W13); wall clock, migration/snapshot time conversion, Linux
  timer ABI; any crate, module path, or public API beyond the contracts
  stated here.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Independent vCPU timer state; isolation from Host timer state | [foundations](01-scope-and-foundations.md) §3 ledger; [architecture](02-architecture-and-state.md) §3; [state contracts](03-code-contracts-vcpu-timer-state.md) | W06-DV01, DV06 → P6-V09, P6-V19 |
| Guest timer programming/masking/expiry | [foundations](01-scope-and-foundations.md) §4 (D1, D7); [expiry contracts](04-code-contracts-expiry-and-delivery.md) §2 | W06-DV02 → P6-V09 |
| Entry/exit preservation | [state contracts](03-code-contracts-vcpu-timer-state.md) §3 | W06-DV03 → P6-V10 |
| Eventual delivery when absent (deferred expiry) | [foundations](01-scope-and-foundations.md) §4 (D4); [expiry contracts](04-code-contracts-expiry-and-delivery.md) §3 | W06-DV04 → P6-V09/P6-V10 |
| No state bound to a pCPU; optional evidenced pause/resume | [architecture](02-architecture-and-state.md) §3–§4; [foundations](01-scope-and-foundations.md) §4 (D3, D8) | W06-DV06; pause/resume recorded Reserved |
| Guest-caused error handling (trapped access) | [expiry contracts](04-code-contracts-expiry-and-delivery.md) §5 | W06-DV05 |
| Isolation, untrusted Guest controls, telemetry review | [validation](06-validation-and-handoff.md); [architecture](02-architecture-and-state.md) §6–§7 | W06-DV06, DV07 |
| Factual status and handoff to W10–W13, P7/P8 | [handoff](06-validation-and-handoff.md) §3 | W06-DV08 |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p6-implementation-designs`): the repository is a P0 documentation
scaffold — no Cargo workspace, no Rust sources, no implemented P1–P5
mechanisms; only P0 has one verification record and one proposed design.
Every P6-W06 prerequisite (W05 event basis, P4 Guest entry/exit, P5
authorization/error boundary) is an **assumed contract** with an explicit
entry check and failure boundary, recorded in the ledger in
[01-scope-and-foundations.md](01-scope-and-foundations.md) §3 and the
contract table in §5. No ledger row invents a crate, target, register
behavior, or Guest DTB layout.

## Resolved design decisions and their authority

The numbered decisions (D1–D9), their rationale, and authority basis are in
[01-scope-and-foundations.md](01-scope-and-foundations.md) §4. In brief:
(D1) the Guest sees the EL1 virtual timer, programmed directly at EL1 without
traps, while EL1 physical-timer access traps to a Guest fault; (D2)
`CNTVOFF_EL2` is zero, written per entry, with offset/scaling Reserved;
(D3) the vCPU record is the sole timer-state authority and the hardware
registers are a per-run cache, so no timer state is bound to a pCPU;
(D4) deferred expiry is evaluated at exit and raised through the W07
injection contract, so delivery is eventual (no scheduler wakeup, which
stays P7); (D5) the Guest timer event identity is the architecture-assigned
EL1 virtual-timer PPI INTID, intake-validated, with identity INTID mapping
in P6; (D6) W06 owns conversion of the Guest virtual-timer physical PPI into
the W07 software event under a single-outstanding rule with deferred physical
deactivation, aligned with the W03 lifecycle; (D7) Guest masking is honored
in every evaluation (no injection while masked); (D8) pause/resume timer
behavior is Reserved; (D9) physical and virtual counter domains are distinct
typed values with explicit conversion.

## Work breakdown and loading order

1. Load this README and the Coding Guidelines; complete the coding preflight.
2. Load [01-scope-and-foundations.md](01-scope-and-foundations.md) for the
   prerequisite contracts, failure boundaries, scope classification, and
   decisions D1–D9.
3. Load [02-architecture-and-state.md](02-architecture-and-state.md) for the
   logical modules, the vCPU-owned state, the delivery state machine, and
   the concurrency rules.
4. Implement in the order given in
   [05-implementation-workflow.md](05-implementation-workflow.md), loading
   [03-code-contracts-vcpu-timer-state.md](03-code-contracts-vcpu-timer-state.md)
   for workflow steps 3–4 and
   [04-code-contracts-expiry-and-delivery.md](04-code-contracts-expiry-and-delivery.md)
   for workflow steps 5–7.
5. Close with [06-validation-and-handoff.md](06-validation-and-handoff.md):
   run the validation matrix, record run/not-run evidence in the
   verification record path above, and complete the handoff checklist.

## Explicitly excluded interfaces

W06 authorizes no Guest-visible MMIO, no vGIC Distributor/Redistributor
model, no Guest DTB or machine-layout decision, no scheduler or wakeup API,
and no LR/maintenance surface. Its only named collaborators are: the W05
typed time basis, the W07 injection/query contracts, the W03 classified-PPI
dispatch, the W09 completion report hook, and the P4 vCPU entry/exit
boundary it extends. Crate names, module paths, and file trees are not
designed here; the ADR layering rule holds (system-register sequences live
only in the Arch-domain module).

## Downstream handoff

Per the [plan index](../../../p6/plans/README.md) consumer map:

- **P6-W10** (`../p6-w10-interrupt-semantics/README.md`) receives the defined
  timer behavior — masking honor, expiry condition semantics, deferred
  delivery — as input to the bounded masking/priority/timer-plus-vIRQ
  semantics.
- **P6-W11** (`../p6-w11-validation-guest-interrupt-suite/README.md`)
  receives the Guest-observable timer contracts (program/read/expire, exit
  preservation, deferred delivery) that its VG-TIMER scenarios exercise.
- **P6-W12** (`../p6-w12-fault-isolation-robustness/README.md`) receives the
  Guest-caused error boundary for trapped timer access and the isolation
  guarantees to stress.
- **P6-W13** (`../p6-w13-telemetry-regression-handoff/README.md`) receives
  W06's diagnostics and deferred-delivery counters for the factual records.
- **P7** receives only proven vCPU-owned timer semantics and the reserved
  wakeup extension point — no runnable-state or preemption semantics.
- **P8** receives only proven lower-level Guest-timer facts and known
  limitations; the Linux-visible timer/machine ABI remains P8-owned.

No consumer may treat the W06 register sequences as a hardware contract
beyond the declared QEMU reference environment, or infer a machine ABI from
them.
