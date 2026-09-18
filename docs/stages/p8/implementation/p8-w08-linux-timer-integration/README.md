# P8-W08 Linux Timer Integration — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The Linux-facing time and virtual-timer compatibility boundary —
Guest counter/time source semantics, per-vCPU timer register state, expiry
and deferred delivery, preemption continuity, and WFI wakeup — per
[P8-W08](../../plans/p8-w08-linux-timer-integration.md).  
**Owner/change context:** P8-W08 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P8-W08. P6 already owns
vCPU-owned Guest timer behavior ([P6-W06](../../../p6/plans/p6-w06-guest-generic-timer.md))
and the EL2 event-timer substrate ([P6-W05](../../../p6/plans/p6-w05-el2-generic-timer.md));
P7 owns blocking/wakeup ([P7-W06](../../../p7/plans/p7-w06-block-wakeup.md)) and
context-switch hooks. This design **consumes** those contracts and defines
only the Linux-facing delta: which time source the Guest sees, the register
semantics Linux programs, the invariant set (monotonicity, target-vCPU
delivery, preemption continuity), and the WFI/wakeup integration Linux's
idle path depends on. It deliberately does not choose a counter frequency
value, redesign timer state storage, or change scheduler policy.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

| Assigned work | Load |
|---|---|
| Modules, time-source model, state ownership, invariants, concurrency | [01 Architecture and state](01-architecture-and-state.md) |
| Timer register (CTL/CVAL/TVAL) and counter access contracts | [02 Timer register contracts](02-code-contracts-timer-regs.md) |
| Expiry, deferred delivery, WFI/wakeup, preemption contracts | [03 Expiry and wakeup contracts](03-code-contracts-expiry-wakeup.md) |
| Implement in dependency order | [04 Implementation workflow](04-implementation-workflow.md) |
| Validate and hand off | [05 Validation and handoff](05-validation-and-handoff.md) |

Before editing, the agent must also follow the Coding Guidelines preflight,
including the repository `AGENTS.md`, documentation index, ADR baseline, P8
task book, P8-W08 plan, and the P6/P7 contracts cited above. This document
proposes design only; it contains no implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P8 task book → the P8-W02
machine-contract gate → P8-W08 plan → this design → Coding Guidelines. In
particular:

- ADR §7 assigns virtual-timer state to the vCPU and links scheduler
  block/wakeup with timer deadlines; P6-W06 owns that state and P7-W06 owns
  the wakeup semantics. W08 adds no new state owner.
- ADR-007/§19 make the Guest untrusted: all timer register values are
  validated (width, reserved bits, checked arithmetic on TVAL/CVAL
  conversions), and a Guest that programs absurd deadlines gets defined
  behavior, never Host degradation.
- Task book §8 routes timer details through Specification Investigation in
  detailed design using AArch64/Linux sources, "without choosing values": the
  counter frequency and the frozen timer-class selection are
  [W02](../../plans/p8-w02-machine-contract-governance.md)-gated; monotonicity
  and delivery semantics are owned here.
