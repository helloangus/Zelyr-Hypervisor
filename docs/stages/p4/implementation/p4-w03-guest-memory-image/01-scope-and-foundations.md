# P4-W03 Scope, Foundations, and Resolved Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W03 detailed design](README.md).

## 1. Scope classification detail

### Required (P4-B01–B05)

- Guest RAM object: allocation from the P2 page allocator, bounded size,
  ownership marking through the P2 accounting extension point, deterministic
  initialization, explicit release.
- Temporary IPA layout record: Guest RAM base/size, image load IPA, stack
  region, boot-info IPA, console device page — versioned, documented as a P4
  test contract.
- Image route: flat binary with base-entry convention; validated byte-slice
  delivery; build-time embedding.
- Load-time validation: non-empty image, size bound vs Guest RAM, alignment,
  destination-bounds proof before the first byte is copied, overflow-checked
  arithmetic throughout.
- Deterministic initialization contract and its repeat behavior.

### Reserved (must not be precluded; not implemented in P4)

- Non-identity IPA placement and relocation at load time (re-entry: P8
  machine-layout design).
- Additional image formats (ELF with program headers parsed host-side or in a
  later loader design) (re-entry: first non-Validation Guest boot design).
- A configurable Guest RAM size source beyond the layout record (re-entry:
  VM configuration, P11+).
- Richer ownership states for Guest pages (shared, pinned) (re-entry: ADR
  object-model resolution of P2-ACR-01).

### Out of Scope

The `MemoryObject`/`MemoryRegion` ADR object model (P2-ACR-01 unresolved,
kept visible), filesystems and complex loaders in EL2 (ADR-029/ADR-050),
Guest DTB and boot protocols (P8), virtio and DMA, memory overcommit and
ballooning, snapshot formats, Linux boot, and any multi-Guest memory policy.

## 2. Assumed upstream contracts and failure boundaries

Per the [P4-W01](../p4-w01-entry-contract-reconciliation/README.md) entry
review; a divergence between assumption and delivery becomes a recorded
conflict (W01 §4), and the affected W03 step stops — W03 never patches around
an upstream change.

| ID | Assumed contract | Source | Relied-on behavior | Failure boundary if delivered differently |
|---|---|---|---|---|
| M1 | Typed addresses: `HostPhysAddr`, `GuestPhysAddr`, `PageCount` newtypes with checked arithmetic | P0-W15 plan; W01 R19 | all W03 APIs typed; no raw `usize` addresses | no typed addresses → W03 blocks (Coding-Guidelines requirement) |
| M2 | Page allocator: 4 KiB allocations from non-protected RAM, explicit OOM, contiguous-run or equivalent multi-page capability, accounting with an ownership extension point | P2-W03/W04/W10 plans; W01 R08–R12 | Guest RAM and its page set derive exclusively from this allocator; protected pages unreachable | if protected-range exclusion is not demonstrable, "RAM originates only from allocatable pages" (P4-V03) is unsound → Guest-exposure steps stop |
| M3 | Accounting extension point accepts a Guest-use state transition for allocated pages and release restores accounting | P2-W10 (ownership-extension foundation); W01 R12 | W03 marks pages as Guest RAM; release returns them cleanly | if absent: record gap in W01 list; degrade to plain alloc/free and record the limitation for W09 |
| M4 | Host Stage-1 maps all EL2-managed RAM read-write (loader needs to write Guest RAM through Host virtual addresses) | P1-W08 plan; W01 R04 | loader writes via HVA views of the allocated HPA frames | if Host mapping of the Guest RAM range is not guaranteed, W03 needs an explicit map step owned elsewhere → record dependency, do not improvise mappings |
| M5 | Build embeds a byte blob and exposes it as a static byte slice with known length (mechanism per the workspace/target design) | P0-W03/W04 build governance; W01 R18 | `GuestImage` view construction | embedding mechanism unavailable → the route decision D3 must be revisited in design before code (route change is a design change, not a code workaround) |
| M6 | Logging/trace baseline for load/verify events | P0-W12/W13; W01 A8 | `gm.*` events emitted through it | degrade per M7 of W02's table; no ad-hoc prints |
| M7 | P2 boot map declares the QEMU reference RAM layout facts W03's defaults are sized against (RAM base/size) | P2-W02/W03; W01 R07/R08 | default layout values fit the reference configuration | mismatch → layout values are review findings, not silent constants |

