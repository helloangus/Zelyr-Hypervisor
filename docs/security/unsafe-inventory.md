# Zelyr Unsafe Inventory

**Status:** Normative register.
**Version:** v0.4 — W04 baseline and W05 exception-entry boundaries.
**Owner/change context:** P0-W10 unsafe Rust governance; entries are created
only by real, merged unsafe changes under the policy's review rules.
**Supersedes:** the empty v0.1 register (zero first-party `unsafe`).

Entry schema, lifecycle, and update rules: see the [unsafe Rust
policy](unsafe-rust-policy.md); this register carries entries only. Each
entry carries the full policy-governed schema: `id` (`U-<nnn>`, creation
order, never reused), `status` (`proposed` → `accepted` → `superseded` |
`removed`), `title`, `boundary-category` (exactly one of `arch-register`,
`mmio-volatile`, `memory-mgmt`, `low-level-struct`, `asm-glue`,
`boot-state`), `location`, `necessity`, `safety-preconditions`,
`establishment`, `failure-class`, `authorizing-design`, `owner`,
`review-record`, `validation`, `audit-status`, `permanence`; a partially
filled entry may exist only in the `proposed` state and blocks merge until
completed.

## Entries

### U-001 — `BOOT_CONTEXT` once-publication cell

- **status:** accepted
- **title:** `BootContext` publication through `UnsafeCell` behind a
  publish-once flag
- **boundary-category:** `low-level-struct`
- **location:** `hypervisor/src/boot/context.rs`
  (`BootContextCell`, `unsafe impl Sync`, `publish_boot_context`)
- **necessity:** the boot context must be a `static` readable by
  package-internal consumers after establishment while being written exactly
  once from the Rust entry; safe Rust expresses no once-publication static
  without `std::sync` (unavailable in the `no_std` image) or an `unsafe`
  cell boundary
- **safety-preconditions:** single boot CPU executing with DAIF masked;
  exactly one publication, performed before any consumer exists
  (straight-line establishment order, no re-entry path); the published value
  is fully initialized before the write
- **establishment:** the establishment order (W02 architecture §2) is the
  only writer path; the `published` flag makes a second publication an
  invariant violation routed to the panic route; P1 owns no other executing
  context
- **failure-class:** hypervisor-invariant violation (FC-INVARIANT), terminal
  via the panic route
- **authorizing-design:** [P1-W02 detailed design](../stages/p1/implementation/p1-w02-minimal-rust-el2-runtime/README.md),
  Rust-runtime contracts §2
- **owner:** P1-W02
- **review-record:** PR carrying this change; second-reviewer soundness
  review per the unsafe policy §3
