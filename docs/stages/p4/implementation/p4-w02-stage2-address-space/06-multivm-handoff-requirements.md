# P4 Stage-2 to P7 multi-VM handoff requirements

Chinese readers can use the [Chinese edition](06-multivm-handoff-requirements.zh-CN.md).

**Status:** Producer extension scope authorized on 2026-10-02; W10 plan and proposed detailed design supplied; design owner-approved; implementation and evidence pending.
**Scope:** AUD-004; P4-W02/W04/W09 to P7-IN-05 and P7-W04/W05.
**Version:** v0.1
**Owner/change context:** Owner-directed documentation remediation, 2026-10-02.
**Supersedes:** Unqualified use of P4 single-space activation as a multi-VM switch contract.
**Parent:** [P4-W02 design](README.md).

## 1. Authority and baseline

P4's [task book](../../task-book-v0.1.md) permits one Validation Guest and proves
current-path consistency; multi-pCPU handling and VMID evolution are reserved.
P7's [task book](../../../p7/task-book-v0.1.md) requires multi-VM progress and
address-space isolation, but explicitly forbids repairing Stage-2 inside P7.
No Stage-2 runtime implementation is claimed. This document defines the missing
producer outcome and consumer gate; it does not expand P4 acceptance retroactively.

The earlier design permitted one live space. Its old Active(cpu) shortcut
returns without reinstalling registers on the same CPU. After A→B→A this flag
alone cannot establish which root/VMID is installed. Destroy's current-path
invalidation/deactivation also cannot safely act on A while B is installed.
The revised W02 bodies and [W10 detailed design](../p4-w10-multivm-stage2-handoff/README.md) now replace these shortcuts; implementation and target verification remain pending.

## 2. Required producer guarantees

| Obligation / owner | Required handoff | Acceptance |
|---|---|---|
| P4 Stage-2 owner: space lifetime | Distinct live spaces retain distinct root ownership and VMID identities; no undocumented VMID reuse | Same IPA maps different frames in A and B; identity exhaustion is explicit |
| P4 activation owner: installed context | One authoritative per-pCPU installed-space identity, separate from per-space lifetime and P7 current_vcpu | A→B→A checks actual selected root/VMID and generation; an Active(cpu) flag alone cannot skip installation |
| P4 activation owner: switching | Contract takes expected old context and selected new context; validates readiness and ownership before changing hardware; reports the established resulting context | Repeated same-/different-VM switch, idle and wrong-expected-context cases |
| P4 translation owner: stale translations | Track CPUs that may retain translations, not only current executors; specify mutation visibility, invalidation completion and VMID retirement | Mutate inactive A while B runs, then return to A; no stale permission or frame access |
| P4 destruction owner | No active execution, installed reference, pending selection or stale translation may outlive released tables/backing/VMID; teardown must not deactivate another space | Destroy A while B is current preserves B; failed quiescence retains resources |
| P7 lifecycle owner | Failed post-admission dispatch has an explicit owner operation restoring lifecycle/current slot consistently before requeue | Gate sets Running; activation fails before Guest entry; no duplicate Running or queue membership |

These are Required before the corresponding P7 feature is admitted. The W10 design specifies bounded identities, selection, resident completion and
non-recycled VMIDs; architecture-reference review and runtime evidence remain
producer admission obligations. P3 transport timeout is not invalidation
completion; [W08 timeout ownership](../../../p3/implementation/p3-w08-tlb-shootdown-transport/07-timeout-ownership-remediation.md)
and W08 implementation/evidence remain independent gates; the owner has
selected W08-SYNC A and its documentation contract is reconciled.

For one-pCPU multi-VM rotation, prove distinct contexts and local stale-translation
handling. This does not admit movement or simultaneous use on another pCPU.
P7 placement/affinity paths additionally need the producer's resident-CPU,
invalidation and quiescence guarantees before admitting those placements.

## 3. Failure contract required by the consumer

A producer must distinguish a refusal before hardware change, failure with a
verified known safe resulting context, and an indeterminate/partially changed
hardware context. These are semantic outcomes, not frozen Rust enum names.

Only the first two may reach scheduler retry/idle, after producer cleanup and
P7-owned admission unwind complete. Indeterminate state stops entry on the
affected execution path, retains referenced resources and follows the governing
invariant/failure policy; do not convert it to ordinary requeue or Guest Faulted.
A scheduler flag cannot certify that hardware cleanup succeeded.

P7's gate already commits Running and current_vcpu before activation. Therefore
the existing statement that failure leaves the candidate Runnable is not true
without an explicit W02-owned unwind. Do not directly reset the slot, forge a
Guest exit, or assume queue insertion performs that transition. The [W02 abort design](../../../p7/implementation/p7-w02-scheduler-admission-lifecycle/06-pre-entry-abort.md) now defines this operation, race handling and accounting; its implementation
and cleanup evidence are required before recoverable dispatch failure. The outgoing Guest's completed exit is never rolled back.

## 4. Evidence and admission gates

