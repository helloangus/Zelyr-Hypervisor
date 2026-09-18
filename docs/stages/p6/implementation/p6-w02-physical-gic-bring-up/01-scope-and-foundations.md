# P6-W02 Scope, Prerequisites, and Bring-up Foundations

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P6-W02 detailed design](README.md).

## 1. Prerequisite and assumed-contract inventory

### 1.1 W01 capability decision and probe contracts

Assumed source: [P6-W01](../p6-w01-gic-capability-discovery/README.md) and
its [capability contracts](../p6-w01-gic-capability-discovery/03-code-contracts-capability-model.md).
W02 consumes: the graded decision (`ReadyForP6` or `PhysicalOnly`), the
`ExpectedGicIdentity` facts, and the three probe contracts (distributor,
redistributor, virtualization interface). W02 must not start any
state-changing sequence unless a `Usable`-consistent capability decision
exists and its own confirmation probes agree with it. Failure boundary: a
probe mismatch folds back into W01's taxonomy (`Unsupported`,
`Incomplete`, or `Contradictory`) and bring-up stops before the first
state-changing write; the mismatch is recorded per
[01 §4 of W01](../p6-w01-gic-capability-discovery/01-scope-and-foundations.md)
(Platform Investigation or Architecture Change Request).

### 1.2 P3 pCPU lifecycle, per-CPU state, boot rendezvous, synchronization

Assumed sources:
[p3-w03](../../../p3/plans/p3-w03-physical-cpu-lifecycle.md) (absent /
present / starting / initializing / online / failed states; no CPU usable
before local initialization; no twice-online; failed CPUs stay outside the
online set),
[p3-w04](../../../p3/plans/p3-w04-per-cpu-runtime.md) (independent stack,
logical identity, CPU-local storage, no implicit global current-CPU state),
[p3-w05](../../../p3/plans/p3-w05-smp-boot-synchronization.md) (global
initialization once, local initialization per CPU, SMP-ready only after
declared readiness),
[p3-w06](../../../p3/plans/p3-w06-concurrency-synchronization.md) (mutual
exclusion, IRQ-sensitive sections, atomic ordering rules, lock-order
baseline).

W02 consumes: the local-initialization invocation point (where Phase B
runs), the per-CPU storage to host W02's local context, the failed-CPU
vocabulary for local failures, and the lock/atomic disciplines for the
readiness ledger. Failure boundary: if P3's lifecycle delivers without a
per-pCPU local-init point or without per-CPU storage, W02 has no correct
integration point; the affected design decision stops and the gap is
recorded against the P3 contract (Architecture Change Request if the P6
task book entry table is implicated), not solved with a W02-local global.

### 1.3 P2 platform facts

Assumed source: the P2 handoff per
[p2-w10](../../../p2/plans/p2-w10-p3-p4-handoff-contract.md), which
explicitly reserves "GIC initialization" to a later stage — that reservation
is what makes W02 the first owner. W02 consumes GICD/GICR frame descriptors,
stride, and reservation boundaries from the W01-confirmed platform facts.
Failure boundary: a frame descriptor that fails the probe or is unmappable
at the established mapping contract is a `Malformed`/`Incomplete` capability
finding; W02 records and refuses.

### 1.4 Entry gating

Per the P6 task book §2, W02 implementation begins only after the entry
review finds P6-ENTRY-01/02/06 (P4 guest chain facts are not needed by W02
itself, but P6 entry is stage-level), P6-ENTRY-03/04 (P3 SMP contracts), and
the supporting platform input evidenced. A missing entry condition blocks
W02, is recorded, and is not repaired inside W02.

## 2. Bring-up boundary and invariants

W02 owns exactly these state transitions:

- global: Distributor `Unconfigured → Confirmed → Quiesced → Configured →
  Enabled` (one transition owner: the Phase-A sequence on the boot pCPU);
- local (per pCPU, owner: that pCPU): `Unprobed → Confirmed → Waking →
  LocalConfigured → InterfaceReady → LocalReady`, plus `LocalFailed(reason)`
  from any state;
- ledger: per-pCPU readiness/failure records published by the owning pCPU.

Invariants (all Required; validated by W02-DV rows):

- INV-1 no interrupt is enabled anywhere until its owning consumer
  registers it (W03 surface); at the end of bring-up every SGI, PPI, and
  supported SPI is disabled with lowest default priority.
- INV-2 no local state transition occurs before that pCPU's probe
  confirmation; no global transition occurs before the global probe.
- INV-3 a pCPU never reads or writes another pCPU's GICR frame (exception:
  the Phase-A enumeration may *read* GICR_TYPER of frames it maps to verify
  coverage — reads only, and only pre-wake).
- INV-4 local readiness is published exactly once per pCPU (no
  re-bring-up, no twice-online per P3).
- INV-5 every `unsafe` access lies inside the audited register-access
  surface of [03](03-code-contracts-register-access.md); no other `unsafe`
  exists in W02.
- INV-6 every wait is bounded and every timeout has a named failure outcome.

## 3. Scope classification detail

**Required:** identity confirmation; Distributor quiesce/configure/enable;
GICR wake and per-pCPU configuration; CPU-interface SRE/CTLR/PMR/group
sequencing; initial SPI disable + determinate routing; residual pending /
active survey-and-clear with logging; readiness ledger; failure taxonomy
mapping to P3 states; telemetry events; the register-access surface with
reserved-bit and barrier rules.

**Reserved (with triggers):** EOImode=1 + DIR deactivate split (trigger:
W08 presentation design); extended SPI/SGI/PPI range configuration
(trigger: an approved consumer design requires IDs beyond the base ranges);
priority/preemption tuning beyond defaults (trigger: P6-W10 semantics
design); GICR LPI/LPI-related configuration (trigger: a post-P6 ITS/LPI
design, out of P6); dual-security-state support (trigger: an ADR-level
decision — P6 posture is single-security-state per
[W01 posture](../p6-w01-gic-capability-discovery/01-scope-and-foundations.md) §2).

**Out of Scope:** dispatch and classification (W03); SGI send and route
change (W04); EL2 timer PPI enablement (W05); virtual interrupts, List
Registers, maintenance interrupt (W07–W09); Guest GIC MMIO and vGIC
(P6-W07/W08 hardware-presentation boundary and P8); ITS/LPI/MSI; GICv2
(ADR-033); scheduler behavior (P7); Orange Pi 3B hardware support tier
(P15); any new crate boundary or dependency decision (owned by their P0
packages).

## 4. Platform-revision handling

All GIC register names in this design are architectural. Bit positions,
reset values, and reserved-field write rules are taken from the GIC
specification revision pinned in the W01 implementation record
([W01 workflow](../p6-w01-gic-capability-discovery/04-implementation-workflow.md)
step 1). Where this design says "reserved bits" the implementing rule is:
reads mask to implemented fields; read-modify-writes preserve unknown bits;
writes set architecturally-reserved-as-zero fields to zero unless the pinned
revision says preserve; any field the pinned revision marks
implementation-defined is treated as unknown and never depended upon.
Discovering a conflict between the pinned revision and observed hardware is
a Specification Investigation stop, not a local workaround.
