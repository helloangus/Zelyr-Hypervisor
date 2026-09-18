# P1-W08 Address-Type and Page-Table Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W08 detailed design](README.md).

Pseudocode is an outline, not runnable production code. Exact descriptor
bit positions, `TCR_EL2`/`TTBR0_EL2` field layouts, and instruction
encodings follow the architecture revision recorded in the implementation
record (the W04/W03 discipline). All names are internal boot-scope items
owned by this design. No allocation anywhere.

## 1. Address types and checked arithmetic

```text
Name and stability: VirtAddr(u64); internal newtype; Copy; stable within
  P1. PhysAddr: W02's existing newtype, extended with the checked
  operations below as a recorded design change (W02's contract explicitly
  anticipates this: "checked arithmetic enters with W08's mapping work").
Purpose and caller: the typed-address discipline of the Coding
  Guidelines and the P0-W15 red lines, applied to table walking, region
  sizing, and entry addressing. Callers: the descriptor/table layer; the
  transition body; reviews.
Inputs / outputs: constructors from raw u64 are internal; the audited
  exits to raw values are exactly: the TTBR0_EL2/TCR_EL2 programming
  step, and the descriptor address fields.
Provided operations (all checked; overflow/underflow are invariant
  violations routed via the transition's failure route — they cannot
  arise from the static inventory, so a hit means the inventory is
  wrong):
  VirtAddr::page_base() / PhysAddr::page_base()      (4 KiB align-down)
  align_up_to_page(x)                                 (checked)
  offset_within(region) -> Option<PageOffset>         (bounds-checked)
  page_index_of(va) -> Level3Index                    (0..512 domain type)
  table_entry_pa(l3: &L3Table, i: Level3Index) -> PhysAddr
Preconditions / postconditions: no operation silently wraps or truncates;
  every raw-value exit is one of the named audited points.
State and ownership change: none (value types).
Concurrency/allocation context: no allocation.
Errors and failure guarantee: checked operations fail closed (route),
  never wrap.
Security/authorization checks: the types exist so addresses cannot mix
  with indices, lengths, or raw machine words.
Logic: transparent wrappers; arithmetic only through the named checked
  helpers.
Validation: W08-DV02; a naked-integer address computation anywhere in
  the package is a review failure (W08-DV05).
```

## 2. Descriptor and table types

```text
Name and stability: Descriptor(u64) with typed views TableDescriptor |
  PageDescriptor; L1Table = [Descriptor; 512]; L3Table = [Descriptor;
  512]; STAGE1_TABLES static; all internal; stable within P1.
Purpose and caller: the only page-table representation in P1. Caller:
  the build/verify functions (§4–§5) and the programming step (§6 of the
  transition contracts).
Inputs / outputs: typed accessors:
  Descriptor::table(pa: PhysAddr) -> Self       (points to an L3 page)
  Descriptor::page_4k(pa: PhysAddr, cls: MappingClass) -> Self
  Descriptor::is_valid(); Descriptor::as_page_pa(); Descriptor::class()
Preconditions / postconditions: only documented, reserved-bit-respecting
  fields are set (recorded against the architecture revision); a
  descriptor never encodes a class outside the six of the class table.
State and ownership change: the tables' contents (W08-owned per the
  architecture file §4).
Concurrency/allocation context: boot context; private statics; no
  allocation; plain safe memory writes (the table storage is ordinary
  .bss — no volatile, no aliasing: single-threaded construction).
Errors and failure guarantee: encoding is total over (PhysAddr,
  MappingClass); no error path.
Security/authorization checks: the class parameter is the only source of
  permission bits — a descriptor cannot be constructed with attributes
  outside the class model.
Logic:
  table descriptor: validity + table address + documented hints, per the
    recorded revision
  page descriptor: validity + attribute fields derived from the class
    table entry + PA bits
Validation: W08-DV02 (encoding review against the recorded revision).
```

## 3. MappingClass — the closed attribute vocabulary

```text
Name and stability: MappingClass (Copy + PartialEq enum: CodeRx |
  RoData | DataRw | BootStack | Vectors | ConsoleMmio); internal; stable
  within P1.
Purpose and caller: the six-class model of the architecture file §3;
  the single source from which descriptor attributes derive. Callers:
  the inventory table; descriptor construction; verification; the
  prohibited-attribute review.
Inputs / outputs: per class: memory type, cacheability, X, W (the
  recorded semantics); fn label(self) -> &'static str.
Preconditions / postconditions: closed set; the attribute assignment is
  design authority, not runtime data.
State and ownership change: none.
Concurrency/allocation context: none.
Errors and failure guarantee: cannot fail.
Security/authorization checks: this type is the no-RWX enforcement point:
  no variant carries X and W together, and no variant is executable
  Device memory.
Logic: plain enum; attribute derivation lives with the descriptor
  encoding.
Validation: W08-DV02/DV05.
```

## 4. Table build

