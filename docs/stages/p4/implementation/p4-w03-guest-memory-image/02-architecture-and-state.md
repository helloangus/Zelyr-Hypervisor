# P4-W03 Architecture, Objects, and State Model

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W03 detailed design](README.md).

## 1. Logical modules

| Module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| `gm-layout` | The temporary P4 IPA-layout record: constants and their validation (version tag, alignment, non-overlap, containment in the P2-declared reference RAM) | the record values | P2 boot-map facts (M7) | `GuestLayout` value object | runtime memory decisions; mapping operations |
| `gm-ram` | Guest RAM object: allocate, zero, expose write views, boot-info write, release | page run, lifecycle state, size | allocator, layout | `GuestRam`, `MappingGrant`s, `GuestRamView` | mapping into Stage-2 (W02), image contents |
| `gm-image` | Image representation and validation: view type, bounds checks, route convention (flat/base-entry) | none persistent (view over build-provided bytes) | embedded byte slice (M5) | `GuestImage`, validated copy plan | embedding mechanism itself (build governance), scenario semantics (W05) |
| `gm-loader` | The load procedure: validate → boot-info → copy; deterministic order; event emission | none (operates on `GuestRam`) | `GuestRam`, `GuestImage`, scenario id | loaded-and-initialized Guest input set for W04 | vCPU construction (W04), Stage-2 activation (W02) |

Layering: all modules are Core-side (no architecture registers here); the
only Arch coupling is the HVA write path permitted by the Host Stage-1
contract (M4). No board/SoC/QEMU names; layout defaults cite the P2-declared
reference facts rather than hardcoding QEMU folklore.

## 2. Core objects and ownership

### 2.1 `GuestLayout` (value object, module `gm-layout`)

- Versioned constants (layout version tag; changing values is a version bump
  and a cross-consumer review event): `RAM_BASE_IPA` (`GuestPhysAddr`),
  `RAM_SIZE` (`ByteLen`), `IMAGE_LOAD_IPA`, `STACK_TOP_IPA`, `STACK_SIZE`,
  `BOOT_INFO_IPA`, `CONSOLE_PAGE_IPA`, plus derived derived-invariants
  (image + stack + boot-info all inside RAM, pairwise disjoint, alignment ≥
  page).
- Owner: this package; consumers: W02 (mapping requests), W04 (entry/stack),
  W05 (Guest-side expectations).
- Non-responsibility: it does not describe any future machine layout
  (W01 A4).

### 2.2 `GuestRam` (module `gm-ram`)

- **Owned state:** the allocated page run (HPA range), lifecycle state, the
  Guest-use accounting marks (M3).
