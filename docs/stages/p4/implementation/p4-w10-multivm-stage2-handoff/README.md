# P4-W10 multi-VM Stage-2 producer extension — detailed design

Chinese readers can use the [Chinese edition](README.zh-CN.md).

**Status:** Approved detailed design (project owner) v0.1, 2026-10-02; implementation and runtime evidence are not claimed.
**Parent:** [W10 plan](../../plans/p4-w10-multivm-stage2-handoff.md), X01–X06 / V17–V22.
**Authority:** [P4 task book §9](../../task-book-v0.1.md),
[producer requirements](../p4-w02-stage2-address-space/06-multivm-handoff-requirements.md),
[Coding Guidelines](../../../../development/coding-guidelines.md).
**Supersedes:** No accepted ADR; extends the revised W02 single-CPU mechanism.

**Approval:** Project owner, 2026-10-02, explicit confirmation “我确认批准”; see the [approval record](../../../../testing/documentation-audit/design-approval.md#owner-approval). Approved as the implementation design; its prerequisite, architecture/fit and runtime gates remain in force.

## 1. Baseline and foundation ledger

P4 remains design documentation without production implementation; this checkout has no production Stage-2
mapper, multi-VM execution or P7 scheduler. W02/W03 now consume the
[W12 object/region design](../../../p2/implementation/p2-w12-minimal-memory-objects/README.md)
and [W11 Host access design](../../../p2/implementation/p2-w11-host-allocated-frame-mapping/README.md).
P3-W06/W08's reviewed protocol supplies short data locking and admitted remote
execution, not a Stage-2 retirement proof. No Host fake proves register installation.

| Requirement | Foundation / owner | Design below and required evidence |
|---|---|---|
| X01 multiple live spaces | W02 roots/VMIDs; W12 backing | Separate root, object and non-recycled VMID per space; V17 isolation |
| X02 each dispatch installs context | W10 per-CPU selector; W04 world switch | Exact installed identity and entry lease; V18 A→B→A |
| X03 inactive mutation/destruction | W10 lifetime coordinator | Resident history, frozen transactions and B-preserving invalidation; V19 |
| X04 cross-pCPU retirement | P3-W06 serialization, W08 transport, W10 TLBI adapter | Bound completion from every resident CPU; V20 |
| X05 safe failure | W10 result classification; P7 owns abort | No implicit rollback of scheduler state; V21 injection |
| X06 consumer evidence | P4 evidence producer / P7 admission consumers | Capacity/profile/cpu-set evidence bundle; V22 handoff |

Production prerequisites: evidenced W01–W09 base capabilities, W12/W11 producer
paths and P3 locking/transport. P7 consumer contract review is required, but P7
runtime implementation is not a prerequisite for producing W10 tests. No cycle
from W10 evidence back to P7 runtime is introduced.

Required extension profile: ordinary RAM, 4 KiB Stage-2, non-VHE, one fixed
validated VTCR geometry per manager, multiple roots, non-recycled VMIDs, explicit
cross-CPU completion. Reserved: VMID recycling, migration policy and recovery
from unknown hardware state. Excluded: IOMMU/DMA, dirty tracking, huge pages,
device virtualization, device reassignment and shared direct-console mappings.
W02 base console tests remain separate. W10 rejects a console-backed space
before registration; extension test Guests report through saved exit registers
on a controlled synchronous exception, not a shared PL011 or new hypercall ABI.

## 2. Logical modules, identities and bounded storage

`space_registry` owns space records and their roots/table/Guest leases;
`context_selector` owns installed-context records and execution leases per CPU;
`retirement_coordinator` owns mutation/retirement transactions;
`arch_stage2` owns registers, TLBI, ordering and private completion receipts.
P7 alone owns `current_vcpu`, run-state transitions and run queues. No W10 method
writes those P7 fields or infers execution from them.

Caller-owned stable storage supplies S space records, C CPU records and T
transaction records; S,C,T must be nonzero and fit checked size/alignment budgets.
The initial profile allows one outstanding mutation per space and one per-CPU
selection; global remote transport may further return Busy. SpaceId includes
manager identity and non-wrapping serial. TransactionId includes SpaceId, mapping
epoch and non-wrapping serial. Exhaustion rejects new work, never wraps.
Per-space resident CPU sets have exactly C bits and checked CPU indexes.

VMIDs are monotonically consumed from the hardware-validated 8-bit minimum
profile (reserve 0; usable 1–255), with VTCR.VS selecting that profile where
supported. A failed create after VMID mint burns its value. Destruction marks a
tombstone, never recycles it; repeated-create tests must expect eventual
exhaustion. Extra hardware VMID width is not silently enabled. Root geometry,
physical-address width and table budget are validated against capabilities and
W02's temporary IPA extent before Ready; unsupported profiles fail admission.

One short P3 data lock serializes registry transitions. Before hardware work,
reserve a transaction and all references under the lock, then release it.
Never hold this lock, a W06 data guard or a W12 store guard while invoking W08,
waiting for completion, entering Guest, invoking callbacks or freeing memory.
An admitted P3 facade serializes W12 mutations separately, with no nested lock:
prepare W12 leases first, reserve W10 state second, then reconcile failure with
explicit tokens outside both locks. Receipts bind exact identities across phases.

## 3. Two state machines and execution leases

Space lifetime: `Open -> Frozen(tx) -> Open`, or
`Open -> Retiring(tx) -> Destroyed`; uncertain completion yields `Quarantined`.
A frozen space admits no new selection, Guest entry or mutation. Its existing
installed references can remain while no Guest is executing. Guest execution
is tracked by per-CPU entry leases, not by a lifecycle value named Active.

Per CPU: `Idle | Stable(space, root, vmid, epoch) | Switching(tx, old, new) |
Maintenance(tx, saved) | Unknown(tx)`. Only the architecture selector changes
installed authority. Resident history records every CPU on which the space may
have supplied translations, including uncertain attempted installation; it is
not reduced when that CPU selects another space. CPU offline does not clear it
without a proven retirement handshake. Unknown state retains both old/new roots.

`acquire_entry(cpu, installed_receipt)` verifies Stable with matching space and
mapping epoch, Open lifetime and no existing execution lease; it returns a
non-Copy EntryLease and increments that space's executing reference. W04 consumes
the lease for Guest entry and retires it only after the actual exit boundary.
Entry also requires a current PreparedCode receipt covering this CPU for every
executable extent. W03 owns content/cache preparation; writable publication
invalidates that receipt. Adding a CPU requires quiescent preparation first,
not an assumption that Stage-2 TLBI also synchronizes instructions.
A rejected/pre-entry-cancelled lease uses a distinct cancellation operation;
no caller can claim exit merely by dropping it. Mutation requires zero execution
leases and no selection transaction touching the space. Otherwise return Busy
without changing PTEs. Policy for stopping running vCPUs belongs to P7.

## 4. Selection and pre-entry failure contract

`select_context(cpu, expected_current, desired_space)` runs on that CPU with
Guest execution stopped and local exception-entry discipline held by W04/P7.
It checks the exact expected current identity, Open desired space, supported
profile and capacity. It reserves Switching references to old and new roots and
marks new resident history before releasing the coordinator lock.

The local architecture operation installs the desired VTTBR root/VMID and the
validated common VTCR profile, applies required DSB/ISB ordering and checks the
resulting selector state before returning an unforgeable installation receipt.
It must execute for A→B→A; an old space-local CPU field is never an idempotence
condition. Selecting the same space can avoid writes only if the authoritative
per-CPU installed tuple and mapping epoch match and the backend confirms its
register ownership has remained exclusive. Guest return cannot write these regs.

On success commit Stable(new), release old installed reference (retain its
resident history) and return InstalledReceipt. `select_idle` explicitly detaches
the current context with ordered Stage-2 disable; destruction cannot silently
disable whichever other space happens to be installed.

Failure result is one of:

| Result | Hardware/resource facts | Consumer action |
|---|---|---|
| Rejected / Busy | No register changes; current context unchanged; no new entry lease | P7 may perform its own admission abort |
| Restored | New Guest never entered; exact old/Idle context restored and ordered; temporary refs retired | P7 may abort only after all other prepared subsystems are also safe |
| Indeterminate | Possible partial install or failed restore; CPU Unknown, old/new resources retained | Fail-stop this CPU/dispatch path; no ordinary requeue or release |

The selector returns facts, never edits `current_vcpu`, run-state or queues.
After successful install but before actual Guest entry, cancelling a dispatch
first releases an unused EntryLease and explicitly restores old/Idle context;
a safe result from one subsystem does not prove another subsystem is unwound.

## 5. Mutation, BBM and resident retirement

`map`, `unmap`, `protect` and `destroy` reserve Frozen/Retiring only after all
execution/selection references have drained. Any admission race sees Frozen and
fails Busy. Preparation validates all ranges, retains W12 leases and reserves
all table/journal storage before the first PTE write. Existing valid descriptors
are never overwritten in place. Memory-type changes are rejected in this profile.

For unmap/protect: clear affected leaf descriptors, DSB ISHST, retire old
translations on every CPU in resident history, then wait for exact completion.
For protect, publish new permission descriptors only after old-translation
completion, apply publication ordering and complete conservative invalidation
before reopening. For first mapping, publish invalid-to-valid descriptors,
complete ordering/required invalidation before Open. Unpublished, never-resident
spaces need descriptor ordering but no remote retirement; history is the test,
not whether the space is currently installed.

Intermediate tables detached during a transaction stay retained until the full
walk/translation retirement finishes. Guest region leases likewise remain until
W12 consumes the matching retire receipt. No early table free is allowed.
A partial publication rollback is another hardware retirement transaction;
returning Err alone never returns leases to the caller.

### 5.1 Local and remote architecture operation

All registered spaces use the same validated VTCR profile. With Guest execution
stopped, the target CPU saves its current installed tuple (possibly B or Idle),
enters Maintenance, selects target A's VTTBR root/VMID, executes ordered full
VMID Stage-1-and-Stage-2 invalidation (`DSB ISHST; TLBI VMALLS12E1; DSB ISH; ISB`),
restores the saved root/VMID and enable state, and executes the required ISB
before leaving Maintenance. Register accessors live only in Arch; reserved bits
and unsupported hardware profiles are rejected before production admission.
The TLBI variant here is local; every resident CPU is explicitly targeted.
The operation never leaves A installed when B was the saved context.

A mutation starts only with zero A execution leases. A remote CPU running B may
service maintenance through P3's normal handler: B's execution lease stays live,
its architectural state is preserved by that handler, and B resumes only after
restoration. Maintenance does not create an A execution lease. Failure to restore
B is fatal/indeterminate, not a completed remote operation. The architecture
review must pin the exact supported Arm revision and verify this profile's
TLBI/register/barrier sequence before coding; these are explicit obligations,
not a claim that a Host state test validates hardware behavior.

### 5.2 W08 binding and completion

A stable transaction record stores SpaceId/VMID/root, mapping epoch, operation,
exact resident target set, local completion and per-target restore completion.
W08's opaque payload is a non-reused lookup ID for this record; it is never a
borrowed stack pointer. Local CPU completion is handled separately because W08
excludes the initiator. The record outlives every Pending or late receiver.

The receiver resolves and checks the exact transaction, runs §5.1 and publishes
its completion only after invalidation AND restoration. W08 Completed is accepted
as retirement only for that registered operation and exact target set, with the
local bit also complete. Generic transport acknowledgment is insufficient.

W08-SYNC option A applies: strong-CAS admission, TransportBusy on contention,
no held data guard during collect, independent receivers and checked owner
release. Never overwrite Pending on timeout. If Busy occurs before PTE changes,
undo reservations and reopen without effects. If transport rejection, timeout,
or partial publication occurs after PTE changes, quarantine the space and keep
its transaction, table and region resources. Late acknowledgments are recorded
but do not trigger automatic free. No invisible retry or fairness assumption.

## 6. Destruction and capacity release

`destroy(space)` requires zero installed, execution and selection references;
otherwise Busy with no effect. Callers first explicitly select Idle/another
space on each installing CPU. Resident history still participates in final
retirement. Reserve Retiring, clear/detach all mappings, complete §5 on every
resident CPU, then retire W12 Guest leases, table-use pins and W11 table views
in that order. Return table object handles to W04 only after W12 take_back.
Guest objects remain controlled by W03 and become releasable through W12.

Destroying inactive A while B is selected preserves B's installed identity,
permissions and ability to run. No global Stage-2 disable, root reset or reusable
VMID release occurs. Accounting distinguishes successful frees, retained failure
records and permanently consumed VMIDs. A W04 free failure retains its returned
allocation handle for diagnosis; it is not counted as restored capacity.

## 7. P7-owned admission abort

The companion [P7 abort contract](../../../p7/implementation/p7-w02-scheduler-admission-lifecycle/06-pre-entry-abort.md)
owns the post-gate unwind. Its DispatchAttempt binds CPU, vCPU and an admission
epoch. If Stage-2 fails safely before Guest entry, P7 validates all prepared
P6 timer/LR and W10 context cleanup results, transitions Running→Runnable using
its existing Deschedule event and clears current_vcpu in one owner critical
section. It returns a candidate to W05; it never blindly requeues it. W05 checks
pending stop/pause and current eligibility before one queue insertion.
Indeterminate hardware failure preserves admission/resources for fail-stop.
Outgoing Guest exit and its lifecycle effects are never undone by incoming abort.

## 8. Implementation steps, tests and evidence

1. Implement W12-backed multiple-space registry and non-recycling VMIDs; test
   exhaustion, independent roots, foreign IDs and resource conservation.
2. Implement selector and execution leases with fake registers; enumerate
   A→B→A, failed install/restore, entry-vs-freeze races and stale receipts.
3. Implement local architecture install/retire and reference-QEMU A/B Guest
   fixtures. A local-only pass closes neither X04 nor the full W10 package.
4. Integrate P3 transport/serialized W12 adapter; inject contention, timeout,
   duplicate/late completion and target-set mismatch. Run cross-CPU stale-TLB
   tests and inactive A destruction while B runs.
5. Review and test P7 abort through its owner API; then publish a factual producer
   bundle for all P7 consumers. W10 never edits scheduler fields as cleanup.

| Evidence | Required observation |
|---|---|
| V17 | Simultaneous A/B live roots and distinct VMIDs; same IPA maps different canaries |
| V18 | A→B→A actual Guest accesses observe correct canary/perms; per-CPU installation trace |
| V19 | Inactive A protect/unmap/destroy invalidates A history; B regs/mappings/run remain correct |
| V20 | At least two admitted CPUs; old A translations cannot survive completion; offline/late targets retain safely |
| V21 | Failure at every publish/install/restore/collect boundary; safe abort vs fail-stop, no early free or queue duplication |
| V22 | Profile, geometry, capacities, CPU topology, transaction IDs and producer evidence linked from all seven P7 edges |

Model tests prove accounting and transition invariants; register/Guest/QEMU tests
prove only the declared runtime profile. No hardware or DMA claim follows.
Future implementation and verification records are
`../p4-w10-multivm-stage2-handoff-record.md` and
`../../verification/p4-w10-multivm-stage2-handoff-verification.md`.
Unsafe is confined to future Arch sysreg/table access and reviewed cross-CPU
facades; each needs separate SAFETY review. No code or public ABI changes here.

## 9. Architecture reference boundary

Arm's [AArch64 virtualization guide, 102142 issue 01](https://developer.arm.com/-/media/Arm%20Developer%20Community/PDF/Learn%20the%20Architecture/Armv8-A%20virtualization.pdf?revision=a765a7df-1a00-434d-b241-357bfda2dd31)
explains VMID-tagged translation contexts. It supports the separation of installed
context and retained translations; it is not, by itself, a complete normative
proof of the proposed maintenance sequence. Production admission additionally
requires the applicable Arm ARM instruction/system-register review and target
evidence listed above. Design completion is not architecture approval.
