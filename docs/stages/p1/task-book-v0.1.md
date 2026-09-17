# Zelyr Hypervisor — P1 Stage Task Book v0.1

**Stage ID:** P1
**Stage name:** AArch64 EL2 Minimum Bring-up
**Status:** Defined planning baseline; implementation and validation are not claimed
**Owner/change context:** P1 planning set established from the existing stage task book
**Supersedes:** the prior unpartitioned P1 task description in this path
**Governing documents:** [Architecture baseline ADR](../../adr/adr-000-architecture-baseline-v0.1.md), [documentation index](../../README.md), and [Plan Agent guide](../../development/plan-agent-guidelines.md)

## 1. Purpose and boundary

P1 establishes a repeatable, observable and diagnosable AArch64 Non-secure EL2
Rust execution environment on QEMU `virt`. Its outcome is a stable EL2 boot-CPU
runtime with known architectural state, exception diagnostics and a controlled
EL2 Stage-1 address space. P1 does not run a Guest and does not provide general
platform, memory, SMP, interrupt-controller or VM services.

The stage is governed by the ADR → task book → frozen contracts → established
contracts → package-local choice authority order. The ADR constraints include
AArch64-first bring-up, QEMU `virt` as reference platform, Core/Arch/SoC/Board
separation, controlled `unsafe`, capability-driven behavior, structured
diagnostics and layered validation.

P0 must supply the documented workspace/toolchain, AArch64 target, build and
QEMU entry points, logging/panic baseline, version/build metadata,
unsafe-governance, address/error conventions and CI gates. Missing P0 inputs
are upstream defects, not permission to redesign P0 inside P1.

## 2. Scope classification

### Required

- canonical QEMU boot contract and entry validation;
- minimal Rust `no_std` EL2 runtime and stable idle state;
- required/optional/future AArch64 capability inventory and report;
- explicit EL2 architectural-state baseline;
- complete EL2 exception-vector baseline with synchronous/fatal diagnostics;
- early console and structured bring-up markers;
- controlled Host Stage-1 address-space transition and post-MMU stability;
- ordered early-initialization lifecycle and failure boundaries;
- automated QEMU smoke/regression and negative/fault-injection validation;
- P1 boot, initialization, host-address-space, exception-diagnostic and
  limitation contracts; and
- P2 handoff contract and stage-gate evidence map.

### Reserved

Replacement of reference-console and static boot storage by P2 discovery and
dynamic memory infrastructure; reusable platform-capability and physical-memory
abstractions; secondary CPU, GIC, timer virtualization, Stage-2, VM/vCPU and
Guest entry; future host virtual-memory services and allocator strategy; and
real-board bring-up. Reserved items may be constrained by P1 contracts but are
not implemented or claimed here.

### Out of scope

Guest EL1 entry, Stage-2, VM/vCPU, HVC ABI or loader; secondary-CPU/SMP/PSCI;
GIC initialization, IRQ routing or vGIC; general DTB parsing, PlatformInfo,
driver probing, BSP framework or Board runtime; page allocator, dynamic heap,
map/unmap service or memory ownership objects; virtio, PCI, SMMU/IOMMU,
passthrough, Control Domain and management ABI; Orange Pi 3B/RK3566 runtime;
and any QEMU-name or board-name branch in generic Core.

## 3. Work-package map

