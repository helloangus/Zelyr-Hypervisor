# P6-W07 Scope, Foundations, and Resolved Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Parent:** [P6-W07 design entry](README.md).

## 1. Governing constraints

1. **Architecture ADR baseline**
   ([../../../../adr/adr-000-architecture-baseline-v0.1.md](../../../../adr/adr-000-architecture-baseline-v0.1.md)):
   Guest untrusted, faults VM-local (ADR-007, section 19); capability +
   rights + generation authorization, no VM-ID/role privilege (ADR-013,
   ADR-051); `VirtualInterrupt` as an object with route/priority/pending/
   active state (section 4); vIRQ injection starts from a software pending
   queue and grows toward List Registers (section 7, ADR-032); typed IDs
   and checked arithmetic; bounded queues and IRQ-context work (section 12);
   telemetry as a designed interface (ADR-048).
2. **P6 task book** ([../../task-book-v0.1.md](../../task-book-v0.1.md)):
   P6-G owned by W07, validated by P6-V11/P6-V12/P6-V14/P6-V18; §1 requires
   an isolated virtual-interrupt lifecycle with deferred delivery and
   prohibits equating a physical IRQ with a Guest IRQ; §8 classifies vIRQ
   state representation, locking, and coalescing as Implementation Choice
   resolved here; §2 makes W03 and the P4/P5 boundaries entry conditions.
3. **P6-W07 plan**
   ([../../plans/p6-w07-virtual-interrupt-core.md](../../plans/p6-w07-virtual-interrupt-core.md)):
   goal, scope (authorization, target isolation, pending/deferred/presented/
   active/completed coverage, repeated arrival, unavailable-vCPU outcomes,
   diagnostics), and acceptance; the plan excludes LR mechanics, Guest MMIO,
   wakeups, passthrough, and the final coalescing algorithm — meaning W07
   defines a bounded policy here and marks its evolution Reserved.
4. **Coding Guidelines**
   ([../../../../development/coding-guidelines.md](../../../../development/coding-guidelines.md)):
   guest input untrusted and validated; no panic from Guest errors; bounded
   locks/queues/IRQ work; semantic newtypes; no hidden state machines.

## 2. Current-state findings

Observable facts (worktree `docs/p6-implementation-designs`, audited
2026-09-18): P0 documentation scaffold only; no implemented W03 physical-IRQ
lifecycle, P4 vCPU boundary, or P5 authorization machinery; no P6 sibling
designs in this worktree yet. W07 designs against the plans' stated handoffs
(§5) with an entry review at workflow step 1. Because W07 is deliberately
controller-independent, it needs no register contract at all — its only
hardware-adjacent input is the INTID range declaration, which comes from
intake, not from W07 code.

## 3. Goal-to-baseline ledger

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| P6-V11: "an authorized event for a specified vCPU is eventually presented, acknowledged, and completed" | No vIRQ mechanism, presentation, or completion exists | W07-owned lifecycle state + inject/claim/complete contracts (this design); W08 presentation and W09 completion (assumed downstream contracts) | "eventually presented/acknowledged/completed" spans W07 state plus the consumers that act on it; W07 owns the preserved state | W07 (state); W08/W09 (presentation/maintenance) | W07-DV02–DV04 with W08/W09 evidence |
| P6-V12: distinct pending events survive deferred presentation | Nothing exists | W07 per-vCPU bank with per-vINTID bits (D2) | multiple pendings must coexist without interference — a structural property | W07 (this design) | W07-DV02/DV03 |
| P6-V14: repeated arrival follows documented policy without corruption | Nothing exists | W07 coalescing + saturating occurrence accounting (D5) | repetition must be a defined policy, not emergent | W07 (this design) | W07-DV04 |
| P6-V18: "an event directed to one vCPU is not received by another" | No multi-vCPU evidence upstream | per-vCPU banks with no cross-bank mutation path (structural); conditional exercise on evidenced multi-vCPU prerequisite | isolation is structural first, evidential when the prerequisite exists | W07 (structure); P3/P4 (prerequisite) | W07-DV06 or recorded block |
| vIRQ request authorization (plan scope) | P5 capability machinery unimplemented | Assumed: P5 dispatch/capability/error boundary (`../../../p5/plans/p5-w02-hypercall-abi-error-boundary.md`, `../../../p5/plans/p5-w05-capability-rights-bootstrap-revocation.md`, `../../../p5/plans/p5-w04-handle-lifecycle-type-safety.md`, closeout `../../../p5/plans/p5-w10-closeout-p6-handoff.md`) | Guest-requested injection must be authorized by the project's authority model, not by W07-private rules | P5 (assumed); W07 validation layer | W07-DV05 |
| Classified Host IRQ lifecycle (distinct domain) | W03 unimplemented | Assumed: W03 physical lifecycle (`../p6-w03-physical-interrupt-lifecycle/README.md`) as the separate Host-side domain W07 explicitly does not merge with | the plan forbids equating physical and virtual IRQs; the boundary must be stated and kept | W03 (assumed); W07 boundary discipline | step-1 review |
| Presentation/consumers | W08/W09 planned | Assumed: W08 claim/return protocol needs (`../p6-w08-gic-virtualization-interface/README.md`), W09 completion reports (`../p6-w09-maintenance-interrupt/README.md`) | W07's protocol exists to serve them; their designs must consume the same protocol | W08/W09 (consumers); W07 (protocol owner) | cross-design reconciliation at step 1 |
| vCPU lifecycle boundary | P4 unimplemented | Assumed: P4 vCPU object/destroy boundary (`../../../p4/plans/p4-w04-vcpu-entry-exit.md`, `../../../p4/plans/p4-w09-closeout-p5-handoff.md`) | drain-on-destroy needs a defined destroy point | P4 (assumed); W07 drain contract | W07-DV03 (unavailable-vCPU cases) |
| INTID range declaration | W01 planned | Assumed: W01 capability conclusions (`../p6-w01-gic-capability-discovery/README.md`) supply the supported SPI window and PPI set | the bank's SPI window size is platform-dependent and must be intake-declared, never hardcoded | W01 (assumed); W07 intake | W07-DV01 |

