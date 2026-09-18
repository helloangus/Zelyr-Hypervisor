# P6-W06 Scope, Foundations, and Resolved Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Parent:** [P6-W06 design entry](README.md).

## 1. Governing constraints

1. **Architecture ADR baseline**
   ([../../../../adr/adr-000-architecture-baseline-v0.1.md](../../../../adr/adr-000-architecture-baseline-v0.1.md)):
   Guest-untrusted boundaries and VM-local fault containment (ADR-007,
   section 19); vCPU-owned virtual timer (section 4 object model, section 7);
   mechanism/policy separation (ADR-004/005); validation Guest before Linux
   (ADR-009/010); typed identities and checked arithmetic (section 13);
   layered telemetry (ADR-048).
2. **P6 task book** ([../../task-book-v0.1.md](../../task-book-v0.1.md)):
   P6-F owned by W06, validated by P6-V09/P6-V10/P6-V19; §1 prohibits
   binding vCPU timer state to a pCPU and requires eventual delivery when an
   expiry occurs while the vCPU is absent; §8 classifies documented Guest
   timer pause/resume behavior as an Implementation Choice (choose one
   bounded behavior; no scheduler policy); §2 makes P4/P5 handoffs entry
   conditions.
3. **P6-W06 plan**
   ([../../plans/p6-w06-guest-generic-timer.md](../../plans/p6-w06-guest-generic-timer.md)):
   goal, scope (independent vCPU timer state, Guest
   programming/masking/expiry, entry/exit preservation, eventual delivery
   when absent, optional evidenced pause/resume, Host-timer isolation), and
   acceptance. The plan names no registers, types, algorithms, or delivery
   mechanisms; this design supplies them.
4. **Coding Guidelines**
   ([../../../../development/coding-guidelines.md](../../../../development/coding-guidelines.md)):
   typed counter quantities with checked arithmetic; guest-controlled values
   untrusted and validated; register sequences with required barriers; no
   scheduler policy smuggled into mechanism.

## 2. Current-state findings

Observable facts (worktree `docs/p6-implementation-designs`, audited
2026-09-18): P0 documentation scaffold only — no workspace, no sources, no
implemented P1–P5 mechanisms, no W05 implementation, no P4 Guest entry/exit
implementation, no P5 authorization implementation. P6-W06 therefore designs
against the plans' stated handoffs (§5) with an entry review at workflow
step 1; sibling P6 designs are being prepared in parallel on this branch and
are referenced by path, not by assumed content.

