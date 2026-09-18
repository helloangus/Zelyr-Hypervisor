# P6-W05 Scope, Foundations, and Resolved Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Parent:** [P6-W05 design entry](README.md).

## 1. Governing constraints

The binding sources, in authority order, and what each contributes:

1. **Architecture ADR baseline**
   ([../../../../adr/adr-000-architecture-baseline-v0.1.md](../../../../adr/adr-000-architecture-baseline-v0.1.md)):
   mechanism/policy separation (ADR-004/005); AArch64-first with GICv3
   baseline (ADR-002/032); SMP from day one, no single-pCPU assumptions
   (ADR-015); static vCPU↔pCPU binding as early policy with scheduler
   evolution reserved (ADR-016/017); typed addresses/IDs and checked
   arithmetic (section 13); structured telemetry as a designed interface
   (ADR-048); layered validation (ADR-049); Core independent of Board/SoC
   names (ADR-043/052).
2. **P6 task book** ([../../task-book-v0.1.md](../../task-book-v0.1.md)):
   P6-E ("Host monotonic time and per-pCPU deadline timer") owned by W05 and
   validated by P6-V07/P6-V08; section 8 classifies timer programming, trace
   encoding, and Host IRQ state representation as Implementation Choice
   resolved here; section 1 prohibits binding vCPU timer state to a pCPU and
   treating QEMU as an architecture definition.
3. **P6-W05 plan**
   ([../../plans/p6-w05-el2-generic-timer.md](../../plans/p6-w05-el2-generic-timer.md)):
   goal, scope, out-of-scope list, work sequence, and the P6-V07/P6-V08
   acceptance wording. The plan names no types, registers, sequences, or
   algorithms; this design supplies them.
4. **Coding Guidelines**
   ([../../../../development/coding-guidelines.md](../../../../development/coding-guidelines.md)):
   typed quantities and checked arithmetic for all counter/deadline values;
   volatile/MMIO discipline and required barriers for register sequences; no
   unbounded work in IRQ context; minimal, audited `unsafe` with `SAFETY`
   justifications; no guest/board/QEMU names in Core.

If any assumption below is contradicted by an actually delivered upstream
record, the affected decision stops and the conflict is recorded as
Architecture Change Request or ADR Required; W05 does not silently re-derive a
predecessor's contract.

## 2. Current-state findings

Observable facts (worktree `docs/p6-implementation-designs`, audited
2026-09-18 via `git ls-files`):

- No Cargo workspace, crate, Rust source, linker script, target JSON, or CI
  workflow exists. Code directories are `.gitkeep` placeholders only.
- P1–P5 plans and task books exist as planning documents only; there are no
  implementation records or verification records for P1–P5 (`p0-w01` is the
  only completed verification; `p0-w02` has a proposed design).
- No P6 implementation designs exist yet in this worktree; sibling P6 designs
  (W01–W04, W09–W13) are being prepared in parallel on the same branch.
- Therefore: W05 cannot inspect any delivered Host IRQ lifecycle, timer-access
  baseline, or per-pCPU runtime. It designs against the plans' stated
  handoffs, each with an explicit entry check and failure boundary
  (§3, §5).

