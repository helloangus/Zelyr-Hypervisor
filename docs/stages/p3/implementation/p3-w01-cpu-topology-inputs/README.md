# P3-W01 CPU Topology Inputs — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The reviewed P2→P3 contract for physical-CPU topology, hardware
identity, logical identity, boot-CPU designation, and availability
classification required by [P3-W01](../../plans/p3-w01-cpu-topology-inputs.md).  
**Owner/change context:** P3-W01 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P3-W01. It converts the bounded
work-package plan into typed, reviewable CPU-topology foundations: identity
newtypes, an availability-classification vocabulary, a frozen topology-input
aggregate, and the intake rules that turn P2 platform-discovery facts into
P3's stable view of the machine's physical CPUs. It deliberately does **not**
start secondary CPUs ([P3-W02](../p3-w02-secondary-cpu-bring-up/README.md)),
define the physical-CPU lifecycle state machine
([P3-W03](../p3-w03-physical-cpu-lifecycle/README.md)), create per-CPU runtime
state ([P3-W04](../p3-w04-per-cpu-runtime/README.md)), or coordinate boot
([P3-W05](../p3-w05-smp-boot-synchronization/README.md)).

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then loads
only the linked supporting file needed for its assigned step:

| Supporting file | Load it for |
|---|---|
| [01-scope-and-foundations.md](01-scope-and-foundations.md) | goal-to-baseline ledger, scope classification, resolved decisions |
| [02-architecture-and-state.md](02-architecture-and-state.md) | logical modules, ownership, immutability and publication model |
| [03-code-contracts-cpu-identity.md](03-code-contracts-cpu-identity.md) | identity newtype contracts and pseudocode |
| [04-code-contracts-topology-intake.md](04-code-contracts-topology-intake.md) | classification and intake contracts and pseudocode |
| [05-implementation-workflow.md](05-implementation-workflow.md) | ordered implementation steps |
| [06-validation-and-handoff.md](06-validation-and-handoff.md) | validation matrix, failure model, handoff checklist |

Before editing it must also follow the Coding Guidelines preflight, including
the repository `AGENTS.md`, documentation index, ADR baseline, P3 task book,
and the P3-W01 plan. This document is a proposed design; it contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P3 task book → P3-W01 plan → this design
→ Coding Guidelines. Binding constraints:

- ADR-002 keeps the first implementation AArch64-first; ADR-003 makes QEMU
  `virt` the reference environment. QEMU behavior observed here is reference
  evidence, never an architecture contract, and no Core code may branch on a
  platform or board name (ADR-043, ADR-052).
- ADR-015 requires early SMP so no single-core assumption forms; this package
  supplies the identity and classification inputs that make per-CPU reasoning
  possible from the first instruction of P3 code.
- ADR-041–ADR-045 require capability-driven behavior and platform layering.
  Core consumes typed topology inputs; only a platform/architecture
  integration module may interpret DTB-derived or MPIDR-specific facts
  (ADR-042).
- ADR-013/ADR-046 newtype discipline: identities are semantic newtypes, never
  naked `usize`/`u64`.
- The P3 task book forbids assuming contiguous hardware identities, equating
  enumerated order with identity, or treating present as online; the plan
  repeats these as explicit rejections. This design encodes each rejection as
  a validation rule, not as prose alone.
- The [P2→P3 handoff contract](../../../p2/plans/p2-w10-p3-p4-handoff-contract.md)
  (P2-W10) is the assumed source of P2 inputs: CPU inventory, boot-CPU
  relation, and PSCI/capability facts, delivered as a documented semantic
  contract ([P2-W02](../../../p2/plans/p2-w02-platform-discovery-normalization.md)
  explicitly withholds concrete `PlatformInfo` APIs from P3). If P2 delivers
  a different shape or no evidence, that is a prerequisite failure boundary
  (stop; do not re-derive discovery in P3).

Classification:

- **Required** for W01 closure: identity types
  ([03](03-code-contracts-cpu-identity.md)), availability classification and
  intake ([04](04-code-contracts-topology-intake.md)), boot-CPU designation,
  the frozen `TopologyInputs` aggregate, topology enumeration diagnostics, and
  the P3-V01 acceptance evidence.
