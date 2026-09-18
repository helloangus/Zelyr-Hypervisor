# P6-W01 Scope, Prerequisites, and Capability Foundations

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P6-W01 detailed design](README.md).

## 1. Prerequisite and assumed-contract inventory

W01 consumes platform facts; it produces none of them. Each input below is an
assumed contract from an upstream plan. The contract citation is by plan path;
the runtime fact instances do not exist in the current repository scaffold.

### 1.1 P2 PlatformInfo GIC, timer, and CPU facts

Assumed source: [p2-w02](../../../p2/plans/p2-w02-platform-discovery-normalization.md)
(P2-B01–B08, P2-C01–C03), delivered onward per
[p2-w10](../../../p2/plans/p2-w10-p3-p4-handoff-contract.md). W01 assumes the
normalized result distinguishes absent / unsupported / usable states and
carries, at minimum:

- GIC distributor frame: base address and size, with memory-type and range
  sanity already validated by P2 intake rules;
- GIC Redistributor frames: base address, per-frame stride, and coverage of
  the declared CPU set (or an explicit per-frame list keyed by affinity);
- GIC version indication as declared by the platform description;
- the CPU inventory and boot-CPU relation (P2-B01) that P3-W01 later
  normalizes into the possible physical-CPU set;
- timer facts including the PPI interrupt IDs assigned to the Host EL2
  physical timer and the GIC maintenance interrupt as described by the
  platform;
- reservation and ownership constraints that the GIC MMIO ranges must not
  violate (P2-B03/P2-W03 ownership rules).

### 1.2 P1 CPU capability facts

Assumed source: [p1-w03](../../../p1/plans/p1-w03-aarch64-capability-inventory.md)
(P1-V05/P1-V06). W01 assumes the inventory distinguishes required from
optional facts and reports, at minimum: CurrentEL and Non-secure EL2 entry
fact, CPU identity/affinity source (MPIDR), and the CPU-side GIC and
virtualization-extension indication (the CPU implementer's GIC CPU-interface
and virtualization-extension presence bits). These are CPU-side facts; the
GIC-side facts come from P2. A CPU-side absence (for example, a CPU without
the GIC system-register interface) is a *CPU capability finding*, not a
platform-mapping defect.

### 1.3 P3 possible-pCPU set

Assumed source: [p3-w01](../../../p3/plans/p3-w01-cpu-topology-inputs.md) and
[p3-w03](../../../p3/plans/p3-w03-physical-cpu-lifecycle.md) via the P6 task
book entry table (P6-ENTRY-03/P6-ENTRY-04). W01 needs the *possible* pCPU set
(the CPUs P3 will attempt to bring online), not a runtime online set. P2's CPU
inventory plus P3's topology classification define it. Redistributor coverage
is required over the possible set: a CPU P3 might bring online must have a
matching GIC Redistributor frame, or the platform is incomplete for P6.

### 1.4 Failure boundary for assumed contracts

