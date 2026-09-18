# P6-W01 Code Contracts — Capability Model and Reconciliation

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P6-W01 detailed design](README.md).  
**Convention:** contracts follow the work-package design checklist §3
template. Names are exact and stage-stable within P6; they are not a frozen
cross-stage ABI. Pseudocode is design logic, not runnable production code;
Rust here means typed outlines (`no_std`, no allocation assumption, no
`unsafe` — W01 contains none).

## 1. Type inventory

All types are `Copy`-able value types or small aggregates of value types; all
ID/address types reuse the semantic newtypes established by the P0 address/ID
type-safety baseline
([p0-w15](../../../p0/plans/p0-w15-address-identifier-type-safety.md) assumed
contract): physical addresses are `PhysAddr`, never naked `usize`/`u64`.

| Type | Kind | Stability | Purpose |
|---|---|---|---|
| `GicFamily` | enum | P6-internal | `V2`, `V3Family` |
| `CpuInterfaceForm` | enum | P6-internal | `SystemRegister`, `MemoryMapped` |
| `SecurityStatePosture` | enum | P6-internal | `SingleNonSecure`, `DualUndetermined` |
| `DecisionGrade` | enum | P6-internal | `ReadyForP6`, `PhysicalOnly`, `Rejected` |
| `CapabilityVerdict` | enum | P6-internal | `Usable`, `AbsentRequired`, `AbsentOptional`, `Unsupported`, `Incomplete`, `Malformed`, `Contradictory` |
| `CapabilityInputId` | enum (12 variants, fixed) | P6-internal | CAP-01…CAP-12 of [01](01-scope-and-foundations.md) §3 |
| `EscalationKind` | enum | P6-internal | `None`, `PlatformInvestigation`, `ArchitectureChangeRequest` |
| `CapabilityFinding` | struct | P6-internal | input id, verdict, observed value slot, bounded reason |
| `ExpectedGicIdentity` | struct | consumed by W02 | see §4 |
| `GicCapabilityDecision` | struct | P6-internal | grade, findings, escalation list, `Option<ExpectedGicIdentity>` |
| `PlatformGicFacts`, `CpuInterruptFacts`, `PossiblePcpuSet` | structs | upstream-owned | assumed-contract input shapes (§1.1–§1.3 of [01](01-scope-and-foundations.md)) |

## 2. `GicCapabilityDecision` and `CapabilityFinding`

```text
Name and stability: GicCapabilityDecision / CapabilityFinding (P6-internal;
  consumers hold them by value or shared reference; never mutated after
  construction)
Purpose and caller: the reviewable capability decision; produced by
  reconcile() (§3); consumed by W02 (confirmation), W03–W05 (range facts),
  W08 (virtualization findings), W11–W13 (taxonomy and report)
Inputs / outputs: construction from findings + grade rules
Preconditions / postconditions: findings contains exactly one entry per
  CapabilityInputId variant; grade is consistent with verdicts per §3 rules;
  ExpectedGicIdentity is Some iff grade ∈ {ReadyForP6, PhysicalOnly}
State and ownership change: none (immutable value)
Concurrency/allocation context: constructed before SMP release and before
  GIC enablement; no lock; fixed-capacity or allocator-backed per P2 contract
Errors and failure guarantee: none fallible; defect outcomes are findings,
  not errors
Security/authorization checks: none (no untrusted runtime input; platform
  facts range-validated upstream and re-checked here for Malformed)
Logic: aggregate value; invariant check available as a host-test helper
Validation: W01-DV02/DV03 reviews and host-side fixture tests
```

## 3. `reconcile()` — the reconciliation function

