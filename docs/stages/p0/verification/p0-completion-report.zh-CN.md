# P0 完成报告

**Translation status:** Current
**Translation source:** [English source](p0-completion-report.md)
**Source blob:** `6c35307c4d0609bf4b23334310b6728b57582ed2`
**Authority:** 本文是供中文读者使用的译文；[英文原文](p0-completion-report.md)具有权威性。

**状态：** P0 阶段完成声明；按阶段流程 L7，只有本文有权作此声明。
**日期：** 2026-09-19（亚洲／上海）。
**依据：** 所有 P0 工作包的[实施记录](../implementation/README.md)和[验证记录](p0-w22-stage-dependency-map-verification.md)
（W01–W22）、[P0 移交图](../p0-handoff-map.md)及 [P0 任务书](../task-book-v0.1.zh-CN.md)。

## 1. 验证矩阵状态

根据移交图中的完成报告契约，每个 P0-Vxx ID 的情况如下：

| ID | 验证 | 状态 | 主要证据 |
|---|---|---|---|
| P0-V01 | 全新克隆评审 | **已验证** | [W01 验证](p0-w01-repository-baseline-verification.md)；[W19 全新克隆 walkthrough](p0-w19-reproducible-development-workflow-verification.md)再次演示（一次性克隆，可到达 S0）。 |
| P0-V02 | 工具链恢复 | **已验证** | [W02 验证](p0-w02-rust-toolchain-baseline-verification.md)：隔离沙箱恢复、幂等性和单一来源扫描。 |
| P0-V03 | Host 构建 | **已验证** | [W08 验证](p0-w08-host-side-testing-baseline-verification.md)：入口编译全部 Host 目标成员；CI `QG-TEST-HOST` 在每个 PR 与 `main` 推送上执行。 |
| P0-V04 | Host 测试 | **已验证** | [W08 验证](p0-w08-host-side-testing-baseline-verification.md)：1 通过／0 失败及失败可见性探针；PR #31／#32／#33／#34 上 CI `QG-TEST-HOST` 通过。 |
| P0-V05 | AArch64 目标构建 | **已验证** | [W03 验证](p0-w03-aarch64-build-target-baseline-verification.md)：构建、增量无操作、干净重建及 ELF 标识；CI `QG-BUILD-TARGET` 通过。 |
| P0-V06 | 格式 | **已验证** | [W07 验证](p0-w07-development-quality-gates-verification.md)：`QG-FMT` 试运行；CI `QG-FMT` 通过，且红色探针证明其为**必需**（PR #32：`QG-FMT` 失败阻止合并）。 |
| P0-V07 | Lint | **已验证** | [W07 验证](p0-w07-development-quality-gates-verification.md)：`QG-LINT` 的两种命令形式与 `QG-WARN` 零警告构建；CI `QG-LINT`／`QG-WARN` 通过。 |
| P0-V08 | CI 与 PR 集成 | **已验证** | [W20 验证](p0-w20-ci-baseline-verification.md)：`main` 配置并强制六项必需检查（`enforce_admins: true`）；红色探针阻止合并；恢复后回到 `CLEAN`；未检查提交的直接推送被拒绝（“protected branch hook declined”）。 |
| P0-V09 | 文档评审 | **已验证** | [W05 验证](p0-w05-documentation-baseline-verification.md)及各工作包的发现／链接评审；CI `QG-DOCS`（链接、≤4 跳可达性、状态头）在每个 PR 与推送上通过。 |
| P0-V10 | ADR 治理评审 | **已验证** | [W06 验证](p0-w06-adr-governance-verification.md)：生命周期、门槛、演练和模板；adr-000 未被修改。 |
| P0-V11 | Unsafe 治理评审 | **已验证** | [W10 验证](p0-w10-unsafe-rust-governance-verification.md)：政策与清单可用于首次 unsafe 变更；确认当时源码树零 unsafe。 |
| P0-V12 | 可移植性评审 | **已验证** | [W11 验证](p0-w11-platform-portability-guardrails-verification.md)：没有规则允许 Core 依赖板卡／QEMU 或 Arch 依赖 Board。 |
| P0-V13 | QEMU 入口评审 | **已验证** | [W09 验证](p0-w09-qemu-automation-entry-baseline-verification.md)：恰好一个已记录的 runner 接口，作为 P0 占位；完成语法 walkthrough。 |
| P0-V14 | 制品／构建身份评审 | **已验证** | [W16 验证](p0-w16-version-build-metadata-baseline-verification.md)：身份 schema、政策与预留；[W17 验证](p0-w17-artifact-naming-baseline-verification.md)：语法、身份映射及双向交叉评审。 |
| P0-V15 | P1 移交评审 | **已验证** | [W22 验证](p0-w22-stage-dependency-map-verification.md)：移交图登记表与消费映射；[W21 P1 演练](p0-w21-stage-plan-implementation-workflow-verification.md)：P1-W10 消费的输入可找到且状态可区分。 |

没有 ID 为 `unverified` 或 `blocked`。

## 2. 退出条件（任务书第 7 节），依据第 1 节回答

