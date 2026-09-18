# P3-W08 TLB Shootdown Transport — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The cross-CPU request, acknowledgement, and completion transport
for future TLB invalidation work — targeting, single/mask/broadcast
selection, request consumption, completion collection, timeout and failure
diagnostics — required by
[P3-W08](../../plans/p3-w08-tlb-shootdown-transport.md).  
**Owner/change context:** P3-W08 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P3-W08. It defines the
*transport only*: how an initiator selects targets, publishes an opaque
request to their reception slots, wakes them through
[P3-W07](../p3-w07-cross-cpu-notification/README.md), collects
acknowledgements, and diagnoses timeout/failure — all with defined
behavior under concurrency, invalid targets, and offline exclusion. It
deliberately does **not** define what is invalidated: Stage-2 address
spaces, IPA/VA/VMID/ASID selection, the TLBI operation choice, its
shareability/barrier requirements, and guest TLB semantics are P4's
(through the [P3-W14](../p3-w14-p4-smp-handoff/README.md) handoff); the
request payload is an opaque descriptor word whose interpretation contract
P4 owns. At P3 the target-side bound operation is an explicit documented
no-op placeholder, and this design records that gap as its central
boundary: nothing here asserts that translation invalidation has been
implemented.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

| Supporting file | Load it for |
|---|---|
| [01-scope-and-foundations.md](01-scope-and-foundations.md) | goal-to-baseline ledger, assumed-contract failure boundaries, scope classification |
| [02-architecture-and-state.md](02-architecture-and-state.md) | transport model, roles, ownership, the single-flight rule, failure model |
| [03-code-contracts-transport-request.md](03-code-contracts-transport-request.md) | `TargetMask`, slot layout, request encoding, target consumption contract |
| [04-code-contracts-transport-initiator.md](04-code-contracts-transport-initiator.md) | initiate/collect/timeout contracts, broadcast and exclusion rules |
| [05-implementation-workflow.md](05-implementation-workflow.md) | ordered implementation steps |
| [06-validation-and-handoff.md](06-validation-and-handoff.md) | validation matrix, failure model, handoff checklist |

Before editing it must also follow the Coding Guidelines preflight. This
document is a proposed design; it contains no implementation or validation
claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P3 task book → P3-W08 plan → this
design → Coding Guidelines. Binding constraints:

- The ADR invariant "任何跨 CPU 修改 Stage-2/IRQ route 的操作必须定义同步与
  invalidation 语义" (any cross-CPU Stage-2/route modification must define
  its synchronization and invalidation semantics) is the reason this
  transport exists *before* Stage-2 does: P4 will inherit a transport
  whose ordering contract is already stated, instead of inventing one
  under pressure. ADR-018 names TLB shootdown as a Stage-2 capability
  built on these foundations.
- The task book's Reserved split is explicit: "P4 chooses Stage-2
  address-space/VMID/IPA semantics and the concrete TLB invalidation
  operation; P3 supplies only transport and completion foundations."
  P3-V08's passing condition likewise "asserts no Stage-2 TLBI semantics."
