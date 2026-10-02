# P4-W02 Code Contracts — Stage-2 Core

**Multi-VM admission (2026-10-02):** [AUD-004 handoff requirements](06-multivm-handoff-requirements.md) bound this design to the P4 single-Guest/current-path scope. Multiple live spaces, per-switch installation, inactive-space retirement and cross-pCPU use require a producer extension and evidence; P7 cannot infer them from this baseline.

**Status:** Approved detailed design (project owner) v0.2, 2026-10-02; implementation and runtime evidence are not claimed.
**Parent:** [P4-W02 detailed design](README.md).  
**Companion:** semantics and state machines in
[02-architecture-and-state.md](02-architecture-and-state.md).

All names below are P4-internal and unstable-by-declaration: P4 asserts no
API stability for them, and P4-W09 must record them as implemented facts
only. Pseudocode is an algorithm outline, not runnable production code; the
Coding Guidelines remain the authority for the final Rust shape.

## 1. Type contracts (Core-visible vocabulary, module `s2-vocab`)

### 1.1 `S2Access`, `S2Execute`, `S2MemType`

- **Name and stability:** `S2Access` (Read/ReadWrite), `S2Execute`
  (Executable/ExecuteNever), `S2MemType` (NormalCacheable /
  Device). Internal, P4-unstable.
- **Purpose and caller:** express Guest-visible permission and memory-type
  intent without descriptor bits; used by W03 (mapping requests), W02
  internally, W06 (diagnostics reporting), never encoding Arm bit values.
- **Inputs/outputs:** value objects; `Copy`; no methods beyond predicates.
- **Preconditions/postconditions:** none beyond type invariants (closed sets).
- **Errors:** none.
- **Security checks:** these types carry no authority; authorization is the
  caller's (host-side) responsibility. Guest influence over flag *values* is
  impossible in P4 (values come from host-side layout constants).
- **Validation:** unit tests: closed-set round trips through the descriptor
  mapping (§2.1) cover every variant.

### 1.2 `S2Flags`

- **Name and stability:** `S2Flags { access: S2Access, execute: S2Execute,
  mem_type: S2MemType }`. Internal.
- **Purpose:** one mapping's permission/type bundle.
- **Contract notes:** constructor validates the meaningful combinations P4
  allows (for example, Write requires Read — Stage-2 has no write-only
  encoding); invalid combinations are construction errors, not silent
  coercions.

### 1.3 `MappingGrant`

Non-Copy, non-Clone `Ram(W12 MemoryRegion)` or `Console(ConsoleWindowGrant)`;
see [ownership and console authority](02-architecture-and-state.md).
A RAM grant binds ObjectId, RegionId, destination SpaceId, extent and allowed
rights. Only W12 can reserve it; snapshots and raw HPA cannot reconstruct it.
The console authority is separate, limited to the one verified base-test Device
RW/XN page, and is rejected by W10. It is never allocator-backed ordinary RAM.
Before publication, Rejected returns the unchanged grant. After publication,
only a completed rollback can retire it; otherwise the space retains it and
returns RetainedId. A blanket `(error, grant)` result is unsound.

### 1.4 `GuestIpaRange`

- **Name and stability:** `GuestIpaRange { base: GuestPhysAddr, pages:
  PageCount }`. Internal.
- **Purpose:** page-granular IPA span for map/unmap/protect/query-bulk.
- **Contract notes:** all arithmetic checked; `end()` returns the exclusive
  end as `GuestPhysAddr` or an overflow error; alignment validated in the
  constructor.

### 1.5 `QueryResult`

- **Name and stability:** `QueryResult` — `Unmapped` or
  `Mapped { flags: S2Flags, frame_base: HostPhysAddr }`. Internal.
- **Purpose:** W06's diagnostic source for mapping state; also W02's own
  negative tests.
- **Contract notes:** snapshot semantics per decision D9 (ledger read under
  lock); `frame_base` is the base frame of the mapping, not a per-page
  computed HPA (callers add the intra-page offset explicitly with checked
  arithmetic).

### 1.6 `Stage2Error`

Errors include capacity/OOM, invalid range/flags, foreign/stale grant, alias
conflict, AlreadyMapped/NotMapped, Busy and permanent VmidExhausted. Results
separate Rejected (no publication; input capability returned), CompletedRollback
(view safely retired), Retained (exact transaction remains owned) and
FatalInvariant (fail-stop; retain). No generic Err promises unchanged hardware.
Only rejected preflight operations have an unconditional no-effect guarantee.

## 2. Arch-internal contracts (module `s2-table`)

### 2.1 Descriptor accessor `S2Desc`

