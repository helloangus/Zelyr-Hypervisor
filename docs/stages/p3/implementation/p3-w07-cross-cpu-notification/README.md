# P3-W07 Cross-CPU Notification — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The minimal, safe physical-CPU event primitive — per-CPU mailbox
slots, targeting, self/concurrent/invalid/offline behavior, wake signaling,
and accounting — required by
[P3-W07](../../plans/p3-w07-cross-cpu-notification.md).  
**Owner/change context:** P3-W07 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P3-W07. It defines the
notification contract that completes the P3 cross-CPU foundation: what a
sender may send, to whom, with what defined outcomes; how a target observes
and accounts for events; and the explicit non-RPC limit. The primitive is a
mailbox-plus-wake transport (per-CPU slot with a monotonically increasing
sequence, WFE/SEV wake), *not* an interrupt: GIC/SGI delivery of
interrupts is [P6-W04](../../../p6/plans/p6-w04-smp-interrupt-routing-sgi.md)
territory, and this design records the boundary explicitly — the P3
primitive is the "IPI mailbox" the ADR names, with interrupt-driven
delivery as a later carrier of the same contract if a later stage chooses
one. It deliberately does **not** define the TLB request protocol
([P3-W08](../p3-w08-tlb-shootdown-transport/README.md) claims kind 1 and
builds its own request/consumption semantics on this primitive), any
scheduler wakeup policy (P7), guest notification (P8+), or a telemetry
catalog ([P3-W11](../p3-w11-smp-observability/README.md)).

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

| Supporting file | Load it for |
|---|---|
| [01-scope-and-foundations.md](01-scope-and-foundations.md) | goal-to-baseline ledger, assumed-contract failure boundaries, scope classification |
| [02-architecture-and-state.md](02-architecture-and-state.md) | slot model, ownership, sender/receiver split, failure model |
| [03-code-contracts-notification-slot.md](03-code-contracts-notification-slot.md) | slot layout, kind encoding, sender/receiver protocol contracts |
| [04-code-contracts-notification-api.md](04-code-contracts-notification-api.md) | notify/poll/wait contracts, targeting, accounting |
| [05-implementation-workflow.md](05-implementation-workflow.md) | ordered implementation steps |
| [06-validation-and-handoff.md](06-validation-and-handoff.md) | validation matrix, failure model, handoff checklist |

Before editing it must also follow the Coding Guidelines preflight. This
document is a proposed design; it contains no implementation or validation
claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P3 task book → P3-W07 plan → this
design → Coding Guidelines. Binding constraints:

