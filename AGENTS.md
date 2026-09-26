# Zelyr agent instructions

Chinese readers can use the [Chinese edition](AGENTS.zh-CN.md); this English
file remains the agent instruction source.

This repository has completed P1 within its declared reference-QEMU scope on
the completed P0 engineering baseline. See the
[P1 completion report](docs/stages/p1/verification/p1-completion-report.md)
for its exact evidence and limits. P6-V29 still owns genuine asynchronous
unexpected-vector execution; P1 completion does not imply that proof.
See the [P0 completion report](docs/stages/p0/verification/p0-completion-report.md)
and [P1 implementation index](docs/stages/p1/implementation/README.md) for
current evidence. Keep VM, GIC, and guest functionality in their owning stages.

## Mandatory reading

Before any non-trivial task, read [`docs/README.md`](docs/README.md) and follow
its routing table.  In particular:

Read English source documents by default. The accepted ADR-000 has a Chinese
authoritative source; its [English edition](docs/adr/adr-000-architecture-baseline-v0.1.en.md)
is a translation for direct agent reading. Apply the [language-edition rules](docs/development/documentation-baseline.md#7-language-editions-and-translation-authority)
when source and translation disagree.

- Any design or architecture decision: read the ADR baseline and the applicable
  stage task book before proposing a change.
- Any detailed design: read the concise [Plan Agent guide](docs/development/plan-agent-guidelines.md).
- Any code change: read the concise [Coding guide](docs/development/coding-guidelines.md)
  and the applicable detailed design. Do not write code from a stage task book alone.

The concise guides are mandatory entry points. They specify when the original
detailed references must also be consulted; do not load the complete references
by default when their routed sections are sufficient.

## Project-local agent skills

Project-local skills are versioned under .agents/skills/. When creating,
reorganizing, or extending a Stage Task Book and its bounded P<stage>-Wxx
work-package plans, read
[zelyr-stage-work-package-planning](.agents/skills/zelyr-stage-work-package-planning/SKILL.md)
after the mandatory planning documents. The skill preserves the separation
between task book, plans, detailed design, implementation, and verification.

When turning one approved P<stage>-Wxx plan into an implementation-level
design for a coding agent, read
[zelyr-work-package-implementation-design](.agents/skills/zelyr-work-package-implementation-design/SKILL.md)
after the mandatory planning documents.  It requires a current-state audit and
explicitly captures foundation deliverables implied by the plan goal before
decomposing implementation steps.

Use the precedence stated by the applicable governing document: Plan work uses
the Plan Agent guidelines; coding uses the Coding Guidelines.  If sources
conflict, stop the affected work and record an architecture-change or
ADR-required issue; do not silently choose a new architecture.

## Scope and delivery rules

- Develop every post-policy change on a new branch. Push it to GitHub, open a
  pull request targeting `main`, wait for every configured required online
  check to pass, and merge only through that pull request. Read the
  [integration workflow](docs/development/integration-workflow.md) before
  preparing a merge; do not develop or push directly to `main`.
- Keep P0, P1, and later-stage work separate.  Do not implement future-stage
  mechanisms merely because a directory already exists.
- Preserve crate layering: generic core must not depend directly on a board,
  SoC, or QEMU-specific implementation.
- Keep Stage Task Book, detailed design, implementation, and verification
  reports in their separate locations under `docs/stages/<stage>/`.
- Update documentation when a change modifies a documented contract.  Never
  edit an accepted ADR to hide an architectural change; add a superseding ADR.
- Report changed files, validation run and not run, new `unsafe`, ABI/public
  API changes, dependencies, and unresolved design conflicts.

## Current repository state

The workspace contains the completed P1 hypervisor and host-test baseline. Other
directories may still be scaffolds. Do not infer API, workspace membership,
target layout, or module trees from names; inspect the current workspace and
the applicable detailed design.
