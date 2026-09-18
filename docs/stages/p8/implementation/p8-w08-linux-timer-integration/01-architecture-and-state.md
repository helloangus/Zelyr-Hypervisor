# P8-W08 Architecture and State

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W08 detailed design](README.md).

## 1. Logical modules

| Module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| M1 Guest time source (GTI) | Present the virtual counter with the per-VM offset; enforce uniform read-only frequency | None (reads physical counter + VM offset) | Physical counter reads, VM offset, machine-gated frequency | Guest counter values (trap handlers for counter/frequency accesses per W05 classification) | Does not own the physical counter (P6-W05); does not set the offset value (machine-gated) |
| M2 Guest timer registers (GTR) | Emulate the exposed timer class's CTL/CVAL/TVAL semantics onto P6-W06 vCPU timer state | None of its own — P6-W06 owns state | Trapped register accesses (vCPU, which register, value) | P6-W06 state updates; re-arm requests to M3 | Does not store timer state, does not deliver |
| M3 Expiry engine (GTE) | Compute the earliest enabled deadline per vCPU; arm/cancel on the P6-W05 EL2 event timer; convert expiry into delivery | Per-vCPU derived "armed deadline" cache (see §3.2) | P6-W06 state changes from M2; vCPU transition events; P6-W05 expiry events | Delivery requests to M4; wakeup requests to M5 | Does not own deadlines' truth (P6-W06 does); no delivery logic |
| M4 Delivery bridge (GTD) | Turn expiry into a timer-PPI injection via [W07](../p8-w07-linux-vgicv3/README.md) with level semantics | None | Expiry events | `vgic_request_irq` calls; telemetry | Does not queue or order (P6-W07 owns pending state) |
| M5 WFI/wakeup bridge (GTW) | Implement the Guest WFI blocking contract over P7-W06 with declared wakeup sources | None | WFI traps, wakeup sources (M4 delivery, [W07] vIRQ pending, Host notification) | P7 block/wakeup calls; telemetry | Does not implement scheduler wait queues (P7-W06) |
| M6 Diagnostics and telemetry (GTT) | Structured events: programming, expiry, deferred delivery, wakeup, latency per P6-W13 method | Bounded per-vCPU counters | All modules | Telemetry; W13/W17 consumption | Not a log; no unbounded buffers |

Layering: M1–M5 contain no board/SoC/QEMU constants; all Guest-visible
numeric facts enter via machine-contract data (README decision D8).

## 2. Core objects and ownership

No new long-lived object class is introduced; the design consumes planned
owners and adds one derived cache:

- **Per-VM time offset** (`time_offset`): machine-gated value applied at VM
  creation, immutable for the VM's lifetime. Owner: VM object. Written only
  at creation; read by M1. A change mid-life would break monotonicity
  (§4 I1) and is therefore structurally excluded.
- **Per-vCPU Guest-timer state** (CTL, CVAL for the exposed class): owner is
  the P6-W06 vCPU timer state — cited, not redesigned. M2 is its only P8
  writer, through P6-W06's update operations, so P6's entry/exit preservation
  contract continues to hold unmodified.