- **Reserved** with recorded triggers: runtime CPU hotplug and any
  post-boot mutation of topology inputs; identity forms for non-AArch64
  architectures (x86_64 APIC IDs, ADR-002 sequencing); `possible`-class CPU
  admission (the class exists here; starting such a CPU is future work);
  NUMA/socket-level topology detail beyond MPIDR affinity levels.
- **Out of Scope:** secondary startup and all of P3-W02's outcome model; the
  runtime lifecycle state machine and its transitions (P3-W03); per-CPU
  runtime state and stacks (P3-W04); boot rendezvous (P3-W05); GIC, timer,
  and interrupt affinity data; guest CPU and vCPU objects (P4+); board- or
  SoC-specific discovery logic; concrete crate and file placement (reserved
  to the workspace-owning approved design; this design fixes logical modules
  only).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| P2→P3 topology input boundary (facts in, typed values out) | [intake contract](04-code-contracts-topology-intake.md) §2–§3 | P3-V01 (W01-DV01, DV02) |
| Hardware identity as MPIDR-typed newtype, no contiguous-identity assumption | [identity contract](03-code-contracts-cpu-identity.md) §2–§3 | P3-V01 (W01-DV01) |
| Logical identity with stable assignment rule | [identity contract](03-code-contracts-cpu-identity.md) §4, [intake contract](04-code-contracts-topology-intake.md) §4 | P3-V01 (W01-DV02) |
| Availability classification: present, possible, unavailable; online/failed vocabulary boundary | [intake contract](04-code-contracts-topology-intake.md) §3, [architecture](02-architecture-and-state.md) §4 | P3-V01 (W01-DV03) |
| Boot-CPU designation and inventory match | [intake contract](04-code-contracts-topology-intake.md) §5 | P3-V01 (W01-DV04) |
| Anti-assumption rejections (order ≠ identity, identity gaps, present ≠ online) | [intake contract](04-code-contracts-topology-intake.md) §3.4, [workflow](05-implementation-workflow.md) step 3 | P3-V01 (W01-DV01, DV03) |
| Topology evidence for declared QEMU CPU counts (1/2/4/8) | [workflow](05-implementation-workflow.md) step 5; matrix items W01-DV05/DV06 in [validation](06-validation-and-handoff.md) | P3-V01 (W01-DV05); repeated-matrix execution is [P3-W13](../p3-w13-qemu-smp-regression/README.md) |
| Contract, limits, and consumers recorded | [workflow](05-implementation-workflow.md) step 6, [handoff](06-validation-and-handoff.md) §3 | W01 closure review (W01-DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p3-implementation-designs` at
`4e631ee`): the repository is a P0 documentation scaffold. There is no Cargo
workspace, no Rust source, and `crates/`, `hypervisor/src/`, `boards/`, and
`soc/` contain only `.gitkeep` markers. P0–P2 planning sets exist as plans;
only P0-W01/P0-W02 have implementation and verification records. No P3
implementation or verification material exists, and the sibling P3 designs
(W06–W15) are being prepared in parallel on this branch — this design
references them by path and P3-Wxx ID without assuming their content. Each
ledger row below states the missing foundation the plan outcome necessarily
requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Reviewed P2→P3 topology contract exists | No P3-side topology artifact of any kind is tracked | The boundary contract in [04](04-code-contracts-topology-intake.md): what P2 semantics are consumed, what P3 types result, what is rejected | Without a declared boundary every downstream package would re-derive its own view of the machine | W01 (this design); P2-W10 supplies input facts | W01-DV01/DV02 reviews; P2 evidence prerequisite per task book §2 |
| Hardware identity is typed and non-contiguity is impossible to assume | No identity type exists | `HardwareCpuId`/`MpidrValue` newtypes with affinity-level semantics and malformed-value rejection | A u64 identity cannot express "identities are MPIDR affinities, not indices" | W01 | W01-DV01 type review plus host-side unit tests |
| Logical identity is stable and boot-CPU-independent | No logical id concept exists | `LogicalCpuId` plus the deterministic assignment rule of [04 §4](04-code-contracts-topology-intake.md) | Consumers (W02 bring-up order, W04 lookup tables) need a dense, reproducible index that is *not* a hardware claim | W01 (rule is stage-local design freedom, recorded) | W01-DV02 mapping-stability tests |
| Present / possible / online / failed are classified without equating present with online | No classification exists | `TopologyClass` input vocabulary plus the documented handoff point where P3-W03's runtime states take over | The task book requires the distinction; conflating it would let bring-up start unavailable CPUs | W01 owns input classes; P3-W03 owns runtime transitions | W01-DV03 classification tests |
| Boot CPU is identified against the inventory | Nothing designates the boot CPU in P3 terms | Boot-CPU integration rule in [04 §5](04-code-contracts-topology-intake.md), including the fatal mismatch path | The boot CPU's identity anchors bring-up and rendezvous; an unmatched boot identity is a machine-model violation | W01; boot MPIDR observation arrives under the P1 entry contract | W01-DV04 |
| Unavailable CPUs are excluded from bring-up candidates | Nothing excludes them | `TopologyClass::Unavailable` rejection reasons and exclusion invariants in [04 §3](04-code-contracts-topology-intake.md) | P3-V01 requires exclusion; silently starting a declared-but-rejected CPU would be opaque | W01 | W01-DV03 |
| Topology acceptance evidence for declared QEMU CPU counts | No QEMU runner is implemented (P0-W09 is a plan); no regression harness (P3-W13 is a plan) | Enumeration diagnostic output reachable at boot, exercised once per declared count via the P0 QEMU entry path | P3-V01 wording requires cross-count evidence; the repeated matrix itself belongs to W13 | W01 for the output and single-pass evidence; W13 for the matrix | W01-DV05/DV06; matrix evidence in W13 |

The P2 open item P2-ACR-01 (recorded in the
[P2 task book](../../../p2/task-book-v0.1.md) §3) concerns `MemoryObject`/
`MemoryRegion` structures and is not consumed by W01; it remains visible for
[P3-W10](../p3-w10-smp-safety-audit/README.md) and P3 closure. No ledger row
above requires this design to select a crate name, target triple, or runtime
policy, so no new decision blocker is outstanding here.

## Resolved design decisions and their authority

1. **Layer split of identity typing.** `HardwareCpuId` is an opaque newtype in
   the generic topology boundary; `MpidrValue` is an architecture-side newtype
   with documented affinity-field layout and is the only sanctioned producer
   of `HardwareCpuId` on AArch64. Rationale: ADR-043/ADR-052 forbid Core from
   seeing architecture registers; ADR-013 newtype discipline forbids raw
   integers. Crate placement is reserved (see decision 6).
2. **Logical id assignment rule.** Logical ids are dense `0..n-1` assigned by
   ascending canonical order of the MPIDR affinity tuple
   `(Aff3, Aff2, Aff1, Aff0)`. The boot CPU takes whatever logical id its
   affinity sorts to; it is never privileged by the rule. Rationale:
   deterministic across boots, independent of DTB enumeration order (the plan
   forbids equating enumerated order with identity), and it makes logical ids
   usable as array indices downstream without assuming hardware contiguity.
   Stage-local design freedom owned here; a future hotplug design may revisit
   the density invariant.
3. **Availability vocabulary split.** W01 owns the input classification
   `TopologyClass ∈ {Present, Possible, Unavailable}` computed at intake;
   `Online` and `Failed` are runtime states owned by
   [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md), and W01 documents the
   relation (`Online ⊆ attempted Present`, `Failed ⊆ attempted`) without
   defining the transition machine. Rationale: the plan requires the
   classification vocabulary, but every mutable transition must have exactly
   one owner, and runtime states belong to the lifecycle package.
4. **Boot-CPU designation is an intake invariant, not a runtime choice.** The
   boot CPU is the CPU executing under the
   [P1 entry contract](../../../p1/plans/p1-w01-reference-boot-contract.md);
   its MPIDR must map to a `Present`-class entry or intake fails fatally
   before any bring-up work. Rationale: continuing without a verified boot
   identity would violate "no CPU usable before its identity is known" and
   every downstream package consumes the boot designation.
5. **Inventory bound.** P3 accepts at most 8 physical CPUs per inventory
   (the declared QEMU matrix is 1/2/4/8). A larger declared inventory fails
   intake closed with a diagnostic; it is never truncated. Rationale: the
   task book's declared validation scope is 1/2/4/8; a silent larger bound
   would be an unreviewed premise. Raising the bound is a recorded revisit
   trigger, not a code change by itself.
6. **Naming and placement.** Type names in this design (`HardwareCpuId`,
   `MpidrValue`, `LogicalCpuId`, `TopologyClass`, `TopologyInputs`) are
   design-level identifiers fixed as stage-local design freedom; the concrete
   crate, module path, and file layout are reserved to the workspace-owning
   approved design (ADR-054 leaves the crate prefix open at ADR level). The
   binding contract here is responsibilities and boundaries, not paths.
7. **Intake adapter ownership.** The conversion from P2 semantic facts to
   `TopologyInputs` lives in the platform/architecture integration layer;
   generic topology code never interprets DTB cells or MPIDR bits. Rationale:
   ADR-042 and the P2-W02 boundary ("P3 may consume only the documented
   semantic contract, not a particular type or implementation").

## Work breakdown and loading order

1. Read [01-scope-and-foundations.md](01-scope-and-foundations.md) for the
   ledger, the Required/Reserved/Out-of-Scope split, and the decisions above.
2. Read [02-architecture-and-state.md](02-architecture-and-state.md) to
   understand the four logical modules, their ownership, and the
   construct-validate-freeze-publish model that keeps topology inputs
   immutable after intake.
3. Implement per the ordered steps in
   [05-implementation-workflow.md](05-implementation-workflow.md), loading
   [03](03-code-contracts-cpu-identity.md) for identity types (steps 1–2) and
   [04](04-code-contracts-topology-intake.md) for classification and intake
   (steps 3–4).
4. Record implementation decisions in
   `../p3-w01-cpu-topology-inputs-record.md` and evidence in
   `../../verification/p3-w01-cpu-topology-inputs-verification.md` only when
   the corresponding work is actually performed; neither this design nor a
   written record may claim W01 complete. Validation conditions and the
   handoff checklist are in
   [06-validation-and-handoff.md](06-validation-and-handoff.md).

## Explicitly excluded interfaces

No CPU-start call, lifecycle transition function, per-CPU storage accessor,
boot-barrier primitive, interrupt routing interface, or guest-visible surface
is designed or authorized by W01. The only behavioral surface is intake:
build, validate, classify, freeze, and report. Downstream packages that need
mutable state must define it in their own designs:
[P3-W02](../p3-w02-secondary-cpu-bring-up/README.md) (start outcomes),
[P3-W03](../p3-w03-physical-cpu-lifecycle/README.md) (transitions),
[P3-W04](../p3-w04-per-cpu-runtime/README.md) (per-CPU areas),
[P3-W05](../p3-w05-smp-boot-synchronization/README.md) (boot phases). Any
"topology update" or "online CPU count" API appearing in W01 code is a scope
violation and must be stopped at review.

## Downstream handoff

- **W02** receives `TopologyInputs` as the bring-up candidate list: the
  `Present`-class set in logical order, the boot-CPU designation, and the
  recorded CPU-start capability facts (PSCI conduit and function identifiers
  from P2). W02 must not attempt `Possible` or `Unavailable` CPUs.
- **W03** receives the identity/classification vocabulary and the invariant
  that intake is immutable after freeze; the registry seeds from
  `TopologyInputs` and owns every runtime transition from that point.
- **W04** receives the density invariant on `LogicalCpuId` (dense `0..n-1`),
  which justifies index-based per-CPU lookup without a hardware assumption.
- **W05** receives the boot-CPU designation as the rendezvous coordinator
  identity and the inventory bound as the expected-attempted-set source.
- **W10** (SMP audit) receives the anti-assumption invariants (no
  enumerated-order identity, no contiguity assumption, present ≠ online) as
  review criteria for P0–P2 infrastructure.
- **P4** consumes the stable pCPU-identity contract through
  [P3-W14](../p3-w14-p4-smp-handoff/README.md); W01 defines pCPU identity
  only and must not be read as defining vCPU identity.