No row requires inventing a crate, dependency, wire format, or scheduler
policy; no decision blocker is outstanding for this design.

## 4. Resolved design decisions and their authority

**D1 — One owner per datum.** Per-vCPU pending/active software truth is
W07's. List-Register residency is W08's datum, maintained through the
claim/return protocol (W08 never edits W07 bits directly). Maintenance
correlation is W09's processing that feeds the same protocol. The timer
condition remains W06's. This is the checklist's one-owner rule applied
explicitly, and the task book's separation of "virtual-interrupt lifecycle"
from "hardware presentation". Authority: task book §1/§8; skill checklist.

**D2 — Identity and range.** A virtual interrupt is identified by
`(VcpuId, VirqId)` with `VirqId` a validated newtype. The per-vCPU bank
covers the architecture-banked SGI range (0–15) and PPI range (16–31), plus
an SPI window `[spi_first, spi_last]` declared by intake from W01 capability
conclusions. SPI vIRQs are per-VM objects routed at injection time to a
target vCPU; P6 stores them in the target's bank (routing is fixed at
inject; re-routing is not a P6 operation). Ranges outside the declared
window are rejected. Authority: task book §8 Implementation Choice; W01
assumed contract; ADR section 13.

**D3 — Queue-and-preserve delivery semantics.** An injected event for an
absent (not currently running) vCPU stays pending until W08 presents it at
a later entry; W07 performs no wakeup and keeps no notion of "now running"
beyond what presentation consumers report. Unavailable-vCPU injection
(vCPU exists, not running) therefore succeeds and defers; injection to a
nonexistent or destroyed vCPU is rejected. Rationale: eventual delivery is
the task-book requirement; wakeup is P7 policy. Authority: task book §1;
W07 plan (unavailable-vCPU outcomes); P7 boundary.

**D4 — Two producer classes, one authorization spine.** (a)
**Host-mechanism producers** (W06 timer expiry; future Host-side sources)
inject with Host authority: the binding is the mechanism's design, and W07
still validates target liveness, range, and priority encoding. (b)
**Guest HVC requests** (Validation Guest scenarios; later policy services)
pass the P5 dispatch and capability boundary first: caller capability,
rights on the target vCPU object, same-VM rule, and the request class. In
P6 the Guest-requestable class is the **SGI range only**; PPI/SPI classes
are Host-mechanism-owned (the timer owns its PPI event; Guest-requestable
SPI needs machine-model routing policy that P8 owns). This class restriction
is stage-local design freedom owned by this design, recorded because the
plan leaves the request boundary open. Guest-chosen priority is not
accepted in P6: guest-class injections carry the VM default priority
(D7). Authority: ADR-013/051; P5-W05 assumed contract; task book §8.

**D5 — Repeated arrivals coalesce with saturating accounting.** Each
`(vcpu, vintid)` holds one pending bit and a saturating occurrence counter.
Injection while pending increments the counter (the event is coalesced,
never queued twice); injection while presented sets pending again
(re-pend); injection while active sets pending (to be presented after
completion under D6). No structure grows with arrival rate. Authority:
task book §8 (coalescing choice); Coding Guidelines bounded-queue rule;
ADR-048 accounting intent.

**D6 — Selection excludes `active` vINTIDs until completion.** W07's
`select_next_pending` never offers a vINTID whose active bit is set; its
pending bit remains set and it is presented after the completion report
clears active. Rationale: presenting pending+active simultaneously through
an LR is a GIC state corner whose Guest-visible semantics belong to W10's
contract; excluding it keeps P6 bounded and correct for SGI/PPI-class
events. Full pending+active presentation is Reserved (W10 evolution).
Authority: task book §8; W10 boundary.

