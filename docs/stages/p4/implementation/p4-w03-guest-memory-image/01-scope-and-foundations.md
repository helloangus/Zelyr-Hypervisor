# P4-W03 Scope, Foundations, and Resolved Decisions

**Status:** Approved detailed design (project owner) v0.2, 2026-10-02; implementation and runtime evidence are not claimed.
**Parent:** [P4-W03 detailed design](README.md).

## 1. Scope classification detail

### Required (P4-B01–B05)

- Guest RAM object: allocation from the P2 page allocator, bounded size,
  adoption and region retention through P2-W12, deterministic
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

- General machine layout and relocation at load time (P8); temporary fixed
  P4 IPA already translates allocator-selected HPA in this design.
- Additional image formats (ELF with program headers parsed host-side or in a
  later loader design) (re-entry: first non-Validation Guest boot design).
- A configurable Guest RAM size source beyond the layout record (re-entry:
  VM configuration, P11+).
- Richer Guest policies such as COW/sharing; common region retention and pins
  already belong to W12 and must not be reimplemented here.

### Out of Scope

Implementation of the common object model (P2-W12 owns it; W03 consumes it), filesystems and complex loaders in EL2 (ADR-029/ADR-050),
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
| M3 | W12 retains backing and every mapping region until exact backend completion | P2-W12 detailed design, ADR-062 option A | GuestRam holds control capability; take_back returns original W04 handle only when all views end | Missing producer blocks real mapping/release; no degraded raw alloc/free path |
| M4 | Explicit writable Host views of allocated Guest-backing frames with retained ownership/borrow lifetime | [P2 mapping-owner decision](../../../p2/implementation/p2-contract-reconciliation-record.md#p2-acr-02--ordinary-allocated-frame-host-mapping-aud-003); `P2-HOST-MAP`; W01 R04 | loader writes through the assigned P2 producer's HVA views, then releases them according to its contract | missing producer/design/evidence blocks loading; P1 bootstrap coverage and physical allocation alone are insufficient |
| M5 | Build embeds a byte blob and exposes it as a static byte slice with known length (mechanism per the workspace/target design) | P0-W03/W04 build governance; W01 R18 | `GuestImage` view construction | embedding mechanism unavailable → the route decision D3 must be revisited in design before code (route change is a design change, not a code workaround) |
| M6 | Logging/trace baseline for load/verify events | P0-W12/W13; W01 A8 | `gm.*` events emitted through it | degrade per M7 of W02's table; no ad-hoc prints |
| M7 | P2 boot map declares the QEMU reference RAM layout facts W03's defaults are sized against (RAM base/size) | P2-W02/W03; W01 R07/R08 | default layout values fit the reference configuration | mismatch → layout values are review findings, not silent constants |

## 3. Authority analysis for contested areas

- **P2-ACR-01:** ADR-062 owner option A assigns the common foundation to W12.
  GuestRam is its Guest-specific adapter, not a competing backing owner.
  Formal ADR integration and production evidence remain separate gates.
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
| D1 | One contiguous Guest RAM region per Guest; size fixed by the layout record for P4; allocated as a run of 4 KiB pages from the P2 allocator | smallest bounded model that satisfies "bounded Guest RAM"; contiguity simplifies bounded view mapping without inventing a general allocator API | task book P4-B01; ADR-014 |
| D2 | Fixed temporary Guest IPA translates allocator-selected HPA; no identity-placement requirement on W04. This replaces the old D2 implementation choice and supports independently backed W10 spaces | Preserves bounded initialization and single ownership | Task-book §8 implementation choice; W12 and W11 contracts |
| D3 | Image route: flat binary, entry at base, embedded in the Hypervisor image at build time, delivered as a validated static byte slice; no parser in EL2; maximum image size bounded by the layout record | ADR-050 (no file-system/format stacks in EL2); ADR-029 (boot-time loading only); a flat copy has one validation surface and deterministic behavior; ELF parsing adds TCB for zero P4 need | task book §1 Reserved/§8; ADR-029/050 |
| D4 | Boot-info block: fixed-offset, magic+version-tagged parameter block written by the loader at a fixed Guest IPA; carries layout facts (ram base/size, image entry, image size, console page, scenario id echo); Guest validates defensively | gives the Guest its parameters without baking Host-side constants into Guest code beyond the block location; demonstrates a defensive parsing pattern that P5's guest-copy work will generalize | task book P4-B02/B05; ADR-007 discipline applied to a semi-trusted block |
| D5 | Console page: layout reserves the reference PL011 frame as a Device, RW, XN Guest mapping; EL2 desists from console writes during the Guest run segment (coordination duty stated for W04's run loop); Guest markers and Host diagnostics interleave only at exit boundaries | the only in-scope Guest output path; keeps EL2 free of any console service; interleaving rules are deterministic because P4 runs one Guest path and AP idle loops do not print | ADR-003 (QEMU reference); task book exit condition (Hello from EL1); P4 has no device-model scope (P8+) |
| D6 | Deterministic initialization: allocation → zero-fill whole region → write boot-info → copy image → (post-condition) documented residual state; no uninitialized byte ever remains | P4-V03 determinism requirement; also removes a class of Guest-visible nondeterminism that would pollute repeatability evidence (W07) | task book P4-B05 |
| D7 | Copy through W11 scoped MaybeUninit byte access, then complete architecture instruction visibility and retire Host writer before Guest mapping; no local raw-pointer writer | Preserves bounded initialization and single ownership | Task-book §8 implementation choice; W12 and W11 contracts |
| D8 | W12 take_back enforces no remaining view/pin; completed W02 retirement precedes release. A raw Stage2Released assertion is not sufficient | Preserves bounded initialization and single ownership | Task-book §8 implementation choice; W12 and W11 contracts |
