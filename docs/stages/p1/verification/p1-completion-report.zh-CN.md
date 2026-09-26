# P1 完成报告 — AArch64 EL2 最小启动

**Translation status:** Current
**Translation source:** [English source](p1-completion-report.md)
**Source blob:** `d236a1e039b81dd107d7ec288c58a78b29e5fd84`
**Authority:** 本文是供中文读者使用的译文；[英文原文](p1-completion-report.md)具有权威性。

**状态：** P1 在声明的参考 QEMU 范围内的阶段完成决策。
**范围：** P1 任务书 v0.2 的门禁与七项退出条件；不宣称 P2、P6 或真机已完成。
**版本：** v0.1。
**所有者／变更背景：** P1 L7 阶段评审，2026-09-26（亚洲／上海），ADR-061 之后。
**替代：** 不替代工作包验证记录；历史 v0.1 NC6 发现保持原样。
**依据：** [当前任务书](../task-book-v0.2.zh-CN.md)、[ADR-061](../../../adr/adr-061-defer-p1-asynchronous-vector-validation-to-p6.md)、[阶段证据图](../contracts/stage-gate-evidence-map.md)及 W01–W12 实施与验证记录。

## 决策与证明边界

**P1 已在声明的单启动 CPU QEMU `virt` 参考范围内完成。** 下述评审认为 P1-V01–P1-V21
以及全部七项退出条件都有证据。本决策不声称真机运行、其他固件／CPU 配置、永久恒等映射 ABI、
IRQ/GIC 服务，或已执行的异步 IRQ/FIQ/SError 投递。尤其是，原 NC6 场景**没有通过**：
[ADR-061](../../../adr/adr-061-defer-p1-asynchronous-vector-validation-to-p6.md)要求在 Host GIC/IRQ
就绪后由 P6-W12/P6-V29 真正执行异步事件。P6 不能从本 P1 决策推断出该证据。

全部 12 个工作包均有 L5 实施记录和 L6 验证记录；W08、W09 另有基础记录。
[实施索引](../implementation/README.md)链接到其所有者。较早的工作包记录有意描述当时的延期工作。
合并后的 W09–W12 记录、ADR-061 治理补充与下述新集成运行才是后续证据；
不能把旧结果改写成当时已经通过。

## 新的集成参考运行

源码是通过 PR #59 合并到 `main` 的 `ccd6fc0e09da1385afd3676c6be63bafd6650560`；
`hypervisor/`、`scripts/` 和 Host 测试下的现有代码最后一次改动在已合并的 W11 实施。
本次重建的默认及 NC2–NC5 镜像与 W11 集成验证具有**相同 SHA-256 值**，因此无需作代码差异推断。
Host：Linux WSL2 `6.18.33.2-microsoft-standard-WSL2` x86_64；QEMU AArch64 8.2.2；
固定 Rust 1.98.1；一个 `virt` 启动 CPU、128 MiB 及已记录的 W10 profile。
所有 QEMU 启动均由正式的 `scripts/qemu-runner` 入口负责。原始采集目录是被忽略的构建树路径
`.worktrees/p1completion/target/p1-completion-evidence/`。其内容现在保存在
[本地证据归档](p1-local-evidence-archive.md)，原有成员路径不变。该归档**不是**持久 CI 制品或远端备份。

