# P4-W03 Code Contracts — Guest Memory and Image

**Status:** Approved detailed design (project owner) v0.2, 2026-10-02; implementation and runtime evidence are not claimed.
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
- **Preconditions/postconditions:** `validate` proves (a) image, stack and boot-info lie inside Guest RAM while the console
  page lies outside RAM, (b) these extents are disjoint and page-aligned,
  (c) `max_image_size` leaves the stack and boot-info regions intact, (d)
  the usable backing size is supported by P2 while IPA values fit W02 geometry (M7). The validate check
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

Inputs: validated layout, W04 allocator and W12 store. Request a contiguous
usable page count through W04 allocate_contiguous(PageCount), then move the returned
AllocatedFrames into W12. GuestRam owns only the resulting MemoryObject control
capability, layout binding and initialization state. W12 retains the full buddy
block while exposing only the requested usable prefix; no `mark_guest_use` or
invented `alloc_run` API is assumed. Failure before adoption returns the original
handle for explicit W04 free; failed free retains its returned handle.

The temporary Guest IPA layout is independent of the selected HPA. A fixed flat
binary can remain linked at its documented Guest IPA without requiring W04 to
allocate a particular physical address. This supersedes old identity placement
(D2), an implementation choice under task-book §8; no P8 machine ABI is created.
The base layout/BootInfo format remains a versioned P4 test convention.

### 3.2 `GuestRam::init_zeroed`

Require an exclusive W12 Host RW region mapped by W11 and no Guest view or
external-use pin. Initialize every usable byte through scoped MaybeUninit writes;
only after completion mark Zeroed. Allocated does not mean Zeroed. Re-init first
stops Guest, retires every old Guest region through W02, obtains a fresh Host
writer and repeats initialization. Stopping a vCPU alone does not retire mappings.

### 3.3 `GuestRam::mapping_grants`

`reserve_guest_regions(&mut self, space_id, storage)` is the replacement API;
this heading retains the old navigation name. Require Loaded, successful
instruction-visibility preparation for code, and completely retired Host writer.
Reserve non-Copy W12 region leases covering disjoint RX code, R test/boot-info
and RW/XN data/stack extents. All remaining RAM has an explicit disjoint default
RW/XN extent; gaps/overlaps are validation failures. Slot capacity is preflighted;
if any reservation fails, cancel every still-Reserved lease without publication.
Once submitted to W02, only its transaction protocol may retire it. Grants cannot
be regenerated from layout constants or queried HPA values.

The console is not RAM: a separate base-console authority produces one exact
Device RW/XN ConsoleWindowGrant from verified platform facts and W04's exclusive
console-use convention. No W12 object or W04 free is associated with it. W10
RAM-only extension fixtures omit this grant and do not share the base console.

### 3.4 `GuestRam::release`

`release(self, store, allocator) -> ReleaseOutcome` asks W12 to take_back the
object. Any Reserved, Publishing, Live, Revoking, Quarantined view or use pin
returns Busy with the object retained. There is no unscoped Stage2Released proof.
On success, free the exact original full allocation handle via W04 free_contiguous. Free failure
returns an owned failure handle; record it as retained, never successful release.
W02 destroy/unmap completion removes its own leases, but cannot free GuestRam.

### 3.5 Host write view (module `gm-ram`)

`write_slice(mapped, offset, src)` is a safe wrapper over W11's scoped byte
callback. Check offset+length and usable-region bounds before stores; write via
MaybeUninit elements, never reconstruct an HVA slice from an HPA or a layout.
Zero-length at end is allowed; one-past-end or overflow rejects without mutation.
The callback lifetime cannot escape, overlap revoke or an external-use pin.
Architecture unsafe resides in W11's audited view construction and P4's
instruction-visibility backend; W03 does not add another raw-pointer writer.

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
  (destination IPA, object-relative offset and length; no HVA authority) or error.
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

Inputs: GuestRam in Allocated/Zeroed, image, scenario and live W11 Host writer;
output is a PreparedGuestInput (entry, stack, boot-info IPA and scenario), not
permission to enter Guest. Validate layout, scenario, complete image copy plan
and BootInfo serialization before the first write. Zero whole usable region,
write boot-info, copy image, and perform P4 architecture code-visibility
preparation for the future Guest execution profile. Finish the byte callback,
fully revoke Host writer, then mark Loaded and permit Guest-region reservation.
After W02 mappings commit, the caller may construct/enter the vCPU through W04.

Validation rejection leaves prior bytes unchanged. A failure after stores begin
may leave partially initialized bytes; mark Failed/Retained, never Loaded and
never enter Guest. If Host mapping can be safely retired, later explicit re-init
may start from a full zero-fill; if completion is uncertain, quarantine until
restart. There is no false all-or-nothing byte rollback guarantee.

For the admitted AArch64 profile, code-visibility preparation cleans the written
code to the required instruction-unification point, orders it, invalidates the
relevant instruction context and completes the required barriers. Its backend
binds the initialized object/code range and intended execution CPU set. Base P4
supports its one admitted CPU; W10 must extend preparation to all entry CPUs
before execution. The backend returns a private PreparedCode receipt bound to ObjectId, code
extent, content epoch and prepared CPU set. A later writable Guest mapping
invalidates it. W02 checks it before enabling execute, and W10 checks CPU-set
coverage before entry; adding another execution CPU requires new preparation
while the code is quiescent. A local data write or TLB acknowledgment is not proof
of cross-CPU instruction visibility. Exact cache-operation/reference review is part
of architecture admission, not delegated to Guest or generic W12.

Tests cover rejection-before-write, deterministic bytes for identical inputs,
mid-copy fatal injection with no GuestInput exposure, instruction-preparation
failure, Host revoke failure, and Guest grant rejection when Host writer remains.
Integrated P4-V03 demonstrates actual target load and entry; Host buffers do not.

## 6. Error model recap

Recoverable preflight errors: OutOfPages, Capacity, ImageEmpty, ImageTooLarge,
Misaligned, DestinationOverlap, RangeOverflow, Busy and unsupported profile.
LayoutMismatch is a host-authored contract violation. After stores/publication,
Retained explicitly reports partial initialization or unknown mapping completion;
unknown hardware state is fail-stop. No error silently converts retained backing
to free pages or treats a partial image as a valid Guest input.
