# Zelyr agent instructions

This repository is in P0.  Establish the documented engineering baseline before
implementing EL2, VM, memory, GIC, or guest functionality.

## Mandatory reading

Before any non-trivial task, read [`docs/README.md`](docs/README.md) and follow
its routing table.  In particular:

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

The directories are intentionally a P0 scaffold, not implemented crates.
Do not infer API, Cargo workspace membership, target layout, or module trees
from their names: those require the relevant Plan Agent design.
