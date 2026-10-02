# Zelyr Hypervisor

**Translation status:** Current
**Translation source:** [English source](README.md)
**Source blob:** `0f18b3e2700977c83d7c204a8f29de3dd1b02137`
**Authority:** 本文是供中文读者使用的译文；[英文原文](README.md)具有权威性。

Zelyr 是以 AArch64 和 Rust 为先的 Type-1 Hypervisor。QEMU `virt` 是参考平台；
Orange Pi 3B／RK3566 是后续的真实硬件目标。仓库包含已完成的 P0 工程基线和
[P1 EL2 启动完成报告](docs/stages/p1/verification/p1-completion-report.md)。
P1 在单个 Host CPU 上启动至稳定的 Non-secure EL2，具有 Host Stage-1 映射、
控制台和有界故障诊断；尚未运行 Guest，也未提供 GIC、动态平台发现或分配服务。

从 [AGENTS.md](AGENTS.md) 和[文档索引](docs/README.zh-CN.md)开始。贡献者与 agent
在开展非简单工作前必须阅读这两份文档；仅阅读本 README 并不授权修改代码或架构。

## 在 QEMU 上构建和启动

在仓库根目录执行。需要安装 `rust-toolchain.toml` 固定的 Rust 工具链、Python 3、
LLVM（`llvm-objcopy`）和 `qemu-system-aarch64`；参见
[工具链基线](docs/development/toolchain-baseline.md)。

```sh
cargo build --target aarch64-unknown-none-softfloat -p hypervisor
scripts/p1-image --output target/p1/manual-boot.img
qemu-system-aarch64 -machine virt,virtualization=on,gic-version=3 \
  -cpu cortex-a57 -smp 1 -m 128M -display none -monitor none \
  -serial stdio -kernel target/p1/manual-boot.img
```

这是供人工观察的交互式启动。出现 `ZELYR P1 STABLE` 后，当前 P2 路径还会输出
intake、discovery 和 boot-map 结果。按 Ctrl-C 停止 QEMU。镜像转换器添加必要的
ARM64 Image 头并记录来源；它拒绝覆盖已有镜像或配套记录，因此重新构建时请选择
新的输出名称。

## QEMU 验证

使用现有自动化入口保存证据并限制执行时间：

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

重复运行时使用新的证据目录。P1 命令检查 P1 标记，不证明 P2 完成。P2 验证配方
覆盖各自工作包的限定范围；W03 还需要 AArch64 GNU binutils。这些检查不证明
硬件、Guest 或整个 P2 阶段完成。场景输入和证据边界参见
[P1 故障验证配方](docs/stages/p1/verification/p1-w11-negative-fault-validation-verification.md)
和 [P2 W03 验证记录](docs/stages/p2/verification/p2-w03-boot-memory-map-ownership-verification.md)。

保留的脚本各有用途：

- `p1-image`：QEMU 配方所需的镜像转换与来源记录。
- `qemu-runner` / `p1_runner.py`：共享的 QEMU 进程、超时与证据管理。
- `p1-boot-regression`：重复启动判定；`p1-w11-verify`：成对故障检查。
- `p1_w10_marker_control.S` 和 `test_p1_*.py`：验证夹具及工具测试。
- `check-doc-translations.py`：CI 文档翻译检查。

## 顶层布局

```text
docs/        架构、治理、阶段工作与验证记录
crates/      Host 测试基线及预留的可复用 crate 空间
hypervisor/  P1 AArch64 EL2 裸机镜像
soc/         预留的 SoC 支持
boards/      预留的板卡／BSP 组合及 quirk
guests/      预留的验证 Guest
control/     预留的 Control／Service Domain 组件
scripts/     可重复的开发和 CI 入口
tests/       Host、QEMU 和集成测试支持
.github/     CI 工作流
```

部分目录仍只是带有 `.gitkeep` 的 P0 占位目录；名称不代表功能已经实现。
修改代码仍需对应阶段已批准的详细设计及仓库编码指南。

## 许可证

Zelyr 使用 [Apache License 2.0](LICENSE)。

## 贡献流程

政策生效后的开发在新分支上进行，且只有配置的必需在线检查通过后才能经 GitHub PR
并入 `main`。参见[分支与 PR 集成流程](docs/development/integration-workflow.md)。
