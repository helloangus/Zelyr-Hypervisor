# P1-W10 QEMU Boot Regression — Verification Record

**Status:** R1, R3–R6 and 100-cycle evidence recorded; W11 NC4 re-anchored R2 on 2026-09-26. The original NC2 R2 control below remains historical.
**Date:** 2026-09-25 (UTC timestamps below).
**Implementation:** [W10 record](../implementation/p1-w10-qemu-boot-regression-record.md).

The original 2026-09-25 table and its provisional R2 wording are preserved
as execution history; the 2026-09-26 NC4 addendum at the end is the current
R2/P1-V16 status.
**Design amendment:** [R4 control](../implementation/p1-w10-qemu-boot-regression/04-marker-control-amendment.md).

## Environment, rules and evidence identity

Host: Linux `6.18.33.2-microsoft-standard-WSL2` x86_64; QEMU system AArch64
8.2.2 (Debian/Ubuntu package `1:8.2.2+ds-0ubuntu1.18`); Rust 1.98.1;
LLVM assembler/linker/objcopy 18.1.3. Source base at execution:
`249e1b3ad6911fc7eb14087a7875ddff750fbfbd` plus this W10 working-tree
change. Final runner source SHA-256:
`b07e20de08600408b9cded2188b8ca06d377d1d86ec1233759498b099865fd27`.
QEMU was invoked only by `scripts/p1-boot-regression` through the single
`scripts/qemu-runner` process owner, using `virt,virtualization=on`,
`cortex-a57`, one CPU, 128 MiB, headless serial and `-kernel` with the named
image. The exact argv and QEMU version are in each `cycle-*/invocation.json`.

Normal image: `target/p1-w10-canonical.img`, SHA-256
`afb9d826213b7424ee9e1f1aeb2e9d3c49eee49033899d7f9aa228bc72fc16d6`
(102,720 bytes), from current default-feature hypervisor ELF SHA-256
`d63f3ab5e411dc8da247a9247f8c937ce27f25bf69c110bdfeb75c4171eaaf92`.
R2 NC2 image: `target/p1-w10-nc2.img`, SHA-256
`3422570efd968b5a918809aca50ac25cf685eb3caa2b33683d0b8f248b5143e6`.
R4-only marker control image: `target/p1-w10-marker-control-v2.img`, SHA-256
`399114e99cddcdf6686e2b04d8632dbc0409be3628aa22098a07caf3a6bd685b`;
tracked fixture source SHA-256
`c497740b700c7c7318aa8c69a62cec6b07df8835acf0158fb59ebc0617cdef47`.
Image-provenance JSON files sit beside each image.

Fixed normal-run predicates: required `ZELYR P1 STABLE`,
`ZELYR P1 PHASE entry`, `ZELYR P1 PHASE runtime`; forbidden
`ZELYR P1 PANIC`, `ZELYR P1 FATAL`, `ZELYR P1 BOOT REJECT`. Default hard
timeout is 8 seconds, post-marker observation is 0.2 seconds, serial and
emulator streams are separately capped at 1 MiB, and a line is capped at
4096 bytes. The normal R1 elapsed 0.257 seconds; an 8-second bound leaves
over 31× headroom on this host. This is host-specific calibration, not a
firmware or hardware timing guarantee.

Raw evidence root (local build tree, not committed):
`/home/angus/dev/Zelyr-Hypervisor/.worktrees/w10full/target/p1-w10-evidence/`.
Keep this worktree or archive evidence before removing it. Each launched
cycle has `invocation.json`, full `serial.log`, separate `emulator.log`, and
`outcome.json`; each scenario has `meta.txt` and `summary.txt`. R5 fails before
launch and therefore has no emulator log. No failure capture was deleted or
overwritten. These local paths are retrievable now but are not a durable CI
artifact store.

## Build and scenario execution

Image creation commands (run in this worktree):

```sh
cargo build --target aarch64-unknown-none-softfloat -p hypervisor
scripts/p1-image --elf target/aarch64-unknown-none-softfloat/debug/hypervisor --output target/p1-w10-canonical.img
CARGO_TARGET_DIR=target/p1-w10-nc2 cargo build --target aarch64-unknown-none-softfloat -p hypervisor --features p1-w11-nc2
scripts/p1-image --elf target/p1-w10-nc2/aarch64-unknown-none-softfloat/debug/hypervisor --output target/p1-w10-nc2.img
llvm-mc -triple=aarch64 -filetype=obj -o target/p1-w10-marker-control-v2.o scripts/p1_w10_marker_control.S
ld.lld -Ttext=0x40080040 -e _start -o target/p1-w10-marker-control-v2.elf target/p1-w10-marker-control-v2.o
scripts/p1-image --elf target/p1-w10-marker-control-v2.elf --output target/p1-w10-marker-control-v2.img
```

