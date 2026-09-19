# P1-W02 Minimal Rust EL2 Runtime — Verification Record

**Status:** Reviews complete; executed matrix entry deferred by contract;
manual-investigation boots recorded as informative evidence.  
**Date:** 2026-09-19 (Asia/Shanghai)  
**Environment:** development host, Linux (WSL2) x86_64,
`qemu-system-aarch64` 8.2.2, pinned toolchain `1.98.1`, branch
`p1/w02-minimal-runtime`.  
**Design:** [W02 detailed implementation
design](../implementation/p1-w02-minimal-rust-el2-runtime/README.md) ·
[implementation record](../implementation/p1-w02-minimal-rust-el2-runtime-record.md)  
**Parent validation IDs:** P1-V03 (ordered establishment), P1-V04 (stable
state without hidden firmware assumptions).

## Validation matrix results

| ID | Task-book ID | Test or review | Result | Evidence and command | Proves / does not prove |
|---|---|---|---|---|---|
| W02-DV01 | P1-V03 | Prerequisite sufficiency review | **passed** | implementation record "Prerequisite sufficiency": W01 entry-state table covers stages 1–9; P0 target baseline hosts the build; W09 items absent by dependency order with the contracted deferral recorded | readiness to build; not that prerequisites are implemented |
| W02-DV02 | P1-V03 | Entry and establishment review | **passed** | `llvm-objdump -d` of `p1_el2_entry` walked against the entry contracts: tier order T1→T2, `msr SPSel,#1` → SP → `msr DAIFSet,#0xf`, BSS loop, transfer; prohibited-content list (§4 of the entry contracts) walked — no control-register access beyond SPSel/DAIF/CurrentEL, no MMIO beyond the two authorized UART consumers, no cache/TLB/FP instructions, no unauthorized loops or branch targets; single stack constant; single-source UART constant (one definition, two contracted consumers) | the entry establishes what it must as designed; not firmware behavior on real hardware |
| W02-DV03 | P1-V03 | Ordered-establishment review | **passed** (within the W02-only scope) | code mapped line-by-line to the establishment order (W02 architecture §2): stages 1–4 assembly in order; stage 7 publication before stage 8 readiness assertions; stages 5–6, 9–12 are the contracted deferred seam with their call sites uncompiled and owners recorded — no stub, no placeholder phase, no direct-to-idle wiring; single lifecycle owner (W09's tracker; W02 declares none) | in-order establishment of the implemented subset as designed; the full ordered lifecycle with tracker events is proven at W09 integration (W09-DV01 reads the same seam) |
| W02-DV04 | P1-V03 | Panic/identity review | **passed** | `p1_panic` inspected: single-entry guard, bounded report (fixed prefix, identity line, message/location truncated to the 128-byte buffer), terminal stop independent of output success; identity resolves or degrades to the recorded literal (all three mechanism-dependent fields degrade by contract); exactly one post-transfer output path | the route exists and is bounded by design; its behavior under an executed panic is additionally observed (below) and formally belongs to W11's NC4 class |
| W02-DV05 | P1-V04 | Hidden-dependency review | **passed** | H1 walk: every register read/memory access in stages 1–8 checked against the W01 entry-state table — the tier reads CurrentEL/x0; establishment touches only SPSEL/DAIF/BSS/stack; no control register is read for a decision (H2); no allocator/GIC/discovery/Stage-2/secondary reach (H3); W01 tier, W09 contracts, P0 baselines consumed, none re-declared (H4) | absence of hidden firmware assumptions as designed; not runtime behavior |
| W02-DV06 | P1-V04 | Repeat-boot evidence on the integrated path | **not run — deferred to W10** | stable-state boots require the W09 sequencer and later phases; W10's regression executes the integrated path. Until then P1-V04's executed half is unproven; no W02 artifact reports otherwise | — (deferred) |
| W02-DV07 | P1-V03 | Consumability review | **passed** | implementation record's handoff read as W03 (live context, `BootContext` API, identity), W04 (asserted facts), W05 (routed panics), W06 (writer supersession seam), W07 (report-body seam), W08 (region inventory), W09 (deferred seam exact), W10/W11 (marker classes) | handoff readiness; not downstream completion |

## Quality gates (development + integration sets, run locally)

All six required gates pass on this branch: `QG-FMT`, `QG-LINT` (host-class
and bare-metal spellings, `-D warnings`), `QG-WARN` (zero warnings across
host build/tests and the target build), `QG-TEST-HOST` (host baseline
executes green), `QG-BUILD-TARGET` (artifact present at
`target/aarch64-unknown-none-softfloat/debug/hypervisor`). These prove the
compile/lint/test gates only — not EL2, boot, or QEMU behavior.

## Informative manual-investigation evidence (not a matrix entry)

Human-investigation boots on the reference recipe of the W01 record
(derivation: pinned-toolchain `rust-objcopy -O binary` + the 64-byte Image
header; not promoted into scripts or CI — the runner entry remains W10's):

1. **Canonical boot** (`-machine virt,virtualization=on -cpu cortex-a57
   -smp 1 -m 128M -kernel hypervisor-boot.img`): the serial capture is
   exactly the bounded seam report and nothing else —

   ```text
   ZELYR P1 PANIC
   identity: profile=unavailable rev=unavailable dirty=unavailable 0.1.0 aarch64
   message: sequencer seam unlinked: P1-W09 owns run_init_sequence and the stable record
   location: hypervisor/src/boot/mod.rs:174:5
   ```

   Zero rejection lines (the W10 forbidden-marker property), the tier
   passed, establishment stages 1–8 completed, and the runtime terminated
   through the recorded route — the honest W02-only terminal state, never
   presented as stable state.
2. **EL rejection** (`virtualization=off`): the capture is exactly
   `ZELYR P1 BOOT REJECT reason=EL` with no runtime output and no
   continuation — the W01 boundary firing pre-transfer.
3. **DTB rejection** (the cargo ELF booted directly via `-kernel`, which
   enters at EL2 with `x0 = 0`): the capture is exactly
   `ZELYR P1 BOOT REJECT reason=DTB`.
4. **Boot-to-boot determinism:** three consecutive canonical boots produced
   byte-identical target serial output (QEMU's own host-side termination
   message excluded).

This evidence supports DV02–DV05; it does not execute W02-DV06 (integrated
stable state) and proves nothing about later phases, other environments, or
real hardware.

## Summary against the parent validation IDs

- **P1-V03:** ordered establishment is implemented and reviewed (DV01–DV05,
  DV07 passed); tracker-recorded ordering arrives with W09's integration and
  is read from the same seam by W09-DV01.
- **P1-V04:** the hidden-assumption half is reviewed (DV05 passed); the
  executed stable-state half is **deferred to W10** and currently unproven.

No entry here proves P1-V01/P1-V02 (W01's matrix owns their executed halves)
or P1-V05 through P1-V21.
