# P7-W10 Guest Test-Asset Contract: Extension, Markers, Harness Seams

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P7-W10 detailed design](README.md).

## 1. Logical artifact groups

W10 is test-asset and scenario work; its logical modules are authoritative
artifact groups, not hypervisor modules.

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Scheduler workload modules (Guest side) | the maintained Validation Guest asset (P4-W05 lineage) | scenario definitions of [01-scenario-suite.md](01-scenario-suite.md) | Guest behavior + markers for VG-SCHED scenarios; does not alter inherited scenario behavior |
| Marker protocol | this design (§3) | P4 debug channel conventions | deterministic, sequence-numbered marker emission; not a machine ABI |
| Host-side expectation tables | the W10 test-support surface (per the P0 crate-governance decisions, wherever the P0-W02 toolchain/W03 workspace designs place test crates) | scenario pass conditions | machine-checkable per-scenario expectations; no automation scripts (W12) |
| Scenario inventory record | `../p7-w10-validation-guest-suite-record.md` | implemented scenario status | truthful per-scenario status; never a completion claim |

## 2. Assumed upstream contracts and failure boundaries

All unevidenced in the current tree. Failure boundary per row: **the affected
scenario is blocked and recorded (P7-V01 reconciliation / verification row);
never improvised.**

| ID | Assumed contract | Source (plan path) | Fails if |
|---|---|---|---|
| G-1 | A maintained, loadable Rust `no_std` Validation Guest with a debug/semihost-like output channel and VG-001–VG-012 scenario scaffolding | [P4-W05](../../../p4/plans/p4-w05-validation-guest.md) | No asset to extend; W10 cannot create a Guest from scratch (that is P4-W05's deliverable) |
| G-2 | Guest memory/image loading conventions and controlled fault triggers | P4-W03/P4-W05 plans (`../../../p4/plans/p4-w03-guest-memory-image.md`, p4-w05) | Scenarios cannot be loaded or cannot trigger contained faults |
| G-3 | HVC dispatch surface with the P5-W07 expected result classes and the two-context setup | [P5-W07](../../../p5/plans/p5-w07-validation-guest-isolation-suite.md), [P5-W02](../../../p5/plans/p5-w02-hypercall-abi-error-boundary.md) | VG-SCHED-06/-07 have no callable surface |
| G-4 | Guest-visible vCPU timer programming interface and SGI reception path with maintained VG-TIMER/VG-IRQ scaffolding | [P6-W11](../../../p6/plans/p6-w11-validation-guest-interrupt-suite.md), [P6-W06](../../../p6/plans/p6-w06-guest-generic-timer.md) | VG-SCHED-03/-04 have no wake source to program |
| G-5 | Scheduler behavioral contracts W04–W07 and telemetry events W09 | `../p7-w04-preemption-context-switch/README.md` .. `../p7-w09-accounting-diagnostics/README.md` designs | A scenario's named contract does not exist; its proof row is void |
| G-6 | Multi-VM (two-context) operation on the scheduled path | P5-W07 setup + [P7-W05](../p7-w05-shared-mn-multivm/README.md) | VG-SCHED-07/-08 topologies are unrunnable |

## 3. Marker protocol

- **Form:** `VG-SCHED-<NN>-<EVENT>[:seq=<n>][,aux=<k>]` on the inherited
  debug channel — fixed prefix, scenario number, event tag, optional
  sequence/auxiliary numbers. Example shape (illustrative, not code):
  `VG-SCHED-03-RECV:seq=7`.
- **Determinism:** events are emitted at defined program points with
  gap-free sequence numbers per stream; no wall-clock, no floating point,
  no timing-dependent content (README decision 5).
- **Boundedness:** per-run marker volume is bounded by declared loop counts;
  the host expectation table knows the exact expected multiset/order per
  scenario.
- **Safety:** markers reveal no Host pointers and no capability/handle
  values (inherited P5-W07 review rule); the Guest cannot affect hypervisor
  behavior by emitting or withholding markers (decision 6 of the README).
- **Stability:** stable within P7 for automation; changes are ordinary suite
  changes recorded in the implementation record — not ABI events (parent
  README exclusions).

## 4. Telemetry correlation rules

- Each scenario's expectation table pairs Guest markers with the
  [P7-W09](../p7-w09-accounting-diagnostics/README.md) trace events and
  counters named in its catalog row, and declares the authority split
  (markers decide Guest-visible claims; telemetry decides scheduler-internal
  claims).
- Correlation keys: scenario ID maps to topology; vCPU identity maps to
  marker streams; sequence numbers order marker streams; telemetry is
  ordered by its monotonic basis. No wall-clock alignment is attempted.
- Discrepancy handling: a marker/telemetry disagreement is a finding recorded
  as a failed or blocked row with both sides' evidence — never resolved by
  adjusting one side to match the other.
- Failure boundaries: if telemetry is absent or trimmed in the build under
  test (W09 trimming classes), scenarios whose authority is telemetry-only
  record blocked, not passed.

## 5. Platform-contract review rules (plan work-sequence item 4)

The following rules keep Guest workloads from redefining platform contracts;
they are review checks in workflow step 4:

1. No scenario may require a Guest-visible interface that no upstream plan
   (P4-W05, P5-W02/W07, P6-W06/W11) already provides. New Guest-visible
   surfaces are prohibited; if a scenario seems to need one, the scenario is
   wrong, not the platform.
2. No marker, register convention, memory layout, or boot expectation may be
   documented as a Guest machine ABI or carried into `docs/abi/` or
   `docs/machine-types/` content by W10 changes.
3. Scenario topologies are declared through the W03/W05 placement/configuration
   surfaces, never by inventing new configuration formats.
4. Inherited scenarios remain byte-for-byte behaviorally unchanged (§2 of the
   suite file); their markers and expected outcomes may not be edited to fit
   P7 behavior. A needed change is a finding against P7, not against the
   scenario.
5. The Guest asset stays `no_std`, Guest-untrusted, and free of Linux,
   firmware, or production-driver dependencies (P4-W05 boundary).
