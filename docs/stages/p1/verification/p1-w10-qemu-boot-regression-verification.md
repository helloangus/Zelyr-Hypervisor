# P1-W10 QEMU Boot Regression — Verification Record

**Status:** Foundation mechanism checks passed; integrated QEMU evidence not run.

**Date:** 2026-09-25

**Implementation:** [Record](../implementation/p1-w10-qemu-boot-regression-record.md).

## Foundation checks

Command: `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s scripts -p 'test_*.py' -v`.
Result: 16 tests passed, no skipped tests. These are host tooling mechanism
checks using controlled Python child processes; they are neither the canonical
Cargo host-test gate nor evidence of EL2, QEMU, MMIO, cache/TLB or hardware.

Covered: observed completion, late fatal after stable, global timeout after
stable, unexpected/silent process exit, missing executable, full capture on
output-limit failure, stderr isolation, final drain on termination, invalid
parameter evidence, refusal to overwrite prior evidence, invalid timeout
domain, equals-form evidence path selection, driver usage-error evidence,
clean-exit missing-marker classification, and fixed-token matching under all
permutations and byte boundaries.
This supplies foundation portions of W10-DV03/DV04 and matcher review for
W10-DV01/DV06; it does not satisfy their integrated execution requirements.

## Outstanding execution

R1 canonical boot, R2 panic detection, R3 real QEMU timeout, R4 missing marker,
R5 invalid image invocation, R6 permutation of real capture and 100 consecutive
boots: **not run**, awaiting W09's integrated runtime. P1-V16 and P1-V17 are
not yet satisfied. No result here closes deferred W01/W02 execution evidence.
The hard-timeout default remains provisional until exploratory timing exists.

## Build and conversion check

`cargo build --target aarch64-unknown-none-softfloat -p hypervisor` passed.
`PYTHONDONTWRITEBYTECODE=1 scripts/p1-image --output target/p1-foundation/hypervisor-boot.img`
produced a 15,891-byte image from the current W03-era ELF using Ubuntu LLVM
objcopy 18.1.3. The image SHA-256 is
`027165ffeedc9a30fda24c5cfa3f0eaa201039c3146ab448c3d4d9354e0185e6`;
the adjacent `target/p1-foundation/hypervisor-boot.json` preserves ELF/image
identities and converter provenance. This is a conversion check, not boot
evidence. `scripts/qemu-runner --version` reports runner/entry contract 0.1.
The exact CI QG-DOCS checker passed locally. The initial working-tree-only
whitespace check missed committed Markdown hard-break spaces. Review caught
six newly added trailing-space lines; these were removed. The complete branch
check `git diff --check origin/main` then passed, and the recorded unittest
discovery command was rerun with all 16 tests passing. This supersedes the
earlier incomplete whitespace-check statement.