- ADR-032 makes GICv3 the interrupt baseline, but interrupt delivery is
  P6; at P3 the wake carrier is
  [P3-W07](../p3-w07-cross-cpu-notification/README.md)'s WFE/SEV event
  (kind 1 claimed by this design per W07's reservation).
- ADR-044/ADR-052: no assumption about hardware TLB broadcast (DVM)
  enters P3 — whether P4's operation uses broadcast TLBI or this per-CPU
  transport is P4's design; the transport exists for the non-broadcast
  case and for operations lacking a broadcast form.
- [P3-W06](../p3-w06-concurrency-synchronization/README.md) is binding:
  the single-flight initiation lock is `SpinLock` class `Infrastructure`
  (LOL rank 3), completion polling obeys BW-1 bounds, and the
  reactive-wait rule BW-4 is what makes the design deadlock-free.
- [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md)'s `OnlineSet` is
  the targeting universe; [P3-W04](../p3-w04-per-cpu-runtime/README.md)
  owns the `TlbReceptionSlot` placement (cache-line aligned, zeroed at
  allocation); this design owns its contents from its init point.

Classification:

- **Required** for W08 closure: `TargetMask` and the target-selection
  rules (single/mask/broadcast, invalid/offline exclusion), the request
  slot layout and encoding, the target consumption contract with the
  no-op bound operation, the initiator's request/acknowledge/collect
  protocol with the single-flight rule, bounded completion collection
  with timeout and unacked-set diagnostics, and P3-V08 acceptance
  evidence.
- **Reserved** with recorded triggers: pipelined/multiple outstanding
  transport operations (trigger: P4's throughput design — P3 is
  single-flight by recorded decision); timer-based timeout measurement
  (trigger: P6 timer baseline; P3 uses the W02-style bounded poll);
  interrupt-carried delivery of requests (trigger: P6 SGI routing,
  reusing W07's extension point); SMMU/DVM-side invalidation transport
  (trigger: P14's IOMMU design — device TLBs are a different transport);
  a general "remote operation" framework (trigger: an approved design —
  this transport is deliberately specific to the TLB use).
- **Out of Scope:** Stage-2 address spaces, IPA/VMID/ASID selection, TLBI
  operation choice, shareability-domain decisions, and guest TLB
  semantics (P4); vCPU or guest targeting (P4+); GIC/SGI registers and
  interrupt enabling (P6); telemetry catalog (W11); the interpretation of
  the descriptor word at P3 (P4's contract via W14); timer-based
  timeouts (P6).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Single-target and mask/broadcast selection | [TargetMask](03-code-contracts-transport-request.md) §2, [initiator](04-code-contracts-transport-initiator.md) §2 | P3-V08 (W08-DV01, DV02) |
| Acknowledgement/completion | [target consumption](03-code-contracts-transport-request.md) §5, [collection](04-code-contracts-transport-initiator.md) §3 | P3-V08 (W08-DV03) |
| Invalid/offline exclusion | [selection rules](04-code-contracts-transport-initiator.md) §2 | P3-V08 (W08-DV04) |
| Concurrent-request behavior | [single-flight](02-architecture-and-state.md) §5 | P3-V08 (W08-DV05) |
| Timeout/failure diagnostics | [timeout](04-code-contracts-transport-initiator.md) §4 | P3-V08 (W08-DV06) |
| Transport boundary does not predesign Stage-2 | [boundary](01-scope-and-foundations.md) §2.3; [descriptor rule](03-code-contracts-transport-request.md) §4 | W08 closure review (W08-DV07) |
| P4-facing transport contract and explicit semantic gap recorded | [handoff](06-validation-and-handoff.md) §3 | W08 closure review (W08-DV07) |
| Integration with observability, stress, regression consumers | [integration map](02-architecture-and-state.md) §7 | W08 closure review (W08-DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p3-implementation-designs`):
P0 documentation scaffold only — no workspace, no sources, no transport
code. Sibling P3 designs W01–W07 exist as proposed designs on this branch;
W09–W15 are being prepared in parallel. W08 consumes W03/W06/W07 (and
W04's slot reservation) as upstream contracts and serves W11–W15 and P4
downstream, referenced by path and P3-Wxx ID. The task book's P1/P2 inputs
remain conditions for implementation, not claims.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| A future TLB invalidation work can use a request/ack/completion transport | No transport exists; W04 reserved the `TlbReceptionSlot`; W07 reserved kind 1 | The slot encoding, consumption contract, and initiator protocol with stated orderings | "Can use" means P4 designs against a stated contract, not against nothing | W08 (this design) | W08-DV03/DV07 |
| Target/mask/broadcast selection is defined | No selection concept exists | `TargetMask` over the dense logical id space with validation against W03's `OnlineSet` and defined initiator exclusion | Target selection that ignores online-ness would send requests to parked CPUs — unaccountable completions | W08 mask type; W03 universe | W08-DV01/DV02 |
| Acknowledgement and completion are diagnosable | None | Per-target state machine (Empty→Pending→Completed) with sequence tags and release/acquire edges | Completion must mean something checkable, or P4 inherits a guessed protocol | W08 state machine | W08-DV03 |
| Invalid/offline exclusion | Nothing excludes | Fail-closed selection: mask ∩ online set, exclusions reported | P3-V08's wording requires the exclusion be *diagnosable*, not silent | W08 rule; W03 gate | W08-DV04 |
| Concurrent-request behavior defined | Undefined | The single-flight initiation rule with its deadlock analysis (BW-4 reactive wait) | Two concurrent initiators would need per-target arbitration — complexity P3 cannot validate | W08 (stage-local freedom, recorded; revisit trigger P4) | W08-DV05 |
| Timeout/failure diagnosable | No timer exists (P6 owns timers) | Bounded-poll collection with a recorded bound and the unacked set in the timeout result | An unbounded collection could hang the initiator forever with no diagnostic | W08 (W02 POLL_BOUND pattern) | W08-DV06 |
| Explicit semantic gap recorded | n/a | The no-op bound operation statement + the opaque-descriptor rule | Work step 6 and P3-V08 require the gap be explicit, not implied | W08 | W08-DV07 |

No ledger row requires fixing crate names, Stage-2 semantics, or
interrupt-controller design; no new decision blocker is outstanding here.

## Resolved design decisions and their authority

1. **Transport only; the descriptor is opaque; the P3 bound operation is
   a documented no-op.** The request carries a u64 descriptor word this
   package never interprets; a completing target at P3 executes the
   transport's `TransportNoop` placeholder and acknowledges. Rationale:
   the task book's Reserved split and P3-V08's "asserts no Stage-2 TLBI
   semantics"; the alternative (designing VA/VMID fields now) would
   pre-design P4 and freeze semantics no authority has fixed.
2. **Single-flight initiation: at most one outstanding transport
   operation system-wide, guarded by a W06 `SpinLock` (class
   `Infrastructure`), held from request publication through completion
   collection.** Concurrent initiators therefore have exactly defined
   behavior (serialize); targets are never blocked (lock-free
   consumption). Rationale: the plan requires defined concurrent
   behavior; per-target pipelining needs throughput evidence P3 cannot
   produce; the lock is bounded and its class citable. The deadlock risk
   this creates (an initiator waiting on a target that is waiting to
   initiate) is removed by W06's BW-4 reactive-wait rule, made binding
   for W08 in [04 §5](04-code-contracts-transport-initiator.md).
   Revisit trigger recorded: P4 pipelining per address space.
3. **Broadcast = online set minus initiator; the initiator is never a
   transport target of its own request.** A shootdown's local invalidation
   is part of P4's *operation*, executed by the initiator itself; the
   transport's completion collection covers the remote set only.
   Rationale: conflating local work with remote transport would make
   "completed" mean two things; P3-V08's transport accounting stays
   clean.
4. **Completion ordering is the contract P4 inherits: a target's
   Completed acknowledgement is release-stored after it has fully
   consumed the request and executed its bound operation; the initiator
   acquire-loads it.** The transport requires P4's bound operation to
   contain whatever barriers its TLBI semantics need (so that "ack
   observed" implies "invalidation performed and visible") — stated as a
   requirement on the P4 contract, not implemented here. Rationale: the
   ADR cross-CPU invariant; this is precisely the "synchronization and
   invalidation semantics" P3 must define at transport level without
   choosing the TLBI operation.
5. **Timeout by bounded poll, unacked set returned; no timer.** The
   collection loop has a recorded bound constant (the W02 `POLL_BOUND`
   pattern); on exhaustion the initiator gets `TimedOut { unacked }` and
   the per-target slot states remain readable for diagnosis. A target
   that never completes leaves its request Pending; a later request to
   the same target supersedes it by sequence. Rationale: no timer exists
   before P6; a hung initiator with no diagnostic is the failure P3-V08
   forbids; superseding-by-sequence keeps the transport usable after a
   timeout (recorded recovery behavior).
6. **Wake through W07 kind 1; targets consume on wake or poll.** This
   design claims kind 1 per W07's reservation and defines its payload as
   opaque to W07 (a hint byte W08 may use, e.g., zero at P3). Targets
   run W08's consumption from their idle/poll path on wake. Rationale:
   W07 owns the event layer; W08 owns its kind's meaning; consumption is
   poll-driven so a CPU is never *required* to sleep to make progress.
7. **Targeting validation is fail-closed and diagnosable.** The requested
   mask is intersected with W03's `OnlineSet` at initiation time;
   requested-but-excluded CPUs are reported in the result; unknown ids,
   offline, or pre-`SmpReady` initiation are named errors with no slot
   write. At P3 there are no post-`SmpReady` offline transitions (no
   hotplug), so the staleness window W03 documents is empty in practice —
   recorded for the hotplug design to revisit. Rationale: mirrors W07's
   targeting rule at transport level, where exclusions are *expected
   inputs* (mask requests) rather than only errors.

## Work breakdown and loading order

1. Read [01-scope-and-foundations.md](01-scope-and-foundations.md) for the
   ledger, assumed-contract boundaries, and scope split.
2. Read [02-architecture-and-state.md](02-architecture-and-state.md) for
   the role model, ownership, single-flight rule, and failure model.
3. Implement per [05-implementation-workflow.md](05-implementation-workflow.md):
   target-side with [03](03-code-contracts-transport-request.md)
   (steps 1–3), initiator-side with
   [04](04-code-contracts-transport-initiator.md) (steps 4–6).
4. Record implementation decisions in
   `../p3-w08-tlb-shootdown-transport-record.md` and evidence in
   `../../verification/p3-w08-tlb-shootdown-transport-verification.md`
   only when the work is performed. Validation conditions and the handoff
   checklist are in [06-validation-and-handoff.md](06-validation-and-handoff.md).

## Explicitly excluded interfaces

No Stage-2 page-table, VMID, IPA, or address-space type; no TLBI
instruction sequence, barrier placement, shareability decision, or cache
maintenance operation (P4's operation content); no guest-visible
interface; no GIC/SGI access (P6); no SMMU/DVM transport (P14); no
general remote-call framework; no timer. The descriptor word must gain no
P3-side interpretation — a helper that "conveniently" decodes it at P3 is
the scope violation this package exists to prevent. W08 also defines no
telemetry catalog entries (W11) and does not modify W03/W04/W06/W07
surfaces.

## Downstream handoff

- **P4** (through [P3-W14](../p3-w14-p4-smp-handoff/README.md)) receives
  the transport contract: target selection, slot protocol, completion
  ordering requirement, single-flight rule and its revisit trigger, the
  opaque descriptor, and the obligation to define (a) the descriptor
  interpretation and (b) the target-side bound operation with its
  barriers. P4 must not assume pipelining, reliability beyond the
  timeout semantics, or interrupt-carried delivery.
- **W11** receives the transport activity surface (per-target states,
  initiation/completion/timeout outcomes) as event-content input; the
  catalog is W11's.
- **W12** receives the transport as the shootdown stimulus surface:
  concurrent initiations, invalid/offline masks, timeout injection (a
  target that stops consuming), and the accounting invariants.
- **W13** receives the transport scenario definitions as regression-row
  inputs (request/ack evidence per CPU count).
- **W14/W15** receive the semantic-gap statement as mandatory content for
  the P4 handoff and stage documentation.