- **Name and stability:** `S2Desc(u64)` newtype with associated
  constructors/inspectors: `invalid()`, `page(frame: HostPhysAddr, flags:
  S2Flags) -> S2Desc`, `is_valid()`, `output_frame() -> Option<HostPhysAddr>`,
  `flags() -> Option<S2Flags>`, `set_flags(S2Flags) -> S2Desc`,
  `as_u64()`, `from_u64(u64) -> S2Desc`. Internal.
- **Purpose:** the only place Stage-2 bit fields are encoded/decoded,
  including reserved-bit handling (MBZ fields forced zero; unknown/reserved
  bits never set).
- **Inputs/outputs:** typed values to/from 64-bit words.
- **Preconditions:** `page()` requires 4 KiB-aligned `frame`.
- **Postconditions:** produced words obey the pinned Arm ARM Stage-2 page
  descriptor layout for the implementation-revision.
- **Security:** no Guest-reachable input reaches this type directly; still,
  `from_u64` must reject unsupported/reserved encodings instead of silently
  masking corruption into a valid descriptor.
- **`unsafe` boundary:** none inside `S2Desc` itself (pure bit manipulation).
  The `unsafe` lives in the write path (§2.2).
- **Validation:** exhaustive unit tests mapping every `S2Flags` variant pair
  to expected field values checked against the pinned spec revision; reserved-bit input rejection tests; round-trip tests.

### 2.2 Table-memory write path `write_desc / read_desc` (unsafe)

- **Name and stability:** `unsafe fn write_desc(table: *mut S2Desc, index:
  usize, value: S2Desc)` and `unsafe fn read_desc(table: *const S2Desc,
  index: usize) -> S2Desc` (or an equivalent single small abstraction over
  volatile word access). Internal; the module's only `unsafe`.
- **Purpose and caller:** mutate/read one descriptor slot in table memory
  owned by a `GuestAddressSpace`; called by `s2-table` walk routines only.
