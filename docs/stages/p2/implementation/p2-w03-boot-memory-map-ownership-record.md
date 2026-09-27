# P2-W03 boot memory map implementation record

**Status:** Implementation record; validation and closure scope are in the
[verification record](../verification/p2-w03-boot-memory-map-ownership-verification.md).
**Scope:** Allocation-free W03 map, classification, provenance and seal boundary.
**Version:** v0.1
**Owner/change context:** User-requested P2-W03 implementation, 2026-09-27.
**Supersedes:** None; records implementation of the reconciled W03 design.

This is the historical PR #70 record. The [runtime completion record](p2-w03-runtime-record.md)
supersedes its deferral of W03-owned storage and target integration.

## Authority and current baseline

The user selected W03 for implementation after the reconciled design was
merged. The [bounded plan](../plans/p2-w03-boot-memory-map-ownership.md) and
[detailed design](p2-w03-boot-memory-map-ownership/README.md) define this work.
The audited starting point was `main@2403db0` (PR #69), with W01/W02 already
implemented. The old DTB-access conflict is resolved in its owning record;
no P1 access workaround is introduced here.

## Changed modules and requirement coverage

| Artifact | Responsibility / requirement |
|---|---|
| `hypervisor/src/platform/bootmap/ranges.rs` | Checked frame/page types, byte-end validation and outward protection rounding; P2-D05 |
| `bootmap/classify.rs` | W02-only reservation intake; image/DTB/artifact protection, original-byte conflicts, duplicate equivalence and retained provenance; P2-D02–D05 |
| `bootmap/build.rs` | RAM deduplication before adjacency merge, overlap rejection, bounded endpoint partition and clip records; P2-D01/D06/D07 |
| `bootmap/seal.rs` | Consume draft, validate metadata extents, record protection, independently audit exact coverage/source sets/classes/accounting; P2-D08 |
| `bootmap/mod.rs`, `platform/mod.rs` | Immutable queries, source identities, anomaly counters and disjoint accounting; P2-G01–G03 |
| `bootmap/tests.rs` | Private boundary/unit tests; no exported fixture constructor |
| `crates/host-test-baseline/tests/p2_platform/bootmap.rs` and its parent harness | Real W01/W02 fixtures and an independent seeded frame oracle |
| `verification/p2-w03-contract-probes.py` | Compile-only target size and typestate rejection probes |

The code uses only stack/value-owned fixed arrays, `core`, existing
`PhysAddr`/`ByteSize`/`Span`, and actual W02 `Fact<T>` lists. No allocator,
arch register, board constant, physical dereference, global registry, lock,
unsafe block, external dependency or feature is introduced.

## Concrete bindings and recorded adaptations

The [Rust binding section](p2-w03-boot-memory-map-ownership/03-code-contracts-bootmap.md#9-rust-binding-for-the-w03-delivery)
records the concrete names and signature choices. Key points:

- `SourceId` includes RAM error provenance as well as protected-source IDs.
  Reservation/artifact ordinals refer to the original W02 lists. Every
  equivalent declaration retains its own ledger slot.
- `SourceSet` uses a private 128-bit bitset for at most 84 ledger indices.
  Storage limits remain 8 RAM banks, 38 initial sources, 46 metadata extents,
  184 map entries and 672 clip records. Input capacities come from current
  W02; changing them requires reviewing these bounds together.
- Queries return borrowed iterators rather than duplicate cached slices.
  `ram_spans()` exposes the sorted merged RAM union; clip `bank` indexes that
  union. `protected_ranges()` is the normalized in-RAM view;
  `source_ledger()` preserves original bytes and rounded extents outside RAM.
- `seal(self, &[PhysFrameRange])` validates only the physical extent projection
  of W04's future plan. W04 still owns placement, region bindings, storage
  layout and mapping. An empty plan is valid map-level evidence, **not** an
  initialized allocator or proof that a future allocator needs no metadata.
- Draft and sealed objects have private state and no `Clone`. Seal consumes
  the draft even on failure, publishing only a complete sealed authority on
  success. There is no retry, re-seal, unprotect or mutation of sealed state.
- `DuplicateSource` detects internal repeated ingestion. `SealRejected::Audit`
  rejects a broken coverage/source/class/accounting invariant. Errors identify
  input sources or bounded storage; they contain no memory dump.

## Downstream handoff and execution limits

W04 can query the draft and pass metadata extents to seal; it must accept only
`&BootMemoryMap` as its allocation-domain authority. W06 can read entries,
RAM spans, source sets, metadata, counters, clips and per-class totals. W08 can
reuse the policy tests, W09 can check totals, and W10 can carry the immutable
source ledger to P4 without introducing memory objects.

Boot execution still stops after W02 discovery. W03 is compiled as production
platform code and exercised on the host, but is not called from `boot/p2.rs`.
The design assigns joint planning to W04 and target accounting to W09. The
future adapter must preserve the P1-authoritative image extent and W01 DTB
lifetime, make map failures terminal, and emit the designed seal marker only
after a real plan has been accepted.

Storage is a material integration constraint: a draft is 40,768 bytes and a
sealed map 40,864 bytes on both measured host and probed AArch64 layouts.
P1's boot stack is 65,536 bytes. Those object sizes alone do **not** prove
stack fit; debug moves, temporaries and existing W01/W02 frames can exceed it.
W04/W09 integration must provide a reviewed storage/lifetime strategy and
compiler/call-path stack evidence before invoking this by-value API at boot.
This change neither enlarges the stack nor introduces unaudited static storage.

`RegionClass::is_protected()` fails closed for every class except Allocatable;
a future class also needs a corresponding accounting slot and tests. DTB
release remains Reserved. Firmware under-declaration remains undetectable by
this map. W04-LAYOUT/W04-MAP remain downstream gates. **P2-ACR-01 remains
ADR Required**, blocking memory-object work only; no `MemoryObject`,
`MemoryRegion`, Guest, Stage-2 or SMP mechanism is implemented.

## API and delivery summary

New stage-local Rust APIs are the frame types, builder, draft/sealed map,
queries and typed error/provenance records. There is no external ABI, wire
format, existing API change, new unsafe or dependency. No new TODO/FIXME is
introduced; deferred integration is explicitly owned above. Validation run and
not run, failure corrections and package closure are recorded separately in
the linked verification record. Delivery follows a branch and PR targeting
`main`, with required online checks before merge.
