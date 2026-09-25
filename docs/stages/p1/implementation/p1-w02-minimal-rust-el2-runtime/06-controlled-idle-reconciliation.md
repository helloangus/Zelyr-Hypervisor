# W02 controlled-idle unsafe boundary reconciliation

**Status:** Proposed detailed-design correction; implementation and validation are recorded separately.
**Scope:** The W02 `controlled_idle()` instruction boundary realized at W09 integration.
**Version:** v0.1
**Owner/change context:** P1-W02/W09 integration, 2026-09-25.
**Supersedes:** The unsafe-boundary count in W02 [implementation/review §4](05-implementation-and-review.md) and any implication in the [runtime contract §5](03-code-contracts-rust-runtime.md) that `wfi` requires no audited unsafe boundary. The `loop { wfi }` behavior remains unchanged.

## Goal, observed gap and authority

W02 requires a controlled, non-returning stable idle loop, but its original
implementation stopped at the unlinked W09 seam. W09 now supplies the ordered
mechanisms and Stable transition, so the concrete W02 `wfi` instruction must
exist for the runtime goal to hold. Rust has no safe stable AArch64 WFI
primitive. The W02 design named the instruction but omitted its `unsafe` from
the review count; that omission cannot authorize an untracked unsafe block.
This correction is limited to the approved W02 idle mechanism and P1-W09
handoff; it neither changes the phase machine nor grants scheduler, IRQ,
power-management, SMP or firmware configuration behavior.

The boundary is **Required** for W02 stable idle and P1-W09 Stable handoff.
Interrupt delivery, low-power policy and secondary CPUs are **Reserved** for
their owning later stages. Changes to `SCR_EL3`, a local low-power guarantee,
and scheduler idle are **Out of Scope**. ADR-008 leaves EL3 firmware owned
outside this image, so the canonical QEMU observation cannot promise that
every firmware permits WFI to complete locally: `SCR_EL3.TWI` may trap an EL2
WFI to EL3. That firmware policy is an explicit platform limitation and must
not be mistaken for a P1-controlled low-power state or an EL2-only execution
guarantee.

## Internal interface and safety contract

`controlled_idle() -> !` stays W02-owned in the boot module and is called only
after W09's `run_init_sequence()` returns and `enter_stable()` advances the
single tracker from Completed(Stage1) to Stable. It loops over the safe
`arch::aarch64::idle::wait_for_interrupt() -> ()` wrapper. The wrapper is the
single **U-015 `arch-register` unsafe boundary** and executes exactly one
`wfi` instruction. The wrapper owns no mutable state, allocation, lock or
retry policy. On a normal WFI return it returns to the caller, which repeats;
it does not promise what power state hardware entered. An architectural trap
or firmware action is outside this wrapper's normal-return guarantee and
cannot be represented as a Rust `Result` here. W05/W07 own any EL2 exception
that reaches their installed vector path; EL3 trap handling is not owned by P1.

The boundary is necessary because safe Rust cannot issue WFI. Its safety
premise is one W01-established EL2 boot CPU, W04-retained masked DAIF and a
completed W09 Stable transition. The W02 straight-line glue establishes the
call order; W09's CAS tracker rejects premature/duplicate Stable transitions;
the arch wrapper has that loop as its sole P1 call site. The single instruction has
no memory or stack operands and preserves flags. A false lifecycle or EL2
premise is FC-INVARIANT and normal boot must not continue. It is not a license
to invoke the wrapper from other contexts. [U-015](../../../../security/unsafe-inventory.md)
records the inventory and independent review.

## Implementation and validation handoff

Implementation steps: first wire W09's phases and Stable transition without a
stub; then add the one arch wrapper and W02 loop; then confirm target assembly
contains WFI and the control path cannot reach it before Stable. Review the
new source `SAFETY` comment and U-015 with a second architecture reviewer.
An absent predecessor mechanism blocks the work; no direct-to-idle fallback
is authorized. The implementation record belongs under
[`../p1-w09-initialization-sequencing-record.md`](../p1-w09-initialization-sequencing-record.md)
and results under
[`../../verification/p1-w09-initialization-sequencing-verification.md`](../../verification/p1-w09-initialization-sequencing-verification.md).

The review passes only if the straight-line sequence is Runtime.complete →
W09 phases → Stable → WFI loop, no new caller of the arch wrapper exists, and
target disassembly shows the instruction. A QEMU boot reaching Stable and
remaining boundedly quiet supports the canonical reference path; it does not
prove hardware power consumption, EL3 trap policy or real-board behavior.
There is no new external ABI, dependency, allocation or rollback mechanism.