| 运行 | 镜像 SHA-256 | 结果及保留证据 |
|---|---|---|
| 正常 100 次循环回归 | `f9895fa0e365631160fa6b38edf5583b8de77b1609187995c672ad44ce79e152` | `requested=100`、`counted=100`，全部 `PASS`；每份串口记录恰好一个 Stable，且无 panic/fatal/reject 标记。`r100/summary.txt`、`r100/cycle-*/{invocation,outcome,serial,emulator}.*`。 |
| NC1 无 EL2 | 同一默认镜像 | 2/2 成对判定通过；每次运行恰好一次明确的 `reason=EL` 拒绝，不正常继续；保留 `nc1/summary.json` 与 `run-{1,2}/` 采集。 |
| NC2 必需能力样本 | `8791edf9d2040fbdea826c77d257a915e0e0d4f137f87bdcf329165b7d2ab5e7` | 2/2 通过；在 `capabilities.enter` 中拒绝必需的 4-KiB granule 样本。属于**样本替换**，不是观测到 CPU 硬件缺少该能力。 |
| NC3 同步未定义指令 | `78f094be85fdfb40499573815069e59a89f6176369340cf9346b78a9710129dd` | 2/2 通过；ESR／PC／阶段和一个有界终止报告。 |
| NC4 有意 panic | `58caf44e10cda431db48a40f945832c94780f1ec85182541ed5e2e295fe12620` | 2/2 通过；panic 身份／位置／阶段及一个有界终止报告。 |
| NC5 MMU 后转换故障 | `62349bf7195e3a0db6dc05ba1f25fc1cd9d21104ff4d4f967c8587bbf1774ad0` | 2/2 通过；ESR.EC `0x25`、FAR `0x5000_0000`、`stage1.complete`、相关现场和一个有界终止报告。 |

五份 `nc*/summary.json` 均写明 `passed=true`，各含两次一致运行，并对照调用记录验证镜像哈希。
另一次只读审计检查了全部 100 份 `outcome.json` 与 `serial.log`：100 个状态 0 的结果，
每循环恰好一个 Stable 标记，且没有 panic/fatal/reject 标记。重复的参考镜像还佐证 W10 较早接受的
[R1–R6 与 100 次循环记录](p1-w10-qemu-boot-regression-verification.md)，包括 NC4 最终的 R2 panic 控制。

新的本地质量检查均通过：`cargo fmt --all -- --check`；Host
`cargo test --workspace --exclude hypervisor`（34 项测试，均未失败或忽略）；Host 和 AArch64 目标
Clippy 均使用 `-D warnings`；AArch64 默认和 NC2–NC5 构建；以及 24 项 Python runner／判定测试。
这些检查不能替代上述 QEMU 证据。未运行目标单元测试框架、真机、硬件故障注入、Guest 或 P6 异步事件。

## P1-V01–P1-V21 评审

下文“已验证”指的是**声明的 P1 范围内的任务书条件**，不是无边界的硬件或平台声明。
表中链接主要 L6 记录；[证据图](../contracts/stage-gate-evidence-map.md)列出更细的限制。

