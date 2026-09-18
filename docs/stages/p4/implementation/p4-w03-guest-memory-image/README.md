# P4-W03 Guest Memory and Image Construction — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** Bounded, deterministically initialized Guest RAM built from safe
Host memory, the temporary Validation Guest IPA-layout test contract, and the
repeatable image load route, as required by
[P4-W03](../../plans/p4-w03-guest-memory-image.md) (P4-B01–B05).  
**Owner/change context:** P4-W03 implementation handoff; this design owns the
Guest RAM object, the temporary layout record, and the image-route decision
for P4.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P4-W03. It converts the bounded
work-package plan into the Guest-memory object model, the load-time validation
contracts, and the construction workflow, with pseudocode rather than
production code. It deliberately does **not** design the Stage-2 mapper
([P4-W02](../p4-w02-stage2-address-space/README.md) owns map/unmap/protect and
this design only defines what it hands over), the vCPU initial state
([P4-W04](../p4-w04-vcpu-entry-exit/README.md) consumes this package's entry/
stack/image inputs), the Validation Guest scenario contents
([P4-W05](../p4-w05-validation-guest/README.md) owns the scenario set and this
design fixes only the loadable-image convention), the P8 machine layout, Guest
DTB, dynamic memory policy, shared memory, ballooning, or snapshot formats.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

- [01-scope-and-foundations.md](01-scope-and-foundations.md) — ledger, assumed
  upstream contracts with failure boundaries, scope classification, and the
  resolved design decisions (image route, identity placement, boot-info
  block, console page) with their authority. Load first.
- [02-architecture-and-state.md](02-architecture-and-state.md) — logical
  modules, the Guest RAM lifecycle state machine, ownership and sequencing
  with Stage-2 and the vCPU consumer, and the deterministic-initialization
  contract. Load for architecture and lifecycle work.
- [03-code-contracts-guest-memory.md](03-code-contracts-guest-memory.md) —
  full function/type contracts with pseudocode: layout record, Guest RAM
  allocation/initialization, boot-info write, image validation and copy, and
  the error model. Load for the code-contract work area.
- [04-implementation-workflow.md](04-implementation-workflow.md) — ordered
  implementation steps with acceptance and failure handling.
- [05-validation-and-handoff.md](05-validation-and-handoff.md) — validation
  matrix (P4-V03), error/security/observability model, and handoff checklist.

Before editing, the agent must also follow the Coding Guidelines preflight
(repository `AGENTS.md`, documentation index, ADR baseline, P4 task book, the
P4-W03 plan, and the [P4-W01](../p4-w01-entry-contract-reconciliation/README.md)
entry review result). This document is a proposed design; it contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is [ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md)
→ [P4 task book](../../task-book-v0.1.md) →
[P4-W03 plan](../../plans/p4-w03-guest-memory-image.md) → this design →
Coding Guidelines. Binding constraints include:

- ADR §6/ADR-018: memory ownership is modeled before mapping; Guest RAM comes
  from owned, allocator-derived pages. ADR-014: EL2 dynamic allocation is
  legitimate backing for this.
- ADR-007/ADR §19: all image and range inputs are validated (range, overflow,
  empty) before use; a bad image must never overwrite protected memory.
- ADR-029/ADR-050: EL2 implements no filesystem or complex loader; the P4
  image route must stay a bounded in-memory copy.
- Task book §1 Required/Reserved: Guest RAM from P2-allocatable memory; the
  temporary IPA layout is a P4 fact, "neither a permanent one-VM architecture
  nor a frozen `rusthv-arm-virt-v1` machine ABI"; "an ELF or flat-binary
  test-image route may be selected during detailed design" — this design
  selects it.
- Task book §1 Out of scope: final P8 machine layout, Linux/firmware boot,
  Guest DTB, dynamic memory policy, shared memory, ballooning, snapshot
  formats.