Commands below use the frozen marker rules and final runner source. Deliberate
negative controls correctly return nonzero; that process status is not a
failed *harness-control* result. Prefix each with `PYTHONDONTWRITEBYTECODE=1`.

| Scenario | Exact command after the prefix | UTC start | Observed result / control assessment | Raw evidence subdir |
|---|---|---|---|---|
| R1 | `scripts/p1-boot-regression --cycles 1 --image target/p1-w10-canonical.img --evidence target/p1-w10-evidence/r1-final` | 15:53:52 | `PASS`; runner 0, Stable present, forbidden absent; passed | `r1-final/` |
| R2 | `scripts/p1-boot-regression --cycles 1 --image target/p1-w10-nc2.img --evidence target/p1-w10-evidence/r2-final` | 15:54:22 | `FAIL-PANIC`; runner 4, driver exit 1, full capture retained; provisional detection passed, NC4 pending | `r2-final/` |
| R3 | `scripts/p1-boot-regression --cycles 1 --timeout 0.001 --image target/p1-w10-canonical.img --evidence target/p1-w10-evidence/r3-final` | 15:54:28 | `FAIL-TIMEOUT`; runner 3, QEMU terminated, driver exit 1; passed | `r3-final/` |
| R4 | `scripts/p1-boot-regression --marker-control --cycles 1 --image target/p1-w10-marker-control-v2.img --evidence target/p1-w10-evidence/r4-final` | 15:53:47 | `FAIL-MARKER`; runner 4, raw QEMU exit 0, entry/runtime true, Stable and forbidden false, driver exit 1; passed | `r4-final/` |
| R5 | `scripts/p1-boot-regression --cycles 1 --image target/p1-w10-absent.img --evidence target/p1-w10-evidence/r5-final` | 15:54:34 | `ERROR-INVOCATION`; runner 2, no cycle counted, driver exit 2; passed | `r5-final/` |

All timestamps are 2026-09-25 UTC. R2 uses W11's existing NC2
missing-required-granule image as the explicitly permitted interim
panic-bearing variant; it does **not** test W11's final NC4 panic trigger or
prove crash-report completeness. A W11 NC4 image must replace this control
in a subsequent R2 run before the final P1-V16 closure review. R4 is a
separate tiny QEMU fixture, not evidence that W09 itself can cleanly exit
before Stable. Its assembly was inspected: the code writes two fixed UART
lines and issues only semihosting `SYS_EXIT_EXTENDED` with aligned arguments
`[0x20026, 0]`. The runner checks its reviewed image hash *before* enabling
semihosting; an unreviewed image returns usage status 1 without QEMU launch.

R4 development evidence is retained: first fixture attempt `r4/` used
`SYS_EXIT` and produced `FAIL-EXIT` with raw QEMU exit 1. It is an
unsuccessful exploratory control, not recast as R4 acceptance. Corrected
`SYS_EXIT_EXTENDED` in `r4-v2/` first produced raw exit 0 and `FAIL-MARKER`;
the final-source confirmation is `r4-final/` above.

## R6 and 100-cycle review

R6 passed against the actual `r1-final/cycle-001/serial.log`: feeding its
lines in original order, reverse order, and benign-noise/interleaved order
through `Matcher` yielded `PASS` in all three cases. The tracked host test
also checks every permutation and byte boundary of the fixed tokens. The
matcher uses token membership/absence only; it does not compare line order,
timing, total line count or full output text. The one-off R6 command was:

```sh
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts python3 -c 'from pathlib import Path; from p1_runner import Matcher,STABLE,START,FORBIDDEN,verdict; lines=Path("target/p1-w10-evidence/r1-final/cycle-001/serial.log").read_bytes().splitlines(keepends=True); variants=(lines,list(reversed(lines)),[b"\r\nQEMU benign noise\r\n"]+lines[::2]+[b"\r\n"]+lines[1::2]); results=[]; exec("for seq in variants:\n m=Matcher((STABLE,)+START,FORBIDDEN)\n for line in seq:m.feed(line)\n results.append(verdict({\"status\":0,\"predicates\":m.predicates()}))"); assert results==["PASS"]*3,results; print(results)'
```

The accepted repetition command, fresh directory and image were:

```sh
PYTHONDONTWRITEBYTECODE=1 scripts/p1-boot-regression --cycles 100 --image target/p1-w10-canonical.img --evidence target/p1-w10-evidence/r100-accepted
```