## 3. Goal-to-baseline ledger

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| P6-V09: "the Guest reads/programs/enables a timer and observes a handled virtual-timer event" | No Guest execution, timer, or vIRQ mechanism exists | Assumed: P4 Guest EL1 entry/exit path (`../../../p4/plans/p4-w04-vcpu-entry-exit.md`, closeout `../../../p4/plans/p4-w09-closeout-p5-handoff.md`); W06-owned vCPU timer state + delivery into W07; W07/W08 presentation | Guest observation requires a running Guest (P4) and an event path (W07→W08); the vCPU-owned state itself is W06's core deliverable | P4 (assumed); W06 (this design); W07/W08 for presentation | P4 evidence + W06-DV02/DV04 |
| P6-V10: "declared Guest exits preserve timer behavior without loss or Host-state contamination" | Nothing exists | W06-owned save/restore and exit/entry evaluation (D3); P4 controlled-exit contract | Preservation is a property of the transition design, not of the timer alone | W06 (this design); P4 exit contract | W06-DV03 |
| Deferred expiry "eventual delivery when an expiry occurs while the vCPU is absent" (task book §1) | Nothing exists | W06-owned exit-time expiry evaluation raising a W07 pending event (D4) | Absent-vCPU expiry must be captured somewhere durable; the vCPU's W07 queue is the designated holder; wakeup policy stays P7 | W06 (this design); W07 queue contract | W06-DV04 |
| P6-V19: "each exercised vCPU has independent timer state; unavailable multi-vCPU prerequisites are recorded as a stage block" | P4 multi-vCPU Guest execution unproven | Per-vCPU state model (structurally isolated); conditional multi-vCPU scenario gated on evidenced upstream capability | Isolation must be structural, then evidenced when the prerequisite exists | W06 (structure); P4/P3 (prerequisite evidence) | W06-DV06, or recorded block |
| Guest timer programming/masking | P1 trap baseline unimplemented | Assumed: P1-W04 `CNTHCTL_EL2` baseline leaving EL1 virtual-timer access untrapped and EL1 physical-timer access trapped (`../../../p1/plans/p1-w04-el2-architectural-state-baseline.md`) | D1's no-trap Guest path and the trapped-access fault path both depend on the delivered baseline | P1 (assumed) | entry check at W06 step 1 |
| Host event basis for deferred-expiry assists | W05 planned only | Assumed: W05 typed domains and expiry record (`../p6-w05-el2-generic-timer/README.md`) | W06 conversions and any host-timer assist consume the W05 contracts | W05 | W05 evidence at step 1 |
| vIRQ delivery for the timer event | W07 planned only | Assumed: W07 injection/dedupe/completion contracts (`../p6-w07-virtual-interrupt-core/README.md`) | D4/D6 delivery raises software events through W07 — W07 is the pending-state authority | W07 | W07 design/implementation at step 1 |
| Guest-caused error boundary (trapped timer access) | P5 boundary planned only | Assumed: P5 Guest-fault/error classification and HVC boundary (`../../../p5/plans/p5-w02-hypercall-abi-error-boundary.md`, closeout `../../../p5/plans/p5-w10-closeout-p6-handoff.md`) | trapped access must produce a controlled, VM-local fault, not a Host event | P5 (assumed); W06 trap policy | W06-DV05 |
| Completion of the presented timer event | W09 planned only | Assumed: W09 completion report hook to W06 (deferred physical deactivation, D6) | the single-outstanding rule releases on Guest-visible completion | W09/W03 (assumed contracts) | W06-DV04/DV06 |

No row requires inventing a crate, dependency, machine ABI, or scheduler
policy; no decision blocker is outstanding for this design.

## 4. Resolved design decisions and their authority

**D1 — Guest-visible timer: EL1 virtual timer, programmed directly at EL1.**
The Guest reads/writes `CNTV_CTL_EL0`/`CNTV_CVAL_EL0` (and reads
`CNTVCT_EL0`) without traps, under the P1 `CNTHCTL_EL2` baseline. EL1
physical-timer register access is trapped and answered with a Guest fault
(one diagnostic counter, one fault per access; no emulation in P6). Rationale:
the virtual timer gives the Guest a counter that Host activity does not
disturb, is the timer every later stage (Linux) expects, and requires no
per-access emulation on the hot path. The choice is stage-local design
freedom under the task book's Guest-timer-behavior Implementation Choice,
constrained by the P1 baseline; if the delivered baseline differs (for
example physical-timer access left untrapped), the baseline is a P1
contract — W06 records a boundary conflict rather than re-configuring traps
locally. Authority: task book §8; P1-W04 assumed contract.

**D2 — `CNTVOFF_EL2` fixed to zero, written per entry.** W06 writes the
virtual offset once per Guest entry as part of the entry sequence and never
derives Guest time from a nonzero offset. Offset and scaling are Reserved
(out of plan scope: advanced scaling). Rationale: a nonzero per-VM offset
introduces conversion policy that only a machine-model owner (P8) should
set; zero keeps the two counter domains related by a single explicit
constant. Authority: P6-W06 plan out-of-scope list; task book §8.

**D3 — vCPU record is the sole timer-state authority; hardware registers are
a per-run cache.** On exit, W06 saves `CNTV_CTL`/`CNTV_CVAL` into the vCPU
record and disables the hardware timer; on entry it restores them. No W06
state lives in per-pCPU storage across runs, satisfying the task book
prohibition on binding vCPU timer state to a pCPU. Rationale: makes the
state structurally portable (P6-V19) and keeps the absent-vCPU timer quiet
(no stray physical PPIs while absent). Authority: task book §1; ADR section 4.