- **validation:** QEMU manual boots recorded in the [W02 verification
  record](../stages/p1/verification/p1-w02-minimal-rust-el2-runtime-verification.md)
  (canonical boot publishes and proceeds; no host coverage exists by the
  build-target baseline's member boundary)
- **audit-status:** audited at creation (this change)
- **permanence:** permanent for the P1 boot path; re-audited when W08's
  address-space work or the W03 report-cell pattern formalizes ownership

### U-002 — early diagnostic writer volatile MMIO

- **status:** accepted
- **title:** polling volatile write to the reference UART data/flag
  registers
- **boundary-category:** `mmio-volatile`
- **location:** `hypervisor/src/boot/writer.rs` (`early_write_bytes`)
- **necessity:** device output is memory-mapped; no safe Rust construct can
  perform a volatile MMIO store
- **safety-preconditions:** the UART base is the single-source boot constant
  (`P1_BOOT_UART_BASE`, defined once in the boot entry module per the W01
  contract's layering reconciliation); the writer is the only post-transfer
  consumer of the data register (temporal single-consumer rule, W02
  architecture §3); accesses are volatile with polling before each store
- **establishment:** the constant is compile-time and asserted non-zero; the
  caller set (panic route, and later W07's body through the extension seam)
  is fixed by the W02 design; no initialization is performed
- **failure-class:** if the base were wrong, stores would target unrelated
  device space — hypervisor-invariant violation, terminal (the caller's
  bounded-stop discipline bounds every path; a stuck transmitter spins in
  the poll and reaches the same stop)
- **authorizing-design:** [P1-W02 detailed design](../stages/p1/implementation/p1-w02-minimal-rust-el2-runtime/README.md),
  panic/identity contracts §3
- **owner:** P1-W02
- **review-record:** PR carrying this change; second-reviewer soundness
  review per the unsafe policy §3
- **validation:** observed by every executed diagnostic scenario — QEMU
  manual boots (canonical report, EL and DTB rejection lines) in the [W02
  verification record](../stages/p1/verification/p1-w02-minimal-rust-el2-runtime-verification.md)
- **audit-status:** audited at creation (this change)
- **permanence:** panic-path permanent (channel-independent route per the
  W06/W07 boundaries); marker output superseded by W06's channel when it
  lands

### U-003 — AArch64 capability identification reads

- **status:** accepted
- **title:** six side-effect-free EL2 identification register reads
- **boundary-category:** `arch-register`
- **location:** `hypervisor/src/arch/aarch64/capabilities/mod.rs`,
  `read_current_el`, `read_mpidr`, `read_id_aa64pfr0`, `read_id_aa64mmfr0`,
  `read_id_aa64mmfr1`, `read_cntfrq`
- **necessity:** Rust has no safe intrinsic for these privileged MRS reads;
  pure decoding and policy stay outside the assembly boundary
- **safety-preconditions:** W01-established EL2 and trusted firmware register
  accessibility; W02 live runtime, boot CPU and masked DAIF
- **establishment:** only the W03 phase body calls the six reads; W09 owns
  invocation after runtime establishment; assembly declares all outputs and
  no memory, stack, or flags side effects
- **failure-class:** FC-INVARIANT if the execution/firmware contract is violated
- **authorizing-design:** [W03 extraction contracts](../stages/p1/implementation/p1-w03-aarch64-capability-inventory/02-code-contracts-fact-extraction.md)
  and [reconciliation](../stages/p1/implementation/p1-w03-aarch64-capability-inventory/05-implementation-reconciliation.md)
- **owner:** P1-W03
- **review-record:** root systems reviewer, 2026-09-24: six isolated MRS
  wrappers, explicit outputs/no memory effects, W01 EL2 precondition;
  boundary sound for the masked boot CPU
- **validation:** host decoder tests and target compilation in the
  [W03 verification record](../stages/p1/verification/p1-w03-aarch64-capability-inventory-verification.md);
  executing register evidence assigned to W09/W10, hardware not run
- **audit-status:** author and independent root soundness review complete
- **permanence:** P1 boot inventory; re-audit if EL/firmware prerequisites change

### U-004 — immutable capability report publication

- **status:** accepted
- **title:** `UnsafeCell<Option<CapabilityReport>>` behind monotone publication state
- **boundary-category:** `low-level-struct`
- **location:** `hypervisor/src/arch/aarch64/capabilities/mod.rs`, `ReportCell`,
  `unsafe impl Sync`, `publish`, `get`
- **necessity:** no_std has no safe static once-cell; no dependency added;
  consumers require retained immutable boot facts
- **safety-preconditions:** exactly one exclusive writer before any reference
  escapes; reads only after a complete publication; value never mutated again
- **establishment:** compare_exchange claims unpublished→publishing once;
  write completes before release-published; readers acquire-published before
  borrowing, and early/repeated access panics before touching storage
- **failure-class:** FC-INVARIANT, terminal via panic for protocol misuse
- **authorizing-design:** [W03 report contracts](../stages/p1/implementation/p1-w03-aarch64-capability-inventory/03-code-contracts-classification-and-report.md)
  and [reconciliation](../stages/p1/implementation/p1-w03-aarch64-capability-inventory/05-implementation-reconciliation.md)
- **owner:** P1-W03
- **review-record:** root systems reviewer, 2026-09-24: exclusive 0→1 claim,
  Option write before Release 2, immutable reads after Acquire 2, report
  fields Sync; boundary sound for the masked boot CPU
- **validation:** complete-draft policy host tests, publication source review,
  and target compilation in the [W03 verification record](../stages/p1/verification/p1-w03-aarch64-capability-inventory-verification.md);
  execution deferred to W09/W10, no hardware or SMP claim
- **audit-status:** author and independent root soundness review complete
- **permanence:** boot report retention through P2 handoff; no reset API


### U-005 — EL2 baseline register reads

- **status:** accepted
- **title:** W04 closed-set architectural control reads
- **boundary-category:** `arch-register`
- **location:** `hypervisor/src/arch/aarch64/baseline/mod.rs` (`read_sysreg`)
- **necessity:** Rust has no safe EL2 system-register read primitive.
- **safety-preconditions:** W01-established EL2, boot CPU only, DAIF masked;
  W03 report published; optional CNTHV accessed only when its fact is Present.
- **establishment:** the W04 phase checks C1 and upstream facts; only the
  static specifications select writes; masks preserve unowned fields and
  constants honor RES1. No memory or stack effects; the closed selector excludes arbitrary system registers.
- **failure-class:** FC-INVARIANT, terminal; pre-vector register faults retain
  the documented unowned-vector limitation.
- **authorizing-design:** [W04 design](../stages/p1/implementation/p1-w04-el2-architectural-state-baseline/README.md)
  and its preflight synchronization amendment.
- **owner:** P1-W04 architecture baseline.
- **review-record:** independent arch/systems soundness review by root agent,
  2026-09-25: closed selector, guarded CNTHV and execution premises accepted.
- **validation:** [W04 verification](../stages/p1/verification/p1-w04-el2-architectural-state-baseline-verification.md);
  host tests cover masks, not hardware effects. QEMU execution is deferred to
  W09/W10 and real-hardware validation to its owning stage.
- **audit-status:** author and independent soundness review complete.
- **permanence:** P1 boot boundary; re-audit on control-set, feature-guard,
  synchronization or execution-context changes.

### U-006 — EL2 baseline register writes and ISB

- **status:** accepted
- **title:** W04 closed-set architectural control writes and ISB
- **boundary-category:** `arch-register`
- **location:** `hypervisor/src/arch/aarch64/baseline/mod.rs` (`write_sysreg`)
- **necessity:** Rust has no safe EL2 system-register write primitive.
- **safety-preconditions:** W01-established EL2, boot CPU only, DAIF masked;
  W03 report published; optional CNTHV accessed only when its fact is Present.
- **establishment:** the W04 phase checks C1 and upstream facts; only the
  static specifications select writes; masks preserve unowned fields and
  constants honor RES1. Each write is immediately followed by ISB; no nomem annotation hides control effects.
- **failure-class:** FC-INVARIANT, terminal; pre-vector register faults retain
  the documented unowned-vector limitation.
- **authorizing-design:** [W04 design](../stages/p1/implementation/p1-w04-el2-architectural-state-baseline/README.md)
  and its preflight synchronization amendment.
- **owner:** P1-W04 architecture baseline.
- **review-record:** independent arch/systems soundness review by root agent,
  2026-09-25: masks, guards, readback, per-write ISB and compiler effects accepted.
- **validation:** [W04 verification](../stages/p1/verification/p1-w04-el2-architectural-state-baseline-verification.md);
  host tests cover masks, not hardware effects. QEMU execution is deferred to
  W09/W10 and real-hardware validation to its owning stage.
- **audit-status:** author and independent soundness review complete.
- **permanence:** P1 boot boundary; re-audit on control-set, feature-guard,
  synchronization or execution-context changes.

### U-008 — W05 exception-register access and synchronization

- **status:** accepted
- **title:** closed VBAR_EL2/FAR_EL2 access and context synchronization
- **boundary-category:** `arch-register`
- **location:** `hypervisor/src/arch/aarch64/exceptions/mod.rs` (`vbar_write`, `vbar_read`, `exception_barrier_isb`, conditional FAR read)
- **necessity:** Rust has no safe primitive for these EL2 registers or ISB.
- **safety-preconditions:** W01 EL2 and masked boot CPU; W04 C1–C4 established; aligned executable vector page; FAR is read only for architecturally valid synchronous classes.
- **establishment:** W04 declaration checked before install; linker symbol and runtime alignment checked; ISB occurs after VBAR write before dependent readback; pure validity classifier guards FAR.
- **failure-class:** FC-INVARIANT terminal if entry/control premises fail; before vector installation the documented unowned-window limit applies.
- **authorizing-design:** [W05 design](../stages/p1/implementation/p1-w05-el2-exception-entry-baseline/README.md) and its preflight/reconciliation amendments.
- **owner:** P1-W05.
- **review-record:** independent root review, 2026-09-25: closed register set, placement, barrier/readback order and FAR validity accepted.
- **validation:** [W05 verification](../stages/p1/verification/p1-w05-el2-exception-entry-baseline-verification.md); target build and host validity tests pass, QEMU fault execution deferred to W11.
- **audit-status:** author and independent soundness review complete; execution proof pending.
- **permanence:** P1 EL2 entry boundary; re-audit if register set, vector mapping or validity table changes.

### U-009 — W05 guard-first vector assembly and frame bridge

- **status:** accepted
- **title:** sixteen vector slots, exclusive guard/frame storage and terminal Rust bridge
- **boundary-category:** `asm-glue`
- **location:** `hypervisor/src/arch/aarch64/exceptions/entry.S` and `mod.rs` (`p1_exception_rust_entry`, guard initialization)
- **necessity:** exception entry cannot preserve original GPRs, claim a guard and capture machine state through safe Rust alone.
- **safety-preconditions:** one masked boot CPU; W02 stack and BSS initialized; W04 EL2 baseline established; landing code, guard, frame and stack remain accessible; no return path.
- **establishment:** sixteen 128-byte branch slots in one aligned page; TPIDR_EL2 holds original x0; guard is read/taken before frame or stack access; fixed repr(C) offsets and size are compile-time checked; captured SP is retained before switch to W02 boot-stack top; Rust receives one initialized frame.
- **failure-class:** FC-INVARIANT terminal; recursive entry after guard claim silently stops, while inaccessible entry code/guard remains outside containment.
- **authorizing-design:** [W05 design](../stages/p1/implementation/p1-w05-el2-exception-entry-baseline/README.md), [preflight](../stages/p1/implementation/p1-w05-el2-exception-entry-baseline/00-preflight-amendment.md) and [reconciliation](../stages/p1/implementation/p1-w05-el2-exception-entry-baseline/06-implementation-reconciliation.md).
- **owner:** P1-W05.
- **review-record:** independent root review, 2026-09-25: all original GPRs and SP, guard-before-store, frame validity, stack alignment, non-return and linker layout accepted.
- **validation:** [W05 verification](../stages/p1/verification/p1-w05-el2-exception-entry-baseline-verification.md); static link inspection and host tests pass, known-register/recursive fault injection deferred to W11.
- **audit-status:** author and independent soundness review complete; execution proof pending.
- **permanence:** P1 single-CPU terminal entry; re-audit for SMP, TLS, return or MMU mapping changes.

## History

No superseded or removed entries yet.
