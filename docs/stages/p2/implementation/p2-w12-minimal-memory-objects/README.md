# P2-W12 minimal MemoryObject and MemoryRegion — detailed design

Chinese readers can use the [Chinese edition](README.zh-CN.md).

**Status:** Approved detailed design (project owner) v0.1, 2026-10-02; implementation and runtime evidence are not claimed.
**Parent:** [W12 plan](../../plans/p2-w12-minimal-memory-objects.md), N01–N06 / V15.
**Authority:** [ADR-062](../../../../adr/adr-062-p2-minimal-memory-object-foundation.md),
[P2 task book](../../task-book-v0.1.md),
[Coding Guidelines](../../../../development/coding-guidelines.md).
**Supersedes:** The direct-allocation ownership sketches in W11 and the raw-range
ownership sketches in P4-W02/W03; their revised designs consume this contract.

**Approval:** Project owner, 2026-10-02, explicit confirmation “我确认批准”; see the [approval record](../../../../testing/documentation-audit/design-approval.md#owner-approval). Approved as the implementation design; its prerequisite, architecture/fit and runtime gates remain in force.

## 1. Current state and foundation ledger

The current workspace has `hypervisor` and `crates/host-test-baseline`.
[W04 design](../p2-w04-physical-page-allocation/README.md) is the future producer.
W04 is unimplemented and excluded from this submission. W12 requires
non-Copy, non-Clone `AllocatedFrames`, checked owner/serial identity, full buddy
`range()`, separately bounded `usable_count()`, and ownership-preserving free
failure. Neither W04 allocation nor W04-MAP has admitted implementation/evidence in
this submission. W12, W11 and P4 runtime adapters do not exist in production code.

| Goal | Required foundation and owner | Design disposition / evidence required |
|---|---|---|
| N01 sole backing owner | W04 actual allocation handle | Store moves the handle once; no HPA import or second allocator ledger; exact return/free tests |
| N02 identity and capacity | W12 caller-owned storage and one boot identity authority | Checked non-reused store/object/region identities; exhaustion tests |
| N03 bounded views | W12 range, permission and alias validator | Usable prefix only; same memory type; conflict rejection matrix |
| N04 retention | W11 / P4 architecture completion adapters | Linear region lease and exact receipt; real target completion separately required |
| N05 rollback/accounting | W12 state reducer and retained records | Every partial publication fault keeps backing; failure injection |
| N06 consumption | W11, P4-W02/W03; P3 for SMP | Adapter contracts below; Host fakes establish logic only |

Required: ordinary owned RAM, stable identity, bounded regions and explicit
retirement. Reserved: admitted P3 serialization of this same store. Excluded:
heap, MMIO ownership, COW, paging, Guest policy, DMA, automatic Drop reclamation,
new allocator tags, and production fake-completion constructors.

## 2. Logical units and storage

`object_store` owns the actual W04 handles and object records; `region_store`
owns pending/live/revoking view records; `view_rules` is pure checked validation;
`completion_adapter` binds architecture receipts to transactions. These are
logical modules under the platform-neutral memory layer, not a new workspace
crate. Architecture adapters cannot allocate object IDs or free backing.

Boot composition supplies stable storage for `O` object records and `R` region
records before initialization, plus one exclusive `IdentityAuthority`. It may
be linker-owned or caller-owned storage whose lifetime covers every handle;
no heap or mapped-heap bootstrap is assumed. Byte budgets use checked
`O * size_of::<ObjectRecord>() + R * size_of::<RegionRecord>()` and alignment;
configuration with zero capacity, overflow or insufficient storage fails before
Ready. Maximum live objects is O and maximum outstanding regions is R; a region
holds one contiguous usable-page subrange of exactly one object. A bounded scan
of R records provides alias checking; no unbounded list, recursion or allocation.
W12 does not choose consumer RAM sizes or silently consume W04 buddy padding.

One boot authority issues nonzero monotonically increasing StoreIds. Each store
issues non-wrapping object and region serials; slot indexes alone are never
identity. Exhaustion disables further creation in that domain; it never wraps.
ObjectId = (StoreId, object serial); RegionId = (ObjectId, region serial).
Allocation owner identity remains private to the original W04 handle.

`MemoryObject` is a non-Copy, non-Clone control capability with a store lifetime,
not a second copy of its allocation. `MemoryRegion` is a non-Copy retained view
lease; address/range snapshots are diagnostic values, never authority. Records
own resources even if clients forget capabilities. Store teardown with occupied
records is rejected; Drop does not free backing. Forgotten tokens leak bounded
capacity safely. Production initialization consumes a sealed boot-storage lease over stable
`&'static mut` record storage; it never returns that storage to the caller.
Thus forgetting all software tokens cannot make an asynchronous backend pointer
dangle. Local-lifetime store fixtures are host-test-only and cannot register a
production backend. The registry is pinned while any backend can refer to it.

## 3. Internal API contracts

These are designed Rust-like interfaces, not claims about current exported APIs.
All errors are enums; no implicit retry, panic, raw-pointer import or blocking.

| Operation | Preconditions and state effects | Failure and ownership |
|---|---|---|
| `adopt(frames, ceiling, NormalWb) -> MemoryObject` | Valid actual W04 handle; reserve object slot, store full handle, expose only usable prefix; ceiling bounds individual views; no view may combine W+X | Before commit returns the identical handle; after commit the store is sole owner |
| `reserve_region(&object, pages, rights, BackendId, SpaceId) -> MemoryRegion` | Checked nonempty aligned subrange within usable prefix; rights subset of ceiling; exact store identity; reserve record and retention before returning | Range/alias/capacity/identity error changes no count or backend state |
| `begin_publish(region, transaction) -> Publication` | Adapter consumes the reserved lease; binds non-reused transaction ID and destination identity before first descriptor write | Invalid identity returns unchanged reserved lease; no hardware access |
| `commit(publication, PublishReceipt) -> LiveRegion` | Exact transaction, region, backend, destination, range and permissions match; backend confirms publication ordering | Receipt mismatch quarantines retained transaction; never frees |
| `cancel_reserved(region)` | Reserved, never submitted to publication | Consumes region and drops exactly one retention; after begin_publish use rollback protocol |
| `finish_rollback(publication, RetireReceipt)` | Hardware exposure absent or fully revoked by the bound backend | Removes record only after checked receipt; uncertainty quarantines |
| `begin_revoke(live) -> Revocation` | No active byte borrow or external-use pin; preserve retention | Busy returns same live lease; successful reservation blocks new accesses |
| `finish_revoke(revocation, RetireReceipt)` | Exact backend transaction proves no access/translation remains within its declared CPU scope | Removes region/count once; duplicate or foreign receipt cannot retire it |
| `take_back(object) -> AllocatedFrames` | No regions in any state, no access/pin; consumes object control capability and record | Busy returns object capability; returns original full W04 handle on success |

The allocator is called only after `take_back`; its `FreeFailure` returns the
handle and must remain owned by the caller. A failed free is not a free-page
accounting success. W12 maintains object/region retention counts, not a second
physical free/allocated bitmap.

`begin_reprotect(live, new_rights)` reserves a same-region attribute transaction
without dropping retention: validate object ceiling and all other regions, block
new access, retain old and pending rights and enter Revoking. A backend-bound
old-retirement receipt is required before new publication; PublishReceipt then
commits the new rights/epoch to Live. The region remains counted once throughout.
Reject before publication returns the original Live lease. Failure after possible
hardware effects quarantines; it cannot expose an unretained new mapping.
Memory-type changes and region splitting are unsupported in the initial profile.

## 4. Alias and access rules

View rights are R, RW or RX; write implies read, W+X is rejected.
An object ceiling may permit both RW initialization and later RX execution;
it does not itself create a view or authorize simultaneous write/execute access. Initial memory type
is Normal WB, Inner Shareable. Overlapping reservations or live/quarantined
regions of one object are permitted only when both are read-only (R or RX) and
have identical memory type. A writable region conflicts with every overlapping
region, even if another backend or virtual address is used. Distinct objects
cannot alias because only unique W04 handles enter the store. Within a region,
only its bound backend can authorize byte access; public metadata cannot.

Host byte access uses W11's scoped `MaybeUninit<u8>` view. A mutable slice is
never issued for RX, a hardware-active stack, or a table concurrently read or
written by hardware. External use (stack, page-table walk) requires a non-Copy
`UsePin` bound to the region, consumer and epoch. While pinned there are no
ordinary Rust byte borrows and no revoke. The owning architecture adapter may
perform its narrowly defined volatile/table operations under its own safety
contract. Pin retirement needs that consumer's audited quiescence receipt;
a public boolean or caller assertion is not enough. Pins are stored in the
same bounded record, with at most one exclusive external-use owner per region.

The initial store is boot-CPU-only and !Send/!Sync. Cross-CPU access requires a
P3-owned serialized facade that exposes these same transitions under a bounded
short data lock; it returns operation tokens before any backend call or wait.
That adapter must audit token movement and store lifetime before implementing
Send/Sync. W10 cannot manufacture this permission with an unsafe cast.

## 5. State machine, completion authority and failure

```
Reserved -> Publishing -> Live -> Revoking -> Removed
    |           |                    |
 cancel      rollback             retire
    |           |                    |
 Removed <------ completed receipt ---+
                uncertainty -> Quarantined (retention preserved)
```

An object remains Held until the last region and pin is gone, then explicit
`take_back` removes it. Quarantined records count as occupied and retain the
backing, destination and transaction storage until restart or a separately
reviewed recovery operation; this design supplies no optimistic recovery.

Publication/retirement receipts have private constructors in the audited W11
and P4 architecture adapter modules. They are non-Copy and bind the complete
identity tuple, transaction serial, phase, covered range and completion scope.
The store validates all fields and current phase before consuming a receipt.
Test-only adapters are behind host-test configuration, never production feature
fallbacks. If Rust module separation requires an extensible backend trait, its
implementation is an unsafe contract with explicit completion/alias obligations;
ordinary safe external code cannot implement it and forge a receipt.

Prepare reserves every record before hardware work. A pre-publication rejection
returns all capabilities unchanged. After possible publication, either completed
rollback retires the view or the records stay quarantined. A `Result::Err` alone
never proves absence of translations. No lock is held while calling a backend,
waiting for another CPU or releasing to W04. Completion transitions reacquire
the short lock, recheck exact transaction identity, and commit once.

## 6. Consumer protocols

- **W11:** consumes a Host-bound region, owns VA/table transactions, and returns
  retirement to W12 after Stage-1 invalidation. It never owns AllocatedFrames.
- **P4-W03:** holds object control capabilities. Zero/load through a Host RW
  region, finish all byte borrows and instruction-visibility maintenance, revoke
  Host region completely, then reserve disjoint Guest RX/RW regions for W02.
- **P4-W02:** owns Guest region leases, table-object capabilities and pinned
  W11 table views. It does not free mapped Guest backing. Table pins retire
  only after no installed/walking context or stale translation can use them.
- **P3 stacks:** external-use pins live through CPU stop and stack switch; an
  allocation handle or CPU lifecycle label alone cannot retire an active stack.
- **W05:** no automatic heap rewrite. A future explicit heap adapter must obey
  the same pin/region contract without making W12 depend on the heap.

## 7. Implementation order and validation

1. Add bounded IDs/storage and actual W04-handle adoption in the existing crate
   layout; test foreign handles, serial exhaustion, usable-prefix and full-handle
   return. No architectural code is needed for this first step.
2. Implement range/alias checks and the state reducer. Enumerate invalid phase
   calls, all overflow and capacity failures; counts must equal occupied records.
3. Add test-only completion adapters and failure injection at reserve, publication,
   commit, invalidation and completion. Verify no premature `take_back`, no duplicate
   decrement, and retention after token loss. Test W04 free rejection ownership.
4. Compile-fail tests reject copied capabilities, escaped Host borrows, direct
   receipt construction and boot-only cross-thread use. Tests that merely mirror
   enum setters are insufficient: exercise object reuse and foreign transactions.
5. Integrate W11 then P4 adapters only after their approved backend and producer
   prerequisites. Real Host map/unmap/reuse and Guest unmap/destroy evidence is
   required separately; a fake receipt establishes no TLB or CPU quiescence.

V15 records exact capacities, identity exhaustion, alias matrix, phase failures,
retained resources and returned handle identity. N01–N06 each needs a result row.
Future records belong in `../p2-w12-minimal-memory-objects-record.md` and
`../../verification/p2-w12-minimal-memory-objects-verification.md`; they do not
exist merely because this design names them. New unsafe is expected only in
future architecture/storage adapters, must be individually justified and counted.
No dependency, public ABI or production unsafe is added by this document.
