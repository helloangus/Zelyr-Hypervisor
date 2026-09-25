# W10 R4 marker-control amendment

**Status:** Proposed detailed-design correction; no implementation or verification claim.
**Scope:** R4's missing-Stable detection control only.
**Version:** v0.1.
**Owner/change context:** W10 scenario execution after W09 integration.
**Supersedes:** The R4 suggestion of a W11/truncated production boot path where it would require changing the production image.

## Current-state and foundation finding

The canonical W09 image reaches Stable and then idles. The existing runner
classifies a missing marker after a *clean child exit* as `FAIL-MARKER`; a
panic, abnormal exit, or hard timeout has its own distinct outcome. No
existing image starts, exits cleanly, and omits Stable. Thus R4 cannot be
executed honestly with the existing normal or NC2 image. Its bounded missing
artifact is a host-side control image, not a new EL2 production behavior.

## Bounded control contract

- **Required:** A tracked, minimal AArch64 control fixture emits the fixed
  `entry` and `runtime` start tokens through the W01 reference UART, emits no
  Stable/panic/fatal/reject token, and invokes QEMU's semihosting application
  exit. It is a *harness control*, not a hypervisor boot or W09 lifecycle
  validation image. A build helper may use installed LLVM assembler/linker and
  the existing `scripts/p1-image` conversion; it records tool/image identity.
- **Required:** The single `scripts/qemu-runner` process owner supplies a
  `p1-marker-control` profile that differs from `p1-boot-smoke` only by
  enabling semihosting. The W10 driver exposes an explicit, default-off
  `--marker-control` switch selecting that profile. Both profiles use the same
  fixed marker rules, bounds, evidence layout and verdict function.
- **Required safety boundary:** semihosting grants the image a host-facing
  service in QEMU, so the profile is never selected for the canonical image,
  100-cycle acceptance, W11 fault scenarios, CI default, or real hardware.
  The control source is tracked and reviewed. Before launch, the runner
  verifies the image SHA-256 against the reviewed fixture identity and
  rejects any other image with usage status 1. This is a local
  trusted-developer test path, not an API for external or untrusted images.
  Serial bytes remain untrusted and cannot select commands or paths.
- **Out of scope:** changing `hypervisor` Rust, W09 lifecycle, W11 injection,
  the W01 boot contract, target ABI, or QEMU command ownership. The fixture
  introduces no Rust `unsafe`; its AArch64 assembly is tooling-only, not part
  of a production image or the unsafe-Rust inventory.

The runner profile takes only the existing `boot-smoke=IMAGE` parameter and
the common finite timeout/evidence arguments. Unknown profiles and malformed
parameters retain P0 status 1. `--marker-control` is rejected for 100 cycles,
so the positive stage gate cannot accidentally use a markerless fixture.
`FAIL-MARKER` is accepted for R4 only when the outcome record shows raw QEMU
exit 0, expected start tokens present, Stable absent and forbidden tokens
absent. Otherwise R4 fails; no `FAIL-EXIT`, `FAIL-PANIC` or `FAIL-TIMEOUT` is
relabelled.

## Execution and evidence

Build fixture from tracked source into a fresh `target/` image, record its
SHA-256 in the runner's allowlist under review, then invoke
`scripts/p1-boot-regression --marker-control --cycles 1 --image <image>
--evidence <fresh-dir>`. Preserve source/image SHA-256, exact assembler/linker
and converter versions, exact command, raw serial, QEMU exit, verdict and
timestamp in W10's verification record. Review the fixture bytes to confirm
the expected sequence and no extra host operation. Success proves only the
runner detects absence of Stable after a clean observed control exit; it does
not prove W09 behavior or validate a production boot path.