| 门禁 | 决策与主要证据 |
|---|---|
| P1-V01 | **已验证：** [W01](p1-w01-reference-boot-contract-verification.md) 的规范镜像／入口契约；[W10](p1-w10-qemu-boot-regression-verification.md) 正常参考启动及新 100 次循环。 |
| P1-V02 | **已验证：** [W11 NC1](p1-w11-negative-fault-validation-verification.md) 和新 NC1 成对运行中，同一默认镜像在无 EL2 时 2/2 被拒绝。 |
| P1-V03 | **已验证：** [W02](p1-w02-minimal-rust-el2-runtime-verification.md) 评审栈／BSS／上下文／panic／身份顺序，[W09](p1-w09-initialization-sequencing-verification.md) 验证集成的入口到 Stable 有序路径。 |
| P1-V04 | **已验证：** [W02](p1-w02-minimal-rust-el2-runtime-verification.md) 源码评审隐藏寄存器依赖；[W10](p1-w10-qemu-boot-regression-verification.md) 和新运行均为 100/100 Stable。未变更固件进行验证。 |
| P1-V05 | **已验证：** [W03](p1-w03-aarch64-capability-inventory-verification.md) 的有界能力分类／Host 测试；[W09](p1-w09-initialization-sequencing-verification.md) 的一次集成启动 CPU 报告。 |
| P1-V06 | **已验证：** [W03](p1-w03-aarch64-capability-inventory-verification.md) 的必需／可选策略测试，[W11](p1-w11-negative-fault-validation-verification.md) 与新成对运行的 NC2 样本拒绝 2/2。不声称硬件缺少该能力。 |
| P1-V07 | **已验证：** [W04](p1-w04-el2-architectural-state-baseline-verification.md) 评审所有权明确的控制项、掩码、保护、回读与顺序；[W09](p1-w09-initialization-sequencing-verification.md) 集成基线启动。不声称逐循环寄存器转储。 |
| P1-V08 | **结构上已验证：** [W05](p1-w05-el2-exception-entry-baseline-verification.md) 评审全部 16 个入口槽和有界分类／现场契约；异步投递执行仍属 P6-V29。 |
| P1-V09 | **已验证：** [W11](p1-w11-negative-fault-validation-verification.md) 与新成对运行中的 NC3／NC5 同步 ESR／位置／终止证据。 |
| P1-V10 | **已验证：** [W06](p1-w06-early-console-logging-verification.md)、[W09](p1-w09-initialization-sequencing-verification.md) 的早期回放／实时标记；新正常及 NC2–NC5 采集可按阶段归因。向量安装前的输出仍受限。 |
| P1-V11 | **对适用的致命类别已验证：** [W07](p1-w07-fatal-crash-diagnostics-verification.md) 的有界报告字段，[W11](p1-w11-negative-fault-validation-verification.md) 与新成对运行对 NC2–NC5 的字段检查。panic 使用 N/A syndrome，不伪造 ESR。 |
| P1-V12 | **在已执行路径内验证：** [W07](p1-w07-fatal-crash-diagnostics-verification.md) 的不递归保护评审，[W11](p1-w11-negative-fault-validation-verification.md) 与新成对运行中 NC2–NC5 恰好一个 END 且不继续。未进行递归物理故障压力测试。 |
| P1-V13 | **已验证：** [W08](p1-w08-mmu-activation-verification.md) 的封闭映射类别／权限和链接放置；[W11 S2](p1-w11-negative-fault-validation-verification.md) 的无 W+X 审计。恒等 VA=PA 仍为临时措施。 |
| P1-V14 | **已验证：** [W08](p1-w08-mmu-activation-verification.md)／[W09](p1-w09-initialization-sequencing-verification.md) 映射后的启动，[W11](p1-w11-negative-fault-validation-verification.md) 与新成对运行中 MMU 后 NC5 的 ESR／FAR／阶段及诊断路径。 |
| P1-V15 | **已验证：** [W09](p1-w09-initialization-sequencing-verification.md) 的状态机／顺序／失败路径源码及 Host 评审与有序启动；新正常／负面采集佐证 Stable 与阶段归因。 |
| P1-V16 | **已验证：** [W10](p1-w10-qemu-boot-regression-verification.md)、[W11](p1-w11-negative-fault-validation-verification.md) 的 R1–R6 控制（包括基于 NC4 的最终 R2）；新 runner／判定测试通过。 |
| P1-V17 | **已验证：** [W10](p1-w10-qemu-boot-regression-verification.md) 已接受 100/100，加上保存采集的新 100/100 同镜像参考运行。 |
| P1-V18 | **按 ADR-061 已验证：** [W11](p1-w11-negative-fault-validation-verification.md) 和新成对运行的 NC3、NC4、NC5 故障诊断。NC6 不属于修订后的 P1 门禁，在 P6-V29 仍为开放项。 |
| P1-V19 | **对已评审基线验证：** [W11 S1–S6](p1-w11-negative-fault-validation-verification.md) 覆盖输入／范围、无 RWX、unsafe 清单 U-016/U-017、阶段边界、故障路径控制与默认触发隔离。镜像哈希相同、代码自 W11 后未变，因此该评审在此仍适用。 |
| P1-V20 | **已验证：** [W12](p1-w12-p1-documentation-handoff-verification.md) 的八项契约一致性和 P2 可消费性，已按 ADR-061 更新；本完成评审检查修订后的移交与限制。 |
| P1-V21 | **已验证：** [W12](p1-w12-p1-documentation-handoff-verification.md) 的计划／依赖图／链接／证据图评审、当前 12 个计划与 21 个 ID 枚举，以及本次 L7 无过度声明评审。 |

