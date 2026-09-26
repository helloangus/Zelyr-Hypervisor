# P1-W08 constant table initialization correction

**Status:** Verified bounded correction; no expansion of P1 completion scope.
**Scope:** Debug-build boot-stack use during temporary table construction.
**Version:** v0.1
**Owner/change context:** P1-W08 follow-up found during P2 integration, 2026-09-26.
**Supersedes:** No historical P1 evidence.

P2's first post-stable read found the W08 `STARTED` flag false. Inspection of
the linked debug image showed approximately 32 KiB in `enable_host_stage1`,
28 KiB in `Tables::new`, another 4 KiB in `Table::zeroed`, plus page alignment
and callers. That construction path can overrun the 64 KiB boot stack into
adjacent BSS. A stable marker alone did not check this flag after construction.

The correction evaluates `Tables::new()` in a const block. The zero table
value is compiled into the image and copied into the caller's existing local;
the nested runtime constructor frames disappear. The table contents, size,
attributes, and ownership remain W08's existing contract. The post-MMU check
now rejects a clobbered startup flag through W09's existing terminal route.
No unsafe boundary, external API, dependency or stack-size change is added.

Validation on Rust 1.98.1, local QEMU reference recipe:

- `cargo test --workspace --exclude hypervisor`: all 34 tests passed.
- Host and AArch64 Clippy with `-D warnings`: passed.
- `cargo build --target aarch64-unknown-none-softfloat -p hypervisor`: passed.
- `scripts/p1-image --output target/p1/const-init.img`: image SHA-256
  `df491893348513d8822e0d3581126a91745ff7228811c4fe7cee9f63e3826405`.
- `scripts/qemu-runner run --profile p1-boot-smoke --timeout 8s --param
  boot-smoke=target/p1/const-init.img --evidence-dir
  target/p1/const-init-smoke-ok`: status 0, all normal predicates, no forbidden
  marker; the new startup-flag postcheck passed before stable.
- The initial runner invocation omitted `run`, returned usage status 1 and
  launched no QEMU; the corrected invocation above is the execution evidence.

This is a local constructor correction and reference-QEMU smoke, not a whole
program stack-depth proof, hardware validation, or a rerun of all P1 negative
scenarios. The independently linked P2 integration image also passed the
previously failing flag read with this correction; its evidence belongs to P2.