## 3. Authority analysis for contested areas

- **P2-ACR-01 (`ADR Required`, unresolved):** W03 is the package closest to
  the ADR `MemoryObject` concept. To avoid resolving it locally, W03 models
  Guest RAM as the stage-local `GuestRam` handle (single object, single owner,
  no sharing/pinning/DMA state) and routes any future object-model evolution
  through the Reserved re-entry point. The handle is not presented as the ADR
  object system, asserts no API stability, and P4-W09 records it as a
  temporary P4 fact.
- **Image route selection:** explicitly delegated to detailed design by the
  task book (§1 Reserved, §8 Implementation Choice). Decision D3 in §4 fixes
  it for P4 with rationale; changing it later is a design change recorded in
  the implementation record and reflected by W09, not a code-level switch.
- **Console direct map:** neither the ADR nor the task book names a Guest
  console for P4; P4 has no device model (P8+). The direct map of the
  reference console page is a stage-local test convention this design owns
  (D5), justified because the Stage-2 exit criteria require Guest-produced
  output ("Guest 打印 Hello from EL1") and no other output path exists within
  P4 scope. It is recorded as temporary, QEMU-reference-scoped, and not a
  device model.

## 4. Resolved design decisions

| ID | Decision | Rationale | Authority basis |
|---|---|---|---|
| D1 | One contiguous Guest RAM region per Guest; size fixed by the layout record for P4; allocated as a run of 4 KiB pages from the P2 allocator | smallest bounded model that satisfies "bounded Guest RAM"; contiguity simplifies mapping and identity placement without inventing a general allocator API | task book P4-B01; ADR-014 |
| D2 | Identity placement: Guest IPA of RAM == its HPA; console page mapped at its Host MMIO address as IPA | keeps the temporary layout trivially reviewable; isolation is enforced by Stage-2 mapped-range exclusivity (W02), not by address scrambling; non-identity is Reserved | task book §1 Reserved (temporary layout); W02 D-decisions |
| D3 | Image route: flat binary, entry at base, embedded in the Hypervisor image at build time, delivered as a validated static byte slice; no parser in EL2; maximum image size bounded by the layout record | ADR-050 (no file-system/format stacks in EL2); ADR-029 (boot-time loading only); a flat copy has one validation surface and deterministic behavior; ELF parsing adds TCB for zero P4 need | task book §1 Reserved/§8; ADR-029/050 |
| D4 | Boot-info block: fixed-offset, magic+version-tagged parameter block written by the loader at a fixed Guest IPA; carries layout facts (ram base/size, image entry, image size, console page, scenario id echo); Guest validates defensively | gives the Guest its parameters without baking Host-side constants into Guest code beyond the block location; demonstrates a defensive parsing pattern that P5's guest-copy work will generalize | task book P4-B02/B05; ADR-007 discipline applied to a semi-trusted block |
| D5 | Console page: layout reserves the reference PL011 frame as a Device, RW, XN Guest mapping; EL2 desists from console writes during the Guest run segment (coordination duty stated for W04's run loop); Guest markers and Host diagnostics interleave only at exit boundaries | the only in-scope Guest output path; keeps EL2 free of any console service; interleaving rules are deterministic because P4 runs one Guest path and AP idle loops do not print | ADR-003 (QEMU reference); task book exit condition (Hello from EL1); P4 has no device-model scope (P8+) |
| D6 | Deterministic initialization: allocation → zero-fill whole region → write boot-info → copy image → (post-condition) documented residual state; no uninitialized byte ever remains | P4-V03 determinism requirement; also removes a class of Guest-visible nondeterminism that would pollute repeatability evidence (W07) | task book P4-B05 |
| D7 | Copy path: host-virtual write views into the allocated frames; the `unsafe` surface is one bounded, reviewed writer; all destination arithmetic is checked and proven in-bounds before the first store | Host Stage-1 already provides HVA access (M4); a single audited writer keeps the unsafe inventory minimal | ADR-006; Coding Guidelines unsafe rules |
| D8 | Guest RAM release ordering is part of the object contract: address-space unmap/destroy (W02) must precede `GuestRam::release`; the loader-side sequencing is explicit and verified by review, not discovered at runtime | prevents free-while-mapped windows; keeps each object's owner clear (Plan Agent guardrail) | ADR §19 ownership invariant |
