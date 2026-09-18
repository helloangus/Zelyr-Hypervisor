# P3-W07 Scope, Foundations, and Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W07 detailed design](README.md).

## 1. Goal-to-baseline ledger

The entry README carries the goal-to-baseline ledger; this section records
the foundation reasoning and the assumed-contract failure boundaries.

### 1.1 What must concretely exist for the plan goal to be true

"A minimal, safe physical-CPU event primitive for later coordination"
requires five concrete artifacts:

1. The **slot contract**: layout, encoding, and ordering rules for the
   per-CPU reception slot W04 reserves —
   [03-code-contracts-notification-slot.md](03-code-contracts-notification-slot.md)
   §2–§3.
2. The **protocol contracts**: sender CAS protocol with concurrent-sender
   semantics and coalescing; receiver observation protocol — same file
   §4–§6.
3. The **API surface**: `notify` with targeting and named errors,
   `poll`, `wait`, and the accounting counters —
   [04-code-contracts-notification-api.md](04-code-contracts-notification-api.md).
4. The **wake rules**: WFE/SEV ownership, bounds, and the idle-wait
   classification — same file §6, cross-referenced by W06 BW-3/BW-5.
5. **Evidence**: targeted, self, concurrent, invalid, and offline
   behaviors demonstrated within declared limits —
   [06-validation-and-handoff.md](06-validation-and-handoff.md).

Without (1)–(4) the primitive is prose; without (5) P3-V07's
"defined and recoverable target behavior with explainable arrival/type
accounting" has nothing to review.

### 1.2 Prerequisites treated as assumed contracts

| Prerequisite | Source plan/design | Assumed content | Failure boundary if delivered differently |
|---|---|---|---|
| `NotificationSlot` reserved in every `PerCpuArea` | [P3-W04](../p3-w04-per-cpu-runtime/README.md) 03 §5.1 | Cache-line aligned, fixed capacity, zeroed at allocation; W07 owns contents after its init point | If the slot does not exist or is smaller than this design's layout, that is a W04/W07 design conflict to resolve — W07 does not invent a second per-CPU area |
| Targeting universe | [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md) 04 §4–§5 | `OnlineSet` snapshot; `Eligibility` gate; no post-admission `Failed` at P3 | If W03's online set or eligibility shape differs, the targeting rule is re-pointed by a recorded change, never a W07-private copy |
| Availability gate | [P3-W05](../p3-w05-smp-boot-synchronization/README.md) 03 §1, §5 | `BootPhase` with `SmpReady`; acquire-read gate pattern | If the phase vocabulary differs, the gate check changes with W05's owner; W07 does not add a second phase word |
| Synchronization policy | [P3-W06](../p3-w06-concurrency-synchronization/README.md) | AP-1/AP-2 ordering patterns; BW-3 (WFE/SEV owned here), BW-4 (reactive wait), BW-5 (idle wait as sanctioned exception); MIS items | If W06's rules change, this design's protocol statements are updated in the same change; a conflict is raised, not absorbed |
| Lifecycle authority | [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md) | `Failed` is unreachable after admission at P3 (no hotplug) — grounds the "validated target stays valid" statement | A hotplug design invalidates that statement and must revisit W07 targeting (recorded revisit trigger) |
| Telemetry governance | [P0-W13](../../../p0/plans/p0-w13-trace-event-namespace-baseline.md), [P3-W11](../p3-w11-smp-observability/README.md) | Event catalog/ids owned by W11; W07 provides counters and activity, not catalog entries | Missing W11 does not block W07: slot counters are the P3 evidence surface (README decision 7) |

### 1.3 Why no hidden essential deliverable remains

- "Integrate the event boundary with TLB transport, telemetry, stress, and
  regression consumers" (plan step 3) is realized by the kind reservation
  (1 → W08), the counter surface (→ W11), the stimulus/accounting surfaces
  (→ W12/W13), and the handoff section — designed surfaces, not implied.
- "Collect targeted, self, concurrent, invalid, and offline-target
  evidence" (plan step 5) is bounded: W07 delivers the behaviors and their
  host-side demonstrations within declared limits; repeated SMP execution
  at scale belongs to W12/W13 (stated in the matrix).
- "Review host-only scope and platform-capability boundaries" (plan step
  4) is the scope statement in §2.3 plus the no-platform-constant rule —
  verified by the DV07 review rather than left implicit.

## 2. Scope classification

### 2.1 Required

- Slot layout, packed word encoding, kind space (0 Doorbell; 1 W08; 2–3
  consumer-reserved; ≥ 4 invalid), opaque 8-bit payload.
- Sender protocol (CAS, release), receiver protocol (acquire observe,
  local last-seen tracking), coalescing semantics and its counter.
- `notify` with targeting (W03 `OnlineSet` + W05 `SmpReady` gate) and
  named errors; self-notification; `poll`; `wait` (idle context).
- WFE/SEV wake rules with the broadcast-side-effect statement.
- Slot-resident accounting (arrivals, coalesced, per-kind) and the
  non-RPC limit statement.
- Host-side protocol evidence within declared limits.

### 2.2 Reserved (must not block a future design; not implemented now)

- SGI/interrupt carrier for the same contract; per-target wake; trigger:
  [P6-W04](../../../p6/plans/p6-w04-smp-interrupt-routing-sgi.md).
- Larger payload, multi-event queues, or ordering guarantees; trigger: an
  approved consumer design (P3's model is one pending event, latest wins
  under concurrency).
- Retry/acknowledged reliable delivery; trigger: an approved design that
  requires it (W08 builds its own acknowledgement at its layer).
- Scheduler wakeup integration (blocked→runnable transitions); trigger:
  P7's scheduler design.
- Post-hotplug targeting staleness handling; trigger: an approved hotplug
  design revisiting W03/W07.

### 2.3 Out of Scope

- General message queues, RPC, channels, reliable transport (plan).
- Guest notification, vIRQ injection, guest IPI (P6/P8).
- GIC/SGI registers, interrupt enabling, interrupt-controller mechanics
  (P6).
- Scheduler policy and wakeup semantics (P7).
- TLB request semantics, shootdown protocol, completion collection (W08).
- Telemetry catalog, event ids, log formats (W11); timers and wall-clock
  timeouts (P6).
- Board/SoC constants of any kind (ADR-043/ADR-052); the wake pair is
  AArch64 architectural, not platform configuration.
- Any modification of W02's parked loop or W05's coordinator wait (their
  WFE integration is their recorded extension point).

## 3. Resolved decisions — authority notes

The entry README carries the numbered decisions; authority basis:

- Decision 1: ADR §7 (IPI mailbox) + ADR P3 roadmap + task book Reserved
  split (P6 owns GIC) + the W02/W05 reserved triggers; carrier choice is
  stage-local freedom *within* the plan's "minimal event primitive"
  boundary.
- Decision 2: plan's "concurrent sender behavior" + W06 AP/BW rules;
  stage-local freedom owned here with the recorded coalescing limitation.
- Decision 3: task book invalid/offline requirement; targeting universe
  authority is W03's; availability gate is W05's (frozen-contract tier).
- Decision 4: plan's "defined self-notification"; stage-local freedom.
- Decision 5: W02/W05 reserved triggers + W06 BW-3 ownership transfer to
  this design; bounded by the ≤ 8 CPU inventory (W01).
- Decision 6: plan's "minimal type information"; kind-claiming forces
  consumer designs to own their types.
- Decision 7: P3-V07 accounting wording; W11 catalog ownership per task
  book (ADR-048 governance).
- Decision 8: plan work step 6 and acceptance sentence (explicit non-RPC
  limit).
