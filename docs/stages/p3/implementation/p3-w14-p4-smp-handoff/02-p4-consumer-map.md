# P3-W14 P4 Consumer Map and Contract Structure

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W14 detailed design](README.md).

## 1. Status and role of this file

This file fixes the *structure* of the handoff contract artifact and its
consumer mapping. The artifact itself
(`docs/stages/p3/implementation/p3-w14-p4-smp-handoff-contract.md`) is
created only when W14 executes with a real evidence inventory; nothing
here asserts its content beyond structure and source obligations.

## 2. Contract structure (R1–R9)

The contract document contains, in order: a versioned status header
(status, version, supersedes, date, owner); the reliance sections R1–R9;
limits and non-guarantees; the unresolved-items register; the
independence list; the consumer map (§3 condensed); and a pointer table
to every cited plan/design/record. Each reliance section has the same
internal shape:

```text
Rn <name>
  P4 may rely on: <one-sentence guarantee, P3's own terms>
  Source contract: P3-Wxx — <plan path>, <design path>
  Evidence: <verification-record path> — status: evidenced | planned | blocked
  Bound: <what the guarantee does not cover>
```

A section whose evidence status is not `evidenced` is rendered as a
limitation (it states the intended source and the missing evidence), and
the item enters the unresolved-items register. No section may state a
guarantee its evidence does not carry.

| ID | Reliance section | Primary P3 source(s) |
|---|---|---|
| R1 | Logical pCPU identity and topology classification: `LogicalCpuId` density and stability, `HardwareCpuId` opacity, boot-CPU designation, class vocabulary, inventory bound ≤ 8, frozen-after-intake | [P3-W01](../p3-w01-cpu-topology-inputs/README.md) |
| R2 | Secondary bring-up outcomes: per-CPU terminal results with phase attribution, bounded-poll semantics, induced absent-CPU failure input, quarantine of failed-CPU scaffolding | [P3-W02](../p3-w02-secondary-cpu-bring-up/README.md) |
| R3 | Physical-CPU lifecycle: registry as sole authority, state machine and legal transitions, exactly-once online admission, `Eligibility` gate, `OnlineSet` snapshot, Failed-is-terminal (no hotplug/recovery at P3) | [P3-W03](../p3-w03-physical-cpu-lifecycle/README.md) |
| R4 | Per-CPU state foundations: `PerCpuArea` layout and header, per-CPU stacks, `TPIDR_EL2`-based `current()`, reserved regions (notification/TLB/exception slots, opaque P4 reservation — contents undefined at P3), no-Drop/no-free lifetime | [P3-W04](../p3-w04-per-cpu-runtime/README.md) |
| R5 | Boot synchronization: `BootPhase` ordering authority, publication gate, per-CPU ready gate, one-shot rendezvous, `SmpReadyState::{Ready,Degraded}` with failed-set accounting; continuation policy status | [P3-W05](../p3-w05-smp-boot-synchronization/README.md) |
| R6 | Shared-state synchronization rules: the W06 lock/atomic/IRQ-context policy and lock-order rules; allocator/registry SMP-safety classification per the W10 audit | [P3-W06](../p3-w06-concurrency-synchronization/README.md), [P3-W10](../p3-w10-smp-safety-audit/README.md) |
| R7 | Cross-CPU notification and TLB transport: W07 targeting universe over the online set, send/receive semantics and invalid-target behavior; W08 transport request/acknowledgement/completion framing (Stage-2 TLBI *semantics* explicitly excluded — P4 designs the invalidation operation on top) | [P3-W07](../p3-w07-cross-cpu-notification/README.md), [P3-W08](../p3-w08-tlb-shootdown-transport/README.md) |
| R8 | CPU-attributed faults and exception foundations: per-CPU exception/interrupt local state and attribution guarantees (with the P1 baseline), fatal-path CPU attribution | [P3-W09](../p3-w09-cpu-local-exception-interrupt/README.md), P1-W05 (by reference) |
| R9 | Audit and telemetry rules: observability contract summary (counters, event inventory, attribution model, read surfaces), non-guarantees (no timing criteria, no cross-CPU coherence, event ids versioned not frozen), regression matrix contract and its evidence status, stress/failure evidence status and limits | [P3-W11](../p3-w11-smp-observability/README.md), [P3-W12](../p3-w12-smp-stress-failure-tests/README.md), [P3-W13](../p3-w13-qemu-smp-regression/README.md) |