P1-V17 requires 100/100 `PASS` with Stable present and no forbidden class
across the complete evidence set, no retry/exclusion and stop-on-first-failure.
`r100-accepted/` returned exit 0, `requested=100`, `counted=100`,
`passed=100`, zero `FAIL-*` or `ERROR-*`, from 15:55:25.962 to
15:55:55.022 UTC (29.06 seconds total). Per-cycle elapsed times ranged
0.247–0.260 seconds (median 0.249 seconds). It was run without another QEMU
process by coordination with the W11 worker. The 100 invocation intervals
are sequential and non-overlapping; every cycle used the same image SHA-256,
`p1-boot-smoke` profile and effective 8-second timeout. All 100 cycle
directories have the four required files; every serial capture has exactly
one Stable token, no forbidden token, and runner status 0. Each invocation's
runner-source hash matches the final tracked script. The machine-checkable
`r100-accepted/summary.txt` SHA-256 is
`3740b967bf11d2be02f10a0cef8cade4b50ab51224eb390ea2d58865244d0428`.
This meets P1-V17 for this exact image and reference host, subject to keeping
the local raw evidence available; it is not a stage-completion or hardware
claim.
Earlier `r100/`, `r100-final/` and `r100-final2/` all returned 100/100
`PASS`, but are exploratory; notably `r100-final2/` overlapped another QEMU
control near its end, so it is not used as the strict no-concurrent-load
acceptance set.

## Quality gates and proof boundary

| Design validation | Status | Evidence / remaining boundary |
|---|---|---|
| W10-DV01, DV02 | Passed | Fixed-token source review, 19 host tests, R1/R6; rules fixed before the accepted 100-cycle set. |
| W10-DV03 | Passed | R3 timeout and R5 environment error distinct from boot outcomes; outcome-table host tests. |
| W10-DV04 | Passed with provisional R2 anchor | R2/R4 full failure captures and every attempted-cycle record retained; W11 NC4 still needs to replace NC2 for final R2. |
| W10-DV05 | Passed | R1 final-source canonical QEMU boot. |
| W10-DV06 | Passed | R6 real-capture permutation and tracked matcher tests. |
| W10-DV07 | Passed | Strict one-image, sequential, stop-on-first-failure procedure reviewed against `r100-accepted` metadata. |
| W10-DV08 | Passed for this image/environment; handoff open | `r100-accepted` 100/100 audited; W11 NC4 re-anchor and W12 evidence-map archival remain downstream work. |

P1-V16's objective-verdict mechanism and controls are exercised, but its
final acceptance awaits the W11 NC4 R2 re-anchor. P1-V17 has the 100/100
reference-run evidence above. Other P1 validation IDs are not closed by W10.

`PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s scripts -p
'test_*.py' -v` passed 19 host mechanism tests, including profile default-off,
one-cycle limit and unreviewed-image rejection before QEMU launch. The W10
target default-feature build and NC2-feature build passed. No target unit
test harness, Guest boot, multi-CPU run, real-board test, CI QEMU gate,
hardware cache/TLB proof or W11 NC1–NC6 suite ran as part of W10. This
change adds no target Rust `unsafe`, target ABI/public Rust API, or runtime
dependency. R1/100 are reference-QEMU evidence for the named image only.

## W11 NC4 R2 re-anchor (2026-09-26)

The preceding R2 NC2 entry is the original provisional control, not the
final panic-specific source. On W11's branch rebased to W10 main `5319995`,
the same `scripts/p1-boot-regression` entry ran one cycle with the explicit
NC4 intentional-panic image `target/p1-w11-final-nc4.img` (SHA-256
`58caf44e10cda431db48a40f945832c94780f1ec85182541ed5e2e295fe12620`):

```sh
PYTHONDONTWRITEBYTECODE=1 scripts/p1-boot-regression --cycles 1 --image target/p1-w11-final-nc4.img --evidence target/p1-w11-r2-nc4-post-w10
```

The driver exited 1 as expected for a negative control; its summary records
`requested=1`, `counted=1`, `passed=0`, `FAIL-PANIC`. The cycle runner
record has status 4, `reason=forbidden-marker`, and a matched panic token.
Its image hash matches the W11 NC4 paired-run image; the W11
[scenario verification](p1-w11-negative-fault-validation-verification.md)
independently checks the exact static panic message, `fatal-path.complete`
phase, W07 fields, single terminal marker, and both concordant NC4 runs.
Raw evidence is retained under
`/home/angus/dev/Zelyr-Hypervisor/.worktrees/w11full/target/p1-w11-r2-nc4-post-w10/`.
This completes the previously provisional R2 panic-control anchor and
supports P1-V16 on this local reference-QEMU baseline. It does not repeat
the W10 100-cycle set or extend its image/environment claim.