**D4 — Deferred expiry evaluated at exit; delivery is eventual at/before the
next entry.** At exit, after saving state, W06 evaluates the architectural
condition (expired, unmasked); if met, it raises the timer event through the
W07 injection contract so it is pending before the next entry, and W08
presents it at entry. If the deadline is still in the future at exit, the
restored hardware raises it naturally during the next run. W06 never wakes
an absent vCPU and never arms host hardware for Guest deadlines in P6;
timer-driven wake of absent vCPUs is Reserved for P7. Delivery latency while
absent is therefore bounded by the next entry — a documented limitation, not
a defect. Chosen per the task book §8 rule (one bounded behavior, no
scheduler policy). Authority: task book §1/§8; P7 boundary.

**D5 — Event identity and INTID mapping.** The Guest timer event is injected
as vINTID equal to the architecture-assigned EL1 virtual-timer PPI INTID
(PPI 14 → INTID 30), taken from intake data reconciled with the W01
capability review and validated against the P4 Validation Guest's interrupt
layout (its DTB), never hardcoded in Core (layering, ADR-041–045). P6 uses
identity INTID mapping (Guest-visible INTID == physical INTID number)
because P6 has no Guest-visible vGIC MMIO model; any remapping is P8's
machine-model decision. Authority: task book §8 (Implementation Choice);
P8 boundary.

**D6 — W06-owned physical-PPI conversion with a single-outstanding rule.**
The EL1 virtual timer signals through a physical PPI (INTID 30) that arrives
at EL2 via the W03 lifecycle. W06 owns its conversion policy: while the
vCPU is loaded and the condition fires, W06 converts the physical PPI into
the W07 software event (vINTID 30) and holds the physical interrupt
outstanding (acknowledged but not deactivated) until the Guest visibly
completes the event — reported by W09 — or until exit processing releases
it. Holding the interrupt outstanding prevents level-refire storms while the
software event is in flight, and releasing it after Guest completion yields
architecturally correct level semantics (a Guest that handles the event
without clearing the condition is re-interrupted). This requires the W03
lifecycle to express acknowledge-without-deactivate (split completion) and
W09 to report Guest completion of vINTID 30 to W06. Both are assumed
contracts with an escalation path: if the delivered W03/W09 designs cannot
express the split, the conflict is raised (Architecture Change Request
against the physical-IRQ lifecycle) — W06 will not busy-poll, drop
interrupts, or disable the PPI to fake the semantics. Authority: task book
§8 (state representation and locking are design choices); W03/W09 assumed
contracts ([05](05-implementation-workflow.md) step 1).

