# P4-W02 Code Contracts — Stage-2 Core

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
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
  NormalNonCacheable / Device). Internal, P4-unstable.
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

- **Name and stability:** `MappingGrant { frame_base: HostPhysAddr,
  page_count: PageCount, flags: S2Flags }`. Internal, stage-local (decision
  D8). Produced only by P4-W03's Guest RAM construction.
- **Purpose:** transfer validated frame ownership-in-P4 (the allocator retains
  global accounting; the grant transfers *use*) into the mapper.
- **Inputs/outputs:** as above; consumed by `map`.
- **Preconditions:** `frame_base` page-aligned; `page_count ≥ 1`; the frames
  are allocator-derived (P2) and not previously granted.
- **Postconditions:** on success the grant is consumed (moved); on failure it
  is returned intact to the caller.
- **Security checks:** constructor rejects zero pages and overflow when
  computing the covered range (checked arithmetic).
- **Validation:** unit tests: malformed grants rejected without allocation.

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

- **Name and stability:** `Stage2Error` — `OutOfPages` (allocator exhausted),
  `RangeOverflow` (IPA span invalid for the space or arithmetic overflow),
  `AlreadyMapped`, `NotMapped`, `Misaligned`, `InvalidFlags`, `VmidExhausted`,
  `Depleted(TableAllocation)` — internal error values.
- **Purpose and caller:** every fallible W02 entry point; consumed by W03/W04
  call sites and mapped to VM-facing failure handling.
- **Failure guarantee:** error returns leave the space state-unchanged
  (checked before any descriptor mutation begins; see per-function
  pseudocode). `VmidExhausted` is permanent for the P4 allocator (D4) but not
  a hypervisor fault.
- **Security:** errors never echo more address context than the caller
  supplied.

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
- **Postconditions:** produced words obey the pinned Arm ARM Stage-2 block
  descriptor layout for the implementation-revision.
- **Security:** no Guest-reachable input reaches this type directly; still,
  `from_u64` must tolerate arbitrary bits (mask unknown fields) because it
  may read hardware-written memory in future Reserved walks.
- **`unsafe` boundary:** none inside `S2Desc` itself (pure bit manipulation).
  The `unsafe` lives in the write path (§2.2).
- **Validation:** exhaustive unit tests mapping every `S2Flags` variant pair
  to expected field values checked against the pinned spec revision; reserved
  bits preserved-then-cleared tests; round-trip tests.

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
  (2) the caller holds the space lock, so no concurrent descriptor writer
  exists;
  (3) `index < entries_per_table` derived from the granule;
  (4) the table pointer is a valid, aligned mapped Host address (Host Stage-1
  maps all of EL2-managed RAM per P1-W08).
- **Postconditions:** the slot contains `value` (write is a single aligned
  8-byte store, ordered per §6 of [02](02-architecture-and-state.md): the
  caller performs required `DSB`/`TLBI`/`ISB` around sequences — this function
  itself only stores).
- **Concurrency/allocation:** no allocation; lock held by caller; no IRQ
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

- **Name and stability:** `fn walk_to_leaf(&mut self, ipa: GuestPhysAddr,
  alloc: TableAlloc) -> Result<LeafRef, Stage2Error>` on the table owner.
  Internal.
- **Purpose:** descend from root to the leaf slot for `ipa`, allocating
  intermediate tables when `alloc` says `Allocate` and reporting `NotFound`
  when `alloc` is `RequireExisting`.
- **Inputs/outputs:** validated IPA; returns a `LeafRef` (table pointer +
  index) valid only under the held lock and before any table free.
- **Preconditions:** space lock held; ipa within the space's IPA span.
- **Postconditions:** on success the path root→leaf exists; on
  `OutOfPages` failure during `Allocate`, already-allocated intermediates on
  this walk are freed before returning the error (no partially built paths
  persist).
- **Errors:** `OutOfPages`, `NotFound` (RequireExisting mode with an invalid
  intermediate).
- **Logic:**

```text
walk_to_leaf(ipa, alloc):
    level = root_level; frame = root_frame
    loop:
        index = extract_index(ipa, level)          // checked by caller span
        entry = read_desc(frame, index)            // SAFETY via walk context
        if level == leaf_level: return LeafRef(frame, index)
        if entry.is_valid():
            frame = table_frame_of(entry)          // next-level table
        elif alloc == Allocate:
            new = alloc_table()?                   // may fail OutOfPages
            write_desc(frame, index, S2Desc::table(new))  // ordered store
            frame = new
        else: return Err(NotFound)
        level -= 1
```

- **Validation:** unit tests over simulated multi-level trees: empty walk
  with allocation, failure atomicity, deep paths, span-boundary indices.

