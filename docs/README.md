# Zelyr documentation index and governance

**Status:** Normative documentation governance (routing, precedence, and the
documentation layout).  
**Scope:** How to find and classify repository documentation; taxonomy and
metadata detail live in the [documentation
baseline](development/documentation-baseline.md).  
**Version:** v0.1  
**Owner/change context:** P0 engineering baseline; updated whenever the
documentation layout changes.  
**Supersedes:** None.

## Reading rules

Every agent and contributor must read this file and the repository-root
[`AGENTS.md`](../AGENTS.md) before non-trivial work.  Then load the documents
for the task using this table.

| Work type | Required documents |
|---|---|
| Any architecture-affecting work | [ADR baseline](adr/adr-000-architecture-baseline-v0.1.md), [ADR lifecycle and process](adr/README.md), and the applicable stage task book |
| Stage planning / detailed design | ADR baseline, applicable task book, mandatory [concise Plan Agent guide](development/plan-agent-guidelines.md), then routed detailed-reference sections |
| Coding | ADR baseline, applicable task book, approved detailed design, mandatory [concise Coding guide](development/coding-guidelines.md), then routed detailed-reference sections |
| Quality-gate questions (format, lint, warnings, tests, builds, docs checks) | [Quality gates](development/quality-gates.md) |
| Repository contribution / integration | [Branch and pull-request integration workflow](development/integration-workflow.md), applicable work-package plan, and the relevant implementation/verification record |
| New-contributor onboarding (clone to merged PR) | [Contributor workflow](development/contributor-workflow.md) |
| CI checks, required-check, or branch-protection questions | [CI baseline](development/ci-baseline.md) |
| Toolchain setup, restoration, or update | [Toolchain baseline](development/toolchain-baseline.md) and the root `rust-toolchain.toml` manifest |
| Target/build-class or AArch64 build-path work | [Build-target baseline](development/build-target-baseline.md) and the root `Cargo.toml` workspace |
| New feature/profile/build switch, or switch-classification questions | [Build-profile / feature governance](development/build-profile-governance.md) |
| Creating, versioning, or classifying documentation | [Documentation baseline](development/documentation-baseline.md) |
| ABI or machine-model change | Above, plus `abi/` and/or `machine-types/` contracts |
| Platform / BSP work | Above, plus `platform/` contracts; preserve Core/Arch/SoC/Board layering per the [platform portability rules](development/platform-portability-rules.md) |
| Testing / completion claim | Applicable task book plus `testing/` contracts and the stage verification record |
| Writing or reviewing host-side tests | [Host-test baseline](testing/host-test-baseline.md) |
| QEMU automation or runner-entry work | [QEMU runner entry contract](testing/qemu-runner-entry.md) (P0: interface-only placeholder) |
| Introducing or reviewing `unsafe` Rust | [Unsafe Rust policy](security/unsafe-rust-policy.md) and the [unsafe inventory](security/unsafe-inventory.md) |
| Failure-path design or fatal-path review | [Failure classification](security/failure-classification.md) |
| Address/identifier semantics in a new interface | [Address & identifier type-safety requirements](development/address-identifier-type-safety.md) |
| Diagnostic channel/level/visibility questions | [Diagnostics baseline](development/diagnostics-baseline.md) |
| Trace-event naming or telemetry namespace questions | [Trace event namespace](development/trace-event-namespace.md) |
| Artifact identity / version-metadata questions | [Version & build metadata baseline](development/version-build-metadata.md) |
| Naming an artifact that leaves the build tree | [Artifact naming baseline](development/artifact-naming.md) |
| Adding or changing a dependency | [Dependency governance](development/dependency-governance.md) and the [dependency register](development/dependency-register.md) |

## Normative documents and precedence

The current normative sources are the [Architecture baseline
ADR](adr/adr-000-architecture-baseline-v0.1.md), applicable stage task books,
approved detailed designs and frozen contracts, plus the relevant concise agent
guide and any detailed-reference sections it routes. Use the governing
guideline's exact ordering:

- Plan work: ADR → current stage task book → frozen interface/ABI/machine model
  → established module contract → stage-local design freedom.
- Coding: explicit task → detailed module design → interface/ABI/state-machine
  contract → ADR → Coding Guidelines → personal preference.

The baseline ADR, concise guides, and detailed references are versioned source
documents. The concise guides are mandatory entry points; detailed references
are mandatory when their routing trigger applies. Do not silently alter their
decisions. Propose a new ADR for an architecture change.

## Documentation layout

```text
adr/            architecture decisions and their lifecycle
architecture/   cross-cutting architecture descriptions
abi/            versioned ABI and wire-format contracts
machine-types/  guest virtual-machine model contracts
platform/       PlatformInfo, support tiers, BSP and quirk contracts
testing/        test strategy, environments, and evidence
security/       safety, threat-model, and unsafe-audit records
development/    contributor, Plan Agent, and Coding Agent guidance
stages/<id>/    task book, detailed plans, implementation notes, verification
templates/      approved templates for new project documents
```

For each stage, keep work strictly separated:

```text
task-book-v*.md             what must be done
plans/                      approved implementation-level design
implementation/             implementation notes and traceability
verification/               evidence and completion report
```

New normative documents must state status, scope, version, owner/change
context, and what they supersede where applicable.  Informative notes must say
they are informative.  Update affected contracts in the same change as code.
