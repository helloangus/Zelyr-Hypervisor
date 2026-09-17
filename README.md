# Zelyr Hypervisor

Zelyr is an AArch64-first, Rust-first Type-1 Hypervisor P0 scaffold.  QEMU
`virt` is the reference platform; Orange Pi 3B/RK3566 is the first
real-hardware target.  The repository currently contains the P0 project
scaffold and its normative baseline documents; it does not yet contain a
runnable hypervisor.

Start with [AGENTS.md](AGENTS.md) and the [documentation index](docs/README.md).
Contributors and agents must read both before non-trivial work; reading this
README alone does not authorize code or architecture changes.

## Top-level layout

```text
docs/        architecture, governance, stage work, and verification records
crates/      future reusable Rust crates; boundaries await detailed designs
hypervisor/  future binary composition root
soc/         SoC-specific support
boards/      board/BSP-specific composition and quirks
guests/      validation guests
control/     Control/Service Domain components
scripts/     reproducible developer and CI entry points
tests/       host, QEMU, and integration test support
.github/     CI workflows
```

The source directories are placeholders deliberately created in P0.  Their
detailed contents await later approved designs.  Intentionally reserved empty
directories contain a tracked `.gitkeep` marker, so a fresh clone does not
require manual `mkdir` steps.  Adding code or Cargo manifests requires the
applicable P0/P1 detailed design.

## License

Zelyr is licensed under the [Apache License 2.0](LICENSE).

## Contribution flow

Post-policy development is performed on a new branch and reaches `main` only
through a GitHub pull request whose configured required online checks pass.
See the [branch and pull-request integration workflow](docs/development/integration-workflow.md).