## 3. `s2-space` public contracts

### 3.1 `GuestAddressSpace::create`

- **Name and stability:** `fn create(allocator: &mut PageAllocator, ipa_span:
  GuestIpaRange) -> Result<GuestAddressSpace, Stage2Error>`.
  Internal, P4-unstable.
- **Purpose and caller:** construct an independent translation context;
  called once by the P4 minimal VM setup (W04 consumer).
- **Inputs/outputs:** allocator handle; requested Guest IPA span; returns the
  space or a named error.
- **Preconditions:** allocator satisfies M2 (no protected pages);
  `ipa_span` non-empty, page-aligned, representable at the chosen root level
  (D1); P4's single-live-space discipline holds (caller-side; a second
  concurrent create is a caller invariant violation → fatal per D10 if
  enforced by explicit check).
- **Postconditions:** space in `Constructed`; root table allocated and
  zeroed; VMID assigned (D4); no mappings; no hardware state touched (no
  activation here).
- **State/ownership change:** allocator: pages → space-owned table frames;
  `s2-vmid`: one VMID reserved.
- **Concurrency/allocation:** may allocate; no locks held other than the
  allocator's own; not callable from IRQ context.
- **Errors:** `OutOfPages` (root), `VmidExhausted`, `Misaligned`,
  `RangeOverflow`; failure leaves no partial allocation (root freed, VMID
  released).
- **Security:** VMID uniqueness gives TLB isolation between spaces even
  before any mapping exists; the zeroed root translates nothing.
- **Logic:**

```text
create(allocator, span):
    validate span (alignment, size vs D1 root coverage) else Misaligned/RangeOverflow
    root = allocator.alloc_zeroed_page() else return OutOfPages
    vmid = vmid_alloc.allocate() else { allocator.free(root); return VmidExhausted }
    return GuestAddressSpace { root, tables:[root], vmid, ledger: empty,
                               state: Constructed, lock: new }
```

- **Validation:** create/destroy unit tests; VMID exhaustion test with a
  stubbed allocator set; determinism of initial state (no stale entries).

### 3.2 `map`

- **Name and stability:** `fn map(&mut self, ipa: GuestIpaRange, grant:
  MappingGrant) -> Result<(), (Stage2Error, MappingGrant)>`. Internal.
- **Purpose and caller:** install IPA→HPA translations for Guest RAM and the
  Guest console page; caller: P4-W03 loader/attach path.
- **Inputs/outputs:** page-aligned IPA range and an equal-size validated
  grant; returns success or the error with the unconsumed grant.
- **Preconditions:** `ipa.pages == grant.page_count`; space in `Constructed`
  or `Active` (D6); lock held via `&mut self` (P4 single-writer discipline —
  the lock is the borrow in P4's single-owner object; if P3-era code requires
  interior locking, the lock is taken internally; both shapes satisfy D5).
- **Postconditions:** every IPA page in the range translates to the
  corresponding frame with `grant.flags`; ledger updated; if the space is
  `Active`, BBM + current-path invalidation applied for the range
  ([02 §6](02-architecture-and-state.md)).
- **Errors:** `AlreadyMapped` (any page overlapping the range is mapped),
  `Misaligned`, `RangeOverflow`, `OutOfPages` (intermediate tables).
- **Failure guarantee:** check-then-mutate is performed in two passes —
  pass 1 validates the whole range against the ledger (no mutation); pass 2
  mutates. A failure in pass 2 (allocation) rolls back as in §2.3. The
  grant is never partially consumed.
- **Security:** input validation covers alignment, zero-size, overflow;
  descriptor content derives only from the validated grant; no Guest control
  over flags exists in P4.
- **Logic:**

```text
map(ipa, grant):
    if ipa.pages != grant.page_count: return Err(Misaligned, grant)
    ledger.check_all_unmapped(ipa)?                  // pass 1, no mutation
    leaf_refs = reserve_intermediates(ipa)?          // may roll back
    active = self.state == Active
    for (page, idx) in zip(grant.pages(), ipa.pages()):
        if active: bbm_invalidate(page)              // clear+DSB+TLBI
        write leaf S2Desc::page(grant.frame_base + page, grant.flags)
    ledger.insert(ipa, grant)                        // pass 2 commit
    if active: dsb(); invalidate_range(ipa)          // §6 ordering
    emit s2.map
```

- **Validation:** P4-V02 mutation tests; overlapping/zero/overflow negative
  tests (host-side, host-testable logic); QEMU-level mapped-access proof via
  W05/W08.

### 3.3 `unmap`

- **Name and stability:** `fn unmap(&mut self, ipa: GuestIpaRange) ->
  Result<(), Stage2Error>`. Internal.
