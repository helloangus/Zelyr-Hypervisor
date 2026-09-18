# P3-W01 Scope, Foundations, and Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W01 detailed design](README.md).

## 1. Goal-to-baseline ledger

The entry README carries the goal-to-baseline ledger
([README §Current-state findings](README.md)); this section adds the
foundation-deliverable reasoning that the checklist requires and that the
abbreviated table compresses. Nothing here supersedes the ledger.

### 1.1 What must concretely exist for the plan goal to be true

The plan goal is "a reviewed contract from P2 platform discovery to P3 for
physical CPU topology, hardware identity, logical identity, boot-CPU
designation, and availability classification." For that to be true, four
concrete artifacts must exist:

1. Typed identity values (`HardwareCpuId`, `MpidrValue`, `LogicalCpuId`)
   whose construction and comparison rules make the forbidden assumptions
   (contiguity, enumerated-order identity, present-equals-online)
   unrepresentable or rejected at intake —
   [03-code-contracts-cpu-identity.md](03-code-contracts-cpu-identity.md).
2. A frozen aggregate (`TopologyInputs`) that downstream packages consume as
   the single machine view, with boot designation and per-class counts —
   [04-code-contracts-topology-intake.md](04-code-contracts-topology-intake.md).
3. An enumeration diagnostic that prints the classification and mapping in a
   machine-parseable, human-reviewable form, so cross-count evidence
   (P3-V01) can be captured from boot output.
4. Recorded acceptance evidence per
   [06-validation-and-handoff.md](06-validation-and-handoff.md), created only
   when evidence actually exists.

### 1.2 Prerequisites treated as assumed contracts

P3-W01 begins implementation only when the task book §2 entry conditions
hold. Each prerequisite is an assumed contract with an explicit failure
boundary if it delivers differently:

| Prerequisite | Source plan | Assumed content | Failure boundary if delivered differently |
|---|---|---|---|
| P2 platform-discovery semantics: CPU inventory, boot-CPU relation, PSCI/capability facts | [P2-W10](../../../p2/plans/p2-w10-p3-p4-handoff-contract.md), [P2-W02](../../../p2/plans/p2-w02-platform-discovery-normalization.md) | A documented semantic contract (not a concrete P2 type) enumerating described CPUs with identity and enablement facts, the boot CPU, and PSCI start capability | Stop; raise a prerequisite conflict as an Architecture Change Request against the P2 handoff; P3 must not re-derive DTB discovery |
| P1 entry contract: boot CPU executes Non-secure EL2 and can observe its own identity | [P1-W01](../../../p1/plans/p1-w01-reference-boot-contract.md), [P1-W03](../../../p1/plans/p1-w03-aarch64-capability-inventory.md) | Boot MPIDR readable under the documented entry state; identity/affinity among the inventoried capability facts | Stop; the boot-CPU designation invariant (README decision 4) fails closed |
| P0 quality gates: diagnostics and type-safety governance | [P0-W12](../../../p0/plans/p0-w12-logging-diagnostic-baseline.md), [P0-W13](../../../p0/plans/p0-w13-trace-event-namespace-baseline.md), [P0-W15](../../../p0/plans/p0-w15-address-identifier-type-safety.md) | Logging levels, trace-event namespace, and newtype/id governance that the enumeration diagnostic and newtypes follow | Stop and record; do not invent a private diagnostic scheme |
| Workspace, target, and crate placement | P0 build-target baseline plan | A place for the logical modules to live | Implementers park module code in the location fixed by the approved workspace design; if none exists, that is a blocker to record, not a license to create an unreviewed crate tree |

### 1.3 Why no hidden essential deliverable remains

- The plan's step 1 ("inspect P0–P2 contracts and actual evidence") is a
  precondition check, not a deliverable; it is the workflow §1
  precondition review of the implementation file.
