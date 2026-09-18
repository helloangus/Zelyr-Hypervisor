# P6-W08 Implementation Workflow and Acceptance

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Parent:** [P6-W08 design entry](README.md).

## 1. Preconditions and failure boundary

Complete the Coding-Guidelines preflight and load
[01](01-scope-and-foundations.md), [02](02-architecture-and-state.md), and
the contract file for the step at hand. W08's upstream dependencies (W01,
W02, W07, P4; downstream W09) are planned or designed in parallel, so step
1 is a real entry review against [01](01-scope-and-foundations.md) §5.
Stop and record a blocker instead of guessing when: W01 does not declare
virtualization support (then W08 records the readiness-failure path as its
deliverable for this platform and does not fake presentation); W02's
delivered group/physical policy differs from the assumed Group-1/NS
baseline (amend [04](04-code-contracts-lr-presentation.md) §2 against the
delivered policy — never diverge silently); W07's protocol landed with
changes (cross-design amendment with W07/W09); P4 lacks a transition hook
or extension area (boundary conflict with the P4 design); or a step
appears to require maintenance policy, Guest GIC MMIO, or hardware-mapped
interrupts (scope violation — W09, P8, Reserved respectively).

## 2. Ordered implementation steps

### Step 1 — entry review, specification lock, and cross-design reconciliation

Target: implementation record
(`../p6-w08-gic-virtualization-interface-record.md`, created in this step).

Work: inspect delivered evidence for every §5 contract in
[01](01-scope-and-foundations.md). Additionally lock the authoritative
GIC/architecture specification revision for this work (task book §8) and
record it; from it, fix the exact register names, field encodings, and any
additional ordering requirements beyond the contract tables in
[03](03-code-contracts-interface-context.md) §5 and
[04](04-code-contracts-lr-presentation.md) §6. Reconcile the maintenance
boundary (D8) with the W09 design explicitly.

**Acceptance:** every contract has a recorded status; the specification
revision, exact register surface, and boundary reconciliation are recorded
before any register-touching code exists. **Failure/blocker:** missing or
contradictory inputs stop the affected surface at this step with a blocker
naming the owning package.

### Step 2 — place the logical modules and declare the unsafe boundary

Target: workspace layout chosen by the workspace-owning packages (logical
modules `vgic-readiness`, `vgic-lr-table`, `vgic-context`,
`vgic-maintenance-boundary` per [02](02-architecture-and-state.md) §2).

Work: map logical modules onto the actual tree (Arch domain; Core must see
none of it). Declare every register accessor in the `unsafe` inventory
with draft `SAFETY` justifications.

**Acceptance:** all register access lives in the four modules; the unsafe
inventory lists every accessor; no Core-layer module references a GIC
symbol. **Failure/blocker:** a workspace that cannot express the boundary
is a workspace-package blocker.

### Step 3 — discovery and readiness

Target: `vgic-readiness` (contract
[03](03-code-contracts-interface-context.md) §1).

Work: implement discovery (capacity, widths, interface availability), the
sanity envelope, the W01/W02 gating, and the disabled-by-default posture;
record discovery telemetry.

**Acceptance:** readiness on the declared QEMU configuration succeeds and
leaves the interface disabled; each simulated mismatch produces its named
error and no enablement. **Failure/blocker:** a platform without declared
support is a W01 rejection — W08 delivers the failure path and stops.

### Step 4 — context preservation and transition sequences

Target: `vgic-context` (contracts
[03](03-code-contracts-interface-context.md) §2–§5).

Work: implement `VcpuVirtInterfaceContext`, `vgic_on_entry`,
`vgic_on_exit` with the §5 barrier table, including the invariant checks
and force-consistency recoveries; wire into the P4 transition points with
the W06 sequencing fixed at integration.

**Acceptance:** after every exit the interface is disabled, every slot is
empty, and the images match the purged state; after every entry the
hardware matches the images plus fresh loads. **Failure/blocker:** a
missing transition hook is a P4 boundary blocker.

### Step 5 — LR presentation and pressure

Target: `vgic-lr-table` (contracts
[04](04-code-contracts-lr-presentation.md) §1–§4).

Work: implement the slot table, `load_from_selection` (hw=0, group per
delivered W02 policy), priority carriage, and `fill_to_capacity`.

**Acceptance:** slots mirror hardware; pressure fills exactly to
discovered capacity with excess left pending in W07; priority order
survives carriage. **Failure/blocker:** any need for eviction/preemption
is out of P6 scope — stop and record as Reserved.

### Step 6 — maintenance boundary primitives

Target: `vgic-maintenance-boundary` (contract
[04](04-code-contracts-lr-presentation.md) §5).

Work: implement the status read, `clear_completed_slot`, and `refill`;
enable only the maintenance sources the W09 contract names, at the entry
sequence point ([03] §3.1).

**Acceptance:** primitives are policy-free; a maintenance condition with
no W09 consumer stays disabled; completions route to W07 exactly once.
**Failure/blocker:** an unreconciled W09 boundary is a step-1-class
blocker for enabling the corresponding sources.

### Step 7 — acceptance scenarios and evidence

Target: verification record
(`../../verification/p6-w08-gic-virtualization-interface-verification.md`).

Work: run the validation matrix ([06](06-validation-and-handoff.md) §1):
single-presentation, over-capacity, vCPU transition, priority carriage,
cross-vCPU isolation (conditional on the multi-vCPU prerequisite),
maintenance-adjacent progress with W09. Record commands, environment,
outputs, timestamps, and explicit not-run/blocked entries; complete the
implementation record (changed files, unsafe delta, specification lock,
deviations).

**Acceptance:** every W08-DV row has a run status with evidence or an
explicit reason; P6-V11/V16/V18-related claims are made only in the
verification record and only for W08's share.

### Step 8 — closure review and handoff

Work: run the [06](06-validation-and-handoff.md) §3 handoff checklist;
verify the P8 non-ABI constraint (nothing here names a machine layout or
Guest-visible model) and that handoff statements match delivered evidence.

**Acceptance:** checklist complete.

## 3. Evidence destinations

- Implementation record:
  `../p6-w08-gic-virtualization-interface-record.md` (created at step 1;
  holds the specification lock, decisions taken, changed files, unsafe
  inventory delta, deviations).
- Verification record:
  `../../verification/p6-w08-gic-virtualization-interface-verification.md`
  (created when evidence exists).
- No completion claim may appear in any design file.

The validation matrix, error/security/observability model, and handoff
checklist that close this workflow are in
[06-validation-and-handoff.md](06-validation-and-handoff.md).
