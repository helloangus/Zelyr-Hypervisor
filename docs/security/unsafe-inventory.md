# Zelyr Unsafe Inventory

**Status:** Normative register.
**Version:** v0.9 — adds the independently reviewed default-off W11 NC3/NC5 fault instruction boundaries.
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

### U-010 — reference early-console volatile MMIO

- **status:** accepted
- **title:** closed PL011 DR/FR/CR read/write boundary
- **boundary-category:** `mmio-volatile`
- **location:** `hypervisor/src/boot/console.rs`, `read_register` and `write_register`
- **necessity:** Rust cannot safely access physical device registers with volatile semantics through ordinary references
- **safety-preconditions:** W01 canonical QEMU virt PL011 at `0x0900_0000`, 32-bit aligned register offsets in a 4 KiB window, single masked boot CPU writer; W08 preserves the window as Device-nGnRE after MMU enablement
- **establishment:** fixed base and closed offsets in W06, W01 boot premise, W08 mapping obligation; no external input chooses register addresses
- **failure-class:** FC-PLATFORM for a false reference-device premise; W09 routes init readback failure and does not continue normally
- **authorizing-design:** [W06 channel contracts](../stages/p1/implementation/p1-w06-early-console-logging/02-code-contracts-channel.md) and [reconciliation](../stages/p1/implementation/p1-w06-early-console-logging/05-implementation-reconciliation.md)
- **owner:** P1-W06
- **review-record:** `/root/integration_audit`, 2026-09-25: independently checked fixed base, closed aligned offsets, volatile read/write, UARTCR preservation and readback, single-CPU writer, and W08 Device-nGnRE premise; no soundness defect found; see W06 verification record
- **validation:** target compilation and source review in [W06 verification](../stages/p1/verification/p1-w06-early-console-logging-verification.md); QEMU/post-MMU exercise belongs to W09–W11
- **audit-status:** author and independent soundness review complete; integrated execution pending W09–W11
- **permanence:** P1 reference transport only; re-audit at W08 mapping and P2 discovery replacement

### U-011 — W07 fatal-context register reads

- **status:** accepted
- **title:** closed CurrentEL and handler-entry SP/LR capture
- **boundary-category:** `arch-register`
- **location:** `hypervisor/src/boot/fatal.rs` (`current_el`) and `hypervisor/src/boot/panic.rs` (`p1_panic`)
- **necessity:** Rust has no safe primitive to capture these EL2 execution coordinates at the terminal entry.
- **safety-preconditions:** W01 established AArch64 EL2 and a valid boot stack; W07 report guard is held before CurrentEL; the panic handler captures SP/LR before calling the report body.
- **establishment:** fixed read-only asm instructions with explicit x9/x10 outputs for SP/LR, avoiding accidental overwrite of x30; no memory/stack writes or hidden clobbers; the values are labeled handler-entry approximations, not the panic call-site state.
- **failure-class:** FC-INVARIANT terminal if execution premise fails; before W05 vectors the documented unowned-window limit remains.
- **authorizing-design:** [W07 design](../stages/p1/implementation/p1-w07-fatal-crash-diagnostics/README.md) and its [reconciliation](../stages/p1/implementation/p1-w07-fatal-crash-diagnostics/00-implementation-reconciliation.md).
- **owner:** P1-W07.
- **review-record:** `/root/validation_audit`, 2026-09-25: independently reviewed explicit x9/x10 outputs, handler-entry provenance, side-effect-free CurrentEL read, and W05/W07 cross-path guard discipline; no soundness defect found. The review required explicit FC-INVARIANT comments and accurate clobber wording, now applied.
- **validation:** [W07 verification](../stages/p1/verification/p1-w07-fatal-crash-diagnostics-verification.md); host formatter tests and target build/Clippy pass; EL2 fault execution pending.
- **audit-status:** author and independent soundness review complete; integrated execution pending W08–W11.
- **permanence:** P1 terminal report boundary; re-audit for compiler entry-layout, SMP or report-source changes.

### U-012 — W08 EL2 Stage-1 register and instruction boundary