- The ADR names the mechanism class: "跨 CPU 操作统一通过 IPI mailbox"
  (cross-CPU operations unified through an IPI mailbox) — ADR §7. The ADR
  P3 roadmap requires "建立 cross-CPU IPI/SGI primitive". The interrupt
  carrier (GIC SGI) is explicitly P6 (task book Reserved split: "P6 owns
  virtual GIC ... and physical interrupt lifecycle"); the P3 deliverable
  is the mailbox/event primitive that W02 and W05 already anticipated by
  reserving their WFE/SEV parking for "P3-W07's event primitive".
- ADR-015 (early SMP), ADR-044/ADR-052 (capability-driven, no platform
  names): the primitive uses no board or platform constants; the wake
  instruction pair (WFE/SEV) is AArch64 architecture, not a platform
  fact.
- The task book requires targeting, acknowledgement/completion-adjacent
  behavior (completion is W08's), invalid/offline behavior, concurrency
  expectations, and diagnosable failure; P3-V07 requires defined and
  recoverable target behavior with explainable arrival/type accounting
  and explicitly does *not* prove a general communication facility.
- The plan's out-of-scope list removes general message queues, RPC, guest
  interrupts, vIRQ delivery, scheduler policy, and detailed
  interrupt-controller mechanics.
- [P3-W06](../p3-w06-concurrency-synchronization/README.md)'s policy is
  binding: the slot word follows AP-1 (single-variable publication), the
  WFE/SEV semantics and their bounds live here (BW-3 ownership), and the
  idle-context `wait` is the sanctioned BW-5 exception.
- [P3-W04](../p3-w04-per-cpu-runtime/README.md) owns the
  `NotificationSlot` placement (cache-line aligned, zeroed at allocation);
  W07 owns its contents from its init point onward.
- [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md)'s `OnlineSet` is
  the targeting universe; [P3-W05](../p3-w05-smp-boot-synchronization/README.md)'s
  phase gate decides when notification becomes available at all.

Classification:

- **Required** for W07 closure: the slot layout and kind encoding, the
  send/observe protocol with its ordering rules, targeting and its
  invalid/offline outcomes, self-notification behavior, concurrent-sender
  semantics, WFE/SEV wake rules with bounds, arrival/type/coalescing
  accounting, the non-RPC limit statement, and P3-V07 acceptance
  evidence (targeted, self, concurrent, invalid, offline).
- **Reserved** with recorded triggers: interrupt (SGI) carrier for the
  same contract (trigger: [P6-W04](../../../p6/plans/p6-w04-smp-interrupt-routing-sgi.md));
  per-target wake without broadcast side effects (trigger: same); deeper
  queues or payload expansion (trigger: an approved consumer design — the
  one-pending-event model is the plan's "minimal type information");
  retry/reliable delivery (trigger: an approved design that needs it —
  P3 explicitly has none); scheduler-integration wakeup semantics (P7).
- **Out of Scope:** general message queues, RPC, channels (plan);
  guest-visible notification, vIRQ injection (P6/P8); scheduler wakeup
  policy (P7); TLB request semantics (W08); telemetry catalog and event
  ids (W11); the GIC and any interrupt-controller register (P6); timeout
  measurement via timers (P6 baseline owns timing).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Targeted CPU-to-CPU notification with minimal type information | [slot/protocol](03-code-contracts-notification-slot.md) §2–§4 | P3-V07 (W07-DV01, DV02) |
| Defined self-notification | [api](04-code-contracts-notification-api.md) §3 | P3-V07 (W07-DV03) |
| Concurrent sender behavior | [protocol](03-code-contracts-notification-slot.md) §5 | P3-V07 (W07-DV04) |
| Defined invalid/offline-target outcomes | [targeting](04-code-contracts-notification-api.md) §2 | P3-V07 (W07-DV05) |
| Delivery, accounting, and error/failure outcomes | [protocol](03-code-contracts-notification-slot.md) §6, [accounting](04-code-contracts-notification-api.md) §4 | P3-V07 (W07-DV02, DV06) |
| Integration with TLB transport, telemetry, stress, regression consumers | [handoff](06-validation-and-handoff.md) §3; [integration map](02-architecture-and-state.md) §7 | W07 closure review (W07-DV07) |
| Host-only scope and platform-capability boundary review | [scope](01-scope-and-foundations.md) §2.3 | W07 closure review (W07-DV07) |
| Reusable event contract and explicit non-RPC limit recorded | [limits](04-code-contracts-notification-api.md) §5; [handoff](06-validation-and-handoff.md) §3 | W07 closure review (W07-DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p3-implementation-designs`):
P0 documentation scaffold only — no workspace, no sources, no cross-CPU
code. The P3 sibling designs W01–W05 exist as proposed designs on this
branch; W06 exists as a proposed design; W08–W15 are being prepared in
parallel. W07 consumes W03/W04/W06 as upstream contracts (published
designs for W03/W04/W06; plan-level goals for the parallel siblings) and
serves W08 and W11–W15 downstream, referenced by path and P3-Wxx ID. The
task book's P2/P1 inputs remain conditions for implementation, not claims.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| A minimal event primitive exists | No cross-CPU event path exists; W02's parked loop and W05's coordinator wait busy-poll with WFE explicitly Reserved for this package | The per-CPU `NotificationSlot` protocol plus `notify`/`poll`/`wait` with defined outcomes | "Available for later coordination" means W08 and the stress harness can build on it without redesigning it | W07 (this design) | W07-DV01/DV02 |
| Targeting is safe against invalid/offline CPUs | Nothing targets anything | Targeting through W03's `OnlineSet` + W05's phase gate, fail-closed errors, no slot write on refusal | A send to a failed/never-started CPU would write memory a parked CPU will never observe — an unaccountable event; P3-V07 forbids it | W07 targeting rule; W03 gate; W05 gate | W07-DV05 |
| Self-notification is defined | Undefined | Same-protocol local delivery, documented as such | An undefined case becomes an ad-hoc convention in the first consumer | W07 (stage-local freedom, recorded) | W07-DV03 |
| Concurrent senders have defined behavior | Undefined | Hardware-CAS serialization with monotonic sequence and defined coalescing | Two senders racing a two-word protocol would corrupt the slot; CAS makes the race structurally safe | W07 protocol | W07-DV04 |
| Arrival/type accounting is explainable | No accounting exists | Slot-resident counters (arrivals, coalesced, per-kind) with the coalescing limitation stated | P3-V07 requires explainable accounting; silently dropped events would make accounting unexplainable | W07 (counters in the W04-reserved slot); W11 catalog later | W07-DV06 |
| Wake without interrupts | No GIC at P3 (P6) | WFE/SEV wake pair with broadcast side effects documented and bounded | The primitive must actually reach an idle CPU or consumers must poll forever | W07 (WFE/SEV ownership per W06 BW-3) | W07-DV02 |
| Non-RPC limit is explicit | n/a | The limit statement (one pending event, 8-bit payload, no delivery guarantee beyond observability, no retry) | The plan requires the boundary recorded so no consumer assumes reliability | W07 | W07-DV07 |

No ledger row requires fixing a crate name, an interrupt-controller
design, or runtime policy owned elsewhere; no new decision blocker is
outstanding here.

## Resolved design decisions and their authority

1. **Transport is a mailbox slot plus WFE/SEV wake, not an interrupt.**
   Rationale: the GIC and SGI routing are P6's (task book Reserved split
   and plan out-of-scope "detailed interrupt-controller mechanics"); W02
   and W05 already reserved their WFE/SEV parking for this trigger; the
   ADR's "IPI mailbox" names the abstraction, not the carrier. The
   interrupt carrier is Reserved: if P6 delivers SGI routing, the *same*
   notify contract may be carried by an SGI without changing the slot
   protocol — that is the extension point recorded for
   [P6-W04](../../../p6/plans/p6-w04-smp-interrupt-routing-sgi.md), not a
   P3 mechanism.
2. **One pending event per target, monotonically increasing sequence,
   defined coalescing.** The slot is a single packed word
   (sequence/kind/payload) updated by compare-exchange (AP-1 with AP-2's
   CAS form); the receiver tracks its last-observed sequence CPU-locally.
   Under concurrent senders both CAS operations succeed (two sequence
   increments), but only the latest kind/payload is observable — earlier
   concurrent events are *coalesced*, counted, and never silently lost
   (the gap between sequences exposes the count). Rationale: the plan
   requires "concurrent sender behavior" to be *defined*; a two-word
   protocol with a cross-CPU lock would violate W06 BW-2 discipline for
   marginal benefit, and a notification primitive whose worst case is
   "receiver sees the latest event and knows one was coalesced" is
   honestly minimal. Recorded as stage-local design freedom owned here.
3. **Targeting = W03 gate + W05 phase gate, fail closed.** `notify`
   refuses with a named error unless the boot phase is `SmpReady` and the
   target's logical id is in W03's `OnlineSet` snapshot at send time;
   refused sends never touch the target slot. At P3 there are no
   post-SmpReady offline transitions (no hotplug; `Failed` is
   unreachable after admission), so a validated target remains a valid
   target — the staleness window W03's contract documents is empty in
   practice, recorded for the hotplug design to revisit. Rationale: the
   task book's invalid/offline requirement must be structural, and the
   universe authority is W03's, not a W07 copy.
4. **Self-notification is allowed and delivered through the same
   protocol.** A CPU notifying itself performs the identical CAS on its
   own slot (no wake needed — it is running) and returns the same
   success. Rationale: one code path for consumers; "defined" means a
   consumer may rely on the event appearing in its own accounting.
5. **Wake semantics: `sev` after a successful send; `wait` tolerates
   spurious wakeups.** SEV is architecturally broadcast: every WFE-ing
   CPU wakes, re-checks, and re-parks. With ≤ 8 CPUs the herd cost is
   bounded and acceptable at P3; per-target wake is Reserved (P6
   carrier). `wait()` is the sanctioned idle-context terminal state (W06
   BW-5 cross-reference): it is not a wait for progress, it tolerates
   spurious wakes by re-checking the slot, and it must never be called
   while holding a lock or with reception duties unserviced (W06 BW-4).
   Rationale: the plan's minimal primitive plus the siblings' reserved
   triggers name exactly this shape.
6. **Kind space is fixed and small; the payload is opaque.** Four valid
   kind values at P3: 0 = `Doorbell` (generic wake, payload ignored), 1 =
   reserved for [P3-W08](../p3-w08-tlb-shootdown-transport/README.md)
   (claimed in its design), 2–3 reserved for consumer designs via their
   own plans; kind ≥ 4 is refused (`InvalidKind`). The 8-bit payload is
   opaque and interpreted only by the kind's owner. Rationale: "minimal
   type information" in the plan; a fixed small space keeps per-kind
   accounting inside the slot's cache line and forces consumers to claim
   kinds in their designs instead of improvising.
7. **Accounting lives in the slot; W11 owns the catalog later.**
   Receiver-owned counters (total arrivals, coalesced, per-kind) live in
   the W04-reserved slot, giving P3-V07 its evidence surface without
   inventing telemetry; when [P3-W11](../p3-w11-smp-observability/README.md)
   lands, its counter block may aggregate or mirror these (catalog
   ownership is W11's). Sender-side accounting is the `notify` return
   value plus W11 counters when they exist. Rationale: one fact, one
   owner — event *meaning* is W11's, event *counts at the slot* are W07's.
8. **No delivery guarantee, no retry, no reliability.** Observability is
   the only promise: a successful send is eventually observable by a
   target that polls or waits; nothing more. A consumer needing
   reliable/ordered delivery needs a design change to this package.
   Rationale: the plan's explicit non-RPC limit (work step 6).

## Work breakdown and loading order

1. Read [01-scope-and-foundations.md](01-scope-and-foundations.md) for the
   ledger, assumed-contract boundaries, and scope split.
2. Read [02-architecture-and-state.md](02-architecture-and-state.md) for
   the slot model, ownership split, and failure model.
3. Implement per [05-implementation-workflow.md](05-implementation-workflow.md):
   slot and protocol with
   [03](03-code-contracts-notification-slot.md) (steps 1–3), API and
   targeting/accounting with
   [04](04-code-contracts-notification-api.md) (steps 4–6).
4. Record implementation decisions in
   `../p3-w07-cross-cpu-notification-record.md` and evidence in
   `../../verification/p3-w07-cross-cpu-notification-verification.md`
   only when the work is performed. Validation conditions and the handoff
   checklist are in [06-validation-and-handoff.md](06-validation-and-handoff.md).

## Explicitly excluded interfaces

No general message queue, channel, RPC, or reliable-delivery surface; no
TLB request format or completion protocol (W08); no GIC/SGI register
access or interrupt enabling (P6); no scheduler wakeup policy or blocked/
runnable state change (P7); no guest-visible notification or vIRQ (P6/P8);
no telemetry event ids or log formats (W11); no timer-based timeout (P6).
W07 does not modify W02's parked loop or W05's coordinator wait — their
integration with `wait()` is a recorded extension point executed by their
own designs if they are revised; at P3 they remain bounded polls, which is
acceptable and does not block W07. A "notify with payload pointer" or any
heap-carrying event is a scope violation to stop at review.

## Downstream handoff

- **W08** receives the event primitive with kind 1 reserved for it: its
  design claims the kind, defines its payload meaning, and builds
  request/acknowledgement semantics on `notify`/`poll` — W07 carries
  events, not TLB requests.
- **W11** receives the slot-resident counters as the event-activity
  evidence surface and the kind space as the event vocabulary input; the
  catalog and ids are W11's.
- **W12** receives the primitive as the notification-storm stimulus
  surface: concurrent sends, self-notification, invalid/offline targets,
  and the accounting counters are the explainable-accounting inputs
  P3-V12 requires.
- **W13** receives the notification scenario definitions (targeted, self,
  concurrent, invalid, offline) as regression rows' building blocks.
- **W14/P4** receive the notify contract as the Host SMP cross-CPU
  notification foundation; P4 defines its own higher-level protocols per
  the plan's handoff sentence, without extending W07 locally.
- **P6 (future)** receives the recorded extension point: SGI-carried
  delivery of the same contract, if its design chooses it.
