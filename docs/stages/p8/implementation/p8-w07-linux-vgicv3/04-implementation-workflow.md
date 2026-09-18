# P8-W07 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W07 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the parent
README, the [Coding Guidelines](../../../../development/coding-guidelines.md)
(preflight included), and the P6 contracts cited there. Useful read-only
discovery: `git ls-files` (confirm the actual crate layout produced by the
approved workspace decision) and a search for any existing interrupt
registration the design must not contradict.

Stop and obtain direction instead of guessing when any of the following
occurs:

- the W02 gate has not approved the GIC address map and INTID table — no
  address or INTID may be embedded ([01 §2](01-architecture-and-state.md));
  record the blocker;
- a P6-W07/W08/W09 contract cannot express a transition required by
  [03](03-code-contracts-interrupt-flow.md) — record an `Architecture Change
  Request` against the P6 contract; never fork the lifecycle locally;
- the P4 MMIO fault path cannot attribute write faults with IPA/width — M1
  has no input basis; block at the P4/W07 seam;
- a declared Linux scenario appears to require a register outside the
  implemented subset — extend the subset through this design's revision, not
  an ad-hoc handler;
- implementation seems to need ITS/MSI/LPI, security-state registers, or
  EOImode 1 — all Reserved or Out of Scope (README classification).

## 2. Ordered implementation steps

### Step 1 — machine facts registration

Target: `MachineInterruptTable` and region-map population from approved
machine-contract data.

Work: register the GICD/GICR regions, per-vCPU frame map, SPI table (incl.
`<CONSOLE-SPI-INTID>` consumption point for W09 and timer-PPI consumption
point for W08), and TYPER/IIDR facts — all from the approved gate values, no
literals in handlers. Fail VM creation on absence/inconsistency.

**Acceptance:** all Guest-visible numeric GIC facts derive from one approved
source; inconsistency fails creation (P8-V05-consistent).  
**Failure/blocker:** missing gate approval is a recorded blocker (§1).

### Step 2 — access routing (M1)

Target: `vgic_mmio_access` with region decode, width/alignment rules, and
Reject classification application ([02 §1](02-code-contracts-vgic-mmio.md),
[02 §6](02-code-contracts-vgic-mmio.md)).

Work: implement region membership, per-vCPU frame identification, telemetry
on every access class, and the W05-classified outcomes. Verify bounded
worst-case work per access.

**Acceptance:** in-region access routes; out-of-region/illegal access yields
the declared contained outcome with W13 diagnostic context; no allocation.  
**Failure/blocker:** missing write-fault attribution from P4 blocks per §1.

### Step 3 — Distributor model (M2)

Target: the GICD register subset of
[02 §2](02-code-contracts-vgic-mmio.md) with SPI-table transitions and M5
requests.

Work: implement the table row by row; keep enable/disable pending-latching
exact; route pending set/clear and active transitions through M5 only.
Bit-handling uses checked field extraction (no out-of-bounds table indexing).

**Acceptance:** each implemented register behaves per IHI 0069 for the
declared subset; unimplemented offsets are RAZ/WI; GICD_SGIR never injects.  
**Failure/blocker:** a semantics conflict with IHI 0069 stops the register
for review — a wrong register is a contained guest fault, not a Host bug
source.

### Step 4 — Redistributor and interface registers (M3)

Target: the GICR subset of [02 §4](02-code-contracts-vgic-mmio.md) and the
ICC_*_EL1 interface state (PMR, CTLR read-as-implemented bits, IGRPEN1),
including WAKER handshake behavior.

Work: implement per-vCPU frames, the enumeration probe pattern (TYPER,
last-frame), the WAKER Quiescent transitions, and interface register
read/write semantics consistent with P6-W08's context preservation.

**Acceptance:** Linux-style enumeration and secondary GICR init sequences
([05 §1](05-validation-and-handoff.md) S1/S2) pass their register-level
checks; WAKER state agrees with the quiesce/resume hooks.  
**Failure/blocker:** an ICC classification conflict with W05/P6-W08 stops at
that seam.

### Step 5 — injection/EOI/SGI flow (M4/M5)

Target: [03 §2–§5](03-code-contracts-interrupt-flow.md) — SGI generation,
the device-facing injection entry, EOI/deactivate mapping, and the
presentation tick.

Work: implement the bridges as pure calls into the P6 lifecycle operations
per the bridging rule; verify the gate computation (enable/mask/group) and
the over-capacity behavior (stay pending, no loss).

**Acceptance:** end-to-end pending→presented→active→completed cycles are
exact at the contract level; repeated/spurious EOI is benign; over-capacity
preserves pending state.  
**Failure/blocker:** any missing P6 operation blocks per §1 — do not
substitute a local queue.

### Step 6 — lifecycle hooks and consumer seams

Target: `vgic_quiesce`/`vgic_resume` and cross-package agreement.

Work: wire the hooks into the vCPU transition path and confirm signatures
with [W06](../p8-w06-psci-virtualization/README.md) (CPU_OFF call site),
[W08](../p8-w08-linux-timer-integration/README.md) (injection entry), and
[W09](../p8-w09-virtual-console-single-cpu-linux/README.md)/
[W10](../p8-w10-linux-smp-bringup/README.md) (consumers). Mismatches resolve
toward the owning design.

**Acceptance:** hook agreement recorded; no duplicated lifecycle logic
anywhere.  
**Failure/blocker:** disagreement stops integration and is recorded for the
coordinator.

### Step 7 — scenario execution and closure review

Work: run the [05](05-validation-and-handoff.md) matrix as far as current
integration permits; evidence to
`../../verification/p8-w07-linux-vgicv3-verification.md`; decisions to
`../p8-w07-linux-vgicv3-record.md`. Completion claims only in the
verification record, only for what ran.

**Acceptance:** matrix entries passed/failed/blocked/not run with evidence;
P8-V10 rows marked run have real output.  
**Failure/blocker:** a failed fidelity row (lost/dupe pending) is recorded
and fixed through the design's state model, never by weakening the scenario.

## 3. Ordering rationale

Facts (Step 1) → routing (Step 2) → register models (Steps 3–4) → flow
(Step 5) mirrors the Guest's own dependency order (Linux discovers, then
enables, then receives), so each step is exercisable against the next stage
of a real Linux boot log.