Classification: the Guest RAM object, layout record, boot-info block, image
validation/copy contracts ([03](03-code-contracts-guest-memory.md)) and the
deterministic-initialization contract ([02 §5](02-architecture-and-state.md))
are **Required** for P4-B01–B05. Non-identity IPA placement, image formats
beyond the selected route, a configurable Guest RAM size policy, and any
ownership-accounting sophistication beyond the P2 extension point are
**Reserved**. The `MemoryObject`/`MemoryRegion` ADR-level object model,
filesystems, ELF loaders in EL2, Guest DTB, virtio, DMA, and the P8 machine
ABI are **Out of Scope** (P2-ACR-01 remains `ADR Required` and unresolved;
this design keeps its memory handles stage-local).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| P4-B01 Guest RAM originates only from P2-allocatable pages | [contracts](03-code-contracts-guest-memory.md) §3.1; [foundations](01-scope-and-foundations.md) M2 | P4-V03 ("RAM originates only from allocatable pages") |
| P4-B02 temporary IPA-layout facts recorded as a test contract | [contracts](03-code-contracts-guest-memory.md) §2; [foundations](01-scope-and-foundations.md) D2 | recorded, cited, non-ABI (P4-V03 review row; W09 records facts) |
| P4-B03 repeatable image route with validation | [foundations](01-scope-and-foundations.md) D3; [contracts](03-code-contracts-guest-memory.md) §3.4 | P4-V03 (forbidden-overwrite, empty/invalid rejections) |
| P4-B04 range/entry/overflow/empty validation | [contracts](03-code-contracts-guest-memory.md) §3.4, §3.5 | P4-V03 negative evidence |
| P4-B05 deterministic initial memory state | [architecture](02-architecture-and-state.md) §5; [contracts](03-code-contracts-guest-memory.md) §3.2 | P4-V03 determinism evidence |
| No forbidden Host range exposure via mapping or copy | [architecture](02-architecture-and-state.md) §4; [contracts](03-code-contracts-guest-memory.md) §3.4 | P4-V03; joint with W02 isolation evidence |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p4-implementation-designs` at
`4e631ee`): documentation-only repository; no workspace, crate, allocator,
mapper, or guest. `guests/validation-aarch64/` and `hypervisor/src/` contain
only `.gitkeep`. The P2 allocator and boot-memory map are planned, not
delivered (W01 rows R08–R12). Every foundation below is an assumed contract
with a recorded failure boundary, not an observable artifact.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Guest RAM originates only from allocatable pages | No allocator exists (P2 planned) | The P2 page allocator as sole page source (assumed M2) plus W03's allocation discipline | "Only from allocatable pages" is meaningless without the allocator's protected-range exclusion as the guarantee source | P2-W04 plan (assumed); W03 consumes | P4-V03; upstream P2-V06 when delivered |
| Temporary IPA layout exists as a test contract | No layout values recorded anywhere | The P4 layout record ([03 §2](03-code-contracts-guest-memory.md)) with versioned values | Consumers (W02 mapping, W04 entry, W05 Guest) need one shared source of truth | W03 (this design) | layout review; cross-consumer consistency checks |
| One repeatable image route | Task book leaves ELF vs flat binary open | Route decision D3 (flat binary, embedded) and the image validation contract | "Repeatable" requires a fixed representation with fixed validation | W03 (Implementation Choice per task book §8) | image-route evidence in P4-V03 |
| Forbidden overwrite impossible | Nothing exists | Destination-bounds pre-validation and copy contract ([03 §3.4](03-code-contracts-guest-memory.md)) | A loader without proven destination bounds could corrupt Host memory | W03 | P4-V03 negative evidence |
| Deterministic initialization | Nothing exists | Init contract: zero-fill, boot-info write, image copy in fixed order ([02 §5](02-architecture-and-state.md)) | P4-V03 requires deterministic initialization across declared repeat scenarios | W03; W07 repeats it | P4-V03 determinism evidence |
| Entry/stack inputs for W04 | Nothing exists | Layout-derived entry IPA, stack top, boot-info IPA in the record | W04's reproducible vCPU construction is a pure function of these inputs | W03 provides; W04 consumes | consumer design consistency (W04 README mapping) |

No row requires a decision outside this design's authority; the route and
layout choices are Implementation Choices the task book explicitly assigns to
detailed design.

## Resolved design decisions and their authority

Summarized; full rationale in [01 §4](01-scope-and-foundations.md):

1. **Guest RAM shape:** one bounded, contiguous, allocator-derived RAM region
   per Guest for P4, with size taken from the layout record. Task book §1
   (temporary one-Guest scope; not a permanent architecture).
2. **Identity placement:** the temporary layout maps Guest IPA == HPA offsets
   (Guest RAM is mapped at its Host physical base). Non-identity relocation is
   Reserved. Isolation still holds because Stage-2 maps only the Guest ranges
   (W02), so any Host range outside them faults regardless of identity.
3. **Image route:** flat binary, entry at image base, built as its own
   artifact and embedded into the Hypervisor image at build time, delivered to
   the loader as a validated byte-slice view. No ELF parser exists in EL2.
   Task book §8 (Implementation Choice); ADR-050/ADR-029 spirit.
4. **Boot-info block:** the loader writes a small, versioned, magic-tagged
   parameter block into Guest RAM at a fixed IPA; the Guest validates it
   defensively. This is a P4 test convention, not a boot ABI (P8 owns boot
   protocols).
5. **Console page:** the layout reserves one device-memory page for the
   reference PL011 console, mapped into the Guest by W03's mapping request
   (Device type, XN, RW) so the Guest can emit markers without any EL2
   service; EL2 does not write the console while the Guest segment runs
   (handoff coordinated with W04's run loop). Temporary QEMU-reference
   convention, not a machine ABI.
6. **Ownership boundary:** W03 owns Guest RAM pages from allocation until
   release; it hands *mapping grants* ([P4-W02](../p4-w02-stage2-address-space/README.md)
   D8) to Stage-2 and *parameters* (entry, stack top, boot info) to W04. It
   does not define the ADR-level memory-object model (P2-ACR-01 stays
   visible and unresolved).

## Work breakdown and loading order

1. Load [01-scope-and-foundations.md](01-scope-and-foundations.md): ledger,
   assumed contracts (M-series), and decisions D1–D7.
2. Load [02-architecture-and-state.md](02-architecture-and-state.md) for
   module boundaries, the Guest RAM state machine, the sequencing contract
   with Stage-2 destroy and vCPU stop, and determinism rules.
3. Implement per [04-implementation-workflow.md](04-implementation-workflow.md),
   loading [03-code-contracts-guest-memory.md](03-code-contracts-guest-memory.md)
   for each contract; the copy path's `unsafe` carry type has mandatory
   SAFETY requirements.
4. Record validation in
   `../../verification/p4-w03-guest-memory-image-verification.md` and
   implementation facts in `../p4-w03-guest-memory-image-record.md` only when
   work starts; no completion claims.

## Explicitly excluded interfaces

Not designed or authorized by W03: the Stage-2 mapper API (W02's), the vCPU
context construction (W04's), the Guest scenario set and its marker protocol
(W05's), any hypercall/management ABI (P5), ELF/other image parsing in EL2,
Guest DTB or PSCI virtualization (P8), shared-memory or grant tables, DMA/
IOMMU interaction, and any QEMU-conditional Core behavior. The layout values
are test contracts; no consumer may treat them as the machine ABI
(W01 assumption A4).

## Downstream handoff

Per the [plan index consumer map](../../plans/README.md):

- **P4-W04** receives validated Guest entry IPA, stack top, boot-info
  location, and scenario-parameter channel (via the boot-info block), i.e.
  the inputs vCPU construction is a pure function of
  ([03 §2](03-code-contracts-guest-memory.md), §3.3).
- **P4-W05** receives the maintained image route: the flat-binary convention,
  boot-info block format, and console-page contract its Guest builds against.
- **P4-W02** receives the mapping-request shapes (`MappingGrant` production
  per range: RAM as Normal cacheable RW/XN-per-layout; console page as
  Device RW/XN; code page R+X as the layout demands) and the sequencing duty
  (address-space destroy before Guest RAM release).
- **P4-W06/W07/W08** receive the deterministic-initialization and layout
  facts their fault correlation, repeatability, and automation rely on.
- **P4-W09** records only implemented facts: selected route, layout values as
  test contracts, limitations — never a machine-ABI freeze.
