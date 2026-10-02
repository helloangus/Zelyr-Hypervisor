# P4-W02 Architecture, Objects, and State Model

**Status:** Approved detailed design (project owner) v0.2, 2026-10-02; implementation and runtime evidence are not claimed.
**Parent:** [W02](README.md). Common backing/view rules come from
[W12](../../../p2/implementation/p2-w12-minimal-memory-objects/README.md);
multi-CPU extension comes from [W10](../p4-w10-multivm-stage2-handoff/README.md).

## 1. Logical modules

| Module | Owns | Does not own |
|---|---|---|
| s2-space | Space identity, root/table object capabilities, Guest region leases, mutation journal | Guest backing allocation or loader policy |
| s2-table | Descriptor encoders, bounded walks and architecture table-use pins | W04 allocation accounting |
| s2-vmid | Monotone VMID issuance and permanent consumed set | Reusable VMID pool |
| s2-context | Per-CPU installed tuple and Guest execution leases | P7 current_vcpu or scheduler lifecycle |
| s2-tlb | Identity-bound ordering and completion receipts | Transport acknowledgment semantics or caller-owned raw proofs |

## 2. Core objects and ownership

### 2.1 `GuestAddressSpace`

A space holds a W12 object capability for each table allocation, a W11 mapped
region and architecture-use pin for table access, and W12 Guest-region leases
for RAM mappings. The W12 store retains actual W04 handles. GuestRam retains
its object control capability; a mapping never duplicates that backing owner.
Table handles are returned to W04 only after table-use, Host mapping and object
retention have all ended. There is no automatic Drop reclamation.

### 2.2 `MappingGrant`

`MappingGrant` is a non-Copy enum: `Ram(MemoryRegion)` or
`Console(ConsoleWindowGrant)`. RAM is W12-authorized and bound to this SpaceId;
HPA/count/flags snapshots cannot construct it. The console case is separately
minted once by the P4 base console authority from verified P2 platform facts:
one exact PL011 Device RW/XN page, no allocator ownership or generic MMIO import.
It requires W04's exclusive base-console execution convention; W10 rejects
console-bearing spaces. No console page is ever returned to W04 as RAM.

### 2.3 `Vmid`

Validate a supported 8-bit VMID profile; reserve 0 and monotonically consume
1–255. Exhaustion is permanent for this boot. Failed construction after mint
burns the value. Destroy tombstones it; no release makes it allocatable again.

### 2.4 Per-pCPU activation register

The sole authority is `s2-context`: Idle, Stable(space/root/VMID/epoch),
Switching, or Unknown. Space lifecycle does not encode which context happens
to be installed. W04 owns Guest entry/exit and must hold a matching execution
lease while Guest may run. The single-CPU base has one such record; W10 extends
its storage and synchronization, not the meaning of an Active flag.

## 3. Address-space lifecycle state machine

`Open -> Frozen(transaction) -> Open`; destruction uses
`Open -> Retiring -> Destroyed`. Uncertain publication/invalidation enters
Quarantined and retains leases/tables. Freeze requires zero Guest execution
leases and no in-flight selection involving the space. Installed-but-idle is
not running. No new entry, selection or mutation may start while Frozen.

An `ever_resident` CPU history survives detachment. A never-selected space can
avoid TLBI when no hardware exposure occurred; an inactive but previously used
space cannot. Losing a handle leaks resources instead of freeing reachable RAM.

## 4. Activation and register programming

`activate` checks the authoritative CPU tuple and installs root/VMID/profile
with ordered architecture operations. Same-space return is allowed only after
checking that tuple, epoch and exclusive register ownership; an old stored CPU
number is not evidence. Failure before writes is Rejected; a proved restoration
is Restored; unknown register state is Indeterminate and retains roots.
W04 may acquire an entry lease only for Open and matching installed epoch.
Explicit `select_idle` detaches the context; destroy does not disable an
unrelated currently installed space. W10 specifies A→B→A and remote behavior.

## 5. Permission and memory-type model (P4 subset)

Normal WB RAM views use R, RW/XN or RX; W+X is rejected. Bounds and ceilings
are validated against W12, including Host aliases. Guest code is loaded through
a Host RW region which is fully retired before publishing RX. RAM memory type
is immutable in this profile; unsupported changes return InvalidFlags before
mutation. Base console is Device RW/XN under its separate authority.
Query results are snapshots, never handles or permissions to dereference HPA.

## 6. Stage-2 ordering, barrier, and invalidation semantics

### 6.1 Break-before-make (BBM)

Prepare all records, tables and journal capacity before PTE publication. Clear
old valid entries; DSB ISHST; perform identity-bound old-translation invalidation;
DSB ISH and ISB before replacing entries or releasing anything. Permission
replacement then publishes new descriptors and completes publication ordering
and required invalidation before reopening. Detached intermediate tables remain
retained until the full walk/translation completion, never merely until unlink.

### 6.2 First-time map

Only invalid-to-valid insertion is supported. Prevalidate all leaf slots and
W12 leases, prepare zeroed tables while unpublished, publish with ordering, then
commit. Any post-publication failure requires completed rollback or quarantine.

### 6.3 Invalidation operations and seam

The base backend targets the one admitted CPU and exact VMID, including after
detachment. It saves/restores the currently installed tuple when selecting the
retiring target for TLBI. It emits a private retirement receipt only after
invalidation and restoration both complete. Never-resident unpublished mappings
can use an absence-of-publication receipt from this same trusted backend.
A public bool, a Stage2Released value without identities or transport ACK is
not a receipt. W10 §5 defines the resident-set extension and architecture sequence.

### 6.4 Cross-pCPU reservation

Base W02 rejects other-CPU exposure. Cross-pCPU consumers require the separate
W10 implementation and V20 evidence. Local tests do not discharge that gate.

## 7. Concurrency model

Bounded short state-lock operations reserve and commit transactions. Hardware
work, W12 transitions, frees and transport waits occur outside the lock. The
Frozen owner token authorizes table writes without holding a data guard through
TLBI or remote collection. Queries during mutation return Busy or an explicitly
versioned snapshot; they never present a half-updated ledger as committed.
No IRQ allocation, recursive mapping or nested Guest entry is authorized.

## 8. Telemetry points (W02-scope)

Creation, map/protect/unmap, selection and retirement events include SpaceId,
ObjectId/RegionId where applicable, transaction/epoch, CPU, range and outcome.
Retained failures are distinguishable from returned backing and successful frees.
P0 owns trace transport; W02 never adds ad-hoc console writes while Guest owns it.
