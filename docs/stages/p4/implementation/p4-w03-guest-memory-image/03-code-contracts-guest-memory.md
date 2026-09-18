# P4-W03 Code Contracts — Guest Memory and Image

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W03 detailed design](README.md).  
**Companion:** module and lifecycle semantics in
[02-architecture-and-state.md](02-architecture-and-state.md).

All names are P4-internal and unstable-by-declaration; P4-W09 records them as
implemented facts only. Pseudocode is an outline, not production code; the
Coding Guidelines govern the final Rust shape. Typed-address newtypes
(`HostPhysAddr`, `GuestPhysAddr`, `ByteLen`, `PageCount`) are assumed from
the P0 baseline (M1) and are not redefined here.

## 1. `GuestLayout` record (module `gm-layout`)

- **Name and stability:** `GuestLayout` — associated constants
  (`LAYOUT_VERSION: u32`, `ram_base: GuestPhysAddr`, `ram_size: ByteLen`,
  `image_load: GuestPhysAddr`, `stack_top: GuestPhysAddr`,
  `stack_size: ByteLen`, `boot_info: GuestPhysAddr`,
  `console_page: GuestPhysAddr`, `max_image_size: ByteLen`) plus
  `fn validate(&self) -> Result<(), LayoutError>`. Internal, P4-unstable.
- **Purpose and caller:** single source of truth for the temporary P4 Guest
  IPA layout (P4-B02); consumed by W02 mapping requests, W04 vCPU
  construction, W05 Guest expectations, W08 automation documentation.
- **Inputs/outputs:** constants; validation returns disjointness/alignment/
  containment errors.
- **Preconditions/postconditions:** `validate` proves (a) every special
  region lies inside RAM, (b) regions are pairwise disjoint and page-aligned,
  (c) `max_image_size` leaves the stack and boot-info regions intact, (d)
  values fit within the P2-declared reference RAM (M7). The validate check
  runs once at construction setup and aborts setup on failure (fatal setup
  invariant — the layout is host-authored, so failure here is a build
  contract error, not a Guest event).
- **Errors:** `LayoutError` (disjointness/alignment/containment violations).
- **Security:** values are host-authored constants; the record exists so that
  every consumer cites one source, preventing divergent duplicated constants.
- **Validation:** unit test evaluates `validate` and asserts the documented
  invariant set; review row: W09 records values as test facts, not ABI.

## 2. `BootInfo` block (module `gm-loader`)

- **Name and stability:** `BootInfo` — explicit-layout block: magic `u32`,
  `layout_version: u32`, `block_size: u32`, `ram_base: u64`, `ram_size: u64`,
  `image_entry: u64`, `image_size: u64`, `console_page: u64`,
  `scenario_id: u32`, reserved-to-zero fields, `checksum: u32` over the
  preceding bytes. Internal; the *format* is a P4 test convention (D4) and is
  shared with W05's Guest parser by citation, not by shared code (the Guest
  re-derives and validates it independently).
- **Purpose and caller:** pass construction parameters into the Guest
  without embedding Host constants in Guest code.
- **Inputs/outputs:** built from (`GuestLayout`, `ValidatedImagePlan`,
  `ScenarioId`); serialized into Guest RAM by the loader.
- **Preconditions:** target IPA inside RAM, block-size bound checked,
  reserved fields zeroed.
- **Postconditions:** block content is a deterministic function of its
  inputs (no clocks, no ASLR-like variation).
- **Errors:** construction errors if any field exceeds its width (checked
  conversions).
- **Security:** the Guest treats the block as untrusted-until-validated
  (magic, version, size, checksum all verified Guest-side); EL2-side, the
  block is host-authored so no Guest-influenced field exists.
- **Validation:** round-trip unit tests (serialize → independent Guest-style
  validate → fields agree); checksum and truncation negative tests.

## 3. `GuestRam` contracts (module `gm-ram`)

### 3.1 `GuestRam::allocate`

- **Name and stability:** `fn allocate(allocator: &mut PageAllocator,
  layout: &GuestLayout) -> Result<GuestRam, GuestMemoryError>`. Internal.
