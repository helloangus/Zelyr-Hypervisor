# Zelyr Hypervisor

Zelyr is an AArch64-first, Rust-first Type-1 Hypervisor. QEMU `virt` is the
reference platform; Orange Pi 3B/RK3566 is a later real-hardware target.
The repository contains the completed P0 engineering baseline and the
[P1 EL2 bring-up completion report](docs/stages/p1/verification/p1-completion-report.md).
P1 boots a single Host CPU to stable Non-secure EL2 with Host Stage-1 mapping,
console and bounded fault diagnostics; it does not yet run a Guest or provide
GIC, dynamic platform discovery or allocation services.

Start with [AGENTS.md](AGENTS.md) and the [documentation index](docs/README.md).
Chinese readers can start with the [Chinese documentation index](docs/README.zh-CN.md).
The [Chinese edition of this page](README.zh-CN.md) is also available.
Contributors and agents must read both before non-trivial work; reading this
README alone does not authorize code or architecture changes.

## Top-level layout

```text
docs/        architecture, governance, stage work, and verification records
crates/      host-test baseline and reserved reusable-crate space
hypervisor/  P1 bare-metal AArch64 EL2 image
soc/         reserved SoC-specific support
boards/      reserved board/BSP composition and quirks
guests/      reserved validation guests
control/     reserved Control/Service Domain components
scripts/     reproducible developer and CI entry points
tests/       host, QEMU, and integration test support
.github/     CI workflows
```

Some directories remain P0 placeholders with `.gitkeep`; names do not imply
implemented functionality. Code changes still require their owning stage's
approved detailed design and the repository's coding guidance.

## License

Zelyr is licensed under the [Apache License 2.0](LICENSE).

## Contribution flow

Post-policy development is performed on a new branch and reaches `main` only
through a GitHub pull request whose configured required online checks pass.
See the [branch and pull-request integration workflow](docs/development/integration-workflow.md).