| Package | Required outcome | Primary validation |
|---|---|---|
| [P1-W01](plans/p1-w01-reference-boot-contract.md) | Canonical reference boot contract and entry rejection boundary | P1-V01, P1-V02 |
| [P1-W02](plans/p1-w02-minimal-rust-el2-runtime.md) | Stable Rust EL2 runtime and controlled idle state | P1-V03, P1-V04 |
| [P1-W03](plans/p1-w03-aarch64-capability-inventory.md) | Capability inventory and report | P1-V05, P1-V06 |
| [P1-W04](plans/p1-w04-el2-architectural-state-baseline.md) | Explicit EL2 architectural-state baseline | P1-V07 |
| [P1-W05](plans/p1-w05-el2-exception-entry-baseline.md) | EL2 vector coverage and fatal boundary | P1-V08, P1-V09 |
| [P1-W06](plans/p1-w06-early-console-logging.md) | Reliable early console and markers | P1-V10 |
| [P1-W07](plans/p1-w07-fatal-crash-diagnostics.md) | Non-recursive crash and panic diagnostics | P1-V11, P1-V12 |
| [P1-W08](plans/p1-w08-host-stage1-address-space.md) | Host Stage-1 mapping and MMU transition contract | P1-V13, P1-V14 |
| [P1-W09](plans/p1-w09-initialization-sequencing.md) | Ordered initialization lifecycle and failure contract | P1-V15 |
| [P1-W10](plans/p1-w10-qemu-boot-regression.md) | Automated verdict and 100-cycle regression plan | P1-V16, P1-V17 |
| [P1-W11](plans/p1-w11-negative-fault-validation.md) | Negative and intentional fault validation coverage | P1-V18, P1-V19 |
| [P1-W12](plans/p1-w12-p1-documentation-handoff.md) | P1 contracts, limitations and P2 handoff | P1-V20, P1-V21 |

Each package has exactly one plan in [plans/](plans/README.md). Implementation
traceability belongs under [implementation/](implementation/); commands,
environments, results and completion evidence belong under
[verification/](verification/). This task book and its plans make no
implementation or validation claim.

## 4. Dependency and execution map

```text
W01 -> W02 -> W03 -> W04 -> W05 -> W06 -> W07 -> W08 -> W09 -> W10 -> W11 -> W12
  |      |      |      |      |      |      |      |      |      |      |
  +------+------+- - -+------+------+- - -+------+------+- - -+------+------+
```

The primary chain is conservative: later packages consume earlier contracts.
W06 and W07 may be designed in parallel after W05; W10 and W11 consume the
initialized runtime, exception and MMU contracts. The graph is acyclic.

## 5. Requirement-to-validation traceability

| Requirement group | Validation IDs | Passing condition |
|---|---|---|
| T01 boot contract | P1-V01, P1-V02 | canonical boot reaches validated Non-secure EL2; unsupported entry conditions fail explicitly and do not continue normally. |
| T02 runtime | P1-V03, P1-V04 | runtime state, boot context, panic route and identity are established before stable idle without accidental register assumptions. |
| T03 capabilities | P1-V05, P1-V06 | required capability absence is fail-fast; supported, optional and future facts remain distinguishable. |
| T04 EL2 baseline | P1-V07 | required EL2 state is explicitly owned by Hypervisor and consistent across clean boots. |
| T05 exceptions | P1-V08, P1-V09 | synchronous/IRQ/FIQ/SError paths are valid and diagnostic, with a defined recoverable/fatal outcome. |
| T06–T07 diagnostics | P1-V10–P1-V12 | markers, panic, syndrome, fault location and core context remain observable before and after MMU transition. |
| T08–T09 MMU/lifecycle | P1-V13–P1-V15 | required mapping classes have explicit attributes and post-MMU startup follows declared prerequisites and failures. |
| T10 regression | P1-V16, P1-V17 | automated verdict exists and 100 clean boots reach the same stable marker without panic. |
| T11 negative validation | P1-V18, P1-V19 | unsupported entry, synchronous fault, panic, post-MMU fault and unexpected vector are bounded and reproducible. |
| T12 documentation | P1-V20, P1-V21 | contracts, limitations, evidence locations and P2 assumptions are reviewable and scope-safe. |

## 6. Stage validation matrix