- **Purpose:** revoke translations (isolation negatives, teardown).
- **Preconditions:** space `Constructed` or `Active`; range mapped.
- **Postconditions:** every page in range unmapped (descriptor invalid,
  ledger removed); empty intermediate tables may be freed (P4 frees eagerly to
  keep destroy simple); if `Active`, invalidation for the range applied
  **before** returning success.
- **Errors:** `NotMapped` (whole-range-or-nothing semantics: the call maps
  exactly to the ledger's range granularity; P4 does not implement partial
  range splits).
- **Failure guarantee:** all-or-nothing per whole range; no partial unmapped
  states persist on error.
- **Security:** unmap of a never-mapped or foreign range is a host-side
  programming error surfaced as `NotMapped`; a Guest cannot request unmaps in
  P4 (no guest-facing API).
- **Logic:**

```text
unmap(ipa):
    entry = ledger.exact(ipa) else return NotMapped
    for page in ipa.pages():
        leaf = walk_to_leaf(page, RequireExisting)   // cannot fail post-ledger
        bbm_sequence: write S2Desc::invalid(); dsb_ishst(); if Active: tlbi(page)
    ledger.remove(ipa); free_empty_intermediates(ipa)
    if Active: dsb(); isb()
    emit s2.unmap
```

- **Validation:** unmap-then-access-faults evidence (P4-V02/V07 via W05/W08);
  repeated map/unmap cycle test (ledger and page-accounting restoration).

### 3.4 `protect`

- **Name and stability:** `fn protect(&mut self, ipa: GuestIpaRange, new_flags:
  S2Flags) -> Result<(), Stage2Error>`. Internal.
- **Purpose:** change permission/type of existing mappings (P4-V08).
- **Preconditions:** range mapped; `new_flags` valid combination.
- **Postconditions:** all pages in range carry `new_flags`; invalidation per
  §6.1 two-step sequence (invalidate old translation *before* writing the new
  valid descriptor) so no stale-permission window remains for the current
  path.
- **Errors:** `NotMapped`, `InvalidFlags`.
- **Failure guarantee:** all-or-nothing; validation precedes mutation.
- **Security:** this is the mechanism behind "Guest writes read-only memory →
  permission fault"; correctness is an isolation requirement, covered by the
  two-step ordering above.
- **Logic:**

```text
protect(ipa, new_flags):
    entry = ledger.exact(ipa) else return NotMapped
    validate_flags(new_flags) else return InvalidFlags
    for page in ipa.pages():
        leaf = walk_to_leaf(page, RequireExisting)
        old = read_desc(leaf)
        write S2Desc::invalid() at leaf; dsb_ishst(); if Active: tlbi(page)
        write S2Desc::page(old.frame, new_flags) at leaf
    ledger.set_flags(ipa, new_flags)
    if Active: dsb(); isb()
    emit s2.protect
```

- **Validation:** P4-V08 permission negatives (write-to-RO, execute
  comparison) through W05/W08; immediate-effect test: protect → guest write →
  fault without intervening unmap/map (proves no stale-permission reliance).

### 3.5 `query`

- **Name and stability:** `fn query(&self, ipa: GuestPhysAddr) ->
  QueryResult`. Internal.
- **Purpose and caller:** diagnostics (W06), negative-test assertions (W02's
  own evidence), W03 attach checks.
- **Preconditions:** none beyond a valid IPA value (out-of-span IPAs are
  simply `Unmapped`).
- **Postconditions:** returns a snapshot; never mutates.
- **Concurrency:** shared read under the space lock; O(log n) ledger lookup.
- **Errors:** none (result type carries the answer).
- **Security:** exposes only what the caller already knows (IPA → mapping
  facts); no Guest-untrusted input involved.
- **Logic:** ledger lookup; compute `frame_base` by mapping the containing
  ledger entry.
- **Validation:** query agreement tests after map/unmap/protect sequences
  (P4-V02 query evidence).

### 3.6 `activate`

- **Name and stability:** `fn activate(&mut self, cpu: PcpuId) -> Result<(),
  Stage2Error>`. Internal; the only legitimate caller is the P4-W04 entry
  path.
- **Purpose and caller:** install the space as the current Stage-2 context
  (P4-A06) before Guest entry.
- **Inputs/outputs:** identity of the pCPU performing activation (from the
  P3 CPU-local mechanism, M6); success or error.
- **Preconditions:** space `Constructed` (first activation) or already
  `Active` on the same pCPU (idempotent re-activation by the same path);
  Guest execution is not running; interrupts masked per W04's entry protocol
  (context switch between host and guest is atomic with respect to the run
  path).
