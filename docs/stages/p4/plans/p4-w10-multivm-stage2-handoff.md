# P4-W10 — Multi-VM Stage-2 producer handoff

Chinese readers can use the [Chinese edition](p4-w10-multivm-stage2-handoff.zh-CN.md).

**Status:** Authorized work package; detailed design owner-approved on 2026-10-02; implementation and validation pending
**Version:** v0.1
**Owner/change context:** AUD-004 / S2-MULTIVM-SCOPE; owner approved the reviewed P4 extension on 2026-10-02 ("同意")
**Parent:** [P4 task book, extension supplement](../task-book-v0.1.md#9-owner-authorized-p7-producer-extension)
**Prerequisites and consumers:** [P4 plan index](README.md)

## Goal

Provide an evidence-backed Stage-2 producer contract for P7 to select among
multiple live VM address spaces and retire translations/resources safely,
without implementing Stage-2 mechanisms inside the scheduler.

## Scope

Required P4-X01–X06, with P4-V17–V22 defined in the task-book supplement:
separate live roots and distinct non-recycled VMID identities; bounded capacity
and explicit exhaustion; authoritative per-pCPU installed context separate from
space lifetime; validated context switching and failure outcomes; inactive-space
mutation, resident-CPU translation tracking, invalidation completion and retained
resources on failed quiescence; a capability/evidence handoff for P7.

This is a separately accepted extension after the base W01–W09 contract. It does
not expand P4-V01–V16 or retroactively change the P5 single-Guest handoff. A
local-only milestone may admit one-pCPU rotation, but cannot close W10 or admit
P7 cross-pCPU placement without the remaining producer evidence.

Required inputs are evidenced base P4 contracts; P2 allocated/mapped backing and
ownership; and P3 CPU identity, synchronization, notification and completed
transport, including timeout lifetime and TransportBusy handling. Missing inputs
block corresponding implementation/evidence, not the writing of this plan.
The [AUD-004 producer requirements](../implementation/p4-w02-stage2-address-space/06-multivm-handoff-requirements.md)
provide counterexamples and handoff obligations, not an approved detailed design.

Reserved: VMID recycling/rollover, CPU hotplug, dynamic VM provisioning,
migration policy, larger-page/batching and throughput optimization. These
reservations do not remove the resident-CPU and retirement obligations of
already-admitted P7 placement.

## Out of scope

P7 run queues, scheduling/affinity policy, fairness and lifecycle transition
ownership; P5 public ABI/capabilities; P8 machine definition; P2/P3 foundation
implementation; automatic recovery of indeterminate hardware state; new public
APIs, module layouts, register sequences or algorithms prescribed by this plan.
P7-W02 owns post-gate/pre-entry admission unwind; W10 supplies its safe-context
precondition, never directly resets scheduler state.

## Work sequence

1. Reconcile actual P2/P3 and base P4 evidence with the producer requirements;
   enumerate missing inputs and distinguish base completion from this extension.
2. Produce and review the detailed design for context authority, identity,
   mutation/residency, completion, retirement, failure retention and telemetry.
   Reconcile P7-W02/W04 consumer requirements without making their implementation
   a circular prerequisite; obtain detailed-design admission before coding.
3. Implement the approved producer extension and integrate with base P4 backing,
   table and entry/exit ownership, preserving the single-Guest regression path.
4. Verify host-model ownership, selection, exhaustion, inactive mutation and
   teardown, safe/indeterminate failure outcomes and quiescence retention;
   record the tested model limits separately from target execution.
5. Establish reference-QEMU local and cross-pCPU context/translation evidence,
   including equal-IPA/different-backing isolation and failed-completion resource
   retention, with repeatable inputs and evidence custody.
6. Publish the producer capability matrix, detailed-design/implementation records,
   evidence and open restrictions for W09 and P7-W01/W02/W04/W05/W08/W10/W11.
   Keep any unsupported consumer capability blocked.

## Acceptance and closure

P4-V17–V22 must each have actual evidence at their declared layer. Creation
restriction removal alone is insufficient: A→B→A must select the right root and
VMID; mutation of inactive A and destruction of A must preserve current B;
translation residency includes CPUs no longer executing that space. No table,
backing allocation or identity can be reused based on timeout or a scheduler
flag. Distinguish refusal before hardware change, verified safe context and
indeterminate context; the last retains resources and prevents Guest entry.

Full W10 closure requires both local and cross-pCPU producer guarantees. A Host
model is not QEMU execution; QEMU is not hardware proof. P7 post-admission unwind
and scheduler workloads remain P7-owned evidence (P7-V09/V11/V17/V24 as applicable),
not W10 producer completion. No acceptance row is satisfied by this plan.

## Handoff

W09 records the separately evidenced extension without rewriting base P4/P5
completion. P7-W01 consumes condition-specific capability status; W02/W04 consume
known-safe context outcomes and own admission unwind; W05 consumes independent
VM selection; W08 consumes only proven resident-CPU/quiescence capabilities;
W10/W11 consume isolation/failure scenarios. P7 may not infer readiness from
scope authorization or the base single-space contract.

Detailed design and implementation traceability belong in ../implementation/;
actual evidence belongs in ../verification/. The owning design resolves
Implementation Choices (identity capacity, state representation, serialization),
Specification Investigations (architecture ordering/invalidation) and Platform
Investigations (declared reference configuration). Any ADR conflict remains ADR
Required; this scope authorization does not approve changing an accepted ADR.

Owner follow-up (2026-10-02): [P2-W12](../../p2/plans/p2-w12-minimal-memory-objects.md)
is the selected common backing/view producer. W10 must reconcile W02/W03 adapters
against its approved contracts and actual evidence; no independent frame owner
or raw MappingGrant range substitutes for that authority. Formal ADR integration
and W12 delivery are pending and do not invalidate the W10 scope authorization.

The [detailed design](../implementation/p4-w10-multivm-stage2-handoff/README.md) is owner-approved on 2026-10-02; implementation and runtime evidence remain separate.
