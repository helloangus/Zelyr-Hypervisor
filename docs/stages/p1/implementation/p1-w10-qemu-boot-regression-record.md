# P1-W10 QEMU Boot Regression — Implementation Record

**Status:** Runner foundation implemented; integrated runtime evidence pending W09.

**Date:** 2026-09-25

**Design:** [W10 design](p1-w10-qemu-boot-regression/README.md).

**Evidence:** [Verification record](../verification/p1-w10-qemu-boot-regression-verification.md).

The initial baseline is `4c1f554`. The merged foundation correction assigns
W10 the executable implementation of P0's frozen v0.1 runner interface.
Python 3 standard-library scripts introduce no third-party package dependency
or Rust-toolchain change. Host tooling requires Linux/POSIX process groups,
Python 3.9 or newer, QEMU system AArch64 and LLVM objcopy. Installed tool
versions are recorded with executions; no host-tool version is silently pinned.

## Entries and owned state

- `scripts/qemu-runner`: the only automated QEMU process owner. Implements
  `run --profile p1-boot-smoke [--param boot-smoke=IMAGE] [--timeout SECONDS]
  [--evidence-dir FRESH_DIRECTORY]`, and `--version`. Unsupported classes or
  values return usage status 1; the reserved P0 classes are not redefined.
- `scripts/p1-boot-regression`: sequential `--cycles 1|100`, stop on first
  failure, `--image`, `--timeout`, `--evidence`. Uses the same runner function;
  contains no QEMU launch recipe. Driver statuses: 0 all requested passes,
  1 boot failure, 2 invocation/environment failure. These do not alter the
  runner's P0 statuses 0–5.
- `scripts/p1-image`: converts the already-built ELF with `llvm-objcopy`,
  adds W01's 64-byte ARM64 Image header, records source/image SHA-256 and
  converter identity in adjacent JSON. Existing output is rejected. It does
  not build or execute the hypervisor. The default ELF is Cargo's **debug**
  artifact, correcting the historical W01 recipe's build/path mismatch.

Defaults are repository-relative even when invoked elsewhere. Image location:
`target/p1/hypervisor-boot.img`; evidence defaults to a fresh
`target/p1-evidence/run-<unique-id>` directory. This is build-tree layout,
not a published artifact name. Exported reports must follow P0 artifact naming.
An explicit evidence directory must not already exist, preventing overwrite.
Every cycle preserves `invocation.json`, `serial.log`, `emulator.log` and
`outcome.json`; the driver adds `meta.txt` and incremental `summary.txt`.
All serial captures are retained, including all 100 passes. Emulator stderr
is separate and cannot forge a target marker. Larger artifacts are referenced
by absolute path and SHA-256, not copied into evidence.

## Selected marker and resource rules

Stable token: `ZELYR P1 STABLE`; start context: `ZELYR P1 PHASE entry` and
`ZELYR P1 PHASE runtime`; forbidden prefixes: `ZELYR P1 PANIC`,
`ZELYR P1 FATAL`, `ZELYR P1 BOOT REJECT`. These selections precede integrated
verdict execution and must be checked against W07/W09 emission before R1.
The provisional hard timeout is 10 seconds; observation after stable or a
forbidden marker is 0.2 seconds. W09 exploratory timing must confirm headroom
before official R1. Timeout remains failure even when stable was observed.

Matcher state uses fixed-token membership with bounded overlap, not output
order or full-text equality. Stream complete bytes to disk. Limit target
output to 1 MiB and lines to 4096 bytes; crossing either bound terminates and
fails the run, retaining every received byte through drain. Emulator output
also has a 1 MiB bound. Termination sends SIGTERM to the process group, then
SIGKILL after 0.5 seconds if pipes remain; reap the process and drain capture.
Early child exit never passes solely because stable appeared.

## Delivery boundaries

Changed files are the five tooling entries/modules/tests, this record and its
verification record, plus linked runner/index documentation. No runtime code,
new unsafe, target ABI/public Rust API, Cargo dependency or CI gate changes.
W10's CLI is the first concrete realization of the existing P0 contract.
W11 will add its bounded scenario profiles to this runner, never another
QEMU process path. W09 integration, timeout calibration, R1–R6 and the
100-cycle run remain outstanding; no package or stage completion is claimed.