- **Inputs/outputs:** table base pointer (from the space's owned frame),
  validated index, descriptor word.
- **Preconditions (SAFETY obligations to justify at each call site):**
  (1) the table frame is owned by the calling space and not freed;
  (2) the caller owns the exact Frozen transaction, so no concurrent descriptor
  writer or new Guest entry exists;
  (3) `index < entries_per_table` derived from the granule;
  (4) the table pointer is a valid, aligned writable Host address whose
  mapping/borrow remains live under the assigned P2 producer's
  [P2-HOST-MAP contract](../../../p2/implementation/p2-contract-reconciliation-record.md#p2-acr-02--ordinary-allocated-frame-host-mapping-aud-003).
  Physical allocation or P1 bootstrap coverage alone does not establish this
  precondition; missing producer contract/evidence blocks table access.
- **Postconditions:** the slot contains `value` (write is a single aligned
  8-byte store, ordered per §6 of [02](02-architecture-and-state.md): the
  caller performs required `DSB`/`TLBI`/`ISB` around sequences — this function
  itself only stores).
- **Concurrency/allocation:** no allocation; transaction authority held by caller; no IRQ
  context use in P4.
- **Errors:** none (programming errors are the caller's invariant violations).
- **Security:** Guest cannot reach this path; indexes are computed from
  validated IPA ranges with checked arithmetic in the walk layer.
- **Logic:**

```text
write_desc(table, index, value):
    // SAFETY: caller justifies table ownership, lock, index bound, mapping.
    store_volatile_u64(table as *mut u64 + index, value.as_u64())
```

- **Validation:** host-side unit tests with simulated table buffers; code
  review that every call site carries the four-point SAFETY justification;
  unsafe inventory entry per P0-W10 (assumed M8-era governance).

### 2.3 Walk-for-mutation `walk_to_leaf`

The bounded walk uses checked IPA indexes (512 entries/table, supported root
level through L3, no blocks). Prepare walks first count missing paths and reserve
all W12 table objects, W11 views/pins and journal entries without publishing.
Initialize new tables to invalid, then link them under the Frozen transaction.
`LeafRef` is scoped to that transaction and table pin, never a free-standing PA.

Pre-publication OOM releases only unpublished resources with their proper Host
view/pin retirement. If an intermediate link was possibly published, rollback
unlinks only transaction-owned entries, completes walk/TLB retirement, then
releases tables. An OOM after publication cannot simply free the new tables.
Partial invalidation or failed restoration quarantines the whole transaction.
Tests inject failure at every intermediate allocation/link, including an
existing-path prefix which rollback must preserve.

## 3. `s2-space` public contracts

### 3.1 `GuestAddressSpace::create`

Inputs: checked IPA span, caller-owned ledger/journal capacities, W12/W11 and
VMID authorities. Validate profile and full capacity before allocating; adopt
table allocations into W12, obtain/pin W11 Host views, zero unpublished tables,
then mint a VMID and commit Open. Pre-publication failure retires views and
returns handles explicitly; a minted VMID stays consumed. No physical pointer
cast establishes Host table access. Root/table identity and capacities are
recorded; no dynamic hidden heap is assumed.

### 3.2 `map`

`map(space, ipa, grant) -> MapOutcome` consumes the exact bound grant.
Check nonempty/aligned extent, IPA width, rights/type ceiling, destination and
alias restrictions, vacant leaves and complete resource budget before freeze.
Executable mappings additionally require current PreparedCode for the exact
object/code extent; writable publication invalidates any older code receipt.
Only a space with no execution/selection lease may freeze. Reserve the W12
publication transaction before first link/write, publish invalid entries with
ordered architecture completion, then commit the ledger and W12 Live region.
Rejected returns the untouched grant; completed rollback retires it; uncertain
rollback returns RetainedId with resources still in the space. Console follows
its own base authority; it is not admitted into W12 RAM.

### 3.3 `unmap`

Require a whole registered region extent in the initial profile; partial-region
unmap returns UnsupportedRange before mutation (split views are reserved).
Validate and freeze; clear leaves and detach empty intermediate tables into the
retained journal; invalidate all old translations for the space's resident
history; only after completion retire the W12 region and detached table pins,
Host views and object handles. Commit the ledger removal last. No table or
Guest backing is freed before completion; W03 remains the Guest object owner.
NotMapped is a no-effect rejection. Uncertain completion quarantines.

### 3.4 `protect`

Require the whole registered region, supported unchanged memory type and
requested rights within its object ceiling. Reserve the new W12 view attributes
against all aliases while the old view remains retained; this is a transaction
on the same region, not a second overlapping writable reservation. Mark the
region Revoking and block access, execute BBM, complete old translation retirement,
publish the new attributes, complete publication and commit W12 first, then the
space epoch while keeping the space Frozen until both succeed. A failed new publication retains the object and quarantines the
region; no unsafe rollback to a more permissive old mapping is assumed.
R→RW must be rejected if any overlapping read-only alias remains. Publishing
any Guest-writable permission invalidates that extent's PreparedCode receipt.
Enabling execute requires a fresh receipt bound to object/range/content epoch and
admitted CPU set. Thus RX→R→RX is permitted while content stayed immutable;
RW→RX without fresh preparation is rejected before mutation. To prepare code
again, retire Guest views, reload/maintain through an exclusive W11 Host writer,
retire that writer and map the new Guest executable view. There is no implicit
Host alias or unimplemented live self-modifying-code protocol.

### 3.5 `query`

Under the short state lock return a committed, versioned mapped/unmapped
snapshot with rights, memory type and frame plus checked offset. Frozen or
Quarantined returns Busy/Retained rather than pretending partial PTE state is
committed. Query does not borrow backing, authorize access or mint grants.

### 3.6 `activate`

Delegate to the per-CPU selector with expected current identity and desired
SpaceId/epoch. Install and order registers, then return an installation receipt;
W04 acquires a separate execution lease before actual Guest entry. An old
space-local Active(cpu) flag cannot skip register installation. Safe failures
are Rejected or proved Restored; Indeterminate retains involved roots and stops
entry. W10 specifies multiple spaces and P7-owned post-gate abort.

### 3.7 `invalidate` seam operations

Inputs bind space/root/VMID, epoch, transaction, range and resident CPU set.
Completion includes translation/walk retirement AND restoration of the CPU's
previously installed context. No-op is permitted only for a proven never-exposed
transaction, never merely because the space is inactive. Architecture receipt
constructors are private; simulated receipts are host-test-only. Base W02 has
one admitted CPU; W10 supplies cross-CPU completion and timeout retention.

### 3.8 `destroy`

Require zero execution, installed and selection references; Busy otherwise.
The caller must explicitly detach via select_idle/another space first. Reserve
Retiring, remove mappings and complete resident retirement, then retire Guest
region leases and table-use pins, revoke Host table views, take back/free table
objects. W03 frees its own objects only when W12 permits it. Preserve an unrelated
installed context throughout retirement. VMID is permanently tombstoned, never
returned to a reusable pool. Any uncertain step retains exact remaining resources;
free errors preserve the allocator-returned handle. Success records Destroyed.

## 4. Error and failure classification recap

Caller errors and capacity failure before publication are recoverable and leave
hardware untouched. Completed rollback is separately observable. Partial
publication, timeout, failed restoration or invalid receipt retain resources;
unknown hardware state is fail-stop. Guest-caused faults use W06 VM-facing
classification, never the Host invariant route merely because Guest faulted.