## 3. Goal-to-baseline ledger

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| P6-V07: "a future deadline yields one attributable Host timer IRQ and controlled rearm/cancel result" | No timer mechanism exists; no Host IRQ lifecycle exists | Assumed: W03's classified-IRQ dispatch boundary that can route a per-pCPU timer PPI to a W05 handler and complete it; supplied by P6-W03 | Without a classified dispatch and completion path, an expiry cannot be attributable or safely completed | P6-W03 (plan `../../plans/p6-w03-physical-interrupt-lifecycle.md`; design `../p6-w03-physical-interrupt-lifecycle/README.md`, prepared in parallel) | W03 evidence (P6-V20/V22) inspected at W05 step 1 |
| P6-V07: attributable one-shot event | No deadline timer exists | W05-owned deadline-event module with arm/cancel/rearm and per-pCPU expiry records | The outcome requires the mechanism itself; it is the package's core deliverable | W05 (this design) | W05-DV04/DV05/DV07 |
| P6-V08: repeated per-pCPU events "remain stable in the declared environment" | Nothing exists | W05-owned rearm semantics with a declared catch-up policy and stability checks | Repeated behavior must be defined, not emergent from consumers | W05 (this design) | W05-DV05/DV08 |
| P6-V08: "entry/exit does not regress observed monotonic time" | No time source exists | W05-owned monotonic clock read discipline (typed counter reads, ordering, frequency check) plus an explicit across-Guest-entry/exit regression check | Monotonicity must be a designed property with an observation method, not an assumption | W05 (this design) | W05-DV03/DV08 |
| Per-pCPU ownership ("per-pCPU monotonic-time and deadline-event behavior") | No per-pCPU runtime exists | Assumed: P3 per-CPU local-state foundation with owner-pCPU-only access rules; supplied by P3-W04/W06/W14 | A per-pCPU timer is only safe if local-state ownership and synchronization rules exist | P3 (plans `../../../p3/plans/p3-w04-per-cpu-runtime.md`, `../../../p3/plans/p3-w06-concurrency-synchronization.md`, `../../../p3/plans/p3-w14-p4-smp-handoff.md`) | P3 handoff evidence inspected at W05 step 1 |
| Timer capability intake | No PlatformInfo evidence exists | Assumed: P2 PlatformInfo timer facts (frequency, presence) via W01 reconciliation; plus a W05-local intake check of `CNTFRQ_EL0` equality across online pCPUs | Programming a timer without a validated frequency/counter basis can silently mis-convert time | P2/P6-W01 (plans `../../../p2/plans/p2-w10-p3-p4-handoff-contract.md`, `../p6-w01-gic-capability-discovery/README.md`); intake check W05 | W05-DV01 intake check |
| Timer-access baseline (which EL2 timer may be used, trap posture) | P1-W04 unimplemented | Assumed: P1-W04 delivers a documented `CNTHCTL_EL2`/timer-access baseline that leaves the EL2 physical timer usable at EL2 | The register surface (D1) depends on how P1 configured timer traps/access | P1-W04 (plan `../../../p1/plans/p1-w04-el2-architectural-state-baseline.md`) | Entry check at W05 step 1; mismatch → boundary failure, not local reconfiguration |
| Integration with physical IRQ lifecycle and per-pCPU diagnostics | Nothing exists | W05 integration step producing a classified PPI handler + diagnostics counters inside the W03 lifecycle | "Integrated ... with the physical IRQ lifecycle and per-pCPU diagnostics" is an explicit plan work item | W05 + W03 contract | W05-DV06 |
| Mechanism permits later scheduling without deciding scheduler policy | No scheduler exists (P7 plans only) | Design rule: no policy in W05's API; consumer-driven rearm; reserved extension points | Prevents W05 from pre-empting P7 design space | W05 (this design, D5) | W05-DV10 closure review |

No row above requires inventing a remote, license, dependency, or architecture
choice, so no decision blocker is outstanding for this design.

## 4. Resolved design decisions and their authority

**D1 — Register surface: per-pCPU EL2 physical timer over the physical
counter.** W05 programs `CNTHP_CVAL_EL2`/`CNTHP_CTL_EL2` and reads
`CNTPCT_EL0` (counter) and `CNTFRQ_EL0` (frequency). The EL2 virtual timer
(`CNTHV_*`) is Reserved pending any E2H-based baseline; the EL1 timers are not
W05 surface at all. Rationale: the EL2 physical timer is available at EL2
regardless of VHE mode, so it is the baseline-independent Host timer; the
choice is exactly the "timer programming" Implementation Choice the task book
assigns to detailed design. Condition: P1-W04's timer-access baseline and the
W01 platform intake must leave these registers usable at EL2; if the delivered
baseline differs (for example an E2H=1 configuration that re-homes the EL2
timer surface), this decision must be re-derived by a design amendment, not a
local code change. Authority: task book §8 Implementation Choice; P1-W04
assumed contract with failure boundary.

**D2 — Compare-value programming with checked arithmetic.** Deadlines are
computed as `counter + delta` in full 64-bit checked arithmetic and programmed
into the compare-value register. `CNTHP_TVAL_EL2` is not used on the arm path
(its 32-bit signed width would truncate and force range decisions into every
caller). Overflow of the 64-bit deadline is a named error, not a wrap.
Rationale and authority: Coding Guidelines typed-quantity and checked-arithmetic
rules; ADR section 13 newtype requirement.