- **Immutable after construct:** base HPA, size.
- **Not owned:** mapping state in Stage-2 (W02's ledger), image bytes
  (build-owned), the vCPU (W04).
- **Destruction:** `release` returns pages to the allocator and clears
  accounting marks; illegal only after Stage-2 mappings over the region are
  destroyed (D8 sequencing; asserted by parameter — the caller passes
  evidence of unmap, enforced by API shape per
  [03 §3.6](03-code-contracts-guest-memory.md)).

### 2.3 `GuestImage` (module `gm-image`)

- Non-owning view `{ bytes: &[u8] }` over the build-embedded blob with
  identity metadata (build identity from the P0 version baseline for
  diagnostics).
- Validation is separate from representation: a `GuestImage` may exist
  unvalidated (for diagnostics); loading uses only a `ValidatedImagePlan`.

### 2.4 `BootInfo` (module `gm-loader`)

- Fixed-layout, little-endian, explicit-width block written into Guest RAM
  (P4 test convention D4). Representation is explicit per field
  (Coding Guidelines: never serialize raw Rust struct memory as a contract).
- Written once by the loader; read-only to the Guest after validation; not
  modified again by EL2 during the run.

## 3. Guest RAM lifecycle state machine

```text
            allocate()            init+load()            release()
  [absent] -----------> Allocated(Zeroed) ----------> Loaded ------> Released
                             |                          |
                             |     (mapping grants)     |
                             +------> Mapped-in-S2 ----+  (W02 destroy first,
                                     (W02 ledger)        then release)
```

Rules:

- `Allocated(Zeroed)` is the deterministic base state: after allocation, the
  whole region is zero-filled before anything else (D6). Reinitialization for
  a same-session repeat returns the region to exactly this state (the
  re-init entry re-zeroes; W07 consumes this).
- `Loaded` is reached only through the loader's full validation+copy
  procedure; partial loads are impossible (all-or-nothing contract).
- Mapping into Stage-2 is *not* a state of `GuestRam`; it is W02 ledger state
  over grants produced here. The sequencing duty (D8) is recorded on both
  sides: W02 destroy must precede `release`.
- `Released` is terminal; use of a released `GuestRam` is a type-level
  impossibility (consuming move) plus an accounting check.

No ADR VM-lifecycle states are implemented here; this machine is the memory
object's own lifecycle, deliberately narrower than the future VM `Loading`
stages (P10+ owns those).

## 4. Sequencing and integration contract

Construction order for one P4 Guest run (consumers shown in parentheses):

```text
1. create address space            (W02)
2. allocate + zero Guest RAM       (W03)
3. produce mapping grants for RAM, console page, and code-execution view
                                   (W03 -> W02 map)
4. validate image plan             (W03)
5. write boot-info + copy image    (W03)   [Guest RAM now deterministic]
6. construct vCPU from layout + scenario id (W04)
7. activate space; enter Guest     (W02/W04)
...
N. stop vCPU                       (W04)
N+1. destroy address space         (W02)   [before any page release]
N+2. release Guest RAM             (W03)
```

Same-session repeat (W07's P4-V10 scenario): repeat steps 2–7 with a fresh
`GuestRam` (or a verified re-init to the `Allocated(Zeroed)` base); no state
may leak between iterations through this package's objects. Which repeat form
the repeatability design selects is W07's; W03's duty is that both forms are
supportable (fresh allocation and in-place re-init).

## 5. Deterministic-initialization contract

- Order is fixed: zero whole region → boot-info block → image bytes at
  `IMAGE_LOAD_IPA`. Nothing else writes Guest RAM during construction.
- After a load, every byte of Guest RAM has a defined value: zeros except the
  boot-info block and the image span. Reads-before-writes by the Guest
  therefore observe defined data (supports VG-002-class determinism).
- The boot-info block's content is a pure function of (layout, image plan,
  scenario id) — no timestamps, no randomness, no addresses that vary run to
  run (P4-V03 determinism; W01 A8 evidence conventions).
- The image bytes themselves are build-deterministic inputs; W03 does not
  transform them (no relocation processing — that would be an ELF-property).

## 6. Concurrency model

- All W03 operations run during Guest setup/teardown, before the Guest runs
  and after it stops; no Guest execution is concurrent with loader writes
  (W04's run-loop ownership).
- The allocator's own locking (P3-era semantics, assumed) covers allocation;
  `GuestRam` methods take `&mut self`, so object-level races are impossible
  by construction in P4's single-setup-thread model.
- The write path (D7) may run with interrupts masked for the duration of the
  bounded copy (copy size ≤ layout image bound), or unmasked if the copy is
  restartable — P4 fixes the simpler masked variant and records the latency
  non-goal; a preemption-friendly design is later-stage work.
- AP behavior during setup: P3-established idle semantics; no cross-CPU
  invalidation is needed because Stage-2 mappings are created before
  activation and destroyed after deactivation (W02 sequencing).

## 7. Telemetry points (W03-scope)

Routed through the P0 baseline (W01 A8): `gm.ram.allocate`, `gm.ram.release`,
`gm.load.plan`, `gm.load.copy`, `gm.bootinfo.write`, each carrying region
identifiers and sizes, never Guest data content. W07 consumes counts for
repeatability; W06 can correlate fault IPAs with layout constants through the
record.
