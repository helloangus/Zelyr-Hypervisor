# P1-W10 QEMU Boot Regression — Implementation Record

**Status:** Runner, R1–R6 controls and 100-cycle execution recorded; R2 is a provisional NC2 panic-detection control pending W11 NC4 re-anchoring.

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
  The additional `p1-marker-control` profile uses the same process/evidence
  mechanism and fixed marker rules but explicitly enables QEMU semihosting
  only for the W10 R4 fixture. A prelaunch SHA-256 allowlist pins the
  reviewed fixture bytes (`399114e99cddcdf6686e2b04d8632dbc0409be3628aa22098a07caf3a6bd685b`);
  other images fail with runner usage status 1 before QEMU launch. This is a
  trusted-developer-local path, not an untrusted-image service. It is not the
  default/canonical path.
- `scripts/p1-boot-regression`: sequential `--cycles 1|100`, stop on first
  failure, `--image`, `--timeout`, `--evidence`. Uses the same runner function;
  contains no QEMU launch recipe. Driver statuses: 0 all requested passes,
  1 boot failure, 2 invocation/environment failure. `--marker-control` is
  default-off and rejected with `--cycles 100`. These do not alter the
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
Every launched cycle preserves `invocation.json`, `serial.log`, `emulator.log`
and `outcome.json`; an invocation rejected before launch has no emulator log
but retains invocation, empty serial and outcome. The driver adds `meta.txt`
and incremental `summary.txt`.
All serial captures are retained, including all 100 passes. Emulator stderr
is separate and cannot forge a target marker. Larger artifacts are referenced
by absolute path and SHA-256, not copied into evidence.

## Selected marker and resource rules

Stable token: `ZELYR P1 STABLE`; start context: `ZELYR P1 PHASE entry` and
`ZELYR P1 PHASE runtime`; forbidden prefixes: `ZELYR P1 PANIC`,
`ZELYR P1 FATAL`, `ZELYR P1 BOOT REJECT`. These selections precede integrated
verdict execution and must be checked against W07/W09 emission before R1.
The calibrated default hard timeout is 8 seconds; observation after stable or
a forbidden marker is 0.2 seconds. On the W10 reference host, R1 consumed
0.253–0.299 seconds and the 100-cycle run's per-cycle maximum was below
0.3 seconds; 8 seconds leaves more than 26× observed headroom while still
bounding a hang. This is a host-specific choice, not a hardware timing
guarantee. Timeout remains failure even when stable was observed.

The [R4 detailed-design amendment](p1-w10-qemu-boot-regression/04-marker-control-amendment.md)
adds `scripts/p1_w10_marker_control.S`, a tracked tooling-only AArch64 image
which writes start-context tokens through the W01 reference UART and issues
only semihosting `SYS_EXIT_EXTENDED` with the clean-exit argument block. It
does not enter W09, change `hypervisor`, or add a target runtime interface.
The fixture's assembler/linker are host LLVM 18.1.3 components already
installed alongside the existing image converter; no Cargo, Python package,
or hypervisor-TCB dependency was introduced. The generated fixture lives
under `target/`, not in the source/production image.

Matcher state uses fixed-token membership with bounded overlap, not output
order or full-text equality. Stream complete bytes to disk. Limit target
output to 1 MiB and lines to 4096 bytes; crossing either bound terminates and
fails the run, retaining every received byte through drain. Emulator output
also has a 1 MiB bound. Termination sends SIGTERM to the process group, then
SIGKILL after 0.5 seconds if pipes remain; reap the process and drain capture.
Early child exit never passes solely because stable appeared.

## Delivery boundaries

This execution follow-up changes only the W10 runner/driver module, host
tests, fixture source, and W10 design/implementation/verification/index
documents. It adds no hypervisor runtime code, new Rust `unsafe`, target ABI
or public Rust API, Cargo/Python package dependency, or CI gate. W10's CLI
remains the one concrete P0 runner entry; the profile addition is scoped
within its unchanged v0.1 grammar and six-status taxonomy. W11 can consume
`scripts/qemu-runner run --profile p1-boot-smoke` raw evidence for its own
scenario-specific verdict without W10 adding NC policy. The R2 control must
be re-anchored to W11 NC4 after that image lands; W12 receives the reference
environment and evidence paths. No P1 stage completion is claimed here.
