# P2-W11 Host allocated-frame mapping — approved detailed design

Chinese readers can use the [Chinese edition](README.zh-CN.md).

**Status:** Approved detailed design (project owner) v0.2, 2026-10-02; implementation and runtime evidence are not claimed.
**Parent:** [W11 plan](../../plans/p2-w11-host-allocated-frame-mapping.md), M01–M05 / V14.
**Owner:** P2-ACR-02 / AUD-003. **Supersedes:** v0.1 direct-allocation sketches.
Read the [P2 task book](../../task-book-v0.1.md),
[Coding Guidelines](../../../../development/coding-guidelines.md) and
[W12 detailed design](../p2-w12-minimal-memory-objects/README.md).
Internal names below are designed interfaces, not existing Rust APIs.

**Approval:** Project owner, 2026-10-02, explicit confirmation “我确认批准”; see the [approval record](../../../../testing/documentation-audit/design-approval.md#owner-approval). Approved as the implementation design; its prerequisite, architecture/fit and runtime gates remain in force.

## 1. Baseline and foundation ledger

The workspace contains `hypervisor` and `crates/host-test-baseline`.
[Stage-1](../../../../../hypervisor/src/arch/aarch64/stage1/mod.rs) owns the P1
bootstrap mapping; [DTB access](../../../../../hypervisor/src/arch/aarch64/stage1/dtb.rs)
is a separate RO/XN aperture. Neither grants arbitrary writable allocation access.
[W04](../p2-w04-physical-page-allocation/README.md) is unimplemented and will be
restarted later; required allocation-handle interfaces are not yet delivered. Boot integration still reports allocator absent;
W04-MAP is independently pending. W12's design now owns backing and view retention;
there is no production W12 or W11 implementation in this checkout.

| Requirement | Missing runtime foundation / owner | Design resolution |
|---|---|---|
| M01 owned-frame coverage | W04 boot allocation and W12 | Consume a W12 Host region, never raw HPA or AllocatedFrames |
| M02 attributes and aliases | W11 architecture adapter | RW/XN Normal WB only; W12 global alias checks and protected-range exclusion |
| M03 lifetime | W12 region records and W11 access scopes | Stable map identity, scoped uninitialized bytes and explicit external-use pins |
| M04 rollback | W11 journal and completion backend | Reserve before publication; retain on any unproven revoke |
| M05 bounded boot access | Image-owned tables/window and exclusive CPU authority | Fixed geometry below; final linker fit and target execution remain admission evidence |

Required: boot-CPU ordinary RAM views and explicit revoke. Reserved: separately
admitted P3 serialization and shootdown adapter. Excluded: Guest Stage-2, MMIO,
executable Host mappings, heap, all-RAM direct map and replacement of W04-MAP.

## 2. Logical units and owned state

The platform-neutral `HostFrameMapper` owns map slots, VA reservations and
transaction IDs. Its records hold W12 `MemoryRegion` leases, never allocation
handles. The AArch64 adapter owns image-backed tables and descriptor journals.
Boot composition owns both storages for the entire translation lifetime.
Consumers hold non-Copy `MappedFrames` capabilities, not physical owners.
All objects are !Send/!Sync in the initial profile. Lost tokens retain capacity
and backing; Drop never unmaps or frees. No heap, recursion, wait or IRQ-path
allocation is involved.

Approved boot profile: 4 KiB pages, one 128 MiB VA window
`[0x1_0000_0000, 0x1_0800_0000)`, Stage-1 L1 slot 4, one L2 plus 64 L3
image-owned tables (65 pages). It is disjoint from P1 identity, W01 DTB slot 2,
and the W04 metadata slot 3 proposal. This is a VA reservation, not a physical
RAM assumption. The architecture adapter must verify the actual P1 root geometry,
root-slot vacancy, architectural address-size support and every existing mapping
before Ready; mismatch fails initialization without modifying the root.

Caller storage supplies 64 map records and a journal for at most 32,768 leaves.
One record covers a contiguous VA interval and one contiguous W12 region. A
bounded first-fit scan allocates VA runs; fragmentation returns Capacity even if
total free pages suffice. Table pages are linked once during initialization,
zeroed before linking, and retained for boot lifetime; mapping calls only edit
leaves. Their full image size, journal and records must fit the linker/bootstrap
read envelope and memory ownership inventory. No large journal lives on a boot
stack. Init failure after possible root publication retains all static tables;
it must not continue as Ready. Final image-fit evidence is required, not presumed.

## 3. Design interfaces and lifecycle

| Operation | Contract | Failure boundary |
|---|---|---|
| `init(storage, root_authority, boot_cpu)` | Validate geometry, disjointness, table coverage, capacities; publish zeroed static table tree with Stage-1 ordering | Pre-publication leaves root untouched; uncertain publication stops boot with retained tables |
| `map_owned(region) -> MappedFrames` | Region must be W12 Host-bound RW/XN Normal WB; reserve VA/slot/journal, bind transaction, publish invalid leaves only, commit W12 receipt | Rejected returns original reserved region; after publication return RetainedId unless completed rollback retires it |
| `with_uninit_bytes(&mut mapped, callback)` | Check identity, Live, no external-use pin; callback has a fresh non-escaping lifetime over exactly the usable mapped byte range | No raw pointer/PA-to-reference API; no initialized reads implied; callback result cannot borrow slice |
| `coverage(&mapped)` | Snapshot ObjectId, RegionId, VA, physical extent and attributes | Snapshot is not mapping/ownership authority |
| `pin_use(mapped, consumer)` | Consume access capability into one W12 external-use pin; disable ordinary byte callbacks | Consumer must supply its scoped quiescence receipt to regain capability |
| `unmap_owned(mapped)` | No borrow or pin; begin W12 revoke, invalidate leaves, complete Stage-1 invalidation and commit retirement | Busy returns Live capability; uncertain hardware completion quarantines map/VA/region; success returns no allocation handle |

Object owners call W12 `take_back` separately after all views/pins have retired.
This prevents Host-map completion from accidentally freeing backing still viewed
by Stage-2 or another read-only alias. W12 rejects overlapping writable views
before this adapter touches descriptors.

```
Vacant -> Reserved -> Publishing -> Live -> Revoking -> Vacant
                        |                     |
                        +---- Quarantined ----+
```

Slots and transactions use non-reused checked serials. Quarantine keeps the VA
reservation, W12 lease and journal. Ordinary errors cannot turn it into Vacant.
Only Reserved entries cancel without a backend receipt. Once publication begins,
rollback clears only leaves owned by that exact transaction, performs invalidation
for its whole range and completes ordering before returning a retirement receipt.
Uncertain rollback stops the operation with RetainedId; it never reports the
original region as safely reusable. Tables owned by init are never reclaimed by
per-map rollback.

## 4. Architecture and access boundary

The non-VHE EL2 Stage-1 adapter validates supported granule/root geometry from
current boot capabilities; it reuses the existing MAIR Normal WB data index but explicitly encodes
Inner Shareable W11 leaves; P1 leaves currently have no SH bits and must not
be copied unchanged. Existing P1 MAIR/TCR and mappings retain their contracts. Core sees
no descriptor bits or sysregs. For initial map, prepare invalid leaf slots and
zeroed backing as required by its consumer; write descriptors, DSB ISHST, perform
local EL2 Stage-1 invalidation for the window (conservative full local EL2
invalidation is permitted), DSB ISH, ISB, then issue PublishReceipt. For revoke,
clear leaves first, DSB ISHST, invalidate, DSB ISH, ISB, then RetireReceipt.
No AP may have used this window in the boot-only profile. Once AP access is
admitted, local completion alone is insufficient and this backend is rejected.

Every descriptor write and VA-to-slice construction needs a SAFETY explanation:
valid table storage, bounds/alignment, unique writer, effective attributes,
completed publication, no conflicting W12 view, and scoped borrow lifetime.
Byte length uses checked usable pages times 4096; buddy padding is inaccessible.
The callback receives `&mut [MaybeUninit<u8>]`, never an unchecked initialized
slice. Consumers initialize before typed reads and own type/alignment validity.
Guest instruction loading also requires the P4 architecture instruction-visibility
operation before releasing the Host writer and publishing executable Guest views.

External stack/table use is a W12 UsePin, not a long-lived Rust mutable slice.
Table-specific volatile writes remain in its architecture owner; W11 does not
pretend that hardware table reads satisfy Rust exclusive-reference rules.

## 5. Errors, order and validation

Error classes: Rejected (nothing published; input lease returned), Busy (a live
borrow/pin prevents revoke), Retained (publication/completion uncertainty; exact
retained identities returned for diagnosis), FatalInvariant (state/receipt or
hardware contract corruption; fail-stop and retain). There is no hidden retry.

Implementation order: bounded model and W12 integration → static table/window
init → architecture publication/revoke → scoped byte adapter → boot consumer
composition. Review APIs and capacity before backend unsafe; then validate actual
linker fit and execute target cases. No step bootstraps its own tables through
the mapping it is constructing.

V14 must cover foreign/stale IDs, usable-prefix overflow, VA fragmentation/full
capacity, alias conflicts, callback escape compile-fail, pinned revoke rejection,
rollback at every leaf write, stale-TLB read prevention after revoke, same-VA reuse
with a different object and exact W12/W04 accounting after all views retire.
Host models prove state and arithmetic only; QEMU/architecture evidence must show
actual mapping, revoke and reuse. AP access is a negative admission test, not an
SMP completion claim.

Remaining execution gates: formal ADR integration; W12 implementation and
receipts; W04-MAP boot allocator; final architecture/reference and linker review;
reference-QEMU V14. Geometry and ownership are specified here, no longer missing
design work. P2-HOST-MAP remains unproved until the production path passes.
Future records: `../p2-w11-host-allocated-frame-mapping-record.md` and
`../../verification/p2-w11-host-allocated-frame-mapping-verification.md`.