| ID | Evidence sought | Success condition |
|---|---|---|
| P1-V01 | Canonical boot review/test | entry/image assumptions, EL/security state, boot parameters, DTB treatment and failure policy are documented and reference boot reaches EL2. |
| P1-V02 | Unsupported-entry evidence | disallowed environment is rejected with a reason and no normal runtime continuation. |
| P1-V03 | Runtime review/test | stack, Rust data state, boot context, panic route and build identity are established in order. |
| P1-V04 | Stable-state boot evidence | repeated normal boots reach stable EL2 without hidden initial-register assumptions. |
| P1-V05 | Capability report evidence | EL, CPU, affinity, PA/VA/translation/granule/timer/virtualization facts are classified and reported. |
| P1-V06 | Capability negative review | required absence fails explicitly; optional absence does not become an unrelated panic. |
| P1-V07 | EL2 baseline review | routing, traps, FP/SIMD, debug/performance, timer, EL1/EL0 preparation and translation controls are explicitly established. |
| P1-V08 | Vector coverage evidence | synchronous, IRQ, FIQ and SError have valid EL2 entry and origin/context classification. |
| P1-V09 | Synchronous exception evidence | intentional/unexpected paths expose syndrome and location, then follow the defined outcome. |
| P1-V10 | Console/marker evidence | diagnostics work from entry through stable state and identify the failed phase. |
| P1-V11 | Crash-report review/test | fatal output includes build, CPU/EL, PC/return, syndrome, fault address, phase and useful register context. |
| P1-V12 | Non-recursion evidence | panic, early failure and faults do not silently recurse into an unobservable crash. |
| P1-V13 | Host-map review | code, data, stack, vectors, boot data and MMIO have explicit permission, execution and memory attributes. |
| P1-V14 | Post-MMU evidence | MMU-enabled execution, console, vectors and fatal diagnostics work without an identity-map contract. |
| P1-V15 | Lifecycle review | stages have visible prerequisites, order and failure results; hidden dependencies are not accepted. |
| P1-V16 | Automated-verdict evidence | QEMU test independently determines pass/fail with bounded markers, timeout/exit behavior and preserved failure evidence. |
| P1-V17 | Repetition evidence | 100 consecutive clean boots reach the same stable marker with no random startup failure. |
| P1-V18 | Fault-injection evidence | panic, synchronous, post-MMU translation/access fault and unexpected vector produce required diagnostics. |
| P1-V19 | Scope/security review | input/range checks, no unintended RWX, unsafe inventory and P2–P4 boundary are reviewable. |
| P1-V20 | Documentation review | boot, initialization, address-space, diagnostics, reference-environment and limitations contracts are consistent. |
| P1-V21 | Governance review | one plan per package, objective conditions, resolving links, acyclic dependencies and no completion claims. |

## 7. Exit criteria and handoff

P1 may be marked complete only when P1-V01 through P1-V21 have evidence and:

1. QEMU `virt` reaches stable Non-secure EL2 Rust runtime through the canonical path.
2. Required capabilities and explicit EL2 baseline are validated; missing requirements fail fast.
3. Vectors, console, panic and fatal diagnostics remain useful before and after Host Stage-1 MMU enablement.
4. Required code/data/stack/vector/MMIO mapping classes have explicit attributes without a permanent identity-map promise.
5. Automated verdict and 100-cycle clean-boot evidence exist.
6. Negative/fault evidence covers unsupported entry, synchronous fault, panic, post-MMU fault and unexpected vector.
7. P1 contracts, limitations, unsafe/API/dependency reporting and P2 handoff are reviewable, while Guest/SMP/GIC/discovery/allocator mechanisms remain outside the stage.

P2 may rely on a stable boot CPU, Non-secure EL2, Rust runtime, early console,
diagnosable exception path, capability knowledge, known EL2 state and Host
Stage-1 runtime. P2 still owns DTB-to-PlatformInfo discovery, physical-memory
discovery and dynamic allocation.

## 8. Completion review

Before a completion claim, review which entry assumptions were eliminated,
which temporary reference assumptions remain, whether fatal paths preserve
phase/location/syndrome, whether post-MMU behavior avoids an ABI promise, and
whether any Guest, SMP, GIC, platform-discovery, allocator or board-runtime
mechanism entered P1. Any negative answer prevents completion.
