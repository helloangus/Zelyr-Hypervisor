# P1-W02 Minimal Rust EL2 Runtime — Implementation Record

**Status:** Implemented on branch `p1/w02-minimal-runtime`; verification
evidence in [the verification
record](../verification/p1-w02-minimal-rust-el2-runtime-verification.md).  
**Date:** 2026-09-19 (Asia/Shanghai)  
**Design:** [W02 detailed implementation
design](p1-w02-minimal-rust-el2-runtime/README.md) (consumed unchanged) ·
sibling contracts: [P1-W01](p1-w01-reference-boot-contract/README.md) (tier,
reporter, constant — implemented verbatim here), [P1-W09
design](p1-w09-initialization-sequencing/README.md) (deferred seam, below).

## Prerequisite sufficiency (workflow step 1)

- The W01 entry-state table covers every establishment stage's needs: E1/E7
  are the tier's own checks; stages 2–4 establish everything else they use;
  stage 7 consumes x0–x3 per §4 of the boot contract. Verdict: sufficient.
- The P0 target/build baseline hosts the build as delivered (target, pinned
  toolchain, `no_std` semantics, reserved linking extension position — used
  for `link.ld` + `build.rs`, recorded below).
- W09's tracker/sequencer items do not exist yet (dependency order W02
  before W09). This is not a seam mismatch: the W02 design's workflow step 5
  contracts exactly this state — the call sites are not compiled, the
  deferred link and its owner are recorded, and nothing is stubbed. See
  "Deferred seam" below.

## Physical module placement (recorded decision)

All boot-path items live in the binary member's boot module tree; extraction
into architecture crates remains a later design decision (W02 design §1
leaves physical placement to recorded choice; no crate boundary is
authorized to change in this package):

| Item | Location |
|---|---|
| `p1_el2_entry` (assembly), W01 tier + rejection reporter, boot stack, shared constant, transfer | `hypervisor/src/boot/mod.rs` (`global_asm!`) |
| `el2_rust_entry`, establishment stages 7–8, deferred-seam terminal route | `hypervisor/src/boot/mod.rs` |
| `PhysAddr`, `BootContext`, once-publication boundary | `hypervisor/src/boot/context.rs` |
| `BuildIdentity`, readiness assertion | `hypervisor/src/boot/identity.rs` |
| `early_write_bytes`, writer readiness assertion | `hypervisor/src/boot/writer.rs` |
| `p1_panic`, single-entry guard, bounded report | `hypervisor/src/boot/panic.rs` |
| Boot-image layout (`link.ld`) and its wiring (`build.rs`) | member directory |

Layering note (per the W01 contract §8, preserved): the reference-platform
UART base `0x09000000` is defined once (`P1_BOOT_UART_BASE`,
`hypervisor/src/boot/mod.rs`) and consumed by both the assembly rejection
reporter and the Rust early writer through that single definition. No
board/QEMU-name conditional exists; the bring-up binary is the
reference-platform composition point and is expected to be replaced
piecewise by P2 discovery.

## Changed artifacts

| Artifact | Change |
|---|---|
| `hypervisor/src/main.rs` | replaced the P0 build probe with the P1 crate root (attributes + `mod boot`); the probe's `_start` park loop and placeholder panic handler are superseded by the W01/W02 contracts, exactly as the probe's own comments anticipated |
| `hypervisor/src/boot/mod.rs` (new) | entry assembly: W01 tier verbatim (T1 `EL`, T2 `DTB`), establishment stages 2–4, rejection reporter with fixed tokens, `el2_rust_entry`, `__p1_boot_stack`, `P1_BOOT_UART_BASE` |
| `hypervisor/src/boot/context.rs` (new) | `PhysAddr`, `BootContext`, `BOOT_CONTEXT` once-publication (audited boundary U-001) |
| `hypervisor/src/boot/identity.rs` (new) | `BuildIdentity` with recorded degradation; readiness assertion |
| `hypervisor/src/boot/writer.rs` (new) | `early_write_bytes` (audited boundary U-002), writer readiness assertion |
| `hypervisor/src/boot/panic.rs` (new) | `p1_panic`, guard, bounded report, bounded stop |
| `hypervisor/link.ld` (new) | load address `0x40080040` (post-header body address of the canonical Image-mode boot), section order, `__p1_bss_start`/`__p1_bss_end`, `.text.boot` placement |
| `hypervisor/build.rs` (new) | single `rustc-link-arg` wiring the layout script; host-side tooling only |
| `hypervisor/Cargo.toml` | `build = "build.rs"`; probe-state comment updated; still zero dependencies/features |

## Implementation-selected values and decisions

- **Boot stack:** 64 KiB, 16-byte aligned, defined once
  (`P1_BOOT_STACK_SIZE_BYTES`); the entry computes the top as
  `__p1_boot_stack + size` with the size substituted from the same constant
  (the design's `__p1_boot_stack_top` boundary realized as this single-source
  expression). `#[unsafe(link_section = ".bss.__p1_boot_stack")]` pins the
  region into writable BSS — an immutable zeroed static is otherwise emitted
  into read-only `.rodata`, which would contradict the W08 mapping class.
  Sizing arithmetic: establishment and panic paths use well under 1 KiB of
  depth; 64 KiB is the stage-wide bound; overflow behavior is
  undefined-by-absence (P1 limitation for W12).
