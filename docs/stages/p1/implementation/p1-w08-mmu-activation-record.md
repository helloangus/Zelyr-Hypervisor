# P1-W08 Stage-1 activation mechanism implementation record

**Status:** Mechanism target-built in this checkpoint; a later W09 integration boot is recorded in [verification](../verification/p1-w08-mmu-activation-verification.md), with post-MMU fault evidence still pending.
**Scope:** Fixed host Stage-1 tables, checked inventory, EL2 activation and postchecks.
**Version:** v0.1
**Owner/change context:** P1-W08 activation mechanism, 2026-09-25.
**Supersedes:** W08 table/layout foundation records only where this file explicitly adds an activation mechanism.

The [W08 detailed design](p1-w08-host-stage1-address-space/README.md),
[preflight amendment](p1-w08-host-stage1-address-space/00-preflight-amendment.md),
[failure seam](p1-w08-host-stage1-address-space/05-failure-seam-reconciliation.md),
and [activation correction](p1-w08-host-stage1-address-space/06-activation-reconciliation.md)
govern this change. The source-shared model now supplies individual table
pages and static step-qualified failure tokens. A target-only W08 module checks
the W03–W07 predecessor facts, derives seven page-separated linker/console
regions, builds and verifies the five-page model, copies it into a 4 KiB-aligned
20 KiB atomic static, and reads every word back before programming controls.

The EL2 register wrappers in `regs.rs` execute the recorded DSB/TLBI/I-cache
barrier order, MAIR/TTBR0/TCR writes and readbacks, then SCTLR M/C/I activation.
Postchecks read back SCTLR, force a read-only image load, and write/read a
writable-data sentinel. `enable_host_stage1` returns a typed error; W09 owns
its sole invocation and terminal failure route. Reentry is rejected. The
non-VHE EL2 one-VA-range page descriptor sets AP[1] (bit 6, RES1) for all six
mapping classes; AP[2] selects read-only mappings. A host test checks both
the class protection rule and mandatory bit 6.

The three proposed new unsafe boundaries are [U-012 through U-014](../../../security/unsafe-inventory.md):
closed EL2 register instructions, address-only linker-symbol declarations,
and one post-MMU volatile read. There is no external ABI or dependency change.
The crate-private `enable_host_stage1` and model page/error accessors are new
internal APIs. The permanent identity-map ABI, dynamic mapping, Guest Stage-2,
SMP and GIC remain outside this change.

The mechanism is compiled but currently uncalled by W09. Link-time garbage
collection can remove its table/sentinels from the boot ELF, so this checkpoint
does **not** demonstrate physical table placement, post-MMU continuation,
vector/console continuity, or stack peak. [Verification](../verification/p1-w08-mmu-activation-verification.md)
records the exact checks and evidence limits. P1-V13/P1-V14 remain open.
