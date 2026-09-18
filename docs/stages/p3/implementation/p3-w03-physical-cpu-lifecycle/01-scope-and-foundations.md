# P3-W03 Scope, Foundations, and Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W03 detailed design](README.md).

## 1. Goal-to-baseline ledger

The entry README carries the goal-to-baseline ledger; this section records
the foundation reasoning and assumed-contract failure boundaries.

### 1.1 What must concretely exist for the plan goal to be true

"Explicit, observable authority for physical-CPU lifecycle and online
eligibility" requires four concrete artifacts:

1. A **total state vocabulary** with encoding and a legal-transition table
   naming one owner per edge —
   [03-code-contracts-cpu-state-machine.md](03-code-contracts-cpu-state-machine.md).
2. A **registry object** seeded from the frozen topology, with immutable
   identity fields and one atomic state word per record —
   [04-code-contracts-registry-and-admission.md](04-code-contracts-registry-and-admission.md).
3. An **admission and gate layer**: exactly-once `Initializing→Online`, the
   eligibility gate, and the online-set snapshot — same file, §3–§5.
4. **Evidence**: transition-event captures and host-side tests proving the
   P3-V03 invariants (no use before local init, no double online, failed
   excluded), per
   [06-validation-and-handoff.md](06-validation-and-handoff.md).

### 1.2 Prerequisites treated as assumed contracts

| Prerequisite | Source plan | Assumed content | Failure boundary if delivered differently |
|---|---|---|---|
| Frozen topology inputs and the identity/class vocabulary | [P3-W01](../p3-w01-cpu-topology-inputs/README.md) | `TopologyInputs` (dense logical ids, classes, boot designation, ≤ 8 entries), `LogicalCpuId`, `HardwareCpuId` | If W01 delivers a different shape, resolve as a design conflict between the two designs before implementation; do not adapt silently |
| Terminal start outcomes and named transition requests | [P3-W02](../p3-w02-secondary-cpu-bring-up/README.md) | Requester calls Present→Starting (request), any→Failed (request/timeout failure); the secondary calls Starting→Initializing and secondary-detected any→Failed | If W02's flow needs an edge this design forbids, that is a cross-design conflict to resolve, not a local table edit |
| Rendezvous completion point for admission | [P3-W05](../p3-w05-smp-boot-synchronization/README.md) | W05's coordinator calls Initializing→Online per CPU when its declared readiness condition holds | If W05 needs admission at a different point, resolve between designs; admission authority never moves to W05's code — only the call does |
| Small-allocation contract for the registry build | [P2-W05](../../../p2/plans/p2-w05-dynamic-small-allocation.md) | Boot-phase small-object allocation with explicit failure, callable on the boot CPU pre-release | Build failure is fatal boot-critical (P0 panic policy); no static fallback is authorized by this design |
| Diagnostics/trace governance | [P0-W12](../../../p0/plans/p0-w12-logging-diagnostic-baseline.md), [P0-W13](../../../p0/plans/p0-w13-trace-event-namespace-baseline.md) | Logging levels and trace-event namespace for lifecycle events | A missing namespace for lifecycle events is a coordination item with [P3-W11](../p3-w11-smp-observability/README.md), not a private scheme |

### 1.3 Why no hidden essential deliverable remains

- The plan's "online-set admission rules" outcome is realized structurally
  (CAS admission + gate + snapshot), not as documentation; workflow steps
  3–4 build it and DV03/DV04 test it.
- "Integrate lifecycle consumers" (plan step 3) is realized by the handoff
  contracts in the entry README and the gate's use in W04/W05/W07 — the
  design defines the surfaces and records the consumers; actual consumer
  wiring is those packages' designs.
- "Collect lifecycle/online-set acceptance evidence, including
  failed-start handling" (plan step 5) is bounded: single-pass captures
  with the W02 failure inputs where the environment allows, host-side
  tests for the invariants, and the repeated matrix explicitly deferred
  to [P3-W13](../p3-w13-qemu-smp-regression/README.md).

## 2. Scope classification

### 2.1 Required

- `CpuLifecycleState` vocabulary and packed encoding, including the
  declared-unreachable reserved states.
- Legal-transition table with per-edge owners and refusal diagnostics.
- `CpuRegistry` and per-CPU `PhysicalCpuRecord` (identity immutable, state
  atomic), built once from `TopologyInputs`.
- Exactly-once online admission; eligibility gate; online-set snapshot.
- Transition events with CPU/from/to/cause attribution (catalog: W11).
- Registry-state diagnostic rendering (SMP-ready dump surface).
- Host-side unit tests for the transition table, admission races (where
  expressible), gate behavior, and snapshot consistency; QEMU capture of
  registry state at SMP-ready including a failed-start case.

### 2.2 Reserved (must not block a future design; not implemented now)

- `Offline` runtime transition (online→offline); trigger: approved
  hotplug/power design.
- `Stopping` graceful teardown path; trigger: approved teardown design.
- `Suspended` low-power state; trigger: approved power-management design.
- `Failed` recovery/re-admission; trigger: approved recovery design.
- Post-boot record creation/removal (hotplug); trigger: approved hotplug
  design that also revisits W01's frozen-topology contract and this
  registry's boot-static assumption together.

### 2.3 Out of Scope

- Issuing start requests and the PSCI boundary (W02).
- Per-CPU runtime areas, stacks, CPU-local accessors (W04).
- Boot phases and the rendezvous protocol (W05; W03 only supplies the
  transition W05 calls).
- Lock primitives and general atomic policy (W06; W03 uses atomics only).
- Notification/TLB transport semantics (W07/W08).
- Exception/interrupt per-CPU enablement (W09).
- vCPU lifecycle, scheduler policy, guest CPU objects (P7/P4+).
- Runtime hotplug (stage out of scope).
- Board/SoC-specific lifecycle behavior (ADR-043/ADR-052).