```text
Name and stability: fn build_stage1_tables(regions: &[RegionSpec]) ->
  Result<(), Stage1Error>; internal; stable within P1.
Purpose and caller: fill STAGE1_TABLES from the inventory. Caller: the
  transition body, before any register is touched.
Inputs / outputs: the verified inventory (each RegionSpec: name, start
  VirtAddr, byte length as a typed region size, MappingClass); ok, or
  the first error.
Preconditions / postconditions: tables start zeroed (.bss); on Ok every
  inventory page has exactly one descriptor, every descriptor maps to an
  inventory page, and the L1 has table descriptors only where an L3 is
  populated. Build never touches hardware.
State and ownership change: STAGE1_TABLES contents.
Concurrency/allocation context: boot context; no allocation.
Errors and failure guarantee: overlapping regions, unaligned bounds, an
  L1 index collision, or a class/region mismatch -> Stage1Error with the
  region's static name; tables left in the pre-build (zeroed) state on
  error and the boot routes — no partial tables are ever enabled (the
  enable step only runs after §5 verifies).
Security/authorization checks: the region list is the trust boundary of
  this package — it is a reviewed static, not runtime input.
Logic:
  zero tables (explicit; do not rely on .bss freshness alone — recorded
    belt-and-braces)
  for region in regions:
    for each page in region (typed arithmetic):
      l1_index = page_index_l1(va); l3 = ensure_l3(l1_index)?
      l3[page_index_l3(va)] = page_4k(page_base_pa(va), region.class)
Validation: W08-DV02; the count/closure checks of §5.
```

## 5. Table verification

```text
Name and stability: fn verify_stage1_tables(regions: &[RegionSpec]) ->
  Result<(), Stage1Error>; internal; stable within P1.
Purpose and caller: the pre-enable check that makes "explicit
  attributes" mechanically true (W08-DV03). Caller: the transition body,
  after build, before barriers.
Inputs / outputs: the same inventory; ok, or the first mismatch.
Preconditions / postconditions: pure read-back over the tables; on Ok:
  every mapped entry's decoded attributes equal its class's recorded
  semantics; the set of valid entries equals exactly the inventory's
  page set (no extra, no missing); both directions are checked.
State and ownership change: none.
Concurrency/allocation context: boot context; no allocation.
Errors and failure guarantee: a mismatch names the region and the
  differing attribute; the boot routes (fatal, `stage1`) — a wrong
  mapping is never enabled.
Security/authorization checks: this is the package's containment gate —
  the check that no RWX, no Device-X, no writable-code mapping can
  reach the hardware.
Logic:
  walk L1: only documented indices valid; each valid -> its L3
  walk each L3: valid entries -> decode pa, attrs; compare against the
    inventory's expected (pa, class) set; count equality both ways
Validation: W08-DV03; the no-RWX review (W08-DV05) cites this gate.
```

## 6. Sysreg and barrier boundary

```text
Name and stability: fn ttbr0_write(pa: PhysAddr), fn tcr_write(v: u64),
  fn sctlr_write(v: u64), fn sctlr_read() -> u64, fn ttbr0_read() ->
  u64, and the asm wrappers fn barrier_dsb_ish(), fn barrier_isb(),
  fn tlb_invalidate_alle2(), fn ic_invalidate_all(); internal; stable
  within P1.
Purpose and caller: the only MMU/system-register and cache/TLB
  instruction access in W08. Callers: the programming step and the
  transition sequence of [mapping and transition](03-code-contracts-mapping-and-transition.md)
  §3.
Inputs / outputs: typed or raw-recorded values; the closed register set
  is { TTBR0_EL2, TCR_EL2, SCTLR_EL2 } — no other register is nameable
  (the W04 ControlId precedent, scoped to this package).
Preconditions / postconditions: executing at EL2 (phase precondition);
  SCTLR read-back immediately after its write (the W04 read-back
  posture); the barrier wrappers exist so ordering intent is visible at
  call sites.
State and ownership change: the written registers; architectural TLB/I-
  cache state.
Concurrency/allocation context: boot context; no allocation.
Errors and failure guarantee: writes cannot fail; mismatch -> the
  transition's failure route with the register identity.
Security/authorization checks: none; translation setup is the
  containment mechanism itself.
Safety justification (P0 unsafe inventory): the compiler cannot type
  `msr`/`mrs`/`tlbi`/`ic`/`dsb`/`isb`; the closed register set and the
  contracted call sites restrict every access; each primitive carries
  its SAFETY note.
Validation: W08-DV03 (sequence review); boundary walk in W08-DV05 —
  the inventory contains exactly these primitives.
```

## 7. `Stage1Error` and the fatal route carrier

```text
Name and stability: Stage1Error { step: Stage1Step, detail: &'static str
  } with fn fail_stage1(e: Stage1Error) -> !; internal; stable within P1.
  Stage1Step enumerates { Premise, Build, Verify, Program, Enable,
  PostVerify }; detail is one of the recorded step-reason statics.
Purpose and caller: carry any transition failure to W09's `fail_phase`
  with `stage1` attribution. Caller: premise assertions, build, verify,
  programming read-backs, post-MMU verification.
Inputs / outputs: step + static reason; the route diverges.
Preconditions / postconditions: called only inside the `stage1` phase
  body; never returns (W09 `fail_phase(Stage1, ..)` -> W07's
  `report_fatal_phase`).
State and ownership change: none of its own.
Concurrency/allocation context: boot context; no allocation.
Errors and failure guarantee: terminal; no retry (W09 T3); the reason
  statics are the transition-state vocabulary W09's matrix names for the
  `stage1` row ("full fatal report including transition state").
Security/authorization checks: none; static labels and step identity
  only.
Logic: route via fail_phase(Stage1, reason) with the detail attached.
Validation: W08-DV03/DV06; W11's NC5 expectations cite the vocabulary.
```