- **Postconditions:** `VTCR_EL2`/`VTTBR_EL2` programmed (root, VMID, T0SZ
  for the space's span, granule, memory attributes) with reserved bits
  preserved; `ISB` executed; space state `Active` with recorded pCPU.
- **State/ownership:** none beyond state; no allocation.
- **Errors:** `SpaceOnWrongPcpu`-style programming-error escalation (fatal
  invariant, D10) if a second pCPU attempts activation — P4 defines no
  legitimate concurrent activator.
- **Security:** VMID + root installed together prevents any window where a
  fresh VMID could tag hostile prior translations; disabled-by-default
  hardware state (Stage-2 off) between Guests is the safe direction.
- **Logic:**

```text
activate(cpu):
    match state:
      Constructed: pass
      Active(p) if p == cpu: return Ok(())        // idempotent
      Active(p)                                   // different pCPU
                     -> invariant violation (fatal escalation, D10)
    vtcr = VtcrValue::for_span(self.span)         // computed at create
    write VTCR_EL2 preserving reserved bits
    write VTTBR_EL2 = { vmid: self.vmid, baddr: phys(root) }
    isb(); state = Active(cpu); emit s2.activate
```

- **Validation:** activation before-entry evidence (P4-V02, P4-V04 via W04);
  deactivation-on-destroy verified by post-destroy Guest-attempt faulting
  (with W05/W08).

### 3.7 `invalidate` seam operations

- **Name and stability:** `fn invalidate_all_current(&mut self)` and
  `fn invalidate_range_current(&mut self, ipa: GuestIpaRange)`. Internal;
  implemented over the `s2-tlb` backend (D7).
- **Purpose:** P4-A07 current-path consistency primitive.
- **Preconditions:** meaningful only when `Active`; no-op otherwise (and
  marked as such).
- **Postconditions:** after return (plus `DSB`), the current pCPU's
  subsequent translations cannot reuse pre-mutation entries for the affected
  scope; `ISB` before guest (re-)entry is the entry path's duty (W04).
- **Errors:** none.
- **Security:** VMID-scoped operations (never a whole-VMID-set flush in P4)
  avoid collateral invalidation of host state; the seam exists so cross-pCPU
  masking can be added without semantic drift.
- **Logic:** encode `TLBI` operation per pinned spec for "current VMID,
  all/range" (range variant requires the architectural range extension —
  capability-gated from the P1 inventory; fallback is a loop over page-granular
  invalidations, which P4 uses if the extension is absent), execute, `DSB`.
- **Validation:** "no stale translation" evidence: map-then-access and
  unmap-then-fault sequences at P4-V02/V07; capability-gate unit tests for
  both invalidation encodings.

### 3.8 `destroy`

- **Name and stability:** `fn destroy(self) -> Result<(), Stage2Error>`
  (consuming). Internal.
- **Purpose and caller:** full teardown; called by the P4 minimal VM teardown
  after the vCPU is stopped (ordering owned by W04/W09).
- **Preconditions:** no vCPU running through this space (caller's
  sequencing duty; an explicit `Active` check enforces deactivation inside
  destroy).
- **Postconditions:** all translations removed; invalidation applied if it
  was active; all table frames freed to the allocator; VMID released; space
  consumed. Guest data frames are *not* freed here (W03's Guest RAM object
  owns them; sequencing: address-space destroy before Guest RAM release).
- **Errors:** allocator free errors surface rather than being swallowed;
  after a free error the space reports destruction as failed and records
  which frames leaked (explicit leak reporting beats silent loss).
- **Failure guarantee:** partially destroyed spaces are impossible to
  reference (consuming move); a failed destroy is terminal and reported.
- **Security:** zeroing descriptor memory before free prevents stale-content
  leakage through recycled frames (cheap, deterministic).
- **Logic:**

```text
destroy(self):
    if self.state == Active(p): deactivate_current_path(p)   // disable S2 + isb
    for range in ledger: write invalid leaves; free empty tables
    dsb(); invalidate_all_current() if was_active
    zero_and_free(table frames)                              // report any free error
    vmid_alloc.release(self.vmid)
    emit s2.space.destroy
```

- **Validation:** lifecycle repeat test (create→map→activate→destroy→create)
  with accounting checks (P4-V02 repeated-lifecycle evidence); destroy-while-
  running is structurally prevented and reviewed.

## 4. Error and failure classification recap

All `Stage2Error` returns are VM-facing recoverable values (D10): they must
not panic, must not log-and-continue silently, and must be surfaced to the
P4 run/diagnostic layer. The fatal class is reserved for hypervisor invariant
violations (second activator, use-after-destroy, allocator returning a
protected page if detectable) and escalates through the P0 failure-
classification contract (M1/M8 assumptions; W01 R19/A2).