- **Purpose and caller:** obtain the bounded Guest RAM region (P4-B01);
  called by the P4 minimal VM setup.
- **Inputs/outputs:** allocator handle and layout; returns the object or a
  named error.
- **Preconditions:** allocator satisfies M2/M3; `layout.validate()` already
  passed; RAM size is a whole number of pages.
- **Postconditions:** a contiguous page run of exactly `ram_size` is owned by
  the object; pages marked Guest-use in accounting (M3); contents are *not*
  yet assumed zero until `init_zeroed` runs (allocation and zeroing are
  separate so a failure between them cannot be mistaken for a deterministic
  state).
- **State/ownership change:** allocator accounting → `GuestRam` ownership;
  state `Allocated`.
- **Concurrency/allocation:** allocates; no other locks held; not in IRQ
  context.
- **Errors:** `OutOfPages`, `AccountingUnavailable` (M3 boundary — recorded
  gap path, degraded to plain alloc/free per
  [01 §2](01-scope-and-foundations.md)).
- **Security:** the allocator's protected-range exclusion is the guarantee
  that no protected page is included; W03 adds no second source of pages.
- **Logic:**

```text
allocate(allocator, layout):
    pages = layout.ram_size / PAGE_SIZE            // exact by precondition
    run = allocator.alloc_run(pages)?              // contiguous per D1
    allocator.mark_guest_use(run)?                 // M3; gap-degradable
    return GuestRam { run, state: Allocated }
```

- **Validation:** unit tests with allocator stubs: success, OOM,
  accounting-failure path; host-side only until P2 delivers (M2 evidence
  gates P4-V03, not this unit work).

### 3.2 `GuestRam::init_zeroed`

- **Name and stability:** `fn init_zeroed(&mut self) -> Result<(),
  GuestMemoryError>`. Internal.
- **Purpose:** establish the deterministic base state (`Allocated(Zeroed)`).
- **Preconditions:** state `Allocated`.
- **Postconditions:** every byte of the region is zero; state
  `Allocated(Zeroed)`; callable again later as re-init (repeat support).
- **Concurrency:** bounded write loop; interrupts masked for the duration
  (D7 discipline; size ≤ layout RAM bound — small in P4).
- **Errors:** none expected; any fault here is a fatal Host-memory invariant
  (escalates per P0 failure classification).
- **Security:** zeroing removes any cross-run residue dependence (P4-V10
  support, W07).
- **Logic:** page-wise zero stores via the write view (§3.5).
- **Validation:** determinism tests: init → read-back pattern checks;
  repeat-init equivalence.

### 3.3 `GuestRam::mapping_grants`

- **Name and stability:** `fn mapping_grants(&self) -> Result<[MappingGrant;
  N], GuestMemoryError>` — RAM grant (Normal, cacheable, RW, XN per
  sub-region plan) plus console-page grant (Device, RW, XN). Internal;
  consumed by W02 `map`.
- **Purpose:** produce the validated mapping inputs (W02 D8) from owned
  pages (P4-B01 → Stage-2 hand-over).
- **Preconditions:** state `Allocated(Zeroed)` or later; grants not yet
  produced (or regenerable deterministically — P4 regenerates from
  constants; the grants carry no uniqueness).
- **Postconditions:** grants cover exactly the RAM region and the console
  page; flags follow the layout's permission plan (the sub-region plan is
  layout-authored, loader-applied: data region RW+XN, code window R+X, and
  a read-only test window carried in boot-info for the W05 permission
  scenarios).
- **Errors:** `LayoutMismatch` if region arithmetic disagrees with the
  validated layout (internal invariant — fatal class).
- **Security:** flags are host-authored; no Guest influence.
- **Validation:** grant-vs-layout agreement tests; joint tests with W02 map
  (positive) in the integrated workflow.
- **Note:** the data/code permission split is recorded as a P4 test
  convention enabling W05's VG-005/VG-006 permission scenarios; it is not a
  general memory-protection policy.

### 3.4 `GuestRam::release`

