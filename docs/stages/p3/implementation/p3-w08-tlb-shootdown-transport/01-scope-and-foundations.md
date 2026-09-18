# P3-W08 Scope, Foundations, and Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W08 detailed design](README.md).

## 1. Goal-to-baseline ledger

The entry README carries the goal-to-baseline ledger; this section records
the foundation reasoning and the assumed-contract failure boundaries.

### 1.1 What must concretely exist for the plan goal to be true

"A future TLB-invalidation transport can target, acknowledge, and diagnose
physical CPUs" requires five concrete artifacts:

1. The **selection surface**: `TargetMask`, validation, exclusion
   reporting — [03-code-contracts-transport-request.md](03-code-contracts-transport-request.md)
   §2 and
   [04-code-contracts-transport-initiator.md](04-code-contracts-transport-initiator.md)
   §2.
2. The **request/completion protocol**: slot encoding, per-target state
   machine, ordering edges, and the consumption contract —
   [03](03-code-contracts-transport-request.md) §3–§6.
3. The **initiator protocol**: initiation under single-flight, wake,
   bounded collection, timeout with unacked diagnostics —
   [04](04-code-contracts-transport-initiator.md) §2–§5.
4. The **deadlock-safety argument**: the single-flight + reactive-wait
   analysis that makes the protocol's only wait non-deadlocking —
   [02-architecture-and-state.md](02-architecture-and-state.md) §5.
5. **Evidence**: request/ack, mask/exclusion, concurrency, timeout, and
   consumption behaviors demonstrated within declared limits —
   [06-validation-and-handoff.md](06-validation-and-handoff.md).

Without (1)–(3) P4 has no contract; without (4) the concurrency story is
asserted rather than argued; without (5) P3-V08 has nothing to review.

### 1.2 Prerequisites treated as assumed contracts

| Prerequisite | Source plan/design | Assumed content | Failure boundary if delivered differently |
|---|---|---|---|
| `TlbReceptionSlot` reserved per CPU | [P3-W04](../p3-w04-per-cpu-runtime/README.md) 03 §5.2 | Cache-line aligned, fixed capacity, zeroed at allocation; W08 owns contents from its init point | If absent/undersized: cross-design conflict with the W04 owner; W08 does not invent second storage |
| Notification primitive, kind 1 reservation | [P3-W07](../p3-w07-cross-cpu-notification/README.md) | `notify`/`poll` with gates and accounting; kind space with 1 reserved for this design | If W07's kind space lands differently, re-point the claim by recorded change; W08 does not build a second wake mechanism |
| Targeting universe | [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md) 04 §4–§5 | `OnlineSet` snapshot; no post-admission `Failed` at P3 | Different shape → re-point by recorded change; no W08-private online-set copy |
| Lock and wait rules | [P3-W06](../p3-w06-concurrency-synchronization/README.md) | `SpinLock` + `LadderClass::Infrastructure`; BW-1 bounds; BW-4 reactive wait; BW-2 no-wait-under-lock (the initiation lock's *waiters* hold nothing) | Conflicts are raised with the W06 owner; W08 does not bend the ladder locally |
| Availability gate | [P3-W05](../p3-w05-smp-boot-synchronization/README.md) | `BootPhase::SmpReady` acquire-read | Different phase vocabulary → resolve with W05's owner |
| TLB/VMID semantics | P4 (future; via [P3-W14](../p3-w14-p4-smp-handoff/README.md)) | Not assumed at P3 — the descriptor is opaque and the bound operation is the recorded no-op | If P4 requires transport changes (pipelining, richer descriptors), that is a W08 design change with P4, not a P4-local fork |

### 1.3 Why no hidden essential deliverable remains

- "Integrate delivery and accounting with observability, stress,
  regression, and P4 handoff" (plan step 3) is realized by the activity
  surface (→ W11), the stimulus/invariant surface (→ W12), the scenario
  definitions (→ W13), and the transport contract section of the handoff
  (→ W14/P4) — designed surfaces, not implied.
- "Review the transport boundary to ensure it does not predesign
  Stage-2" (plan step 4) is enforced by the opaque-descriptor rule and
  the no-op bound operation, checked by the DV07 review — a named review
  step, not prose.
- "Collect request/acknowledgement and failure-path acceptance evidence"
  (plan step 5) is bounded: W08 delivers the behaviors and host-side
  evidence; QEMU-scale repeated execution is W12/W13 territory (stated
  in the matrix).

## 2. Scope classification

### 2.1 Required

- `TargetMask` newtype; single-target and mask forms; broadcast =
  online-minus-initiator; validation and exclusion reporting.
- Slot layout (`control` word + `descriptor` word), request encoding with
  sequence tags, Empty→Pending→Completed state machine.
- Target consumption contract (lock-free, wake- or poll-driven,
  `TransportNoop` bound operation at P3, release-stored completion).
- Initiator protocol: single-flight `SpinLock`, publish + wake, bounded
  acquire-poll collection, `Completed`/`TimedOut{unacked}`/error results.
- Timeout bound constant with recorded limitation; superseding-by-sequence
  recovery.
- Transport boundary statements (opaque descriptor; no Stage-2 semantics).
- Host-side evidence within declared limits.

### 2.2 Reserved (must not block a future design; not implemented now)

- Pipelined/multiple outstanding transport operations; trigger: P4's
  throughput design (single-flight is a recorded P3 decision with this
  revisit trigger).
- Timer-based timeout measurement; trigger: P6 timer baseline.
- Interrupt-carried request delivery (SGI); trigger: P6 SGI routing,
  via W07's extension point.
- SMMU/DVM/device-TLB invalidation transport; trigger: P14's IOMMU
  design.
- General remote-operation framework; trigger: an approved design that
  needs one — this transport stays specific to the TLB use.

### 2.3 Out of Scope

- Stage-2 address spaces, IPA/VMID/ASID fields, TLBI operation choice,
  shareability/barrier *content*, guest TLB semantics (P4; the transport
  *requires* P4's operation to carry its own barriers — a requirement,
  not an implementation).
- vCPU/guest targeting (P4+); GIC/SGI access (P6); SMMU (P14).
- Interpretation of the descriptor word at P3.
- Telemetry catalog and event ids (W11); timers (P6).
- Any W03/W04/W06/W07 surface modification.
- Performance claims of any kind (P3-V08 measures diagnosability, not
  throughput).

## 3. Resolved decisions — authority notes

The entry README carries the numbered decisions; authority basis:

- Decision 1: task book Reserved split ("P3 supplies only transport and
  completion foundations") + P3-V08's explicit no-semantics clause;
  stage-local freedom covers only the transport shape.
- Decision 2: plan's concurrent-request requirement; W06 lock/ladder
  rules; deadlock-safety from BW-4; revisit trigger recorded for P4.
- Decision 3: transport-accounting cleanliness (one meaning for
  "completed"); P4's operation owns local invalidation.
- Decision 4: ADR cross-CPU invariant ("must define synchronization and
  invalidation semantics") — the ordering contract is the P3 deliverable;
  barrier content stays P4's.
- Decision 5: no timer before P6 (same recorded limitation as W02);
  P3-V08's timeout-diagnosability wording.
- Decision 6: W07 kind-1 reservation; poll-driven consumption keeps
  targets independent of the wake mechanism's health.
- Decision 7: task book invalid/offline requirement; W03 universe
  authority; exclusion reporting demanded by P3-V08.