- **status:** accepted
- **title:** closed MAIR/TTBR0/TCR/SCTLR access and activation barriers
- **boundary-category:** `arch-register`
- **location:** `hypervisor/src/arch/aarch64/stage1/regs.rs`, all small register and instruction wrappers
- **necessity:** safe Rust cannot encode EL2 `msr`, `mrs`, `dsb`, `isb`, `tlbi alle2` or `ic iallu`.
- **safety-preconditions:** W01-established EL2 and one DAIF-masked boot CPU; W03 required 4 KiB/PA facts, W04 baseline, W05 vectors, W06 console and W07 fatal route complete; W08 verified five-page tables and fixed control values before the one-time enable.
- **establishment:** W09 will be the sole caller of the single-shot W08 entry; W08 checks the declared predecessors, verifies and copies the full tables, uses explicit asm operands without `nomem` on control writes/barriers, and follows the W08 preflight `DSB SY`/TLBI/I-cache/ISB order. A second entry is rejected.
- **failure-class:** FC-INVARIANT terminal via W09 Stage1 failure route or W05/W07 vector path if an architectural fault occurs mid-transition.
- **authorizing-design:** [W08 design](../stages/p1/implementation/p1-w08-host-stage1-address-space/README.md), [preflight](../stages/p1/implementation/p1-w08-host-stage1-address-space/00-preflight-amendment.md) and [activation reconciliation](../stages/p1/implementation/p1-w08-host-stage1-address-space/06-activation-reconciliation.md).
- **owner:** P1-W08.
- **review-record:** `/root/w08_soundness`, 2026-09-25: independently reviewed closed asm scope, barrier/control order, MAIR/TCR/XN and AP encoding against Arm-maintained reference definitions; identified missing EL2 AP[1] RES1 bit, then re-reviewed and accepted the corrected six-class encoding and U-012 boundary.
- **validation:** [W08 activation verification](../stages/p1/verification/p1-w08-mmu-activation-verification.md); target build/Clippy and six host W08 tests pass; actual MMU execution belongs to W09/W10/W11 integration.
- **audit-status:** author and independent soundness review complete; integrated execution pending.
- **permanence:** P1 fixed reference Stage-1 transition; re-audit for register policy, P2 remapping or SMP.

### U-013 — W08 linker-bound address inventory

- **status:** accepted
- **title:** addresses of twelve fixed linker region bounds
- **boundary-category:** `memory-mgmt`
- **location:** `hypervisor/src/arch/aarch64/stage1/mod.rs`, linker-symbol `unsafe extern` declaration and `inventory`
- **necessity:** linker-defined mapping bounds cannot be declared as Rust-owned statics; their addresses are required to construct the exact Stage-1 inventory.
- **safety-preconditions:** the W08 linker script uniquely defines page-separated symbols; no declared `u8` is dereferenced; the symbol addresses remain in the fixed P1 image window.
- **establishment:** `addr_of!` obtains addresses only, the pure table model rejects unaligned, overlapping, reversed or out-of-window bounds, and the vector base is checked against the vector bound before activation.
- **failure-class:** FC-INVARIANT terminal on a false linker/layout premise; unmapped execution would enter W05/W07 if the premise escaped checks.
- **authorizing-design:** [W08 page-layout record](../stages/p1/implementation/p1-w08-page-layout-record.md) and [activation reconciliation](../stages/p1/implementation/p1-w08-host-stage1-address-space/06-activation-reconciliation.md).
- **owner:** P1-W08.
- **review-record:** `/root/w08_soundness`, 2026-09-25: independently reviewed address-only extern declarations, checked linker-bound inventory and memory-management classification; accepted U-013, with final linked table/sentinel placement reserved for W09 integration.
- **validation:** [W08 activation verification](../stages/p1/verification/p1-w08-mmu-activation-verification.md); merged linker section/bound evidence exists, integrated table placement remains pending W09.
- **audit-status:** author and independent soundness review complete; integrated ELF placement and execution pending.
- **permanence:** P1 fixed linker inventory; re-audit on image layout or relocation changes.

### U-014 — W08 forced post-MMU read-only sentinel