If a named upstream fact is missing, malformed, self-contradictory, or
delivered under a different contract than this section assumes, the affected
reconciliation input is classified `Malformed` or `Contradictory` by
[section 4](#4-verdict-taxonomy-and-escalation) below and the affected P6 work
stops; the mismatch is recorded as a Platform Investigation item (platform
defect) or Architecture Change Request (contract defect against the upstream
plan), per the P6 task book §2. W01 must not patch, default, or infer a
missing upstream fact. "Absent" is a *valid platform answer* for optional
inputs; for required inputs it is a rejection verdict with a diagnostic, not
an inference opportunity.

## 2. Supported GIC posture (Required)

The task book reserves the exact supported GIC revision to detailed design.
This design selects and owns the following posture for the whole P6 stage.
Numeric bit positions and reserved-field handling are taken from the
authoritative GIC specification revision pinned at implementation time
([workflow](04-implementation-workflow.md) step 1); this section fixes the
*shape* of support.

| Aspect | Selected posture | Rationale and authority |
|---|---|---|
| GIC family | GICv3-family (architectural major version 3, including a GICv4-register-level GIC reported as major 3 by identity registers) | ADR-032 mainline; GICv4 register-level devices present a GICv3 programming model for SGI/PPI/SPI |
| GICv2 | Unsupported; presence is an `Unsupported` verdict | ADR-033 keeps GICv2 out of the mainline |
| CPU interface | System-register interface at Non-secure EL2 (ICC_SRE-family SRE posture) required | The P6 task book names the "system register CPU-interface lifecycle"; a memory-mapped CPU interface is GICv2-shaped and Unsupported |
| Affinity routing | ARE required on Distributor and Redistributor paths | Prerequisite for the system-register SGI mechanism (ICC_SGI1R) that W04 consumes and for affinity-keyed GICR lookup |
| Security state | Non-secure single-security-state assumed; dual-security-state detection is a platform-investigation escalation; Secure configuration is never written | ADR-008 (no EL3 firmware ownership); runtime confirmation delegated to W02 |
| Interrupt classes | SGI, PPI, SPI only | Task book scope; ITS/LPI/MSI are Out of Scope for P6 |
| Extended ranges (GICv3.1) | Optional-detected, Reserved | Usable later without redesign; not required for P6 closure |
| Virtualization interface | Required for W08+; absence yields the graded `PhysicalOnly` decision | Plans index gives W08 an evidenced-virtualization-capability prerequisite |
| Priority width | 8-bit priority as declared by identity registers; P6 uses lowest-default init | W02 owns init; W10 owns priority semantics |

## 3. Capability input register

Each row is one reconciled input. `Req` classes: **R** = required for
`ReadyForP6` or as marked; **O** = optional-detected; **R-** = required for a
subset of consumers. Sources are the assumed contracts of §1.

| ID | Capability input | Req | Source | Consumer |
|---|---|---|---|---|
| CAP-01 | GIC family is GICv3-family per declared identity | R | P2 declared + W02 probe | all |
| CAP-02 | Distributor MMIO frame valid, aligned, size-sane, non-overlapping with declared reservations | R | P2 | W02 |
| CAP-03 | Redistributor frames cover every possible pCPU by affinity; stride declared | R | P2 + P3 possible set | W02, W04 |
| CAP-04 | System-register CPU interface available at NS EL2 (CPU-side) | R | P1 + W02 probe | W02, W03 |
| CAP-05 | Virtualization interface present and basic capacity readable (CPU-side virtualization extension plus GIC-side virtual CPU interface) | R- (W08+) | P1 declared + W02 probe evidence | W08, W09 |
| CAP-06 | Supported SPI range determinate and ≥ the declared platform minimum | R | P2 + GICD facts | W02, W03, W04 |
| CAP-07 | Maintenance-interrupt PPI ID declared | R | P2 timer/GIC facts | W08, W09 |
| CAP-08 | Host EL2 physical-timer PPI IDs declared and within the architectural PPI range | R | P2 timer facts | W05 |
| CAP-09 | GIC MMIO ranges do not intersect reserved-memory or other device frames | R | P2 reservations | W02 |
| CAP-10 | Dual-security-state indicators absent or resolvable as single-security-state | R | P2 firmware facts + W02 DS probe | W02 |
| CAP-11 | Extended SPI/SGI/PPI ranges | O | GICD identity probe | Reserved |
| CAP-12 | Interrupt-{Targets,Groups}-extendable / 1-of-N routing features | O | GICD identity probe | Reserved (W04 records) |

## 4. Verdict taxonomy and escalation

Every capability input resolves to exactly one verdict:

| Verdict | Meaning | Action | Escalation owner |
|---|---|---|---|
| `Usable` | Fact present, in range, consistent | include in decision | — |
| `AbsentRequired` | Required fact not present | decision degrades (`Rejected`, or `PhysicalOnly` when the fact gates only W08+) | Platform Investigation record; affected packages blocked |
| `AbsentOptional` | Optional fact not present | recorded as not-available; no degradation | — |
| `Unsupported` | Fact present but below/against the supported posture (e.g. GICv2, memory-mapped CPU interface) | decision `Rejected` for the affected subset | Platform Investigation; posture change would need the task book's Reserved clause re-exercised by a new design |
| `Incomplete` | Partially present (e.g. Redistributor missing for one possible pCPU) | decision `Rejected` naming the deficit | Platform Investigation |
| `Malformed` | Fact unparseable or out of architectural range | treated as `AbsentRequired` with a distinct diagnostic | Platform Investigation; if the upstream parser should have caught it, an upstream defect record |
| `Contradictory` | Two sources disagree (DTB vs CPU inventory vs firmware facts) | decision `Rejected` naming both sources | Architecture Change Request if the upstream contract itself conflicts; Platform Investigation if the platform description is inconsistent |

Escalation records are documentary, located per the P6 task book §7/§8 under
the stage's implementation area; they are not created by this design.

## 5. Layering and policy-separation requirements

The reconciliation engine is pure: same inputs → same decision, no hardware
access, no allocation-dependent behavior, no platform-name branching. The
decision artifact separates:

- *facts* (verbatim upstream values with their sources),
- *findings* (per-input verdict plus the reason), and
- *the decision* (graded outcome plus consumer-facing assumptions).

Core never sees a board name; board-specific content exists only inside
platform fact instances and fixture inputs for host-side tests. Guest-facing
inputs do not exist in W01 — there is no Guest path — and no W01 type may
later be reused as Guest state (P6-W07/W08 own virtual interrupts; this
boundary is restated in
[P6-W03](../p6-w03-physical-interrupt-lifecycle/README.md) and
[P6-W08](../p6-w08-gic-virtualization-interface/README.md)).

## 6. Entry contract published by W01

W01's output artifact is the capability decision record consumed by W02–W13
([validation and handoff](05-validation-and-handoff.md) §3 states its required
content): the posture summary of §2, the per-input findings table, the graded
decision, the expected-identity facts for W02's confirmation, the
open-investigation list, and the consumer assumptions. Until that record
exists with W02's confirmation evidence, no P6 package may touch GIC hardware
(P6-ENTRY gating per the task book §2).