**D7 — Priority is validated carriage, not semantics.** `VirqPriority` is a
newtype validated against the priority-field width declared by the
virtualization capability intake (W01/W08 data); comparison is total
(lower value = higher urgency, matching the architecture convention). W07
orders selections by it with a deterministic tie-break (lowest VirqId
first). The Guest-visible meaning of priorities — preemption, masking
interaction — is W10's; Guest-chosen priorities are not accepted in P6
(D4). Authority: task book §8; W10 boundary.

**D8 — Deterministic destruction.** vCPU/VM destruction drains the bank:
pending events are dropped with a counted total, presentations are
returned by W08 before final drain (protocol ordering: W08 purges at
unload; drain asserts empty), and later injections are rejected. The
destroy path never silently discards presented or active state. Authority:
skill checklist (destruction behavior); P4 lifecycle assumed contract.

## 5. Assumed upstream contracts and failure boundaries

| Contract | Source (plan path) | W07 relies on | If delivered differently |
|---|---|---|---|
| HVC dispatch, error taxonomy, guest-data safety | `../../../p5/plans/p5-w02-hypercall-abi-error-boundary.md`; `../../../p5/plans/p5-w03-guest-data-safety.md` | a dispatch point where the Guest injection request enters; structured denial classes | Guest request path has no entry point → W07-DV05 blocked, recorded; Host-mechanism path unaffected |
| Object handles with generation; capability + rights | `../../../p5/plans/p5-w04-handle-lifecycle-type-safety.md`; `../../../p5/plans/p5-w05-capability-rights-bootstrap-revocation.md` | validated `VcpuHandle` references; rights check for the target object | if handles/capabilities differ structurally, [04](04-code-contracts-authorization-and-errors.md) §2 is amended against the delivered contract — never bypassed |
| P5 closeout: evidenced authority facts for P6 | `../../../p5/plans/p5-w10-closeout-p6-handoff.md` | the documented authority boundary P6 may consume | gap → step-1 blocker for the Guest path |
| vCPU objects, liveness, destroy point | `../../../p4/plans/p4-w04-vcpu-entry-exit.md`; `../../../p4/plans/p4-w09-closeout-p5-handoff.md` | target-liveness queries; destroy hook for drain | no destroy hook → drain integrates at VM teardown with recorded ordering |
| Physical-IRQ lifecycle (distinct domain) | `../p6-w03-physical-interrupt-lifecycle/README.md` (design) | nothing — explicit non-merger: Host physical IRQs never enter W07 state | n/a; boundary violation would be an architecture review failure |
| Supported INTID window | `../p6-w01-gic-capability-discovery/README.md` (design) | declared SGI/PPI bank + SPI window | bank/window resize is an intake change; no hardcoded ranges in Core |
| Presentation claim/return | `../p6-w08-gic-virtualization-interface/README.md` (design) | W08 consumes the protocol as specified in [03](03-code-contracts-virq-lifecycle.md) §5 | protocol amendments are cross-design changes agreed with W08/W09; W07 does not adapt to private W08 state |
| Completion reports | `../p6-w09-maintenance-interrupt/README.md` (design) | W09 reports Guest-visible completions with (vcpu, vintid) | completion remains outstanding → D6 defers re-presentation; recorded limitation, no timeout hacks |

## 6. Scope classification detail

**Required:** `VirqBank` per vCPU (pending/active/presented bits, saturating
occurrence counts, priority slots for the declared window); typed IDs
(`VcpuId`, `VirqId`, `VirqPriority`, claim tokens); `virq_inject` with the
validation spine; `virq_select_next`; presentation claim/return;
`virq_report_guest_completion`; `virq_query_summary`; `virq_drain`; the
Guest HVC request handler consuming P5 authorization; intake of the range
declaration and priority width; diagnostics counters; the P6-V11/V12/V14/
V18-oriented scenarios in [06](06-validation-and-handoff.md).

**Reserved:** pending+active simultaneous presentation (D6); Guest-chosen
priorities and preemption semantics (W10+); multi-target and routing-change
operations beyond per-target injection; device/virtio injection classes
(P9+); cross-VM notification composition (P12); persistent/snapshot
serialization of bank state (P16/P17 must design their own representation —
the in-memory layout is not a format).

**Out of Scope:** LR mechanics and pressure (W08); maintenance processing
(W09); Guest masking/priority semantics (W10); Validation Guest scenario
implementation (W11); storm evidence (W12); telemetry collection/latency
baseline (W13); vGIC MMIO/DTB/machine ABI (P8); scheduler wakeups (P7);
crate/module tree and public API beyond the stated contracts.
