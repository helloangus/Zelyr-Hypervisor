# P2-W03 runtime completion record

**Status:** Implementation record; closure evidence is in the linked verification.
**Scope:** W03-owned storage, lifetimes, boot wiring and stack safety.
**Version:** v0.1
**Owner/change context:** User-requested completion correction, 2026-09-27.
**Supersedes:** PR #70's deferral of W03 runtime foundations to W04/W09.

## Authority and ownership correction

The user explicitly required completing W03 before handing its foundation to
W04. The [runtime design amendment](p2-w03-boot-memory-map-ownership/06-runtime-storage-and-handoff.md)
implements that requirement under the existing W03 plan. The original
[host implementation record](p2-w03-boot-memory-map-ownership-record.md) and
its tests remain historical evidence, but host-only completion was insufficient.
No stage task book or accepted ADR is changed.

## Implementation

- `platform/bootmap/mod.rs`, `build.rs`, `seal.rs`: `MapStorage`, explicit
  Fresh/Building/Draft/Sealing/Sealed/Failed lifecycle and `draft_in` support
  exclusive borrowed backing. The target's draft/sealed handles are 8/96 bytes.
  Only owning host convenience paths return arrays by value. Failed or dropped
  drafts cannot reopen the same storage; sealed handles expose shared queries.
- Remove the 672-element clip cache. The same source-major/bank-major clip
  records are derived as bounded intersections of retained immutable banks and
  sources. No provenance or accounting is dropped. Storage falls from 40,768
  to 13,888 bytes; in-place operations do not copy that backing. Sort at most
  eight banks with bounded insertion sort, avoiding a general recursive sorter.
- `boot/p2.rs`: a separate non-inlined, non-returning `memory_map` owns one
  inline-const-initialized storage after W02 returns. It checks placement
  inside the image, builds from actual discovery, seals, reports conservation
  and retains the DTB window, validated DTB, facts and map until restart.
  Every map failure takes a bounded diagnostic followed by terminal idle.
- `arch/aarch64/stage1/dtb.rs`: expose only the existing checked image extent
  already retained by the DTB owner. No new linker symbols, physical access,
  mapping, unsafe segment or widened safety precondition is introduced.
- Host tests cover owning/borrowed equivalence, exact clip views, failed/dropped
  storage and no second publication. Compile probes reject premature backing
  destruction and concurrent mutable access as well as the earlier typestate
  misuse. Target recipes capture normal/error paths, accounting, artifact
  identity, disassembly frames and full single-instruction W03 SP trajectories.

## Live lifecycle and W04 seam

The map storage lives in the existing P1 boot stack, so the authoritative
image extent already protects it. W03 requires no metadata carve-out or writable
RAM aperture to house its own records. P1's 64 KiB stack and mappings are
unchanged; no global map, heap allocation, lock or singleton is introduced.

Current boot uses an empty metadata list because there is no allocator in the
binary, and explicitly reports `metadata=0 allocator=absent`. W04 owns the
future metadata layout, placement and writable mapping, and inserts its planner
at the existing draft-to-seal call site. It must use the borrowed sealed map
as allocation authority, not invoke the owning host convenience constructor or
try to re-seal the current result. No W03-owned storage foundation remains
assigned to W04 or W09. W09 still owns its full regression matrix.

## Interfaces, safety and delivery

New stage-local Rust surface: MapStorage/new/default, generic map backing,
`draft_in`, StorageUnavailable, and the crate-internal image getter. `clips()`
now yields value records rather than cached references. Existing owning calls
remain source-compatible. No external ABI, dependency, toolchain, feature,
new unsafe or P1 stack-size/layout change. No new TODO/FIXME or unresolved
W03 design conflict remains. P2-ACR-01 remains open only for memory objects;
W04-LAYOUT/W04-MAP remain W04 work. No allocator/Guest/SMP mechanism is added.

Actual checks, trace limits, earlier failed test setups and the corrected
completion claim are in the [runtime verification](../verification/p2-w03-runtime-verification.md).
