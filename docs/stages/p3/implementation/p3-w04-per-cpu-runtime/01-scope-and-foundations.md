# P3-W04 Scope, Foundations, and Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W04 detailed design](README.md).

## 1. Goal-to-baseline ledger

The entry README carries the goal-to-baseline ledger; this section records
the foundation reasoning and assumed-contract failure boundaries.

### 1.1 What must concretely exist for the plan goal to be true

"Every online physical CPU has independent execution and CPU-local state
foundations" requires five concrete artifacts:

1. **Storage**: one page-aligned `PerCpuArea` per CPU with a validated
   header, typed slots, and reserved regions —
   [03-code-contracts-percpu-area.md](03-code-contracts-percpu-area.md).
2. **Execution independence**: one runtime stack per CPU (allocated with
   the areas), plus the recorded transfer rule for W02's provisional
   scaffolding — same file §2, §4.
3. **Access**: a register-based CPU-local mechanism with a safe accessor
   and an installation sequence that runs exactly once per CPU —
   [04-code-contracts-cpu-local-access.md](04-code-contracts-cpu-local-access.md).
4. **Boundaries**: the no-global-current-CPU rule and the
   per-CPU-private/shared-surface split stated as review checks —
   architecture file §5; validated by W04-DV04.
5. **Evidence**: isolation diagnostics per declared CPU count and
   host-side tests, per
   [06-validation-and-handoff.md](06-validation-and-handoff.md).

### 1.2 Prerequisites treated as assumed contracts

| Prerequisite | Source plan | Assumed content | Failure boundary if delivered differently |
|---|---|---|---|
| Topology inputs and identity types | [P3-W01](../p3-w01-cpu-topology-inputs/README.md) | Frozen `TopologyInputs`; dense `LogicalCpuId` (`0..n-1`, n ≤ 8); `HardwareCpuId` | Density or bound deviation is a W01 contract conflict — resolve between designs; do not add a search layer here |
| Lifecycle gate | [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md) | `Eligibility` with `EligibleForLocalInstall`/`Eligible` semantics at install time | A different gate shape is a W03 contract conflict; installation must not proceed ungated |
| Provisional environment transfer | [P3-W02](../p3-w02-secondary-cpu-bring-up/README.md) | Success-side stack ownership transfer; W02's entry tail calls W04 install before the W05 readiness signal | If W02's flow cannot reach the install point, resolve between designs; W04 never installs from the requester side |
| Page allocator | [P2-W04](../../../p2/plans/p2-w04-physical-page-allocation.md) | Page-aligned allocation with explicit OOM; boot-CPU-only use pre-release | Allocation failure is fatal boot-critical (P0 panic policy); no static image arrays as fallback |
| Host Stage-1 map covers the allocations | [P1-W08](../../../p1/plans/p1-w08-host-stage1-address-space.md) | Newly allocated RAM is mapped per the P1 address-space design so areas/stacks are addressable | If allocations fall outside the mapped regions, that is an Architecture Change Request against the P1 address-space design — W04 must not extend the map itself |
| Exception baseline stays vector-side | [P1-W05](../../../p1/plans/p1-w05-el2-exception-entry-baseline.md), [P3-W09](../p3-w09-cpu-local-exception-interrupt/README.md) | Vectors and entry mechanics are P1/W09's; W04 supplies the storage slot only | A request to embed exception state semantics in W04 is a boundary violation to refuse |
| `TPIDR_EL2` availability | [P1-W04](../../../p1/plans/p1-w04-el2-architectural-state-baseline.md) | The P1 baseline does not reserve `TPIDR_EL2` for another purpose | If it does: Architecture Change Request with the P1 owner; W04 does not silently pick another register |

### 1.3 Why no hidden essential deliverable remains

- "Integrate local-state availability with boot synchronization,
  synchronization, notification, exceptions, and audit" (plan step 3) is
  realized by the install-sequence call points (W05 signal), the slot
  reservations (W07/W08/W09/W11), and the invariants handed to W10 — each
  a designed surface, not an implied one.
- "Collect isolation and CPU-identity acceptance evidence" (plan step 5)
  is bounded to what W04's scope produces: per-CPU boot diagnostics and
  host-side tests; repeated/underload evidence is
  [P3-W12](../p3-w12-smp-stress-failure-tests/README.md) and the matrix is
  [P3-W13](../p3-w13-qemu-smp-regression/README.md).

## 2. Scope classification

### 2.1 Required

- `PerCpuArea` layout: header (magic, self pointer, identity, stack
  bounds), typed slots (notification reception, TLB reception,
  exception/interrupt local state, telemetry counter block), reserved
  region (scheduler/current-vCPU capacity, opaque).
- One runtime stack per CPU; `RUNTIME_STACK_SIZE` constant with recorded
  rationale; provisional-stack transfer on success.
- CPU-local access: register-based `current()` with install-time
  validation; lookup table for cross-CPU diagnostic reads only.
- Install sequence: gated by W03, exactly-once, ending in the W05
  readiness signal.
- Isolation diagnostics (per-CPU identity, area and stack addresses).
- Host-side unit tests (layout invariants, install logic with a
  register-fake seam, boundary checks) and QEMU isolation captures.

### 2.2 Reserved (must not block a future design; not implemented now)

- Stack guard pages / overflow detection; trigger: an approved
  host-address-space change owned by the P1-W08 design.
- Generic `PerCpu<T>`-style allocation for arbitrary per-CPU types;
  trigger: a consumer design requiring it (would amend this design).
- Area teardown/reuse on hotplug; trigger: approved hotplug design.
- Boot-stack unification (replacing the boot CPU's P1 boot stack);
  trigger: a later design with the P1 owner.
- Cache-line padding choices beyond the documented slot alignment;
  trigger: W11/W12 contention evidence that justifies a layout change.

### 2.3 Out of Scope

- Notification/TLB slot contents and protocols (W07/W08).
- Exception/interrupt enablement, vectors, trap policy (P1/P3-W09).
- Telemetry event catalog, rates, filters (W11).
- Boot phase ordering and rendezvous (W05).
- Lifecycle transitions and the gate itself (W03).
- Lock primitives (W06); scheduler/current-vCPU semantics (P7/P4+);
  guest state (P4+); DTB/discovery (P2); board/SoC specifics (ADR-043).
