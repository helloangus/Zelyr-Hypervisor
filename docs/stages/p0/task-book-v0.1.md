# Zelyr Hypervisor — P0 Stage Task Book v0.1

Chinese readers can use the [Chinese edition](task-book-v0.1.zh-CN.md).

**Stage ID:** P0  
**Stage name:** Repository, Specification & Toolchain Baseline  
**Status:** Defined; not an implementation-completion claim  
**Owner/change context:** P0 engineering baseline  
**Supersedes:** the previous unstructured P0 task-book layout in this path  
**Governing documents:** [Architecture baseline ADR](../../adr/adr-000-architecture-baseline-v0.1.md), [documentation index](../../README.md), and [Plan Agent guide](../../development/plan-agent-guidelines.md)

## 1. Purpose and boundary

P0 establishes the repeatable, governed engineering baseline required before
runtime hypervisor development. Its completion means P1 can begin AArch64 EL2
minimal-bring-up detailed design without first repairing the repository,
toolchain, build, quality, documentation, or diagnostic foundation.

P0 is deliberately **not** an EL2 bring-up stage. It does not require an EL2
binary to run, a guest to boot, or a QEMU smoke test to pass.

### Required

- repository, workspace, toolchain, host and AArch64 build baseline;
- build/profile, quality, dependency, unsafe, portability, diagnostic,
  versioning, artifact, and documentation governance;
- CI and QEMU-runner entry points, with host-side testing support;
- reproducible development instructions, branch/PR integration policy, and a
  P1 handoff package.

### Reserved

- profiles for minimal, research, embedded, general, secure, and full;
- future guest, Control Domain, machine-model, ABI, and telemetry artifacts;
- future AArch64/QEMU and Orange Pi 3B validation work.

Reserved items must not be made unavailable by P0 decisions, but P0 does not
implement their runtime mechanisms.

### Out of scope

EL2 initialization, exception vectors, DTB parsing, allocator and Stage-2
design, VM/vCPU, GIC, scheduler, hypercalls, capability implementation, device
drivers, virtio, Control Domain, guest boot, Linux, and Orange Pi 3B bring-up.
P0 must not fix their crate boundaries, module trees, data layouts, API
signatures, or algorithms.

## 2. Inputs and non-negotiable constraints

The Architecture Baseline ADR is the source of the following P0 constraints:

- AArch64-first, Rust-first Type-1 hypervisor; QEMU virt is the reference
  platform and Orange Pi 3B/RK3566 is the first real-hardware target.
- Cargo workspace and a no_std hypervisor target are required; necessary ASM
  and reviewed unsafe are permitted.
- Core is independent of architecture, SoC, board, and QEMU; platform behavior
  is capability-driven rather than board-name-driven.
- Guest-facing mechanisms remain unimplemented in P0, but their future
  contracts are versioned and telemetry is a first-class concern.
- An accepted ADR is not silently changed. A conflict is an ADR-required issue,
  not a local work-package choice.

## 3. Delivery hierarchy and reading order

    ADR baseline
      -> P0 task book (this document: required outcomes)
      -> plans/P0-Wxx-*.md (one bounded work package: scope and work sequence)
      -> implementation/ (implementation record and, when needed, approved detailed design)
      -> verification/ (evidence and completion report)

plans/ provides planning granularity, not implementation prescriptions. A Coding
Agent must read the ADR, this task book, the applicable work-package plan, the
Coding Guide, and any approved detailed design before changing code. If a work
package needs a module/object/interface design, create that design in the
stage's implementation/design record; do not infer it from a task-book bullet
or invent it while coding.

The [plan index](plans/README.md) is the authoritative entry point for a
specific work package. It supplies prerequisites and downstream consumers so an
agent can load only the documents relevant to the package.

## 4. Work-package map

| Phase | Work packages | Phase outcome |
|---|---|---|
| A — repository and reproducibility | W01–W03, W19 | a clean, documented, target-capable developer baseline |
| B — build and quality controls | W04, W07–W09, W20 | governed build choices and repeatable automated checks |
| C — documentation and engineering governance | W05–W06, W21–W22 | durable stage workflow and downstream handoff map |
| D — safety and portability guardrails | W10–W11, W14–W15, W18 | reviewable low-level development without architectural leakage |
| E — observability and identity | W12–W13, W16–W17 | coherent diagnostics and identifiable artifacts |

The phase order is a recommended dependency order, not permission to complete a
downstream package without its prerequisites. Exact dependencies are in the plan
index.

## 5. Work-package requirements