```text
Name and stability: fn reconcile(platform: &PlatformGicFacts,
  cpu: &CpuInterruptFacts, cpus: &PossiblePcpuSet) -> GicCapabilityDecision
  (P6-internal; sole constructor of decisions)
Purpose and caller: evaluate the capability input register of
  [01](01-scope-and-foundations.md) §3 into findings and a grade; called
  once by the boot initialization owner after platform discovery, and
  directly by host-side fixture tests
Inputs / outputs: three assumed-contract fact objects → decision
Preconditions / postconditions: inputs are complete upstream results (each
  carries its own absent/unsupported states); output satisfies the
  aggregate invariants of §2; identical inputs produce an identical
  decision (pure)
State and ownership change: none
Concurrency/allocation context: runs to completion pre-SMP, pre-GIC-enable;
  no lock; bounded allocation per [02](02-architecture-and-state.md) §4
Errors and failure guarantee: infallible; every input defect becomes a
  finding; the function never panics on malformed platform data (Coding
  Guidelines: platform/firmware input is validated, not trusted)
Security/authorization checks: re-validates architectural ranges of every
  consumed field (frame alignment/size bounds, affinity width, PPI ID ≤ 31,
  declared SPI range vs ID-bit facts) before classifying
Logic:
  findings = empty fixed-capacity set keyed by CapabilityInputId
  for each input row in the register:
    verdict = classify_row(row, inputs)        # per-row rule below
    findings.insert(CapabilityFinding { row.id, verdict, observed, reason })
  grade =
    if any of CAP-01,02,03,04,06,07,08,09,10 ∉ {Usable, AbsentOptional}
      -> Rejected
    else if CAP-05 ∉ {Usable, AbsentOptional}   # virtualization missing
      -> PhysicalOnly
    else -> ReadyForP6
  escalations = map verdicts: Contradictory -> ArchitectureChangeRequest
                             (when upstream contracts conflict) or
                             PlatformInvestigation (platform inconsistency,
                             decided by the row rule in
                             [01](01-scope-and-foundations.md) §4);
                AbsentRequired/Unsupported/Incomplete/Malformed
                             -> PlatformInvestigation
  expected = derive_expected_identity(platform, findings)  # §4; None iff
                                                           # grade == Rejected
  return GicCapabilityDecision { grade, findings, escalations, expected }

  classify_row highlights (full table in host-test form):
    CAP-01: family from declared identity; V2 -> Unsupported; undeclared ->
            AbsentRequired
    CAP-02: frame base nonzero, size in declared bounds, power-of-two-
            aligned per platform contract; overlap with any reservation ->
            Contradictory (sources: frame vs reservation list)
    CAP-03: for each affinity in cpus.possible(): a GICR frame exists whose
            affinity matches; stride declared and nonzero; any miss ->
            Incomplete (reason names the affinity)
    CAP-04: cpu.gic_interface == SystemRegister else Unsupported;
            undeclared -> AbsentRequired
    CAP-05: cpu.virtualization_ext declared AND GIC-side virtual CPU
            interface declared -> Usable; one side absent -> the missing
            side governs (CPU-side absent -> Unsupported; GIC-side absent
            -> AbsentRequired with PhysicalOnly degradation)
    CAP-06: declared SPI range determinate and ≥ platform minimum declared
            by P2; indeterminate -> Malformed
    CAP-07/08: maintenance PPI and EL2 timer PPIs declared and ≤ 31 else
            Malformed (IDs are architecturally PPI-ranged)
    CAP-09: GIC frames ∩ reserved ranges = ∅ else Contradictory
    CAP-10: firmware hints inconsistent (both single and dual indications)
            -> Contradictory; dual-state indicated -> Unsupported for now
            (posture requires single-security-state; escalation)
    CAP-11/12: declared -> Usable; not declared -> AbsentOptional

Validation: host-side fixture tests per platform fixture class (usable,
  GICv2-input, missing-GICR-frame, malformed-range, contradictory-source);
  each fixture asserts the exact verdict set and grade
```

## 4. `derive_expected_identity()` and `ExpectedGicIdentity`

