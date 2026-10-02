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

## Build and boot on QEMU

Run from the repository root. Install the pinned Rust toolchain from
`rust-toolchain.toml`, Python 3, LLVM (`llvm-objcopy`), and
`qemu-system-aarch64`; see the [toolchain baseline](docs/development/toolchain-baseline.md).

```sh
cargo build --target aarch64-unknown-none-softfloat -p hypervisor
scripts/p1-image --output target/p1/manual-boot.img
qemu-system-aarch64 -machine virt,virtualization=on,gic-version=3 \
  -cpu cortex-a57 -smp 1 -m 128M -display none -monitor none \
  -serial stdio -kernel target/p1/manual-boot.img
```

This is an interactive boot for inspection. After `ZELYR P1 STABLE`, the
current P2 path also reports intake, discovery and boot-map results. Stop QEMU
with Ctrl-C. The image converter adds the required ARM64 Image header and
records provenance; it refuses to overwrite an existing image or sidecar, so
choose a fresh output name when rebuilding.

## QEMU verification

Use the existing automation for captured evidence and bounded timeouts:

```sh
scripts/qemu-runner run --profile p1-boot-smoke \
  --param boot-smoke=target/p1/manual-boot.img --timeout 8
scripts/p1-boot-regression --cycles 100 --image target/p1/manual-boot.img
python3 docs/stages/p2/verification/p2-w01-w02-smoke.py \
  --image target/p1/manual-boot.img --output target/p2-smoke-readme
python3 docs/stages/p2/verification/p2-w03-runtime.py \
  --image target/p1/manual-boot.img \
  --elf target/aarch64-unknown-none-softfloat/debug/hypervisor \
  --output target/p2-bootmap-readme
```

Use fresh evidence directories for repeated runs. The P1 commands check P1
markers, not P2 completion. The P2 recipes check their bounded package scopes;
W03 additionally requires AArch64 GNU binutils. These checks do not establish
hardware, Guest, or whole-P2 completion. See the
[P1 fault-validation recipe](docs/stages/p1/verification/p1-w11-negative-fault-validation-verification.md)
and [P2 W03 verification record](docs/stages/p2/verification/p2-w03-boot-memory-map-ownership-verification.md)
for scenario inputs and evidence limits.

Retained scripts have distinct responsibilities:

- `p1-image`: image conversion and provenance, required by QEMU recipes.
- `qemu-runner` / `p1_runner.py`: shared QEMU process, timeout and evidence owner.
- `p1-boot-regression`: repeated boot verdicts; `p1-w11-verify`: paired fault checks.
- `p1_w10_marker_control.S` and `test_p1_*.py`: verification fixture and tooling tests.
- `check-doc-translations.py`: the CI documentation translation gate.

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
