# P2-W01 verification

**Status:** Passed for the declared reference boot scope; P2-V01/P2-V02 met.
**Scope:** Intake, bounded mapping and consumer handoff; not whole-P2 completion.
**Version:** v0.1
**Owner/change context:** P2-W01, 2026-09-26.
**Supersedes:** No earlier execution evidence; resolves the recorded A1/A2 blocker.

Implementation is described in the [W01 record](../implementation/p2-w01-boot-platform-description-intake-record.md)
and [baseline amendment](../implementation/p2-w01-boot-platform-description-intake/00-current-baseline-amendment.md).
The reproducible source-shared tests are
[`p2_platform.rs`](../../../../crates/host-test-baseline/tests/p2_platform.rs).

## Environment, commands and artifacts

Local Linux x86_64 host, Rust 1.98.1 (`48a229cea`, 2026-09-01), QEMU 8.2.2
(Debian Ubuntu package 1:8.2.2+ds-0ubuntu1.18), AArch64 softfloat debug image.
All commands ran on 2026-09-26; local artifacts are under
`target/p2/reviewed-evidence/` and are not durable/off-host archival evidence.

- `cargo fmt --all -- --check`: passed.
- `cargo test --workspace --exclude hypervisor`: 52 passed, zero failures or
  ignored tests (34 existing baseline tests and 18 P2 tests).
- `cargo clippy --workspace --exclude hypervisor --all-targets -- -D warnings`:
  passed; AArch64 `-p hypervisor` Clippy with `-D warnings`: passed.
- `cargo build --target aarch64-unknown-none-softfloat -p hypervisor`: passed.
- `scripts/p1-image --output target/p2/reviewed-evidence/boot.img`: image size
  225600 bytes, SHA-256
  `7a82feafd10adc0e17d425065efb40d5873db5cb2fc4d36ac6c347763557755a`;
  source ELF SHA-256
  `67f98c0ffc611d2206b44a65043b0bd128e1eafe4e723a5e2df3fb9cd0e2bec5`.
- `python3 docs/stages/p2/verification/p2-w01-w02-smoke.py --image
  target/p2/reviewed-evidence/boot.img --output target/p2/reviewed-evidence/smoke`:
  all four predicates passed; the [recipe](p2-w01-w02-smoke.py) reuses the
  existing P1 QEMU process owner and records command, timestamp, hash and logs.
- `scripts/p1-boot-regression --cycles 1 --image
  target/p2/reviewed-evidence/boot.img --evidence
  target/p2/reviewed-evidence/p1-regression`: one requested cycle, one PASS.

## W01 evidence map

| Validation | Status | Evidence and limit |
|---|---|---|
| W01-DV01–DV03 / P2-V01 | Passed | Placement tests distinguish absent, malformed size, alignment, coverage, arithmetic overflow and overlap before header interpretation; aperture test checks page-rounded containment/RO/XN. Code review establishes checks precede raw reads. |
| W01-DV04 / P2-V02 | Passed | Explicit header-field mutations, span/overlap rejection and truncation at every byte; corrected boot CPU/structure size offsets exercised by discovery. |
| W01-DV05–DV07 / P2-V02 | Passed | Root/name/balance and depth/node/property/reservation caps; overflow/unterminated reservations; every-byte two-bit mutation sweep exercises every certified cursor and discovery without panic. This is a deterministic mutation sweep, not exhaustive fuzzing. |
| W01-DV08 | Passed | Unknown nodes, empty nodes, non-UTF-8 node-name anomaly and zero-based reservations remain bounded and observable. |
| W01-DV09 | Passed | First-failure diagnostic assertions; QEMU out-of-envelope case returns one `DtbUnreachable` and no intake/discovery complete marker. Logs contain static classes and counts, not DTB text. |
| W01-DV10 | Passed | Real W02 consumer uses only the cursor; W03 can read typed range/reservations; same validator compiles in host tests for W07/W08 reuse. |
| U-018 / widened U-012 | Passed | Independent `/root/p2_soundness_review` accepted the owner borrow and single-CPU publication sequence; inventory updated in the same change. QEMU executes the new mapped reads. |

## Executed QEMU cases

All cases use cortex-a57, virtualization enabled, TCG, no guest, and an
8-second deadline with 0.2-second post-marker observation. Successful cases
include P1 stable, intake complete and discovery complete, with no P1 fatal,
panic/rejection or P2 rejection. These are bounded package smoke witnesses,
not W09's full configuration/repetition matrix.

| Case | Result |
|---|---|
| Canonical 128 MiB, one CPU, default GIC | Passed; DTB at `0x44000000`, intake/discovery complete; GIC unsupported, timer/PSCI usable |
| 128 MiB, one CPU, GICv3 | Passed; GIC/timer/PSCI usable |
| 128 MiB, four CPUs, GICv3 | Passed; four CPUs, one RAM bank, boot index zero; GIC/timer/PSCI usable |
| 256 MiB, one CPU, GICv3 | Expected rejection passed; DTB outside the conservative first-128-MiB bootstrap read envelope; no complete marker |

Initial integration attempts rejected `stage1-not-started`. Linked-code review
found P1 debug table constructor stack overflow; PR #66 fixed that upstream
without increasing the stack or bypassing the check. Two host-fixture mistakes
were also corrected: adjacency was incorrectly expected to overlap, and a
one-cell root fixture initially retained two-cell GIC registers. The passing
results above were rerun after those corrections; failures were not ignored.

Not run/claimed: physical hardware, alternate firmware/DMA/SMP writers,
Miri or coverage-guided fuzzing, W07 real RK3566 fixtures, W09 full matrix,
P2 allocators/guest behavior, or 100-cycle repetition. The bootstrap envelope,
immutable-loader premise and new mapping owner are explicit limits. Full P2
completion remains dependent on W03–W10 and P2-ACR-01 remains outside W01.