- **status:** accepted
- **title:** volatile load of an in-image read-only sentinel
- **boundary-category:** `memory-mgmt`
- **location:** `hypervisor/src/arch/aarch64/stage1/mod.rs`, `post_mmu_checks`
- **necessity:** an ordinary read may be folded to a constant and fail to test the RoData mapping after MMU enablement; one volatile read forces the access.
- **safety-preconditions:** the sentinel is a valid aligned `u64` in the linker RoData region and that region is mapped read-only before the access.
- **establishment:** W08 builds and verifies the linker-bounded RoData page descriptors before setting SCTLR.M; the read occurs only after the enable ISB; no pointer arithmetic or write is performed.
- **failure-class:** FC-INVARIANT terminal; a bad mapping faults through the armed W05/W07 path rather than returning success.
- **authorizing-design:** [W08 transition contracts](../stages/p1/implementation/p1-w08-host-stage1-address-space/03-code-contracts-mapping-and-transition.md) §5 and [activation reconciliation](../stages/p1/implementation/p1-w08-host-stage1-address-space/06-activation-reconciliation.md).
- **owner:** P1-W08.
- **review-record:** `/root/w08_soundness`, 2026-09-25: independently reviewed aligned immutable sentinel, one bounded volatile read and the verified RoData-map precondition; accepted U-014, with emitted load and post-MMU execution reserved for W09 integration.
- **validation:** [W08 activation verification](../stages/p1/verification/p1-w08-mmu-activation-verification.md); target compilation is not post-MMU execution evidence, which W09/W10/W11 must collect.
- **audit-status:** author and independent soundness review complete; emitted-access and execution evidence pending.
- **permanence:** P1 post-enable sentinel; re-audit if the mapping check changes or is removed.

### U-015 — P1 stable idle WFI instruction

- **status:** accepted
- **title:** one AArch64 `wfi` instruction inside the stable boot-CPU idle loop
- **boundary-category:** `arch-register`
- **location:** `hypervisor/src/arch/aarch64/idle.rs`, `wait_for_interrupt`
- **necessity:** stable Rust has no safe intrinsic for the W02-designed AArch64 WFI idle operation.
- **safety-preconditions:** W01-established EL2 boot CPU, W04-maintained DAIF mask, W09 `Stable` transition completed; WFI has no memory or stack operand. Normal completion returns to the W02 loop; EL3 firmware may trap it under `SCR_EL3.TWI`, which P1 does not control.
- **establishment:** W02 glue invokes `controlled_idle` only after `run_init_sequence` returns and `enter_stable` succeeds; that loop is the one arch wrapper's sole P1 call site.
- **failure-class:** FC-INVARIANT terminal on a false execution-level or lifecycle premise; an EL2 exception taken through the installed vectors after Stable is routed by W05/W07. EL3 traps are firmware-owned and not claimed by this route.
- **authorizing-design:** [W02 controlled-idle reconciliation](../stages/p1/implementation/p1-w02-minimal-rust-el2-runtime/06-controlled-idle-reconciliation.md), [W02 runtime contracts](../stages/p1/implementation/p1-w02-minimal-rust-el2-runtime/03-code-contracts-rust-runtime.md) §5 and [W09 state machine](../stages/p1/implementation/p1-w09-initialization-sequencing/01-init-state-machine.md) §2.
- **owner:** P1-W09 integration of the W02 deferred seam.
- **review-record:** `/root/w09_unsafe_review`, 2026-09-25: independently reviewed the W02 detailed-design correction, unique Stable-only call path, masked-DAIF premise, WFI asm options and EL3 firmware-trap limitation; accepted the U-015 instruction boundary. Required precise EL2-only exception wording and the linked W09 verification record, both added before merge.
- **validation:** [W09 verification](../stages/p1/verification/p1-w09-initialization-sequencing-verification.md); target build/Clippy and one W10-runner canonical QEMU Stable boot pass; real-hardware power state, wake and EL3 trap behavior are not proven.
- **audit-status:** author and independent soundness review complete; hardware behavior remains out of scope.
- **permanence:** P1 boot CPU idle; firmware WFI trapping and power-state behavior remain reference-environment limitations; re-audit for interrupt delivery, SMP or scheduler idle.

