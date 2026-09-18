# P2-W04 Validation, Error Model, and Handoff Design

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W04 detailed design](README.md).

## 1. Scope of validation for this package

W04's own evidence is host-side property and unit testing over injected
span tables and buffer-backed metadata storage — the entire algorithm is
hardware-independent by design ([01 §7](01-scope-and-foundations.md)).
QEMU evidence (allocator live on the reference platform, accounting across
repeated boots) is W09; stress and negative regression harnesses are W08
(consuming this package's typed errors as fixtures). Planning those here
supplies none of them.

## 2. Error, security, and observability model

- **Error model.** Typed allocate errors (`OrderTooLarge`,
  `OutOfFrames{order, region_stats}`), typed free errors
  (`UnmanagedFree`, `DoubleFree`, `BadFreeRange`), init errors
  (`AllocatorInitError{...}`), all stateless on failure
  ([02 §7](02-architecture-and-state.md)). Boot-phase policy: allocator
  errors are fatal stops with the typed diagnostic — resource exhaustion
  and invalid operation are distinguished per P0-W14. No guest exists to
  cause allocator inputs in P2; the guest-facing fault boundary is a later
  stage's concern and is not implemented here.
- **Security model.** The hard gate is structural (domain subtraction
  before first free list; init audit; table authority; no address-directed
  allocation). The allocator never reads or writes managed frame contents.
  Residual limits, recorded for W10: the order-mismatch detection limit
  ([02 §5](02-architecture-and-state.md)), and inherited
  under-declaration of firmware reservations (W03's limit).
- **Observability.** `AllocationStats` is live-derived and conserved;
  boot emits one "page allocator ready" marker with totals (managed/free/
  reserved frames per region); errors render with region statistics so a
  boot log explains exhaustion. W06 renders stats; W09 compares totals
  across repeated boots (determinism of totals given identical platform
  description).

## 3. Validation matrix

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W04-DV01 → P2-V06 (P2-E07) | Bootstrap + table | Host tests (joint with W03 seal) | Plan/seal/init sequence; seeded seal/domain/conservation mismatches | Init accepts only consistent seal; every mismatch class fatal with its diagnostic | Metadata protected before allocation; not W03's builder |
| W04-DV02 → P2-V06 (P2-E01) | Allocation | Host property test | Random valid allocate sequences over multi-span fixtures | Every returned range inside managed domain, order-aligned; table says Allocated | Allocation correctness; not real-memory behavior |
| W04-DV03 → P2-V06 (P2-E02) | Release | Host property test | Allocate/free in varied orders; re-allocate after free | Freed frames reusable; state identical to pre-allocation baseline at cycle end | Release correctness; not fragmentation quality targets |
| W04-DV04 → P2-V06 (P2-E03) | Alignment/order discipline | Host unit tests | Misaligned counts, order > MAX_ORDER, order-aligned range checks | Typed errors; no misaligned range ever returned or accepted | Discipline enforcement; not large-page policy |
| W04-DV05 → P2-V06 (P2-E04) | Multi-region | Host property test | Fixtures with 1/2/8 spans; allocations spanning region choices | Per-region isolation; no cross-region coalescing or block | Multi-region support; not NUMA policy |
| W04-DV06 → P2-V06 (P2-E05) | OOM | Host tests | Exhaust small pools; exact-count near limits | `OutOfFrames` with region stats; state unchanged (stats equal) | Explicit OOM; not performance under pressure |
| W04-DV07 → P2-V06 (P2-E06) | Debug detection | Host unit tests | Unmanaged/duplicate/unaligned/partial frees; metadata-range free | Each defect yields its typed error, state unchanged | Detector correctness; documented order-mismatch limit applies |
| W04-DV08 → P2-V06/P2-V10 (P2-E07, hard gate) | Never-protected property | Host property/soak test | Long randomized valid+invalid sequences with audit after each | Zero protected frames in any returned range; invariants hold after every op | The P2 safety gate at allocator level; not QEMU or hardware |
| W04-DV09 → P2-V06 (P2-E08) | Accounting | Host tests + property loop | stats() after every operation vs table recount | Conservation exact; fast-copy divergence detection fires when injected | Accounting integrity; not W06 rendering |
| W04-DV10 → W04 closure | Consumer walkthrough | Design review | Read as W05 (backing sufficiency), W06 (stats), W08 (error fixtures), W09 (totals), P3/P4-via-W10 (boundaries) | Each consumer can proceed without new W04 work | Handoff readiness; not consumer implementations |

Evidence statuses are passed / failed / blocked / not run with command,
input, environment, timestamp. Host validation does not prove QEMU
integration, real-metadata-window behavior, determinism on real hardware,
or any P3 concurrency property.

## 4. Handoff checklist

Before handing W04 to review, provide:

- changed-module list; confirmation of zero `unsafe` outside the injected
  window adapter (if any), zero locks/atomics, zero board names, zero
  writes to managed frames, zero allocation outside metadata storage;
- W04-DV01–DV10 evidence statuses with not-run entries (QEMU integration
  → W09; stress harness → W08; real window → P1 integration; SMP → P3);
- confirmed consumer readiness: W05 (frame API + exhaustion semantics),
  W06 (`AllocationStats`), W08 (typed errors as regression fixtures),
  W09 (accounting totals), W10 (concurrency boundary, limits, extension
  points for P3/P4);
- open items recorded, not resolved: P0/P1/W03 assumed contracts,
  physical placement (P0-W03), algorithm freeze reconciliation with ADR
  §18, order-tag extension, poisoning, >`MAX_ORDER` spans;
- explicit statement that W04 adds no public API beyond
  [03](03-code-contracts-pagealloc.md) and that W05's GlobalAlloc surface
  is that package's design.