```text
Name and stability: derive_expected_identity(platform, findings) ->
  Option<ExpectedGicIdentity> (P6-internal; called by reconcile only)
Purpose and caller: compute the identity facts W02's confirmation probes
  must reproduce; prevents W02 from re-deriving expectations (single
  authority for the expected posture)
Inputs / outputs: platform facts + findings → identity record
Preconditions / postconditions: called only when grade ≠ Rejected; the
  record contains no value that was not Usable or AbsentOptional-derived
State and ownership change: none
Concurrency/allocation context: as reconcile()
Errors and failure guarantee: infallible
Security/authorization checks: as reconcile()
Fields of ExpectedGicIdentity:
  family: GicFamily                    # expect V3Family
  affinity_routing_required: bool      # always true in the P6 posture
  sgi_range: RangeInclusive<u8>        # 0..=31 with PPIs: SGI 0..=15, PPI
  ppi_range: RangeInclusive<u8>        #   16..=31 (architectural split)
  max_spi: TypedIntId bound            # typed bound; SPI ids 32..=max
  gicr_stride_hint: Bytes or per-frame list  # from P2 facts, for lookup
  maintenance_ppi: Option<PpiId>       # required fact → Some on Usable
  el2_timer_ppis: [PpiId; 2] slots     # per CAP-08 facts
  security_expectation: SecurityStatePosture
  virtualization_expected: bool        # grade == ReadyForP6
Validation: host fixture tests assert identity contents per fixture
```

## 5. Probe contracts (specified by W01, executed by W02)

These are *contracts only*; no W01 code performs them. Full register-access
rules (volatile access, reserved bits, barriers) are W02 scope
([W02 register-access contracts](../p6-w02-physical-gic-bring-up/03-code-contracts-register-access.md));
W02 must bind each probe result to the matching capability input and fold a
mismatch back into this design's taxonomy.

```text
Name and stability: DistributorIdentityProbe (contract)
Purpose and caller: confirm CAP-01/CAP-06/CAP-11/CAP-12 before any GICD
  state change; executed by W02 step "identity confirmation"
Inputs / outputs: mapped GICD frame (from PlatformInfo) → observed family,
  SPI range, feature bits
Failure guarantee: read-only; safe to run on an unconfigured GIC
Validation: mismatch verdicts Unsupported/Contradictory per §3 rules
```

```text
Name and stability: RedistributorIdentityProbe (contract)
Purpose and caller: confirm CAP-03 per pCPU (frame presence, affinity
  match, Last-marker consistency) before waking that GICR; executed by W02
  per-pCPU bring-up
Inputs / outputs: mapped GICR frame + expected affinity → observed affinity,
  processor number, Last flag
Failure guarantee: read-only; safe pre-wake
Validation: affinity mismatch or missing Last → Incomplete/Contradictory
  finding referencing CAP-03
```

```text
Name and stability: VirtualizationInterfaceProbe (contract)
Purpose and caller: confirm CAP-05 at EL2 (virtualization-control
  registers readable, basic capacity/feature fields in architectural
  range); executed by W02's CPU-interface phase; evidence consumed by
  [P6-W08](../p6-w08-gic-virtualization-interface/README.md)
Inputs / outputs: current pCPU context → observed virtualization capacity
  and feature record
Failure guarantee: read-only; no state change
Validation: unreadable or out-of-range capacity → CAP-05 finding degrades
  to PhysicalOnly grade rules of §3
```

## 6. Report emission

```text
Name and stability: emit_capability_report(decision: &GicCapabilityDecision)
  (P6-internal; called by the report module after reconcile())
Purpose and caller: persist the decision to the W01 record artifact
  location and emit the trace events of [02](02-architecture-and-state.md) §7
Inputs / outputs: decision → record content + events
Preconditions: telemetry namespace contract from P0-W13 available; record
  path per the stage documentation layout
Postconditions: report content mirrors the decision exactly (no editorial
  improvement of findings)
Errors: emission failure is a diagnostic (logged), never a boot-gating
  error; the in-memory decision remains authoritative
Validation: W01-DV06 record review
```

## 7. Explicitly not authorized here

No `unsafe`, no MMIO or system-register access code, no register bit
constants, no GIC initialization sequencing, no handler registry, no Guest
types, no board/SoC constants, and no second decision entry point. Any need
for these is either W02+ scope or an ADR Required escalation.