**D3 — Repeated events are consumer-driven rearms with bounded catch-up.**
There is no autonomous periodic mode. On expiry service, the consumer requests
`rearm`; for anchored repetition the next deadline is
`previous_deadline + period` computed with checked arithmetic, with an
explicit bounded-catch-up cap (a declared maximum of consecutive anchor steps
applied per service) so an overdue consumer cannot enqueue an unbounded burst
of immediate expiries. Rationale: keeps tick policy out of W05 (mechanism/policy
separation) while making repeated-event stability (P6-V08) a defined W05
property; the cap is stage-local mechanism policy owned by this design because
it bounds IRQ-context work. Authority: task book §8 Implementation Choice +
Coding Guidelines IRQ-boundedness rule.

**D4 — Bounded IRQ-context expiry handling.** The timer PPI handler records a
bounded expiry record (event identifier, service counter sample, arm
generation) into the per-pCPU timer state and marks the consumer's event flag.
It does not run consumer policy, allocate, or loop. The consumer picks the
record up in a bounded mainline dispatch. Rationale: Coding Guidelines
prohibition on long/heavy work in IRQ context; ADR-048 observability without
hot-path cost. Authority: Coding Guidelines; task book §1 ("no unbounded work"
is stated at stage level through W03/W12).

**D5 — Mechanism, not policy.** W05 exposes arm/cancel/rearm and expiry
records. It decides nothing about scheduling, tick rates, preemption, or vCPU
selection, and exposes no interface that implies them. P7 consumes the
evidenced mechanism and designs its own policy. Authority: ADR-004/005,
P6-W05 plan handoff wording, P6 task book §7 (P7 consumes mechanisms only).

**D6 — The timer PPI INTID is intake-validated platform/arch data, never a
Core constant.** The architecture assigns the EL2 physical timer to PPI 10
(INTID 26), but W05 consumes the INTID through the W02/W03 Host-GIC intake
(platform/arch layer data reconciled with W01 capability conclusions), so a
platform quirk cannot be papered over by a hardcoded Core value. Rationale:
ADR-041–ADR-045 layering and capability-driven selection. Authority: ADR;
task book §1 ("must not treat a platform name as an architecture decision").

**D7 — Per-pCPU ownership with owner-pCPU-only access.** All W05 mutable state
lives in the owning pCPU's timer object; only the owning pCPU mutates it.
Remote actors have no cancel/rearm path in P6; cross-pCPU timer control is
Reserved for P7 (which owns cross-pCPU coordination policy). Rationale: P3
per-CPU ownership contracts; eliminates a whole class of cross-CPU races from
the first Host timer. Authority: P3-W04/W06/W14 assumed contracts; task book
§1 per-pCPU requirement.