The limits/non-guarantees section must additionally carry the stage-level
statements: no hardware proof (QEMU reference only; P15 owns hardware),
no performance results, no guest/Stage-2 semantics of any kind, and the
P2-inherited limitations (by reference to [P2-W10](../../../p2/plans/p2-w10-p3-p4-handoff-contract.md),
including P2-ACR-01's status).

## 3. Consumer map (P4 packages → P3 deliverables)

Keyed to the [P4 plan index](../../../p4/plans/README.md) and the
[P4 task book](../../../p4/task-book-v0.1.md) §2 consumption list; the
map states what each P4 package consumes and via which reliance sections,
so the sufficiency review (workflow step 5) can walk it mechanically.

| P4 package | Consumes from P3 | Sections |
|---|---|---|
| [P4-W01](../../../p4/plans/p4-w01-entry-contract-reconciliation.md) entry-contract reconciliation | the whole handoff: contract artifact, evidence statuses, unresolved register | R1–R9 |
| [P4-W02](../../../p4/plans/p4-w02-stage2-address-space.md) Stage-2 address space | TLB transport for later invalidation integration; shared-state rules for mapper concurrency; pCPU identity for per-CPU context | R6, R7, R1 |
| [P4-W03](../../../p4/plans/p4-w03-guest-memory-image.md) guest memory/image | allocation availability and ownership accounting as inherited from P2 via P3's audit; online-set awareness for any per-CPU staging | R6, R3 (P2-W10 by reference) |
| [P4-W04](../../../p4/plans/p4-w04-vcpu-entry-exit.md) vCPU entry/exit | CPU-local state foundation (`current()`, reserved capacity — contents P4 designs); exception attribution foundations; lifecycle availability semantics (`Online` eligibility) — explicitly *not* a vCPU state machine | R4, R8, R3 |
| [P4-W05](../../../p4/plans/p4-w05-validation-guest.md) validation guest | nothing directly from P3 beyond the execution foundations above (guest is P4's design) | via P4-W04 |
| [P4-W06](../../../p4/plans/p4-w06-fault-isolation-diagnostics.md) fault isolation | CPU-attributed fault/exception diagnostics; guest-caused vs hypervisor-invariant separation rules as P3 hands them to the panic policy | R8, R5 (attribution context) |
| [P4-W07](../../../p4/plans/p4-w07-repeatability-telemetry.md) repeatability/telemetry | observability contract: namespace, counter/event patterns, attribution model; boot-order repeatability rules | R9, R5 |
| [P4-W08](../../../p4/plans/p4-w08-qemu-integration-regression.md) QEMU regression | the P0 automation contract as exercised by P3's matrix; result-classification conventions; the reference-platform boundary | R9 |
| [P4-W09](../../../p4/plans/p4-w09-closeout-p5-handoff.md) closeout/P5 handoff | the P3 evidence locations and unresolved items as inputs to P4's own closeout | R1–R9 |

Rules: the map names consumption *needs* from the P4 planning set; it
never assigns P4 design decisions. Where a P4 package's need exceeds a
section's bound (e.g. P4-W04 needing more than the reserved capacity's
opacity), that is an insufficiency finding for the review step, recorded
in the contract — not a W14-designed extension.

## 4. Independence list (what P4 must design without P3)

The contract states, with the same prominence as the reliance sections,
that P4 independently designs:

1. Stage-2 address-space semantics: layout, map/unmap/protect algorithms,
   VTCR/VTTBR usage, break-before-make rules.
2. The concrete TLB invalidation operation and its use of the W08
   transport (P3 supplies transport and completion framing only).
3. VMID allocation and TLB-generation caching.
4. VM/vCPU objects and lifecycle (P3's `PhysicalCpu` and lifecycle
   vocabulary must not be read as defining them; static vCPU↔pCPU binding
   is P4/P7 policy).
5. Guest EL1 execution, entry/exit mechanics, and guest PSCI (P8).
6. Any synchronization P4 needs beyond W06's published policy.
7. Guest-visible telemetry and any guest-facing surface.

Each item cites the reserving authority (P3 task book §2 Reserved list;
P4 task book §1 Required/Reserved) so the boundary is checkable, not
traditional.
