# P2-W03 runtime closure verification

**Status:** W03 complete in the bounded reference-QEMU boot scope described
below; W04 allocator and whole-P2 completion are not claimed.
**Scope:** W03-RV01–RV05 plus continued W03-DV01–DV10/P2-V05 evidence.
**Version:** v0.1
**Owner/change context:** User-requested W03 completion correction, 2026-09-27.
**Supersedes:** The host-only package-completion claim in the
[PR #70 verification](p2-w03-boot-memory-map-ownership-verification.md), not its
historical test results.

Design: [runtime storage correction](../implementation/p2-w03-boot-memory-map-ownership/06-runtime-storage-and-handoff.md).
Traceability: [runtime implementation record](../implementation/p2-w03-runtime-record.md).
Baseline: `df72bfb`; branch `p2/w03-runtime-storage`. Environment: x86_64 Linux,
repository Rust 1.98.1, AArch64 soft-float target, existing QEMU TCG and GNU
AArch64 binutils. No dependency or system package was installed.

## 1. Host and build checks

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | Passed |
| Host and AArch64 Clippy with `-D warnings` | Passed |
| `cargo test --workspace --exclude hypervisor` | 74 passed; zero failed/ignored |
| Same host command with `--release` | 74 passed; zero failed/ignored |
| AArch64 hypervisor build, debug and release | Passed |
| `python3 docs/stages/p2/verification/p2-w03-contract-probes.py` | Seven compile probes passed |
| CI documentation link/reachability/header block | Passed |
| Translation checker and seven checker unit tests | Passed |
| `git diff --check` | Passed |

The original 20 W03 tests remain; two new tests exercise exact owning/borrowed
entry/source/clip/summary equivalence, stable backing address, failed build,
failed seal and dropped-draft rejection of a second publication. Seven compile
probes include backing lifetime rejection (`E0515`) and exclusion of a second
mutable borrow while the sealed map is alive (`E0499`). Compiler assertions
check both host-observed and AArch64 layouts:

| Layout | Bytes |
|---|---:|
| MapStorage / owning draft | 13,888 |
| Owning sealed convenience value | 13,984 |
| Borrowed target draft | 8 |
| Borrowed target sealed handle, including summary | 96 |

This closes the large-return-value defect: the actual target uses inline const
backing initialization and `draft_in`. No target symbol for the owning
`BootMapBuilder::draft` or runtime `MapStorage::new` is linked. Clip records are
bounded computed views; they no longer consume a 26,880-byte cache.

## 2. Runtime recipe and artifact identity

The [recipe](p2-w03-runtime.py) uses the existing `p1_runner.profile/capture`
process owner. It records the exact command, image/ELF hashes, serial output,
result and stack evidence. Example invocations, after the two target builds:

```sh
scripts/p1-image --output target/p2-w03-runtime/debug2.img
scripts/p1-image --elf target/aarch64-unknown-none-softfloat/release/hypervisor --output target/p2-w03-runtime/release.img
python3 docs/stages/p2/verification/p2-w03-runtime.py --image target/p2-w03-runtime/debug2.img --elf target/aarch64-unknown-none-softfloat/debug/hypervisor --output target/p2-w03-runtime/debug-verified --trace
python3 docs/stages/p2/verification/p2-w03-runtime.py --image target/p2-w03-runtime/release.img --elf target/aarch64-unknown-none-softfloat/release/hypervisor --output target/p2-w03-runtime/release-verified --trace
```

Each traced case has a 55-second deadline and 0.2-second terminal-marker
observation; ordinary smoke uses eight seconds. Tracing is test-side only:
[the minimal debugger controller](p2_w03_trace.py) stops at the ELF's W03 entry,
checks its PC, enables `cpu,nochain` logging, removes the breakpoint and
continues. One guest instruction per TCG block yields all PC/SP samples,
including indirect callees, after that entry. No target memory/register is
modified by the debugger and no instrumentation is added to the hypervisor.

| Artifact | SHA-256 |
|---|---|
| Debug ELF | `6eef97723399ce8d305595e4d6a9ee65b820ad475a5d681d2430000da787743f` |
| Debug Image | `a28f4856e2406dfb00d0d4b33cc76d5f7f0c72ffce3dcaece5b541a3e80da77e` |
| Release ELF | `c0cfb15ee53529fa13b4b799e3d2145cf6330e6848980cda7fc488bb30f7d9ba` |
| Release Image | `092c61f7c0fadbee5aa9117117996e7bf4b20dbbdb816cfcd99d64f3fd11fe40` |

Compact serial/result/stack evidence is retained in the
[evidence directory](p2-w03-runtime-evidence/README.md). Large compressed raw CPU
logs and disassemblies remain under the recorded local `target/` paths, with
hashes in the evidence manifest; they are not claimed as off-host archives.

## 3. Executed runtime matrix

All cases use 128 MiB, one cortex-a57 boot CPU, GICv3 and no Guest. Canonical
and repeat use QEMU's ordinary kernel/DTB loader. Synthetic input cases use an
explicit loader trampoline to preserve the test DTB unchanged; the trampoline
page itself is reserved. They are adversarial/fragmentation fixtures, not a
claim that the machine's normal firmware reports eight banks.

| Case (debug and release) | Outcome | Debug used / headroom | Release used / headroom |
|---|---|---:|---:|
| Canonical | Sealed, exact RAM/protection conservation | 33,824 / 31,712 | 22,272 / 43,264 |
| Repeat | Same map output and SP bound | 33,824 / 31,712 | 22,272 / 43,264 |
| Eight banks + 32 reservations | Sealed; 32,760 RAM pages, source/clip preservation | 33,824 / 31,712 | 22,272 / 43,264 |
| Reservation overlaps image | Terminal ProtectionConflict; no sealed marker | 33,824 / 31,712 | 22,656 / 42,880 |
| Overlapping RAM banks | Terminal RamOverlap; no sealed marker | 33,824 / 31,712 | 22,656 / 42,880 |
| Present unusable reservation | Terminal UnusableFact; no sealed marker | 33,824 / 31,712 | 22,512 / 43,024 |

Values are bytes and include live ancestor frames, measured against the
unchanged 65,536-byte linker stack. No observed SP leaves the stack. The debug
capacity case records 6,718,527 instruction samples after W03 entry. The
recipe independently recomputes protected page union from the reported actual
DTB extent, ELF image bounds and declared fixture reservations; it checks
`RAM = allocatable + protected` and actual image-bound equality, not merely
presence of a success log. The active map reports `metadata=0 allocator=absent`.

The existing W01/W02 smoke recipe also passed all four cases against the new
debug image: canonical, GICv3, four discovered CPUs (only one executes), and
expected 256 MiB out-of-bootstrap-envelope rejection. This does not widen
W01's trusted 128 MiB access window.

## 4. Static stack review and proof limits

[Disassembly audit](p2_w03_stack.py) records immediate frame reservations,
direct call edges, indirect sites and cycles rather than treating unresolved
edges as zero. The final reviewed inventory is retained in the evidence set.

| Frame | Debug | Release |
|---|---:|---:|
| `el2_rust_entry` | 96 | 16 |
| `boot::p2::run` | 11,536 | 7,008 |
| `boot::p2::memory_map` | 16,144 | 14,688 |
| W02 `normalize` (returns before map entry) | 24,640 | 6,480 |
| Largest W03 direct acyclic path, including map frame | 23,536 | 15,984 |
| Largest direct formatting segment | 2,704 | 976 |

W03's bounded normal/typed-error paths have no recursion: bank sort is bounded
insertion sort, source/entry/clip loops are capped, and formatting is fixed
non-pretty scalar/enum output. Callback review bounds at most four nested
indirect segments: MapFatal → SourceId/SealViolation → scalar → Line writer.
A conservative envelope is ancestor frames + longest direct path + four times
the largest formatting segment: **45,984 bytes debug**, **26,912 release**.
This leaves respectively 19,552 and 38,624 bytes, above the recipe's additional
8 KiB reserve. All linked indirect sites in this closure belong to formatting
or compiler-generated enum dispatch; no arbitrary caller callback is accepted.

Reported panic recursion cycles are outside this envelope: W03 arithmetic and
bounded accesses reject typed invalid inputs before invariant panic, and the
fixed formatting path does not intentionally recurse into panic. This is not
a claim of bounded stack use after arbitrary hypervisor corruption or an
unexpected recursive panic. Dynamic traces corroborate normal/typed-error
paths; they are finite-fixture evidence, not exhaustive state-space proof.

The W03 storage frame is entered only after W02 normalization returns; their
large frames do not overlap. The P2 caller frame is slightly smaller than the
previous baseline. Existing P1 boot/exception behavior, asynchronous IRQ/SMP
stack use, future W04 calls, and different compilers are not proven by this
W03 budget. Recheck it when capacity, compiler or call graph changes.

## 5. Validation mapping and corrections

- **W03-RV01:** passed host lifecycle/equivalence and seven compile probes.
- **W03-RV02:** passed actual boot input, image/storage containment, sealing and
  independent accounting. The stack is included in HypervisorImage protection.
- **W03-RV03:** passed real terminal ownership/RAM/unusable-fact rejection.
- **W03-RV04:** passed debug/release frame review and instruction traces above.
- **W03-RV05:** reviewed live borrowed draft/sealed seam; W04 inserts its own
  planner before sealing and consumes this authority without fixing W03 storage.

Earlier unsuccessful attempts are retained locally and not counted as passes:
QEMU `-dtb` rewrote the RAM-node fixtures; overlapping generic-loader placement
at the automatic DTB address was rejected by QEMU; full-boot/filtered tracing
spent its deadline before W03; the first line-at-a-time trace reader timed out
on the debug capacity case. The final loader uses disjoint placements, tracing
starts at an ELF breakpoint, and chunked trace compression removes that reader
bottleneck without weakening predicates or increasing the deadline. An initial
compile assertion expected a 104-byte borrowed sealed handle; the actual
layout is 96 bytes and the assertion/test were corrected. These are fixture,
instrumentation and expected-layout corrections, not silent policy changes.

## 6. Completion boundary

W03 now supplies its own usable boot map, backing/lifetime, terminal failures
and stack evidence. The earlier host-only completion claim is corrected;
these foundations are no longer deferred to W04/W09. No new unsafe, dependency,
external ABI, stack expansion or unapproved global storage is introduced.

Not implemented/claimed: W04 metadata planner, metadata mappings, live allocator
or allocation/free stress; W05 heap; full W09 RAM/CPU matrix; P3/P4/Guest/SMP;
real hardware; coverage-guided fuzzing or exhaustive proof. Those are their
own packages' work. P2-ACR-01 and DTB-release reservation remain unchanged.
Configured required online checks must pass on the carrying PR before merge.