- **Load address:** `0x40080040` — the canonical recipe (W01 record) loads
  the derived ARM64 boot image at `0x40080000`; the body begins after the
  64-byte header. Entry symbol `p1_el2_entry` at the image body start.
- **Single-entry guard:** `AtomicBool` with `Relaxed` ordering (single boot
  CPU, DAIF masked). The design's "plain static flag inside the audited
  boundary" is realized without `static mut`, so no forbidden-pattern
  exception is needed; the audited-boundary count is thereby two (U-001,
  U-002), not three.
- **Identity degradation:** `build_profile`, `source_revision`, `dirty` are
  the recorded `"unavailable"` literal (P0-W16 defines no embedding
  mechanism; fabricating one is out of scope); `project_version` is the
  single declaration's value (`0.1.0`, version-build-metadata §3.3) and
  `target_architecture` is the delivered baseline's value (`aarch64`).
- **Scoped allows** (each with in-code justification): `dead_code` on the
  `BootContext` accessors and the `PhysAddr` raw field (contracted read API
  whose consumers are W03–W09/P2), `non_upper_case_globals` on
  `__p1_boot_stack` (linker-facing design symbol).
- **Edition-2024 forms:** `#[unsafe(no_mangle)]` / `#[unsafe(link_section…)]`
  as required by the pinned toolchain's edition; no `unsafe extern` block
  exists (linker symbols are referenced only from assembly).
- **Token literals recorded for W10/W11 consumption** (W10 precedent: fixed
  before the first verdict-bearing run): rejection line
  `ZELYR P1 BOOT REJECT reason=EL|DTB\r\n`; panic-route marker prefix
  `ZELYR P1 PANIC\r\n`; report lines `identity: …`, `message: …`,
  `location: …`, bounded to 128 bytes each by the fixed-capacity buffer
  (truncation, never panic).
- **`el2_rust_entry` ABI:** `#[unsafe(no_mangle)] extern "C" fn(u64, u64,
  u64, u64) -> !` — the one recorded Rust/assembly boundary.

## Deferred seam (W09-owned link; recorded, not stubbed)

`el2_rust_entry` implements stages 7–8 and ends at the recorded terminal
route. The contracted call sites that do not compile until W09's items exist:

- stages 5–6: `Entry.enter/complete` and `Runtime.enter` tracker records
  (W09 tracker; W02-owned calls on the W01 transfer guarantee's authority);
- stage 9: `Runtime.complete`;
- stages 10–12: `run_init_sequence()`, the `stable` record, and
  `controlled_idle()`.

Until W09's integration lands, reaching the end of stage 8 is a terminal
invariant failure of the lifecycle contract and routes through the panic
route with the fixed message `sequencer seam unlinked: P1-W09 owns
run_init_sequence and the stable record` — visible, bounded, and never a
silent shortening of the lifecycle. The W09 integration replaces the panic
call with the contracted sequence; its `unsafe`-free, straight-line shape
means no further audited boundary is created.

## Region inventory for W08 (plan work seq 1 input)

| Region | Extent (as linked) | Required attributes |
|---|---|---|
| Text + vectors-carrying `.text.boot`/`.text` | `0x40080040` … `0x40083068` | read-only, execute |
| `.rodata` (incl. rejection/identity literals) | `0x40083070` … | read-only, execute-never |
| `.data`/GOT | (empty at link) | read-write, execute-never |
| `.bss` incl. `__p1_boot_stack` (64 KiB) and `BOOT_CONTEXT` | `0x40083ea0` … `0x40093ef0` | read-write, execute-never |
| Early-writer MMIO | `0x09000000` (UART base; FR at +0x18) | device, read-write |

## Deviations from the design

None in substance. The two realizations worth naming (both recorded above,
both narrower than the design's wording): the guard is an `AtomicBool`
instead of a `static mut` flag (avoids a forbidden-pattern exception), and
`__p1_boot_stack_top` is realized as the single-source `symbol + const`
expression instead of a second linker symbol.

## Handoff

- **W03** inherits the live Rust context, published `BOOT_CONTEXT`
  (`dtb()`/`reserved()` accessors ready), `BuildIdentity`, and the EL entry
  guarantee as its precondition.
- **W04** inherits SPSel=1/DAIF-masked as single-owner-established state.
- **W05** inherits the routed panic path and fixed DAIF discipline.
- **W06** supersedes the early writer for markers per its design; the
  writer's constant stays single-source in this module.
- **W07** owns the panic-report body through the extension seam; handler
  registration, guard, and bounded-stop discipline transfer through its
  design only.
- **W08** consumes the region inventory above.
- **W09** materializes the deferred seam exactly as contracted (see above).
- **W10/W11** consume the recorded marker classes (rejection line; panic
  prefix) and execute the deferred scenarios (integrated stable-state boots;
  NC4 panic).