**D7 — Guest masking is honored in every evaluation.** The architectural
condition is "expired and unmasked" (`CTL.ISTATUS` semantics). W06 never
injects while the Guest masked the timer, and does not treat masked expiry
as loss: at exit with masked-expired state, nothing is queued; the restored
registers reproduce the masked condition at the next run, where a Guest
unmask raises it naturally. Masking never implies completion (aligned with
W10's rule). Authority: architecture semantics; W10 boundary; task book
P6-V13 intent.

**D8 — Pause/resume timer behavior: Reserved.** No evidenced pause/resume
facility exists upstream at P6 (P4 closeout scope has none), so per task
book §8 the only honest bounded behavior is to record the gap: W06 defines
no pause/resume behavior now and marks it Reserved with the trigger "an
evidenced upstream pause/resume facility". Validation rows that would need
it are recorded blocked, not passed. Authority: task book §8; P4 closeout
plan.

**D9 — Distinct typed counter domains.** Physical-counter values
(`CNTPCT_EL0`, W05 domain) and virtual-counter values (`CNTVCT_EL0`, Guest
domain) are distinct newtypes; every conversion is an explicit function
carrying the `CNTVOFF` value (zero in P6, D2). Naked integer equality of the
two domains is prohibited even though they are numerically equal at offset
zero. Rationale: Coding Guidelines typed-quantity rules; prepares P8 for
offset behavior without re-plumbing. Authority: Coding Guidelines; ADR
section 13.

## 5. Assumed upstream contracts and failure boundaries

| Contract | Source (plan path) | W06 relies on | If delivered differently |
|---|---|---|---|
| Guest EL1 entry/exit with controlled recovery and controlled stop | `../../../p4/plans/p4-w04-vcpu-entry-exit.md`; closeout `../../../p4/plans/p4-w09-closeout-p5-handoff.md` | entry/exit hooks at which save/restore runs; exits that return control to EL2 in a defined state | no defined transition points → W06 step 1 blocker; state cannot be preserved |
| vCPU boundary and context extension area | P4 vCPU contract via `../../../p4/plans/p4-w09-closeout-p5-handoff.md` | a vCPU-owned arch-extension area holding W06 fields | if the vCPU record has no extension point, raise a boundary conflict with the P4 design; W06 does not create a parallel registry |
| Trap posture for EL1 timer registers | `../../../p1/plans/p1-w04-el2-architectural-state-baseline.md` | virtual-timer access untrapped; physical-timer access trapped (D1) | boundary conflict against the P1 record; no local `CNTHCTL` reconfiguration |
| HVC/error boundary for Guest-caused faults | `../../../p5/plans/p5-w02-hypercall-abi-error-boundary.md`; closeout `../../../p5/plans/p5-w10-closeout-p6-handoff.md` | Guest-fault classification for trapped access; VM-local containment | P5 taxonomy absent → step 1 blocker for the trap path |
| Typed time domains and expiry records | `../p6-w05-el2-generic-timer/README.md` (design) | conversion basis, late-fire measurement reuse | re-derive conversions in W06 by amendment, not duplication |
| vIRQ injection/dedupe/completion | `../p6-w07-virtual-interrupt-core/README.md` (design) | inject returns a dedupe-aware outcome; completion reports reach W06 | D4/D6 delivery paths redesign required — cross-design amendment |
| Classified PPI dispatch with split completion | `../p6-w03-physical-interrupt-lifecycle/README.md` (design) | acknowledge/hold/decompose of INTID 30 per D6 | escalation: Architecture Change Request against W03 lifecycle |
| Guest completion reports (maintenance) | `../p6-w09-maintenance-interrupt/README.md` (design) | a completion hook naming the completed (vCPU, vINTID) | deferred deactivation needs an alternative release point — cross-design amendment |
| Multi-vCPU Guest execution (for P6-V19) | P3/P4 handoffs (`../../../p3/plans/p3-w14-p4-smp-handoff.md`, `../../../p4/plans/p4-w09-closeout-p5-handoff.md`) | evidenced multi-vCPU execution to exercise isolation | record P6-V19 as a documented stage block (allowed by the task book) |

## 6. Scope classification detail

**Required:** per-vCPU `VcpuTimerState` (saved registers, typed shadow,
outstanding-physical flag, diagnostics); entry sequence (offset write,
restore); exit sequence (save, disable, condition evaluation, deferred
injection request, physical release); pure condition evaluator (hostable);
INTID-30 PPI conversion handler; trapped-access Guest-fault path; intake
validation of the timer event INTID against the platform and Validation
Guest layout; per-vCPU diagnostics (events raised, deferred events,
coalesced deliveries, trapped accesses, unexpected PPIs); the P6-V09/V10/V19
scenarios in [06](06-validation-and-handoff.md).

**Reserved:** timer-driven wakeup of absent vCPUs (P7); pause/resume
behavior (D8); `CNTVOFF` selection per VM/machine (P8); any scaling; use of
a W05 host deadline as a Guest-timer assist beyond measurement; EL1
physical-timer emulation.

**Out of Scope:** LR programming/pressure (W08); maintenance processing
(W09); Guest-visible masking/priority semantic contracts (W10); Validation
Guest scenario implementation (W11); storm evidence (W12); telemetry
collection/latency baseline (W13); Linux timer ABI, Guest DTB, machine
layout (P8); wall clock and migration time conversion; crate/module tree and
public API beyond the stated contracts.