- **Name and stability:** `fn release(self, unmap_proof: Stage2Released)
  -> Result<(), GuestMemoryError>` (consuming). Internal.
- **Purpose:** return pages to the allocator; end the object's life.
- **Preconditions:** `unmap_proof` evidences that all Stage-2 mappings over
  the region are destroyed (W02 destroy's return value; D8 sequencing).
- **Postconditions:** pages freed; Guest-use accounting cleared; object
  consumed.
- **Errors:** free failures surface (leak reporting), never swallowed.
- **Security:** zeroing before free is not required for Guest RAM by the P4
  threat model (Host-owned RAM reuse), but a debug-feature zero is permitted;
  the *accounting* release must be exact (hard P2 safety gate inheritance).
- **Validation:** release-after-destroy test; double-release impossible by
  type; accounting restoration check.

### 3.5 Host write view (module `gm-ram`, `unsafe`)

- **Name and stability:** `unsafe fn write_slice(&mut self, offset: ByteLen,
  src: &[u8])` — the single bounded writer (D7). Internal.
- **Purpose and caller:** loader writes (boot-info, image copy) into Guest
  RAM through Host virtual addresses.
- **Inputs/outputs:** region-relative offset and source bytes.
- **Preconditions (SAFETY obligations):** (1) `self` owns the frames and
  they are Host-Stage-1-mapped (M4); (2) `offset + src.len()` computed with
  checked arithmetic and proven ≤ region size *before* any store; (3) no
  concurrent writer (single setup thread; `&mut self`).
- **Postconditions:** exactly `src.len()` bytes stored at `offset`.
- **Errors:** none internal; the safe wrapper returns
  `RangeOverflow`-class errors when the checked bound fails (never performs
  the store).
- **Security:** the Guest cannot reach this path; bounds are the P4-V03
  "prohibited overwrite" defense.
- **Logic:**

```text
write_slice(offset, src):                       // unsafe entry
    end = offset.checked_add(src.len()) else return Err(RangeOverflow)
    if end > region_size: return Err(RangeOverflow)
    // SAFETY: frames owned+self-mapped via Host Stage-1 (M4);
    //          bound proven above; exclusive &mut self.
    store bytes at hva(base + offset .. end)
```

- **Validation:** boundary unit tests (offset at end, one-past-end refusal,
  zero-length writes); unsafe-inventory entry with the three-point SAFETY
  pattern.

## 4. `GuestImage` and load-plan contracts (module `gm-image`)

### 4.1 `GuestImage::view`

- **Name and stability:** `fn view(bytes: &'static [u8], identity:
  BuildIdentity) -> GuestImage`. Internal.
- **Purpose:** represent the build-embedded flat binary (D3) without
  embedding knowledge of its contents.
- **Contract notes:** no parsing; `identity` comes from the P0 version
  baseline for diagnostics. The embedding mechanism itself is build
  governance (M5), referenced but not designed here.

### 4.2 `validate_image_plan`

- **Name and stability:** `fn validate_image_plan(image: &GuestImage,
  layout: &GuestLayout) -> Result<ValidatedImagePlan, GuestMemoryError>`.
  Internal.
- **Purpose and caller:** the P4-B04 negative-input gate: empty, oversize,
  misaligned, or out-of-position images are rejected before any byte is
  written.
- **Inputs/outputs:** image view + validated layout → copy plan
  (destination IPA/HVA pairs, length) or error.
- **Preconditions:** layout already validated.
- **Postconditions:** plan covers exactly `image.len()` bytes at
  `IMAGE_LOAD_IPA`; plan arithmetic is pre-checked (destination end ≤ RAM
  end; image size ≤ `max_image_size`).
- **Errors:** `ImageEmpty` (zero-length), `ImageTooLarge`, `Misaligned`
  (load IPA not page-aligned — layout invariant), `DestinationOverlap`
  (would hit stack/boot-info/console — layout invariant failure class).
- **Failure guarantee:** rejection happens before any mutation; no partial
  plans exist.
- **Security:** this function is the primary defense for "invalid image
  inputs cannot overwrite protected memory" (P4-V03): destination bounds are
  proven against the layout, and the region itself is allocator-derived
  (M2), so protected-memory overwrite is doubly excluded.
- **Logic:**

```text
validate_image_plan(image, layout):
    if image.len() == 0: return Err(ImageEmpty)
    if image.len() > layout.max_image_size: return Err(ImageTooLarge)
    dest_span = span(image_load, image.len())          // checked arithmetic
    if !dest_span.within(layout.ram_span()): return Err(DestinationOverlap)
    if dest_span.intersects(layout.stack_span() | layout.boot_info_span()):
        return Err(DestinationOverlap)                 // layout invariant
    return ValidatedImagePlan { dest: dest_span, len: image.len() }
```

- **Validation:** negative unit suite per error class (P4-V03 evidence);
  positive plan for the built asset; property-style checks (random sizes/
  offsets always classified).

## 5. Loader contract (module `gm-loader`)

### 5.1 `load_guest`

- **Name and stability:** `fn load_guest(ram: &mut GuestRam, image:
  &GuestImage, scenario: ScenarioId) -> Result<GuestInput, GuestMemoryError>`.
  Internal.
- **Purpose and caller:** the complete P4-B03/B05 construction: validate →
  boot-info → copy, in fixed order; called by the P4 minimal VM setup.
- **Inputs/outputs:** initialized RAM (state `Allocated(Zeroed)`), image
  view, scenario id → `GuestInput` (entry IPA, stack top, boot-info IPA,
  scenario id — the pure-function inputs W04 constructs the vCPU from).
- **Preconditions:** `ram` zeroed; `ram`'s mapping grants already mapped into
  the (not yet active) address space — mapping-before-load is *not*
  required for correctness (loader writes via Host addresses), but the P4
  order maps first so a load can never succeed into an unmappable region;
  scenario id already validated against the W05-owned table.
- **Postconditions:** Guest RAM in `Loaded` state with the deterministic
  content of [02 §5](02-architecture-and-state.md); `GuestInput` returned.
- **State/ownership:** `GuestRam` state transitions to `Loaded`; no ownership
  transfers.
- **Concurrency:** runs in setup context; bounded, IRQ-masked writes.
- **Errors:** all validation errors from §4.2; write-path errors cannot occur
  post-validation (plan proven in-bounds) — any fault is fatal-invariant
  class.
- **Failure guarantee:** all-or-nothing: a failure anywhere leaves RAM in the
  zeroed base state (the copy is preceded by complete validation; boot-info
  write precedes image copy, and both are in-region proven).
- **Security:** Guest input does not exist yet (Guest has not run); all
  inputs are host-authored and validated. The defensive posture exists for
  the *Guest-side* parse (W05) and for repeat safety.
- **Logic:**

```text
load_guest(ram, image, scenario):
    plan = validate_image_plan(image, layout)?     // §4.2
    ram.reinit_zeroed()                            // deterministic base
    write BootInfo block at layout.boot_info       // §2
    ram.write_slice(plan.offset_of_image_load, image.bytes)   // §3.5
    ram.state = Loaded
    emit gm.load.copy
    return GuestInput { entry: layout.image_load, stack_top: layout.stack_top,
                        boot_info: layout.boot_info, scenario }
```

- **Validation:** end-to-end host tests with a stubbed RAM region;
  determinism test: two loads produce byte-identical regions; negative suite
  per §4.2; integrated on-target evidence via W08 (P4-V03).

## 6. Error model recap

`GuestMemoryError`: `OutOfPages`, `AccountingUnavailable`, `ImageEmpty`,
`ImageTooLarge`, `Misaligned`, `DestinationOverlap`, `RangeOverflow`,
`LayoutMismatch`, `ReleaseOrderViolation`. All are VM-facing recoverable
values for consumers; `LayoutMismatch` and any write-path fault are
host-authored-contract violations and escalate through the P0 failure
classification (fatal for setup), because they indicate a broken build
contract rather than a Guest event (W01 A2).