1. **从克隆经工具链、构建、Host 测试到 AArch64 目标构建的干净环境路径**——成立：W02 恢复契约、W19 阶段链（所有阶段均 `reachable`），一次性克隆中的执行（S1／S2／S3）佐证，CI 亦强制执行。
2. **Workspace 和质量门禁支持后续多 crate、多平台工作而不预设最终 crate 边界**——成立：虚拟 workspace（成员变更仅经批准设计）、四类目标模型和门禁登记表的升级门槛。
3. **底层代码开始前已有 ADR／文档／阶段治理、unsafe 政策、可移植性约束与诊断语义**——成立：W06／W05／W21／W10／W11／W12–W15 已交付，当时源码树零 `unsafe`。
4. **`main` 仅接受必需在线检查通过的 GitHub PR 带来的政策后开发变更**——成立：W20 的保护已配置且有证据；强制演练拒绝了失败的必需检查及未检查的直接推送。
5. **P1 具有单一 QEMU runner 入口、可识别的构建制品和可发现的移交材料**——成立：W09 契约（P1-W10 实现它）、W16／W17 的身份及命名，加上本报告与移交图。
6. **P0 交付物不编码未来 EL2、VM、内存、IRQ、设备或 Guest 的实施决策**——成立：各工作包范围评审（各验证记录的未运行／未证明条目、探针的双符号边界、runner 仅接口的状态）。

## 3. 开放问题

| 问题 | 所有者 | 跟踪位置 |
|---|---|---|
| ADR-054（项目正式名称）待定；P0 使用工作名 `zelyr`／`hypervisor`，并记录制品命名迁移路径 | 所有者，公开协议冻结前 | adr-000 登记表；制品命名第 4.1 节 |
| ADR-055–ADR-058 待定项 | 所属阶段（按登记表注记为 P5／P6／P8） | adr-000 登记表（P0 未修改） |
| ADR 治理的狭义编辑规则第 4(b) 节（登记表指针标注）按设计应用；所有者可能拒绝 | 维护者 | W06 实施记录 |
| W16 `platform` 字段必需但未填（平台词汇尚未命名） | 命名平台词汇的设计（P2 平台发现范围） | W16 验证记录的前置说明 |
| W16 `capability_summary` 预留，直到 W04 profile 语义落入源码树 | 首个实现 profile 的设计 | W16 验证记录 |
| W14 管理域不可信输入的最终类别归属 | P5 hypercall／管理 ABI 错误边界设计 | 失败分类第 2 节的已记录开放项 |
| P8 提议设计包含指向自身未来记录的五个前向引用 | P8 实施工作包 | W05 验证记录（已记录观察）；属于已记录的 QG-DOCS 豁免类别 |

## 4. 范围如实陈述

- 预留和范围外边界见 [P0 任务书第 1 节](../task-book-v0.1.zh-CN.md)；按单一归属规则仅作链接。
- **P0 证据不能证明的事：** 没有 EL2 执行、Guest 启动、QEMU 执行或硬件行为；Host 结果只证明
  Host 侧逻辑（[Host 测试证明边界](../../../testing/host-test-baseline.md)）；runner 入口仅是接口
  （[占位标记](../../../testing/qemu-runner-entry.md)）；CI 检查只证明已配置的门禁，不证明未来类别范围
  （[CI 基线第 5 节](../../../development/ci-baseline.md)）；目标构建只证明编译链
  （[构建目标基线](../../../development/build-target-baseline.md)）。

## 5. P1 移交材料（依照移交规则通过链接组装）

按任务书第 7 节，材料包括：

- 本任务书：[task-book-v0.1.md](../task-book-v0.1.zh-CN.md)及本完成报告。
- 工具链与目标／构建基线：[W02](../../../development/toolchain-baseline.md)与 `rust-toolchain.toml`；
  [W03](../../../development/build-target-baseline.md)与 workspace manifests。
- 质量／CI 与测试基线：[W07](../../../development/quality-gates.md)；[W20](../../../development/ci-baseline.md)
  与 `.github/workflows/ci.yml`、`main` 保护；[W08](../../../testing/host-test-baseline.md)。
- QEMU runner 基线：[W09](../../../testing/qemu-runner-entry.md)。
- 诊断／元数据规则：[W12](../../../development/diagnostics-baseline.md)、
  [W13](../../../development/trace-event-namespace.md)、[W16](../../../development/version-build-metadata.md)、
  [W17](../../../development/artifact-naming.md)。
- Unsafe 与依赖政策：[W10](../../../security/unsafe-rust-policy.md)与[清单](../../../security/unsafe-inventory.md)；
  [W18](../../../development/dependency-governance.md)与[登记表](../../../development/dependency-register.md)。
- ADR 与文档流程：[W06](../../../adr/README.md)、[W05](../../../development/documentation-baseline.zh-CN.md)。
- 平台约束：[W11](../../../development/platform-portability-rules.md)。
- Feature／profile 治理：[W04](../../../development/build-profile-governance.md)。
- 失败、类型安全与平台治理：[W14](../../../security/failure-classification.md)、
  [W15](../../../development/address-identifier-type-safety.md)。
- 贡献者路径和阶段流程：[W19](../../../development/contributor-workflow.md)、
  [W21](../../../development/stage-workflow.md)及集成[政策](../../../development/integration-workflow.md)。
- 依赖图：[p0-handoff-map.md](../p0-handoff-map.md)，它是指向上述各项及其状态和证据的入口。