- **Per-vCPU armed-deadline cache** (`armed_deadline: Option<Deadline>`):
  derived truth ("what M3 asked the EL2 event timer to observe for this
  vCPU"). Owner: M3 via the vCPU. Invariant: it is always derived from P6-W06
  state + the vCPU's run state; it is cancelled at every vCPU transition out
  and re-derived at transition in ([03 §5](03-code-contracts-expiry-wakeup.md)).
  It is a cache, never an authority: a mismatch resolves in favor of P6-W06
  state.
- **Per-VM frequency fact**: machine-gated constant surfaced by M1.
  Immutable; VM creation fails without it.

No Guest-writable hypervisor structure exists on this path; every Guest
action enters as a trapped register access.

## 3. Lifecycle

### 3.1 VM/vCPU lifecycle integration

```text
VM creation:  time_offset + frequency facts installed (VM fails to boot without them)
vCPU created: P6-W06 timer state exists (cleared/disabled); armed cache = None
vCPU entry:   M3 re-derives and re-arms (if an enabled deadline exists)
vCPU exit:    M3 cancels the arm; P6-W06 state untouched (preservation is P6's)
vCPU stop
  (CPU_OFF):  M3 cancels arm; state persists for re-ON per [W06]/P7 lifecycle
VM destroy:   all derived caches dropped with the VM; no global residue
```

### 3.2 Deadline lifecycle per vCPU

```text
disabled/masked                    -> no arm; assertion condition false
program CVAL/TVAL while enabled    -> M3 re-arms (earliest deadline wins)
deadline passes (counter ≥ CVAL)   -> asserted: M4 injects PPI (level), M5
                                      wakes the vCPU if blocked
Guest masks (IMASK=1)              -> deasserted at the GIC level per W07's
                                      level-config carrying; arm retained
Guest disables (ENABLE=0)          -> arm cancelled; assertion false
Guest EOIs the PPI while asserted  -> level rule re-asserts via W07 config
                                      carrying (README decision D5)
```

## 4. Invariants (the contracts P8-V11 actually tests)

- **I1 Monotonicity:** for any single Guest observation channel, two reads of
  the virtual counter never regress, regardless of WFI, preemption, or
  pCPU migration. Derived from: immutable per-VM offset + monotonic physical
  counter + no counter-domain switching on the Guest path.
- **I2 Cross-vCPU coherence:** all vCPUs of a VM read the same counter value
  at the same physical instant (within architecture counter-read atomicity)
  — same offset, same source. Linux multi-CPU timekeeping depends on this.
- **I3 Target-vCPU delivery:** a timer programmed by vCPU A asserts and is
  delivered only to vCPU A's timer-PPI, in every state of vCPU A
  (running/blocked/stopped-pending-on); it never migrates to another vCPU.
- **I4 No lost wakeup:** if an enabled, unmasked deadline has passed, the
  target vCPU either runs (delivery pending via P6-W07) or is woken (P7-W06)
  — never both-missed. Deferred delivery and wakeup are complementary, not
  alternative.
- **I5 Continuity:** preemption, scheduling delay, or pCPU migration never
  advances, regresses, or duplicates Guest-visible time ([03 §5](03-code-contracts-expiry-wakeup.md)).
- **I6 Bounded host work:** no Guest programming pattern (including
  zero/already-elapsed deadlines set at trap frequency) causes unbounded,
  unthrottled-host-work behavior beyond the declared, bounded re-arm policy
  ([02 §4](02-code-contracts-timer-regs.md)).

## 5. Concurrency and security model

- **Context:** M1/M2 handlers run in VM-exit context (bounded, no
  allocation); M3 arming runs in VM-exit and vCPU-transition contexts; M4/M5
  issue requests (P6-W07/P7-W06) without joining them.
- **Locks:** per-vCPU timer lock guards P6-W06 state mutations and the
  armed-cache derivation (one critical section: "update state, then derive
  arm"); the P6-W05 event-timer API provides its own arbitration for the
  per-pCPU hardware deadline. Lock order: vCPU timer lock before any VM GIC
  lock ([W07](../p8-w07-linux-vgicv3/01-architecture-and-state.md) order)
  when both are taken (delivery path).
- **Cross-CPU:** expiry on pCPU X for a vCPU blocked and placed elsewhere
  (M:N future) is expressed as P6-W07 injection + P7-W06 wakeup requests via
  the P3 transport — issuance only, never cross-CPU state mutation beyond
  those seams.
- **Untrusted input (ADR §19):** TVAL/CVAL arithmetic uses checked
  operations (Guest-controlled 64-bit values); reserved CTL bits validated;
  counter/frequency registers are read-only from the Guest; a trap on a
  disallowed register is W05-classified. Deadline values are Guest data —
  they never select Hypervisor code paths.
- **Non-leakage:** Guest time never exposes Host uptime beyond the offset
  relation; frequency and offset are machine facts, not Host hardware reads.

## 6. Failure boundaries of assumed prerequisite contracts

| Prerequisite | Assumed contract (cite) | Failure boundary if different |
|---|---|---|
| P6-W06 Guest-timer state owner | `docs/stages/p6/plans/p6-w06-guest-generic-timer.md` — vCPU-owned state, entry/exit preservation, deferred delivery when absent | If state or update operations differ, [02 §3](02-code-contracts-timer-regs.md) blocks; `Architecture Change Request` against P6, no local state |
| P6-W05 EL2 event timer | `docs/stages/p6/plans/p6-w05-el2-generic-timer.md` | Without a cancellable per-pCPU deadline primitive, M3 cannot exist; block at the P6/W08 seam |
| P6-W07 vIRQ injection | `docs/stages/p6/plans/p6-w07-virtual-interrupt-core.md`, consumed via [W07 §3](../p8-w07-linux-vgicv3/03-code-contracts-interrupt-flow.md) | Without an injectable level-configured PPI, delivery blocks; raise at W07/P6 |
| P7-W06 block/wakeup | `docs/stages/p7/plans/p7-w06-block-wakeup.md` | If wakeup sources cannot include timer + vIRQ, [03 §4](03-code-contracts-expiry-wakeup.md) cannot be declared; raise at the P7 seam |
| P7-W04 transition hooks | `docs/stages/p7/plans/p7-w04-preemption-context-switch.md` | Without in/out hooks, I5/continuity is undeclarable; block, do not poll |
| W02 machine gate | `docs/stages/p8/plans/p8-w02-machine-contract-governance.md` | Unapproved frequency/offset/timer-class ⇒ no implementation may embed them |
| W04 DTB timer node | `docs/stages/p8/plans/p8-w04-guest-dtb-contract.md` | Node facts must match the exposed class/frequency; mismatch fails P8-V05, fix in W04's contract |
| W05 classification | `docs/stages/p8/plans/p8-w05-linux-cpu-virtualization.md` | Physical-counter/timer access classification is W05's; divergence wins at that seam |
