# W05 implementation reconciliation

**Status:** Detailed-design amendment; implementation and validation not claimed.
**Scope:** Fatal-entry stack, syndrome validity, concrete assembly/Rust seams.
**Version:** v0.3
**Owner/change context:** W05 current-state audit, 2026-09-25.
**Supersedes:** Conflicting stack and FAR-validity clauses in original entry
and classification contracts; supplements the preflight amendment.

## Foundation audit

| Goal | Current state at 4c1f554 | Required bounded artifact / owner | Proof |
|---|---|---|---|
| All 16 vector slots | No W05 code | Architecture exception module and 4 KiB vector region / W05 | Layout and target inspection |
| Original GPRs and guard-first capture | TPIDR/guard algorithm designed, no code | Assembly-owned guard/frame storage with shared offsets / W05 | Disassembly; later W11 known-register injection |
| Valid Rust fatal path for invalid SP | Boot stack exists; old design assumes faulting stack stays usable | Reuse existing stack top after capture, terminal only / W05 | Ordering review; later fault execution |
| Accurate fault-address reporting | Old SP-alignment/FnV rules incorrect | Total pure validity decoder and tests / W05 | Primary register reference and host tests |
| Baseline assertions | W04 implementation in parallel | W04 category/value queries / W04 | Rebase and compile before PR |
| Full fatal report and tracker | W07/W09 not executing yet | W05 pre-arm summary; W07/W09 wire owned seams | Deferred integration evidence |

## Primary architectural reference

[Arm Architecture Registers for A-profile, DDI 0601, ID032522](https://documentation-service.arm.com/static/6245e828b059dc5ff9a8ccef),
FAR_EL2 pages 747–748, ESR_EL2 Instruction/Data Abort and Watchpoint sections,
TPIDR_EL2 page 2175, is the recorded revision. FAR is meaningful for aborts,
PC alignment and watchpoints, not SP alignment. Abort validity checks FnV
for synchronous external abort FSC 0x10; watchpoints check FnV directly.
Other categories do not inherit a stale synchronous ESR's FAR validity.
HPFAR is always unavailable in P1. Only validity and the previously required
DFSC grouping are decoded; no trap emulation is added.
The original table's "LDP/STP trap group" is corrected to the documented
AArch32 LDC/STC trap, EC 0x06; no invented LDP/STP exception encoding is used.

## Entry and stack contract

The preflight guard-first algorithm remains authoritative. Once the frame
contains all original x0–x30 and the entry SP, assembly loads the existing
`__p1_boot_stack_top` linker symbol into SP before branching to Rust.
The path never returns, never restores the old stack, and never reads old
stack memory for diagnostics. This reclaims the boot stack for terminal
reporting without allocating a dedicated exception stack or adding a region.
The symbol is defined by W02's existing single stack-size constant.

The original prohibition on growing the faulting stack means no stack use
before guard/capture; it does not prohibit Rust report stack use after this
transition. DAIF and SPSel are not changed by entry. Hardware uses SP_EL2
for exception entry; invalid-origin coordinates remain diagnostic evidence.
Guard/frame/landing-code/boot-stack mappings must be valid. Failure before
the guard is taken is outside containment; after Taken, a recursive entry
that can access its guard reaches silent stop without another frame write.
Inaccessible landing/guard code or storage is outside this guarantee.

Assembly owns aligned guard/frame BSS storage, with no Rust static references
before exclusive capture. Rust receives a raw pointer only after fixed fields
are initialized; the guarded Rust bridge borrows the one immutable frame
after setting conditional FAR validity. A repr(C) fixed layout plus compile
time offset/size checks keeps assembly consistent. Original register values
are retained even when the exception itself is fatal. Frame addresses are
diagnostic register values only, never pointers used for enrichment.

## Install and declaration

`arch::aarch64::exceptions::install_el2_exception_entry()` asserts W04 C1–C4
are Established, arms the guard, installs and reads back VBAR, executes ISB,
then publishes `VectorStatus::Established`. Repeated install fails before
rearming. `vector_status()` and `vector_base()` report the retained state;
the atomic base (zero means absent) suffices without a second unsafe cell.
`VectorBase` is an internal semantic HVA wrapper at the sysreg boundary.
Existing designs' raw `Option<u64>` query remains diagnostic retained value.

W05 supplies `ExceptionFrame`, pure `syndrome_class`, `classify`, and
`route_classified`. All origins/categories terminate. Recoverable is reserved.
Pre-arm summary uses W02's writer; W07 readiness/report calls are added by
W07. Before W09 exists phase availability is explicitly `unavailable`, never
an invented current phase. W09 replaces this with its snapshot read.
The W07-owned fatal prefix is coordinated before runtime evidence.
It is `ZELYR P1 FATAL`. The pre-arm line also includes `fc=FC-INVARIANT`,
`site=vector`, and W02 build identity fields to satisfy P0's fatal minimum;
unavailable identity fields retain their honest unavailable tokens. The
fixed 512-byte buffer covers the longest current line with ample slack.

## Scope, validation and review

Required: corrections above, baseline/vector seams, full frame, host validity
tests, assembly inspection, inventory U-008/U-009 and per-package records.
Reserved: dedicated stack, TLS/SMP ownership, recovery and return machinery.
Out of scope: GIC acknowledge/EOI, timers, guests, allocator and ABI changes.

Pure tests cover all origin/category pairs, all EC encodings, DFSC groups,
FnV edges and asynchronous stale-syndrome rejection. There is no recoverable
resource failure or retry; misuse terminates. Assembly checks cover 16 slots,
page/alignment, frame offsets and guard-before-store ordering. W10/W11 own
executed normal, GPR and recursive-fault evidence; host tests cannot prove
EL2 entry or real hardware behavior. The skill checklist finds every missing
foundation owned and every interface/failure boundary explicit. Results and
completion claims belong in separate implementation/verification records.
