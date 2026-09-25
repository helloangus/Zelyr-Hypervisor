# P1-W05 exception-entry implementation record

**Status:** Mechanism implemented; W07/W09 integration and executed fault evidence pending.
**Scope:** P1 single-CPU EL2 vector install, capture, classification and pre-arm termination.
**Version:** v0.1
**Owner/change context:** P1-W05, 2026-09-25.
**Supersedes:** None.

The [design](p1-w05-el2-exception-entry-baseline/README.md),
[preflight amendment](p1-w05-el2-exception-entry-baseline/00-preflight-amendment.md),
and [reconciliation](p1-w05-el2-exception-entry-baseline/06-implementation-reconciliation.md)
govern this implementation. `hypervisor/src/arch/aarch64/exceptions/entry.S`
owns sixteen 128-byte branch-only slots, guard-first landing, original
x0–x30/SP capture, and the terminal stack reset. The fixed `repr(C)` 304-byte
frame and pure syndrome/validity/classification logic live in `model.rs`.
`mod.rs` owns VBAR installation, category assertions against W04, readback,
the retained declaration and the pre-arm summary. The linker script places
`.text.boot` at 0x40080040, then a separate 4 KiB vector page and normal
`.text`; W08 consumes all three distinct intervals.

The assembly guard is armed before VBAR is installed. Every entry writes
original x0 to W05-owned TPIDR_EL2, checks and takes the guard without stack
or frame access, then captures all remaining registers in a unique static
frame. It resets SP to W02's boot-stack top only after capture and branches
to a diverging Rust entry. A second entry branches to silent stop before
rewriting the frame. There is no ERET, interrupt acknowledgment, Guest
semantics or recovery. The frame is not a public ABI.

`install_el2_exception_entry()` requires W04 C1–C4, rejects a second call,
checks 4 KiB table alignment, writes VBAR_EL2, executes ISB before readback,
and publishes its safe atomic declaration only after verification. Invalid
FAR is rendered `na`; synchronous external abort FnV and watchpoint validity
are checked, while SP alignment never claims FAR. HPFAR is always unavailable
in P1. The pre-arm summary uses W02's early writer with W07's frozen
`ZELYR P1 FATAL` prefix, `ph=unavailable` until W09, build identity and an
explicit terminal invariant. W07 later adds the full post-arm route;
W09 owns phase attribution, W11 owns fault injection evidence.

Unsafe boundaries: U-008 is the closed VBAR/FAR and ISB register set;
U-009 is vector assembly plus exclusive frame/guard and bridge protocol.
An independent root soundness review checked guard-before-frame ordering,
all register offsets, single-owner BSS, 16 slot destinations, stack alignment,
VBAR synchronization and validity guards. This review does not prove runtime
EL2 behavior. No new dependency, feature, external ABI or public API was
added. Crate-local W05 types/functions are new internal interfaces. There
are no unresolved architecture conflicts; the later W07/W09 seams are
explicitly deferred, not stubbed as successful.