## 七项退出条件及必需的 L7 问题

| 任务书第 7 节条件 | 决策 |
|---|---|
| 1. 稳定的 Non-secure EL2 Rust 入口 | 由 V01／V03／V04／V15 和新 100/100 参考启动满足。 |
| 2. 能力与 EL2 基线 | 由 V05–V07 满足；NC2 是受控的必需事实样本，不是物理上缺少某功能。 |
| 3. MMU 前后的向量和诊断 | 在已安装向量及已执行的同步／致命范围内由 V08–V12／V14／V18 满足；异步投递属于 P6。 |
| 4. 映射类别及临时恒等映射 | 由 V13／V14／V19 满足；不提供永久恒等映射 ABI。 |
| 5. 自动判定及 100 次干净启动 | 由 V16／V17 满足；记录两组 100/100 和有界 R1–R6 控制。 |
| 6. 负面／故障路径 | NC1–NC5 与结构性向量覆盖分别满足修订后对应门禁；NC6 仍属 P6-V29。 |
| 7. 契约、报告及移交 | 由 V19–V21 和链接的契约集合满足；P2 仍负责发现和分配。 |

代码／评审消除的入口假设：Rust 转交前检查 CurrentEL／EL2 和非零 DTB 指针，
必需能力策略快速失败，W04 明确建立其所有的 EL2 控制项。剩余参考假设：QEMU `virt`／PL011、
一个 CPU、固定镜像／表／栈位置、入口时 MMU／缓存关闭、不解析的 DTB 内容、固件 WFI 行为，
以及一个支持的 CPU 模型；见[已知限制](../contracts/known-limitations.md)。
致命路径保留构建／CPU／EL 与阶段；同步异常还保留相关 syndrome、PC 和 FAR 有效性；
panic／阶段失败把不可用字段标为不可用，而不编造。向量安装前的窗口没有保证由 Hypervisor 拥有的路径。
Host Stage-1 的 VA=PA 窗口只是启动选择，**不是** ABI。[W11 S4](p1-w11-negative-fault-validation-verification.md)
及 W11 以来未变的代码确认 Guest、SMP、GIC、平台发现、分配器和板卡运行时机制未进入 P1。

## 开放项与移交

| 开放项 | 所有者／消费者 | 对本决策的影响 |
|---|---|---|
| 真正的意外异步 EL2 向量（历史 NC6）尚无执行证明 | ADR-061 下的 P6-W12／P6-V29 | 不是 P1-v0.2 门禁；在得到真实证据前**阻止 P6-V29**，且不得引用 P1 完成作为通过证明。 |
| DTB 内容、物理 RAM／保留区、动态分配 | P2，见 [P2 移交](../contracts/p2-handoff.md) | P1 未实施或验证。 |
| 真机、其他固件／CPU、硬件故障及递归故障压力测试 | 后续平台／稳健性验证 | 不属于本参考 QEMU 完成声明。 |
| 原始 QEMU 采集是本地被忽略的构建制品 | 本地证据保管者 | [归档](p1-local-evidence-archive.md)保留清理掉的 worktree 之外的原采集；若需要持久保存，应维护异机副本。单靠 CI 必需检查不能重建这些采集。 |

P2 可消费的材料包括本报告、[任务书 v0.2](../task-book-v0.2.zh-CN.md)、
[八项 P1 契约](../contracts/README.md)、[阶段证据图](../contracts/stage-gate-evidence-map.md)，
以及上述 W01–W12 实施／验证记录。它提供稳定的参考 EL2 启动 CPU、诊断与 Host Stage-1 运行时，
**不提供**已发现的平台、内存分配器、Guest 或 IRQ 服务。