### U-016 — W11 NC3 intentionally undefined instruction

- **status:** accepted
- **title:** one permanently undefined AArch64 instruction in the selected NC3 validation image
- **boundary-category:** `asm-glue`
- **location:** `hypervisor/src/arch/aarch64/validation_fault.rs`, `fault_scenario(Nc3)`
- **necessity:** safe Rust cannot emit a fixed architecturally undefined encoding to exercise the real W05 exception entry
- **safety-preconditions:** W01 EL2 boot CPU, W05 vectors and W07 fatal path established; DAIF remains masked; only the NC3 image contains the call
- **establishment:** W09's call is after `FatalPath.complete`; the architecture module is feature-gated and emits exactly `.inst 0` with no operands or stack effects; a static panic is the unexpected-return fallback
- **failure-class:** FC-INVARIANT if the execution/exception premise is false; terminal W05/W07 exception path or fallback panic
- **authorizing-design:** [W11 trigger reconciliation](../stages/p1/implementation/p1-w11-negative-fault-validation/04-trigger-reconciliation.md)
- **owner:** P1-W11
- **review-record:** `/root/w11_unsafe_review`, 2026-09-25: independently accepted the minimal `.inst 0` boundary, exact W09 post-arm insertion, and Rust-to-W05 exception-entry glue classification. ESR.EC 0 and W05's `cls=unknown` require raw-syndrome checking in executed evidence; static soundness review is not fault-execution proof.
- **validation:** [W11 verification](../stages/p1/verification/p1-w11-negative-fault-validation-verification.md); target build and two scenario-verdict QEMU runs observed ESR.EC `0x00` and a terminal W05/W07 report on the local unmerged branch; hardware behavior is not claimed
- **audit-status:** author and independent soundness review complete; local paired QEMU execution complete, final integrated S1–S6 review pending
- **permanence:** validation-image only; absent from the default image

### U-017 — W11 NC5 post-MMU unmapped-address probe

- **status:** accepted
- **title:** one AArch64 load targeting an intentionally unmapped Stage-1 L2 entry
- **boundary-category:** `memory-mgmt`
- **location:** `hypervisor/src/arch/aarch64/validation_fault.rs`, `fault_scenario(Nc5)`
- **necessity:** the Rust volatile-read contract and emitted instruction cannot be unconditionally established for this deliberately unmapped address and W05 terminal-handler route; one fixed assembly load tests hardware translation without constructing a Rust reference
- **safety-preconditions:** W08 Stage-1 enabled successfully; W05 vectors and W07 fatal path established; `0x5000_0000` remains absent from W08's verified image/console L2 entries; only NC5 image contains the call
- **establishment:** W09's call is after `Stage1.complete`; W08's fixed three-level table verifies invalid holes; assembly uses one typed-VA-derived address operand and no `nomem` option; unexpected return reaches a static fallback panic
- **failure-class:** FC-INVARIANT if the mapping/exception premise is false; terminal W05/W07 exception path or fallback panic
- **authorizing-design:** [W11 trigger reconciliation](../stages/p1/implementation/p1-w11-negative-fault-validation/04-trigger-reconciliation.md)
- **owner:** P1-W11
- **review-record:** `/root/w11_unsafe_review`, 2026-09-25: independently accepted the single-load assembly boundary, no `nomem`/`readonly`/`pure` claim, W08 invalid-hole premise and W09 post-MMU insertion. This is static soundness review, not P1-V18 runtime evidence.
- **validation:** [W11 verification](../stages/p1/verification/p1-w11-negative-fault-validation-verification.md); target build and two scenario-verdict QEMU runs observed ESR.EC `0x25`, FAR `0x5000_0000`, and a terminal W05/W07 report on the local unmerged branch; hardware behavior is not claimed
- **audit-status:** author and independent soundness review complete; local paired QEMU execution complete, final integrated S1–S6 review pending
- **permanence:** validation-image only; absent from the default image

## History

No superseded or removed entries yet.