- The plan's step 5 ("collect topology acceptance evidence") is bounded by
  the observable environment: P3 can capture single-pass enumeration output
  for each declared CPU count through the P0 QEMU entry path once it exists;
  the repeated 1/2/4/8 matrix and cold-boot coverage belong to
  [P3-W13](../p3-w13-qemu-smp-regression/README.md) by the plan index. W01's
  evidence therefore proves mapping stability per configuration at capture
  time, not regression-grade repeatability; the validation matrix states this
  boundary explicitly.

## 2. Scope classification

### 2.1 Required

- `HardwareCpuId`, `MpidrValue`, `LogicalCpuId` newtypes with the contracts
  of [03](03-code-contracts-cpu-identity.md).
- `TopologyClass ∈ {Present, Possible, Unavailable}` classification with
  rejection reasons.
- `TopologyInputs` frozen aggregate with boot-CPU designation, per-class
  counts, and start-capability facts.
- Intake validation enforcing the anti-assumption invariants and the
  8-CPU bound.
- Enumeration diagnostic output with CPU-attributed lines.
- Host-side unit tests for classification, ordering, rejection, and
  mapping-stability behavior.
- The P3-V01 evidence items listed in
  [06-validation-and-handoff.md](06-validation-and-handoff.md).

### 2.2 Reserved (must not block a future design; not implemented now)

- Post-boot mutation of topology inputs (hotplug-driven reclassification);
  trigger: an approved hotplug design.
- Admission of `Possible`-class CPUs (firmware-declared capacity beyond this
  boot's inventory); trigger: an approved hotplug or later-stage design.
- Non-AArch64 identity forms (x86_64 APIC ID newtype); trigger: the ADR-002
  sequencing that introduces x86_64.
- Socket/cluster/NUMA-level topology attributes beyond MPIDR affinity
  levels; trigger: a scheduler or placement design that needs them (P7+).
- Raising the 8-CPU inventory bound; trigger: a stage declaration extending
  the validated matrix.

### 2.3 Out of Scope

- Secondary startup requests and outcomes (P3-W02).
- Runtime lifecycle states and transitions (P3-W03).
- Per-CPU runtime state, stacks, and CPU-local access (P3-W04).
- Boot phases and rendezvous (P3-W05).
- Interrupt affinity, GIC, timer, and SGI/PPI/SPI data (P6+).
- Guest CPU or vCPU objects and any P4 mechanism (P4+).
- DTB parsing, platform discovery, or any board/SoC-specific code (P2; and
  the ADR-042 layering boundary).
- Concrete crate names, module paths, or file trees (workspace-owning
  design; ADR-054).

## 3. Resolved decisions and their authority

The entry README §Resolved design decisions records the seven decisions with
rationale. Summary index:

| # | Decision | Authority basis |
|---|---|---|
| 1 | `HardwareCpuId` opaque in the generic boundary; `MpidrValue` arch-side producer | ADR-043/ADR-052 (no arch leakage into Core), ADR-013 (newtypes) |
| 2 | Logical ids dense `0..n-1` by ascending `(Aff3,Aff2,Aff1,Aff0)`; boot CPU unprivileged | Stage-local design freedom owned here; serves W02/W04/W05 without hardware claims |
| 3 | Input classes `Present/Possible/Unavailable` owned by W01; `Online/Failed` vocabulary handed to P3-W03 | One-owner-per-transition guardrail; P3-W01 plan scope vs P3-W03 plan scope |
| 4 | Boot-CPU designation is an intake invariant with a fatal mismatch path | P1-W01 entry contract; P3 task book "boot-CPU designation" |
| 5 | Inventory bound 8, fail-closed, raise only by recorded trigger | P3 task book declared 1/2/4/8 matrix; no-unreviewed-premise rule |
| 6 | Design-level names fixed; crate/module placement reserved | ADR-054 pending crate prefix; P0 workspace plan owns layout |
| 7 | Intake adapter in platform/arch integration; Core consumes typed inputs only | ADR-042; P2-W02 handoff wording |
