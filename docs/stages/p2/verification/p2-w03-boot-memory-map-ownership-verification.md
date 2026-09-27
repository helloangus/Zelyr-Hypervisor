# P2-W03 boot memory map verification

**Status:** W03 complete within its designed pure-map / host-validation scope;
not runtime map, W04 allocator, W09 QEMU accounting or whole-P2 completion.
**Scope:** P2-V05; W03-DV01–DV10, with downstream execution limits below.
**Version:** v0.1
**Owner/change context:** P2-W03 implementation verification, 2026-09-27.
**Supersedes:** None.

Implementation traceability: [W03 record](../implementation/p2-w03-boot-memory-map-ownership-record.md).
Contract: [W03 validation design](../implementation/p2-w03-boot-memory-map-ownership/05-validation-and-handoff.md).
Base: `2403db0`; branch: `p2/w03-boot-memory-map`. Environment: x86_64 Linux,
repository-pinned `rustc 1.98.1 (48a229cea 2026-09-01)`, target
`aarch64-unknown-none-softfloat`. No packages or dependencies were installed.

## Executed checks

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --exclude hypervisor --all-targets -- -D warnings` | Passed, no warnings |
| `cargo clippy --target aarch64-unknown-none-softfloat -p hypervisor -- -D warnings` | Passed, no warnings |
| `cargo test --workspace --exclude hypervisor` | 72 passed, 0 failed, 0 ignored; includes 20 new W03 tests |
| `cargo test --workspace --exclude hypervisor --release` | 72 passed, 0 failed, 0 ignored |
| `cargo build --target aarch64-unknown-none-softfloat -p hypervisor` | Passed; build evidence only |
| `python3 docs/stages/p2/verification/p2-w03-contract-probes.py` | All five compile probes passed |
| QG-DOCS Python block from `.github/workflows/ci.yml` | Links, reachability and headers passed |
| `python3 -m unittest discover -s tests -p 'test_doc_translations.py'` | Seven tests passed |
| `python3 scripts/check-doc-translations.py --coverage` | 33 translation pairs valid |
| `git diff --check` | Passed |

The host entry's one infrastructure-health test remains infrastructure evidence.
W03 contributes 13 real-pipeline integration tests and seven internal boundary
unit tests, source-sharing the exact production module. The internal tests do
not expose a public fixture constructor or change W02's API.

The seeded property test starts at `0x5eed`, runs 160 different RAM/protection
configurations, and checks 140 physical pages per configuration against a
byte-overlap oracle independent of the builder. It checks holes, allocatable
queries, per-source membership, metadata, sortedness, accounting and repeat
construction determinism. This is deterministic property-style testing, not
exhaustive fuzzing or an allocator stress claim.

## Validation matrix

| ID | Status and evidence | Proof boundary |
|---|---|---|
| W03-DV01 | Passed: conversion test covers unaligned RAM, empty ranges, byte-end/rounded-end/frame-count overflow, outward rounding, predicates and high-address boundary | Checked typed arithmetic; no hardware-address validity claim |
| W03-DV02 | Passed: required image/DTB test checks absent, in-RAM, boundary, partial/outside-RAM image; real pipeline test preserves image class | Synthetic image placement; P1 linker authority is not re-proven |
| W03-DV03 | Passed: actual W01 validation then W02 normalization; exact active DTB extent and one-ingestion header reservations; header/header/node duplicates keep three identities | W01 structural tests also reran; no physical DTB access execution |
| W03-DV04 | Passed: no-map/reusable node flags, initrd, invalid artifact/reservation, and page-only shared protection | No release/reuse policy implied by flags |
| W03-DV05 | Passed at map API: RAM overlap/alignment/overflow, ownership conflicts/different flags and all four non-usable states across every fact list; exact errors/ordinals checked | Real boot-stop diagnostic not run; target adapter is not wired |
| W03-DV06 | Passed: RAM dedupe before adjacency; source duplicate provenance; zero protection/RAM handling, upstream zero-counter separation, outside warnings and hole-spanning clips | Clip bank ordinal refers to normalized RAM union |
| W03-DV07 | Passed: independent seeded frame oracle, sparse banks above 4 GiB, complete RAM totals, sorted/disjoint entries and deterministic reconstruction | Finite host fixture space |
| W03-DV08 | Passed: oracle checks no protected page appears in allocatable spans; audit corruption tests reject wrong class, missing source, wrong coverage and crossing boundary | Map-level hard gate only, not allocate/free behavior |
| W03-DV09 | Passed: valid/empty/full-46 metadata plans; protected/outside/hole/cross-protection/self-overlap/over-capacity rejection; 84-source/8-bank capacity test | W04's actual planner and writable metadata remain unimplemented |
| W03-DV10 | Passed review: immutable/private sealed state, fail-closed class predicate, stable original ordinals, retained flags/bytes, disjoint shared-page accounting, no memory-object/Guest mechanism | Future class must also extend accounting slots; P2-ACR-01 remains open |

The compile probes independently confirm: draft cannot be used where sealed
is required (`E0308`), draft cannot seal twice (`E0382`), sealed has no seal
method (`E0599`), and a borrowed summary cannot be mutated (`E0594`). A fifth
probe compiles constant layout assertions for the pinned AArch64 target.

## Storage and target integration

Host size observation and AArch64 compile assertions agree:

| Object | Bytes |
|---|---:|
| UnsealedMemoryMap | 40,768 |
| BootMemoryMap | 40,864 |
| MapEntry (host observation) | 48 |
| ProtectionSource (host observation) | 56 |
| ClipRecord (host observation) | 40 |

The implementation is bounded and heap-free, but these sizes are **not** a
stack-usage proof. P1 has a 64 KiB boot stack; multiple by-value temporaries
plus W01/W02 activation frames may exceed it. The target build type-checks
W03 but does not execute it; unused code may be removed by the linker.
The W04/W09 adapter must close storage placement/lifetime and stack-fit review
before calling W03 at boot. No boot-stack-size change or new unsafe is made.

## Corrections during development

The first targeted test compilation found an invalid nested `self` import in
the new test module; it was corrected. The first sparse-bank test accidentally
reserved bytes covering the mandatory image/DTB/artifact inputs and correctly
received `ProtectionConflict`. Moving both sparse banks and their reservation
above those inputs corrected the fixture without changing the conflict rule.
Subsequent targeted and full debug/release runs passed. No gate or policy was
weakened, and no flaky online rerun is involved in those local corrections.

## Not run and closure limits

- QEMU map/accounting/negative boot-stop/seal-marker execution: **not run**;
  boot still ends after W02. P2-V11 belongs to W09.
- W04 metadata planner integration, live page allocation/free and mapped
  metadata: **not run**, pending W04-LAYOUT/W04-MAP and target storage fit.
- W06 rendering, W08 full-stage harness, W10 consumer/stage review: not claimed.
- Fuzz engine, Guest, SMP, Orange Pi and real hardware: not run/out of scope.

W03's own map API and P2-V05 host evidence are delivered. This does not close
P2-V06–V13 or the stage hard gate over every allocation/free sequence. W04
must derive all allocation solely from sealed allocatable spans. Firmware
under-declaration is outside the map's ability to detect. Original-DTB release
is Reserved and P2-ACR-01 remains ADR Required for memory-object work.

Documentation consistency and the required online gate results are attached
to the carrying PR; merge requires every configured required check to pass.