| ID | Required outcome | Primary validation |
|---|---|---|
| P0-W01 | Repository root has documented, clone-safe engineering conventions and entry points. | P0-V01, P0-V09 |
| P0-W02 | Rust toolchain is pinned, recoverable, and shared by developers and CI. | P0-V02 |
| P0-W03 | AArch64 bare-metal/no_std target can produce a baseline hypervisor artifact; host, hypervisor, guest, and tooling targets are distinct. | P0-V05 |
| P0-W04 | Build capabilities, runtime configuration, features, and profiles have non-overlapping governance. | P0-V09, P0-V15 |
| P0-W05 | Normative/informative documentation, versioning, and stage-document locations are defined. | P0-V09 |
| P0-W06 | ADR lifecycle and architecture-change handling are enforceable project rules. | P0-V10 |
| P0-W07 | Format, lint, host-test, target-build, documentation, warning, and CI-failure rules are defined. | P0-V06–V08 |
| P0-W08 | Host-testable logic has a CI-executable baseline independent of QEMU. | P0-V03–V04 |
| P0-W09 | One reusable QEMU virt runner entry point can later carry test parameters and evidence. | P0-V13 |
| P0-W10 | unsafe justification, inventory, review, and boundary rules exist before EL2 code. | P0-V11 |
| P0-W11 | Core/Arch/SoC/Board separation and capability-driven platform rules are reviewable. | P0-V12 |
| P0-W12 | Logging, tracing, metrics, panic/crash, release, and version-metadata semantics are governed. | P0-V09, P0-V14 |
| P0-W13 | Trace-event namespace and compatibility governance are defined without a telemetry implementation. | P0-V09 |
| P0-W14 | Hypervisor invariant, guest, resource, unsupported-feature, and platform failures are architecturally distinct. | P0-V09 |
| P0-W15 | Future addresses and identifiers must retain semantic type distinctions. | P0-V09 |
| P0-W16 | Build identity and compatibility metadata have a defined baseline and extension point. | P0-V14 |
| P0-W17 | Artifact names are machine-processable and distinguish relevant dimensions. | P0-V14 |
| P0-W18 | Dependencies are evaluated for TCB, no_std, license, maintenance, unsafe, and platform risk. | P0-V09 |
| P0-W19 | A documented clean-environment path reaches build, test, target build, QEMU entry, and the branch-to-GitHub-PR integration flow. | P0-V01–V05, P0-V13 |
| P0-W20 | CI classifies required, informational, and later manual/hardware checks, and enforces verified PR-only integration to `main`. | P0-V08 |
| P0-W21 | ADR → task book → plan/design → implementation → verification workflow is explicit. | P0-V09, P0-V15 |
| P0-W22 | P0 outputs and later-stage consumers are mapped so P1 need not recreate P0. | P0-V15 |

## 6. Stage validation matrix

| ID | Evidence sought | Success condition |
|---|---|---|
| P0-V01 | Fresh-clone review | no undocumented machine-local prerequisite or manually created required path |
| P0-V02 | Toolchain restoration | declared toolchain and components can be restored consistently |
| P0-V03 | Host build | host tools/tests build through the documented entry |
| P0-V04 | Host tests | all baseline host-side tests pass |
| P0-V05 | AArch64 target build | bare-metal AArch64 artifact builds successfully |
| P0-V06 | Formatting | required formatting check reports no violation |
| P0-V07 | Lint | required lint policy passes |
| P0-V08 | CI and PR integration | required pipeline checks pass and their classifications are visible; GitHub requires a PR with those checks passing before post-policy development changes reach `main` |
| P0-V09 | Documentation review | required documentation paths, links, statuses, and responsibilities are coherent |
| P0-V10 | ADR governance review | lifecycle and supersession path can be followed from repository documents |
| P0-V11 | Unsafe governance review | policy and inventory location exist and are usable for a first unsafe change |
| P0-V12 | Portability review | no rule permits board/QEMU dependencies in Core or Board dependencies in Arch |
| P0-V13 | QEMU entry review | exactly one documented runner interface is available for future P1 smoke evidence |
| P0-V14 | Artifact/build identity review | target artifact can be associated with source and declared compatibility metadata |
| P0-V15 | P1 handoff review | P1 planner can find all P0 inputs without reconstructing missing policy |

## 7. Exit criteria and handoff

P0 is complete only when all validation IDs have evidence and these observable
criteria hold:

1. A clean environment can follow declared inputs from clone through toolchain,
   build, host test, and AArch64 target build.
2. The workspace and quality gate can support later multi-crate, multi-platform
   work without asserting final crate boundaries.
3. ADR/document/stage governance, unsafe policy, portability guardrails, and
   diagnostic semantics are available before low-level code begins.
4. `main` accepts post-policy development changes only through GitHub PRs with
   required online checks passing.
5. P1 has a single QEMU runner entry, identifiable build artifacts, and a
   discoverable handoff package.
6. No P0 deliverable encodes future EL2, VM, memory, IRQ, device, or guest
   implementation decisions.

The P1 handoff package contains this task book, the P0 completion report,
toolchain and target/build baseline, quality/CI and test baseline, QEMU runner
baseline, diagnostics/metadata rules, unsafe and dependency policy, ADR and
documentation workflow, platform guardrails, feature/profile governance, and
the P0 dependency map.

## 8. Completion review

The final review answers:

- Is the engineering environment repeatable and protected by appropriate
  automated checks?
- Are downstream constraints clear without P0 predesigning downstream runtime
  systems?
- Can a P1 Plan Agent start the AArch64 EL2 minimal-bring-up design using the
  stated handoff package?

Any no fails P0. Any attempt to settle later-stage module/API/runtime design
inside P0 is a scope violation, not a substitute for completion evidence.