| Gate | Required artifact | Current status |
|---|---|---|
| S2-MULTIVM-SCOPE | Producer-owned extension plan/design with stage/scope authorization and P7 consumer review | Scope authorized; [W10 plan](../../plans/p4-w10-multivm-stage2-handoff.md) and [design](../p4-w10-multivm-stage2-handoff/README.md) supplied; owner approval recorded; implementation/evidence pending |
| S2-INSTALL | Installed-context ownership and switch/failure contract, architecture review, implementation | Design supplied in W10; design owner-approved; implementation and evidence pending |
| S2-RETIRE | Mutation, resident translations, destruction and VMID/backing retention contract | Design supplied in W10; design owner-approved; implementation and evidence pending |
| P7-DISPATCH-UNWIND | W02-owned post-gate/pre-entry abort and W04 use, with lifecycle/slot/queue accounting | Companion design supplied and W04 linked; implementation/producer cleanup evidence pending |

P4-W09 must carry these limits as missing producer deliverables. P7-W01 must
mark affected IN-05 capabilities blocked rather than infer them from P4 completion.
P7-W04/W05/W08 consume only evidenced capabilities. W10/W11 must include A→B→A
with equal IPA/different canaries, inactive-space mutation, destroy-inactive-A
while B remains current, failed selection before/after hardware change, and
post-gate unwind. Map these to P7-V09/V11, and P7-V17 for cross-pCPU coverage;
P4-V02/V07/V08 single-path evidence is retained at its original scope.

Static review identifies the counterexamples above; no Host, QEMU or hardware
execution occurred in this remediation. No production API/ABI, unsafe,
dependency, accepted ADR or existing stage completion claim is changed.
AUD-004 remains open until producer scope/design and actual evidence close
these gates; the consumer's unsupported readiness and retry claims are removed.

## 5. Producer-scope decision record (authorized)

Owner-approved direction: a separately identified **P4-owned Stage-2 extension**
for the P7 handoff ([P4-W10](../../plans/p4-w10-multivm-stage2-handoff.md)), with its own task-book supplement, bounded
plan, approved detailed design and verification record. Preserve the original
P4 single-Validation-Guest acceptance and P5 handoff; extension evidence is an
additional P7 prerequisite, never a retroactive claim about base P4 completion.

The owner authorized preparation and reconciliation of this producer extension:

- Required: multiple live spaces with separate roots and distinct non-recycled
  VMIDs; explicit bounded capacity/exhaustion; authoritative per-pCPU installed
  context; validated old→new selection and safe failure classification.
- Required: inactive-space mutation visibility, resident-CPU tracking, bounded
  invalidation completion, and resource-retaining retirement. A local-only
  milestone may admit one-pCPU rotation only; P7 placement across CPUs stays
  blocked until resident translation/quiescence evidence exists.
- Required companion: P7-W02 owns post-gate/pre-entry admission unwind; P7-W04
  consumes it only after the producer establishes a known safe hardware context.
  The scheduler neither writes Stage-2 hardware nor invents completion authority.
- Reserved: VMID recycling/rollover, CPU hotplug, dynamic VM provisioning,
  migration policy and throughput optimizations. Reserving these must not remove
  the cross-pCPU residency and retirement needed by admitted P7 placement.
- Dependencies: evidenced base P4 memory/Guest contracts, P2 backing ownership,
  P3 synchronization/transport and their completion/lifetime boundaries.
  Planning authorization is not permission to bypass those implementation gates.
- Acceptance: A→B→A with equal IPA/different backing; inactive-A mutation and
  destruction while B stays valid; identity exhaustion; wrong-old-context refusal;
  safe versus indeterminate switch faults; post-gate unwind without duplicate
  Running/queue membership; cross-pCPU stale-translation and failed-quiescence
  retention cases. Host model and QEMU execution are recorded separately.

The owner approved this packet on 2026-10-02 by answering "同意". The
[P4 task-book supplement](../../task-book-v0.1.md#9-owner-authorized-p7-producer-extension)
and [W10 plan](../../plans/p4-w10-multivm-stage2-handoff.md) now record the
bounded producer scope. P7 retains its "no Stage-2 redesign" boundary.
S2-MULTIVM-SCOPE's ownership/scope choice is resolved; detailed-design owner approval is now recorded; implementation and
consumer runtime acceptance remain pending, and no implementation or runtime
admission follows. No accepted ADR is edited; a detailed-design ADR conflict
must follow the ADR change process.

## 6. Detailed-design delivery (2026-10-02)

W10 now specifies independent roots/non-recycled VMIDs, per-CPU installed
context, execution leases, resident history, BBM, full retirement, retained
failure and V17–V22. Revised W02/W03 bodies replace the earlier Active(cpu),
raw-range authority and premature-release sketches. P7-W02's companion abort
specifies atomic lifecycle/slot cleanup, producer receipts, control races and
queue accounting, and W04 consumes it. SCOPE/INSTALL/RETIRE/UNWIND no longer lack
proposed design bodies; implementation and real architecture/runtime
evidence remain pending. AUD-004 is still a delivery gate, not runtime-closed.
