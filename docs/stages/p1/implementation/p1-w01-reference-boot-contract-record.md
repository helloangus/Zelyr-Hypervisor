# P1-W01 Reference Boot Contract — Implementation Record

**Status:** Implemented on branch `p1/w01-boot-contract`; verification
evidence in [the verification
record](../verification/p1-w01-reference-boot-contract-verification.md).  
**Date:** 2026-09-19 (Asia/Shanghai)  
**Design:** [W01 detailed implementation
design](p1-w01-reference-boot-contract/README.md) (consumed unchanged; no
design edit was required).

## Prerequisite state observed

- Baseline commit: `7688925` (merge of PR #36, the P0 handoff-map fix).
- P0-W03 build-target baseline: delivered. Target
  `aarch64-unknown-none-softfloat` under pinned toolchain `1.98.1`; the member
  builds a warning-free ELF; image-layout control is an explicitly reserved
  extension position owned by the introducing P1 design.
- P0-W09 QEMU runner entry: delivered as an interface-only placeholder —
  invocation grammar, six-status exit taxonomy, and evidence set exist; no
  runner program exists and none was created here (first implementation is
  P1-W10's).
- P0-W16 version/build metadata: delivered as a contract; it defines identity
  fields and policies but deliberately no embedding mechanism.
- W02's and W09's detailed designs exist and are consumed as accepted sibling
  contracts (the tier/reporter land in W02's entry module; the `entry`
  lifecycle records are W02-owned on this design's transfer guarantee).

## Implementation-selected contract fields (design decision 7)

These fields were selected once, at implementation, against the observed
reference platform, and are normative for W02/W10/W11 consumption.

| Field | Selected value | Selection rationale and observation |
|---|---|---|
| Emulator | `qemu-system-aarch64` 8.2.2 | the development host's available emulator supporting the `virt` machine; the runner (W10) records its own emulator identity per the P0-W09 evidence contract |
| Machine | `virt,virtualization=on` | enables EL2; observed to enter the image at Non-secure EL2 (`CurrentEL = 0b10`) |
| CPU model | `cortex-a57`, `-smp 1` | the design's suggested candidate; observed EL2-capable on the reference path; single-CPU canonical environment |
| Memory | `-m 128M` | minimum-memory arithmetic: P1 image well under 1 MiB + QEMU-generated DTB under 1 MiB + 64 KiB boot stack + ≥125 MiB slack; 128M is the `virt` default, deterministic, and leaves the DTB (observed at `0x44000000`) far from the image (loaded at `0x40080000`) |
| Image form | derived ARM64 boot image: 64-byte Image header prepended to the `rust-objcopy`-extracted raw body of the cargo-produced ELF | see the finding below; the P0 artifact (ELF) is unchanged and remains the build output |
| Loader | `-kernel` on the derived image (direct-kernel mechanism, Linux-Image mode) | the only observed mode of the direct-kernel mechanism that delivers the §4.1 boot-parameter contract |
| Console | the reference platform's first serial port: PL011 at `0x09000000` (`ttyAMA0`) | the single-source boot-entry constant (§8 layering statement below) |

### Finding: direct-kernel ELF boot does not deliver boot-protocol registers

QEMU 8.2.2 direct-kernel boot of a bare ELF enters at Non-secure EL2 but sets
`x0 = 0`; the `-dtb` option does not change this. A direct-kernel boot of the
same code in Linux-Image mode (ARM64 Image header with magic `0x644d5241` at
offset `0x38`, `text_offset = 0x80000`, total size at `0x10`) enters through
the emulator's boot stub, which sets `x0 =` DTB base (observed `0x44000000`)
and `x1–x3 = 0` before branching to the image — exactly the §4.1 contract.
The derived image is the standard `booti`-class format, which keeps the
Reserved U-Boot/TF-A recipe trivially reachable later. This selection resolves
the design's open "image form" field; it introduces no second canonical path
and does not modify the P0 build output.

### Canonical invocation recipe (recorded spelling)

Derivation (recipe-level; W10 owns the automated spelling under the runner
entry):

```sh
cargo build --target aarch64-unknown-none-softfloat -p hypervisor
rust-objcopy -O binary \
  target/aarch64-unknown-none-softfloat/release/hypervisor hypervisor.raw
# 64-byte Image header (little-endian): code0 = 0x14000010 (branch over the
# header), code1 = 0, text_offset = 0x80000, image_size = 64 + len(raw),
# flags = 0, magic = 0x644d5241 at 0x38; header || raw -> hypervisor-boot.img
```

Reference invocation:

```sh
qemu-system-aarch64 -machine virt,virtualization=on -cpu cortex-a57 \
  -smp 1 -m 128M -nographic -kernel hypervisor-boot.img
```

The Reserved second recipe is named: delivery to Non-secure EL2 by
U-Boot/TF-A on the reference platform or Orange Pi 3B, consuming the same
derived image via `booti`. It has no acceptance criteria in P1 and is not a
supported path.

## Materialized contract content

The content of
[01-boot-contract.md](p1-w01-reference-boot-contract/01-boot-contract.md)
§2–§8 is materialized as designed, without strengthening any assumption into
a check: the canonical path and its guaranteed properties (§2), the
entry-state table E1–E8 with the assumption/check split (§3 — E1/E7 checked
pre-transfer by Tier A; E2/E3/E5 recorded assumptions; E4/E6/E8 canonical
facts), boot parameters and DTB/memory treatment (§4), the rejection boundary
T1=`EL`, T2=`DTB` with R1–R5 and the fixed line
`ZELYR P1 BOOT REJECT reason=<token>\r\n` (§5), the transfer guarantee that
reaching `el2_rust_entry` implies T1/T2 passed (§6), and the layering
reconciliation (§8).

**Single-source constant (§8):** the reference-platform UART base
`0x09000000` is defined exactly once in W02's boot entry module (the
`P1_BOOT_UART_BASE` constant) and consumed by the assembly rejection reporter
(pre-transfer) and the Rust early diagnostic writer (post-transfer) through
the same definition. No other module may reference it; W06's console owns its
own reference-console assumption. No board/QEMU-name conditional exists
anywhere.

## Boundary implementation and W09 refinement status

- The tier, the rejection reporter, and the shared constant are implemented
  verbatim inside W02's entry module per
  [02-entry-validation-contracts.md](p1-w01-reference-boot-contract/02-entry-validation-contracts.md);
  the W01-only state of this branch implements no code by design (the
  boundary is a W02-delivered artifact).
- The §5 post-transfer routing refinement was cross-checked against W09's
  accepted design (decision 6, the H4/H6 routing rules, and the `entry`/`runtime`
  matrix rows): consistent — the panic route is established before the
  post-transfer window opens, and the delegated `entry`/`runtime` records are
  W02-owned exactly as W09 decision 6 expects. The refinement row stands as a
  recorded matrix amendment; W09's implementation confirms it at W09-DV04.

## Deviations from the design

None. The image-form and invocation selections above are the design's own
implementation-time fields (README decision 7), selected from observed
reference-platform facts and recorded here before any verdict-bearing boot.
No DTB parsing, memory discovery, console abstraction, second canonical
recipe, bootloader framework, or platform-name branch was introduced, and no
`unsafe` belongs to a W01 contract (the tier is check-and-branch assembly
inside W02's entry module).

## Handoff

- **W02** implements the tier, reporter, and constant verbatim (delivered in
  its entry module) and relies on the §3 entry-state table after transfer.
- **W03** re-derives the execution level as a required fact (defense in
  depth) on the EL entry guarantee.
- **W09** owns the tracker the `entry` records target; the §5 refinement row
  is its matrix amendment to materialize.
- **W10** consumes the canonical invocation recipe and treats the rejection
  line as a forbidden marker class on normal boots.
- **W11** executes NC1 against the `EL` rejection class (reference spelling:
  `virtualization=off`). NC2 owns missing-required-capability validation.
  A separate DTB rejection execution closes T2 evidence and must not be
  reported as NC2 capability evidence.
- **W12** assembles this record's contract content into the stage boot
  contract document; the recorded limitations are the Non-secure assumption
  (E3), the single-CPU canonical environment (E4), the unvalidated second
  recipe, and the unexamined DTB pointer (E7).
- **P2** receives the retained, unexamined DTB pointer and owns DTB intake.
