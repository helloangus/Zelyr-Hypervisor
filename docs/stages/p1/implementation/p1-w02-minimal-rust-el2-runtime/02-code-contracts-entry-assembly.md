# P1-W02 Entry Assembly Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W02 detailed design](README.md).

Pseudocode is an outline of the required algorithm, not runnable production
code. The assembly is the only assembly W02 authorizes; every instruction
outside these contracts is a scope violation. No allocation, no stack use
before the stack exists, no FP/SIMD anywhere (the stage-wide no-FP guarantee
of [P1-W04](../p1-w04-el2-architectural-state-baseline/README.md) is already
binding on this module because it executes before the baseline is written).

## 1. Image entry point

```text
Name and stability: p1_el2_entry; assembly, internal (linker-visible as the
  image entry point); stable within P1.
Purpose and caller: the first instruction of the P1 image; entered by the
  canonical boot path per
  [W01's canonical contract](../p1-w01-reference-boot-contract/01-boot-contract.md)
  §2. Caller: the platform's direct-kernel loading mechanism.
Inputs / outputs: machine state per W01 contract §2; x0 = DTB physical
  pointer, x1–x3 reserved per W01 §4.1. No outputs.
Preconditions / postconditions: preconditions are exactly the canonical-path
  properties of W01 §2; postconditions are the establishment discipline of
  [01-architecture-and-state.md](01-architecture-and-state.md) §2 stages 1–4,
  ending in the transfer of §5.
State and ownership change: establishes the boot stack, SPSel, DAIF, and BSS
  state; writes no other machine state.
Concurrency/allocation context: boot CPU only; no allocation; no stack before
  stage 2 completes; instruction- and data-fetch behavior is assumed
  cacheable-free per the canonical path (no cache maintenance here — caches
  stay off until W08 owns enabling).
Errors and failure guarantee: a disallowed environment never reaches stage 2
  (the W01 tier rejects first); any establishment fault is outside P1's owned
  surface (W09 unowned-window limitation) and must not be created by this
  code (no self-modifying behavior, no speculative-state dependence).
Security/authorization checks: delegated entirely to the W01 tier; this
  entry adds no checks of its own beyond ordering.
Logic:
  p1_el2_entry:
    <W01 tier: T1, T2, reject-or-fall-through>   # implemented verbatim
    msr SPSel, #1                                 # use SP_EL2
    ldr x_tmp, =__p1_boot_stack_top; mov sp, x_tmp
    msr daifset, #0xF                             # all-masked; never unmasked in P1
    <BSS clear: see §3>
    b el2_rust_entry                              # transfer, §5
Validation: W02-DV02 entry review; W02-DV05 self-establishment walk.
```

The entry point symbol and the linker entry placement are recorded in the
implementation record against the P0 target baseline's extension points
(assumed contract; failure boundary in
[01-architecture-and-state.md](01-architecture-and-state.md) §6).

## 2. Boot stack

```text
Name and stability: __p1_boot_stack (region), __p1_boot_stack_top (boundary
  symbol); static, stable within P1.
Purpose and caller: the boot CPU's only stack for the entire P1 stage,
  including panic reporting and the sequencer. Callers: the entry (SP load)
  and every post-transfer caller implicitly.
Inputs / outputs: none; a reserved memory region.
Preconditions / postconditions: 16-byte aligned (AAPCS64 public-interface
  requirement); fully inside the image's reserved layout so W08 maps it as
  one writable, non-executable region; contains no initial content the
  runtime relies on before first use.
State and ownership change: none at definition; contents owned by executing
  code thereafter.
Concurrency/allocation context: single consumer (boot CPU); fixed size.
Errors and failure guarantee: overflow is not a designed path — the stack
  size is chosen with the recorded arithmetic of parent README decision 8;
  growth requires a design change, and overflow behavior is undefined-by-
  absence (recorded as a P1 limitation for W12, not mitigated here).
Security/authorization checks: none.
Size: P1_BOOT_STACK_SIZE_BYTES = 64 * 1024, defined once in the boot-path
  module with the sizing arithmetic in the implementation record.
Validation: W02-DV02 (alignment, single definition); W08 consumes the region
  inventory.
```

## 3. BSS zeroing

```text
Name and stability: the BSS-clear stage of the entry sequence; internal.
Purpose and caller: establish Rust static-data readiness (parent README
  decision 3). Caller: p1_el2_entry immediately after the stack/DAIF stage,
  before the transfer.
Inputs / outputs: consumes the BSS bounds from the linker extension points
  (`__p1_bss_start`, `__p1_bss_end` or the baseline's equivalent symbols —
  the exact names follow the P0 baseline and are recorded); no outputs.
Preconditions / postconditions: bounds are word-aligned and ordered; on
  completion every BSS byte is zero and no other memory was touched.
State and ownership change: BSS only.
Concurrency/allocation context: no stack use, no allocation, no loops except
  the store loop; stores are plain integer stores (no FP/SIMD, no
  cache-maintenance calls — caches are off per the canonical path).
Errors and failure guarantee: malformed bounds are a link-time defect caught
  by review (W02-DV02), not a runtime path.
Security/authorization checks: none.
Logic:
  x0 = __p1_bss_start; x1 = __p1_bss_end
  while x0 < x1: store zero at x0; x0 += word size
Validation: W02-DV02; the no-FP rule holds because the loop is integer-only.
```

Static data (`.data`, `.rodata`) needs no processing: the image is statically
linked and non-relocatable per the P0 target baseline. If the baseline ever
delivers relocatable semantics, that is a blocker to record, not a relocation
loop to write here.

## 4. Prohibited content

For review explicitness, the entry module must not contain: control-register
reads or writes other than SPSel/DAIF and the W01 tier's `CurrentEL` read;
any MMIO access other than the two authorized UART consumers (W01 reporter
pre-transfer; early writer post-transfer); any cache or TLB instruction; any
FP/SIMD instruction; any loop other than the two authorized ones; any
branch target outside the module except `el2_rust_entry` and the rejection
stop. W02-DV05 walks this list.

## 5. Transfer to Rust

```text
Name and stability: the transfer arrangement between p1_el2_entry and
  el2_rust_entry; internal; stable within P1.
Purpose and caller: move control from assembly to the Rust entry with the
  boot parameters intact. Caller: p1_el2_entry (final instruction).
Inputs / outputs: x0–x3 forwarded unmodified; no other register contract.
Preconditions / postconditions: establishment stages 1–3 complete; on
  arrival, Rust code may execute with the [W01 entry-state table](../p1-w01-reference-boot-contract/01-boot-contract.md)
  §3 checked fields holding.
State and ownership change: none beyond control transfer.
Concurrency/allocation context: a plain branch; the Rust side declares its
  own ABI boundary (extern "C", no unwinding) once, recorded in the
  implementation record.
Errors and failure guarantee: none — the branch cannot fail; post-transfer
  failures belong to the Rust contracts.
Security/authorization checks: none (the tier already decided).
Logic: b el2_rust_entry (x0–x3 preserved by the establishment stage, which
  uses only x_tmp and stack-relative state).
Validation: W02-DV02 review of the forwarding rule.
```
