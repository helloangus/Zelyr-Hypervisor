# Zelyr Unsafe Inventory

**Status:** Normative register.  
**Version:** v0.2 — first two entries, created by the P1-W02 runtime change.  
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

## History

No superseded or removed entries yet.
