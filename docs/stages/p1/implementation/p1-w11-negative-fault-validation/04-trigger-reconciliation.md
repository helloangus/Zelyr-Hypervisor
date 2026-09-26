# P1-W11 validation-trigger reconciliation

**Status:** Proposed detailed-design correction; execution is not claimed.
**Scope:** NC1 runner profile, NC3–NC5 trigger placement, and NC6 feasibility on the integrated W09 path.
**Version:** v0.1.
**Owner/change context:** P1-W11 implementation, 2026-09-26.
**Supersedes:** The suggested NC5 Rust volatile-read technique in the scenario matrix; all other scenario requirements remain.

## Current-state and foundation check

W09 now has a real straight-line sequencer and Stable handoff. W05 classifies
current-EL synchronous and SError as `fatal-syndrome`; IRQ/FIQ are
`unexpected-event`. W08 maps only its declared image and console pages.
W10's scenario verdict interface is owned by W10 and must be integrated later.
The default image must remain unchanged and trigger-free.

NC3 and NC4 attach after `FatalPath.complete`, before `Stage1.enter`, so
vectors, console and the fatal path are ready. NC5 attaches after
`Stage1.complete`, before Stable, after `enable_host_stage1()` has returned
successfully. Exactly one of `p1-w11-nc3`, `p1-w11-nc4`, `p1-w11-nc5` may be
compiled into a validation image; none is selected by default. These class-1
binary capabilities have no runtime selector or external input. The NC2
feature remains independent but cannot be combined with another scenario.

## Trigger contract and safety correction

`fault_scenario(ScenarioId) -> !` is internal, no-allocation, boot-CPU only,
and is compiled only for a selected scenario. Every call is itself
compile-gated. A post-trigger fallback panics with a static invariant token
if hardware unexpectedly resumes, so the function cannot return normally.
The selector is a closed enum, not a user-supplied number. The NC3 instruction
is the architecturally permanently undefined AArch64 encoding `.inst 0`.
NC4 calls `panic!("P1-W11 NC4 intentional panic")`.
For NC3, the intended undefined-instruction exception has ESR.EC `0x00`;
W05's syndrome vocabulary renders that as `cls=unknown`. Acceptance must
therefore inspect the raw ESR EC together with `cat=sync`,
`disp=fatal-syndrome`, phase and PC, not treat `cls=unknown` alone as proof.

NC5 uses one AArch64 `ldr` from virtual address `0x5000_0000`, aligned to
eight bytes. W08's image occupies the `0x4000_0000` two-megabyte window
(L1 index 1, L2 index 0), while `0x5000_0000` shares L1 index 1 but uses
L2 index 128. The console is at `0x0900_0000` in separate L1 index 0,
L2 index 72. W08 verifies all
other L2 entries invalid, so this NC5 address is in an unmapped L2 entry
and should cause a Stage-1 translation fault. The exact
target is validation-only and is not interpreted as a host pointer by Rust.
NC5 acceptance expects `cls=data-abort-translation` and FAR
`0x5000_0000`, in addition to the W07 report/phase/terminal checks.

The matrix's suggested `ptr::read_volatile` from an intentionally unmapped
pointer cannot be shown to meet all of its language-level conditions for
this W05 terminal-handler route, nor does it fix a single exact load
instruction. This correction uses a single instruction in an isolated
unsafe assembly boundary. `ldr` may fault; W05's installed vector owns that
architectural transfer. If it unexpectedly completes, the static fallback
panic is an explicit failure of NC5, not a successful scenario.

Both NC3 and NC5 need unavoidable architecture instructions that safe Rust
cannot express. Their only unsafe segment is the instruction emission in
the validation-only architecture module. NC3's Rust-to-undefined-instruction
and W05 architectural exception-entry glue is `asm-glue` (U-016);
NC5's translation probe is `memory-mgmt`
(U-017). Neither is a normal Rust memory access.
They require independent arch/systems soundness review before merge. No
new production API, external ABI, state owner, dependency, synchronization,
or recovery route is created.

## NC6 limitation

A synchronous `BRK`, undefined instruction, or branch into a vector slot
does not raise an IRQ/FIQ/SError and cannot prove W05's
`unexpected-event` category. Current P1 masks DAIF and owns no GIC/IRQ
source or safe QEMU event-injection contract. NC6 stays blocked pending a
genuine, deterministic asynchronous category that does not introduce an
out-of-scope GIC/IRQ subsystem or relax normal controls. The verification
record must name this blocker; no proxy event may be recorded as a pass.

## Validation and proof boundary

Compile default and each scenario selection; reject combined selections.
Inspect the linked default ELF and source `cfg` gates for zero reachable
triggers. Run NC3–NC5 twice each through the W10 runner's scenario contract
once available, retaining serial captures and image hashes. Check class,
phase, syndrome/FAR, terminal marker, and absence of continuation. A build
or manual QEMU probe proves none of the required paired-run outcomes by
itself. S1–S6 must be reviewed on the final integrated tree; S6 additionally
requires W10's ordinary image regression.

The W11-specific `scripts/p1-w11-verify` is a verdict layer, not a QEMU
entry: it calls the one W10 `scripts/qemu-runner` entry twice with the
fixed `p1-no-el2` profile for NC1 or `p1-boot-smoke` for NC2–NC5, retains both complete evidence sets, then parses
their serial reports and status records. Its inputs are a closed scenario
name NC1–NC5, an image, finite timeout and fresh evidence root; output is a
paired `summary.json` and status 0 only if both runs match the exact class,
phase, syndrome/FAR expectations and image identity.

## NC1 fixed runner profile after W10 integration

The W10 runner's single `profile` function adds `p1-no-el2` for W11. It uses
the same trusted default or `boot-smoke=<image>` image selection, CPU,
memory, serial capture, timeout and process ownership as `p1-boot-smoke`;
the sole QEMU machine delta is `virt,virtualization=off`. No arbitrary QEMU
option or runtime environment selector is accepted. The required marker is
exactly `ZELYR P1 BOOT REJECT reason=EL`. The forbidden set includes P1 phase,
Stable, panic and fatal prefixes, and a DTB rejection. For this
scenario-specific oracle, runner status 0 means the W01 rejection marker was
observed and no forbidden marker arrived during its bounded post-marker
window. It does not mean the EL2 runtime booted. W11's paired verdict also
requires exactly one complete EL rejection line and no other P1 marker in the
full capture, matching image hashes and two concordant runs.

The W01 rejection loop intentionally does not exit; the runner terminates the
QEMU process after its fixed observation window. This is an environment-only
case using the same default image as a normal boot, not an in-image trigger.
It proves the QEMU `virtualization=off` variant is rejected before Runtime,
not a real-board firmware behavior. NC6 remains independently blocked.