- The plan excludes timer programming mechanics (P6's), frequency values,
  scheduling algorithm, hardware timing guarantees, and real-hardware
  conclusions.

Classification:

- **Required:** Guest time-source contract (counter class, per-VM offset,
  uniform read-only frequency); per-vCPU CTL/CVAL/TVAL semantics for the
  exposed timer class; enable/mask/status behavior; expiry → timer-PPI
  delivery through [W07](../p8-w07-linux-vgicv3/README.md); deferred delivery
  when the target vCPU is not running; WFI blocking with timer/wakeup-source
  integration; preemption/migration continuity; monotonicity invariants;
  diagnostics/telemetry.
- **Reserved:** physical-counter exposure to the Guest (trigger: an approved
  classification change in W05 requiring it); the second (physical) timer
  class as a Guest-programmable timer (trigger: a declared scenario);
  timer-state migration/snapshot conversion (P16/P17 work); frequency
  migration across versions.
- **Out of Scope:** EL2's own event-timer internals (P6-W05); scheduler
  policy and placement (P7); the GIC delivery path (W07 consumes it);
  wall-clock/RTC semantics (not in the v1 machine); Linux-side
  configuration (W15/W03); real-hardware timing behavior (P15).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Counter/time semantics Linux and the machine contract must expose | [02 Timer register contracts](02-code-contracts-timer-regs.md) §2, decision D1/D2 | P8-V11 counter/time rows |
| Timer IRQ delivery | [03 Expiry and wakeup contracts](03-code-contracts-expiry-wakeup.md) §2 (via [W07 §3](../p8-w07-linux-vgicv3/03-code-contracts-interrupt-flow.md)) | P8-V11 timer-IRQ rows |
| Monotonic time | [01 Architecture and state](01-architecture-and-state.md) §4 invariants, [02](02-code-contracts-timer-regs.md) §2 | P8-V11 monotonicity rows |
| Tick/tickless operation, sleep/timeout | [03 Expiry and wakeup contracts](03-code-contracts-expiry-wakeup.md) §3–§4 | P8-V11 sleep rows |
| Preemption and continuity | [03 Expiry and wakeup contracts](03-code-contracts-expiry-wakeup.md) §5 | P8-V11 preemption rows |
| Target-vCPU delivery | [03 Expiry and wakeup contracts](03-code-contracts-expiry-wakeup.md) §2, [01 §4](01-architecture-and-state.md) | P8-V11 target-vCPU rows |
| WFI wakeup | [03 Expiry and wakeup contracts](03-code-contracts-expiry-wakeup.md) §4 | P8-V11 WFI rows |
| Normal, high-frequency, multi-vCPU scenario scope | [05 Validation and handoff](05-validation-and-handoff.md) §1 matrix, decision D7 | P8-V11 matrix |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, worktree branch
`docs/p8-implementation-designs`): documentation scaffold only — no Cargo
workspace, no Rust sources, no P6/P7 implementation records. Every
prerequisite below is a planned contract consumed as an assumption with a
failure boundary.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Linux reads a stable, monotonic Guest time source (P8-V11) | No Guest timer code or contract exists | The time-source contract of [02 §2](02-code-contracts-timer-regs.md) (counter class, offset, frequency consumption) | Linux's clocksource is the architected counter; without a declared source, "time works" is untestable | P8-W08 (this design); value freeze via W02 | P8-V11 counter/time rows (future `../../verification/p8-w08-linux-timer-integration-verification.md`) |
| Guest timer programming works and expires | P6-W06 Guest-timer behavior is planned, not implemented | Register-semantics contracts of [02 §3](02-code-contracts-timer-regs.md) consuming the P6-W06 state owner | Re-owning timer state would fork the P6 contract | P6-W06 (cite, don't redesign) | P8-V11 timer-IRQ rows |
| Expiry reaches the right vCPU even when it is not running | P6-W06 deferred delivery + P7-W06 wakeup are planned | Delivery contracts of [03 §2–§3](03-code-contracts-expiry-wakeup.md) expressed as P6-W07 injection + P7-W06 wakeup | A deadline that silently lapses breaks Linux sleep/timeout paths | P6-W06/W07, P7-W06 seams | P8-V11 deferred/sleep rows |
| WFI wakes on deadline or pending event | P7-W06 block/wakeup planned | The WFI integration contract of [03 §4](03-code-contracts-expiry-wakeup.md) | Linux idle loops depend on it; a missed wakeup is a hang | P7-W06 seam | P8-V11 WFI rows |
| Timer state survives preemption/migration | P7-W04 context switch planned | Continuity contract of [03 §5](03-code-contracts-expiry-wakeup.md) | State bound to a pCPU would corrupt time after migration (P6-W06's explicit non-goal) | P6-W06 ownership; P7-W04 hooks | P8-V11 preemption rows |
| Machine-gated values (frequency, timer class) | Machine ABI unfrozen (ADR §18) | Placeholder tokens (`<CNTFRQ-VALUE>`, exposed-timer-class) owned by the W02 gate | Values frozen outside the gate violate task-book routing | W02 gate owns values; this design owns semantics | P8-V02/V03 review rows |
| Malformed timer programming contained (P8-V24) | No validation code; W05 owns classification | Validation rules in [02 §3](02-code-contracts-timer-regs.md) (reserved bits, checked arithmetic) | Unchecked 64-bit deadline arithmetic is a classic overflow source | W05 categories; this design applies them | W18 abuse scenarios |

No row above selects a final machine value inside this design alone; the
outstanding decision is the W02 gate (frequency value, exposed timer class),
placed by the task book before implementation.

## Resolved design decisions and their authority

1. **Guest time source: the virtual counter with a per-VM offset.** The
   Guest reads the virtual counter (CNTVCT-class access) computed as
   physical-counter minus a per-VM offset fixed at VM creation; the physical
   counter is Hidden-classified for the Guest per
   [W05](../../plans/p8-w05-linux-cpu-virtualization.md) (Reserved: exposure
   change). Rationale: a per-VM offset is the minimal mechanism that gives
   every vCPU of a VM an identical, monotonic, Host-independent time origin,
   matches ADR §7's vCPU/vm time boundary, and keeps Host uptime
   unobservable.
2. **Guest-programmable timer class: the virtual timer (CNTV_*_EL0/EL1
   semantics).** One timer class is exposed in P8; the physical timer class
   is Hidden per W05 (Reserved: second class). Rationale: Linux defaults to
   the virtual timer when presented it, one class keeps the vCPU-state model
   minimal, and P6-W06 owns the state either way. The class choice is frozen
   with W02 (it is Guest-visible via DTB timer node facts owned by W04).
3. **Frequency: uniform, read-only, machine-gated.** All vCPUs of all VMs
   read the same `<CNTFRQ-VALUE>` from approved machine facts. The value is
   W02's; uniformity and read-only enforcement are this design's invariants.
4. **Deadline model: absolute compare plus enable/mask.** The Guest-visible
   assertion condition is `CTL.ENABLE ∧ ¬CTL.IMASK ∧ CVAL ≤ counter`
   (IHI 0069/AArch64 timer semantics), evaluated against the virtual
   counter; TVAL is defined as a write-form (sets CVAL = counter + TVAL) and
   read-form (counter − CVAL, saturated) — not stored state. Rationale:
   avoids a second source of truth and the wrap hazards of stored TVAL.
5. **Expiry delivery is bridged, not re-owned.** Expiry produces a timer-PPI
   injection through [W07 §3](../p8-w07-linux-vgicv3/03-code-contracts-interrupt-flow.md)
   onto the P6-W07 lifecycle, and a wakeup request through P7-W06 when the
   target vCPU is blocked. Level semantics: the PPI stays asserted (per
   enable/mask/compare) until the Guest masks or disables — re-injection
   after EOI follows the level rule, via W07's config carrying.
6. **One physical re-arm per running vCPU, state-owned by the vCPU.** While
   a vCPU runs, its earliest enabled deadline is armed on the P6-W05 EL2
   event timer; on preemption the arm is cancelled and the deadline
   bookkeeping remains with the vCPU ([03 §5](03-code-contracts-expiry-wakeup.md)).
   This is the P6-W06 ownership boundary expressed as integration.
7. **Stress scope bounded and declared:** high-frequency reprogramming and
   multi-vCPU matrices ([05 §1](05-validation-and-handoff.md)) have
   fixture-configured rates (W16); the pass condition is semantic fidelity
   (monotonicity, delivery to the right vCPU, no lost wakeup), not
   performance.
8. **Naming and placement:** logical names only; placement follows the
   approved workspace decision; no board/SoC/QEMU constants (ADR-043).

## Work breakdown and loading order

1. Read this README, then [01 Architecture and state](01-architecture-and-state.md)
   for the time-source model, invariants, and concurrency rules.
2. Implement in the order given by
   [04 Implementation workflow](04-implementation-workflow.md), loading
   [02](02-code-contracts-timer-regs.md) for register/counter work and
   [03](03-code-contracts-expiry-wakeup.md) for expiry/wakeup/preemption
   work.
3. Record implementation decisions in
   `../p8-w08-linux-timer-integration-record.md` when implementation begins
   and evidence in
   `../../verification/p8-w08-linux-timer-integration-verification.md` when
   scenarios run. Neither file may claim W08 complete; P8-V11 is the proof
   surface.

## Explicitly excluded interfaces

No scheduler policy hook, P6 timer-state representation, EL2 event-timer
internals, GIC delivery mechanism, wall-clock source, DTB timer node bytes,
or Linux-side change is designed or authorized here. The Guest-visible
surface is the machine-gated counter/timer register semantics this design
defines; anything beyond it is a scope conflict to stop at review (at minimum
W02 for values, W04 for DTB facts, P6/P7 for owned mechanisms).

## Downstream handoff

- **W09 (console + single-vCPU Linux)** receives the time-source and
  deadline contracts its boot milestones and sleep-based tests depend on
  ([02 §2](02-code-contracts-timer-regs.md), [03 §3–§4](03-code-contracts-expiry-wakeup.md)).
- **W10 (Linux SMP bring-up)** receives the per-vCPU timer continuity
  contract, the multi-vCPU delivery semantics, and the WFI/idle integration
  that per-CPU idle loops use ([03 §4–§5](03-code-contracts-expiry-wakeup.md)).
- **W11 (scheduler integration)** receives the preemption-continuity and
  wakeup contracts it must preserve across M:N scheduling
  ([03 §5](03-code-contracts-expiry-wakeup.md)).
- **W16 (automated regression)** receives the P8-V11 scenario matrix and
  expected markers from [05 §1](05-validation-and-handoff.md).
- **W17 (performance baseline)** receives the timer telemetry events and the
  latency-measurement method inherited from P6-W13.
- **W18 (security isolation regression)** receives the malformed-programming
  and deadline-abuse scenarios ([02 §4](02-code-contracts-timer-regs.md)).