**D8 — Explicit monotonic-read ordering discipline.** Counter reads that order
a decision are preceded by an instruction barrier (ISB) so the read cannot be
speculatively hoisted above prior system operations; arming and disabling
sequences carry the barrier placements listed in
[04-code-contracts-deadline-timer.md](04-code-contracts-deadline-timer.md) §5.
The P6-V08 monotonicity check observes reads across Guest entry/exit windows
instead of assuming the property. Rationale: Coding Guidelines
MMIO/register/barrier rules ("QEMU success is not evidence these hardware
rules can be omitted"); the speculative-read behavior of the counter is an
architecture property that must be respected, not tested into existence.
Authority: Coding Guidelines; Arm architecture reference (revision locked at
implementation per task book §8).

## 5. Assumed upstream contracts and failure boundaries

Each contract below is assumed from an upstream plan because no upstream
implementation exists. W05 step 1 inspects whatever evidence exists at
implementation time; a missing, partial, or contradictory delivery stops the
affected step and records a blocker (Platform Investigation, Architecture
Change Request, or ADR Required as applicable). W05 never repairs or
re-implements an upstream mechanism.

| Contract | Source (plan path) | W05 relies on | If delivered differently |
|---|---|---|---|
| Online pCPU lifecycle, CPU-local state, owner-only access rules | `../../../p3/plans/p3-w04-per-cpu-runtime.md`, `../../../p3/plans/p3-w14-p4-smp-handoff.md` | existence of per-pCPU local data with a defined owning-pCPU access discipline | per-pCPU timer objects cannot be placed; step 1 blocker |
| Shared-state synchronization semantics, IRQ-context rules, lock ordering | `../../../p3/plans/p3-w06-concurrency-synchronization.md` | irq-safe local access and lock-order integration for the timer state vs. W03 dispatch | concurrency model in [02](02-architecture-and-state.md) must be re-derived; design amendment |
| P6-V07: "a future deadline yields one attributable Host timer IRQ and controlled rearm/cancel result" | No timer mechanism exists; no Host IRQ lifecycle exists | Assumed: W03's classified-IRQ dispatch boundary that can route a per-pCPU timer PPI to a W05 handler and complete it; supplied by P6-W03 | Without a classified dispatch and completion path, an expiry cannot be attributable or safely completed | P6-W03 (plan `../../plans/p6-w03-physical-interrupt-lifecycle.md`; design `../p6-w03-physical-interrupt-lifecycle/README.md`, prepared in parallel) | W03 evidence (P6-V20/V22) inspected at W05 step 1 | a registered per-pCPU PPI handler that receives the classified timer PPI and completes it per the W03 lifecycle; condition-cleared-after-ack is classified, not fatal | if W03 cannot classify a condition-cleared timer PPI safely, the cancel/fire race handling in [04](04-code-contracts-deadline-timer.md) §3 must be renegotiated with W03; escalation path recorded there |
| Host GIC and per-pCPU interface readiness (INTID space, group config, PPI routing usable) | `../p6-w02-physical-gic-bring-up/README.md` + `../p6-w01-gic-capability-discovery/README.md` | timer PPI is routable at the local CPU interface; INTID delivered as intake data (D6) | timer events cannot be received; step 1 blocker |
| EL2 timer-access baseline | `../../../p1/plans/p1-w04-el2-architectural-state-baseline.md` | `CNTHP_*`/`CNTPCT_EL0`/`CNTFRQ_EL0` usable at EL2 with the declared trap posture | D1 register surface re-derived by design amendment |
| EL2 IRQ entry path | `../../../p1/plans/p1-w05-el2-exception-entry-baseline.md` | IRQ exceptions reach EL2 with bounded context capture | expiry receipt path does not exist; step 1 blocker |
| Platform timer facts (frequency, presence, counter width) | `../../../p2/plans/p2-w10-p3-p4-handoff-contract.md` + `../p6-w01-gic-capability-discovery/README.md` | PlatformInfo-derived timer facts reconciled at intake; local `CNTFRQ_EL0` cross-check | frequency mismatch across online pCPUs is a W05 intake failure (W05-DV01), blocking timer use |

## 6. Scope classification detail

**Required** (W05 closure depends on all of these): typed time-value module;
monotonic clock read service; per-pCPU deadline-event timer object; arm, one-
shot expiry, cancel, rearm (including anchored repetition with bounded
catch-up); timer PPI receipt integration with the W03 lifecycle; intake checks
(frequency equality, capability/INTID validation); per-pCPU diagnostics
(arm/cancel/fire/late-fire counters, trace events); the P6-V07/P6-V08
acceptance scenarios in [06](06-validation-and-handoff.md).

**Reserved** (named here so later designs do not discover them by accident):
cross-pCPU timer control or remote cancel; timer-driven wakeup of blocked
entities (P7); the EL2 virtual timer (`CNTHV_*`) surface; pause/resume of Host
time; offset/scaled time; migration time conversion; any poll- or
deadline-based scheduling API.

**Out of Scope:** everything in the plan's out-of-scope list — Guest timer
virtualization (W06), scheduler ticks and time-slice policy (P7), wall
clock/NTP, exact timer-register programming decisions delegated here are done
(they are in scope) but no *further* register behavior beyond the stated
surface, timing API design beyond the contracts here, and any crate/module
tree or public API beyond the contracts in [03](03-code-contracts-time-core.md)
and [04](04-code-contracts-deadline-timer.md).
