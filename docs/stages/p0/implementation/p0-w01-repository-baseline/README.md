# P0-W01 Repository Baseline — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The root-level repository contract and clone-safe entry points
required by [P0-W01](../../plans/p0-w01-repository-baseline.md).  
**Owner/change context:** P0-W01 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P0-W01.  It converts the bounded
work-package plan into small, reviewable documentation/configuration changes.
It deliberately does **not** create a Cargo workspace, prescribe crate/module
boundaries, add hypervisor code, or establish build/test/QEMU behavior; those
belong to later P0 packages.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md).  It then
loads only the linked section needed for its assigned step.  Before editing it
must also follow the Coding Guidelines preflight, including the repository
`AGENTS.md`, documentation index, ADR baseline, P0 task book, and P0-W01 plan.
This document is the approved detailed design for those changes; it is not a
completion record.

## Authority, constraints, and traceability

The governing order is ADR baseline → P0 task book → P0-W01 plan → this design
→ Coding Guidelines.  In particular:

- P0 supplies a clone-safe repository baseline only.  It must not infer or
  pre-create future crates, Cargo manifests, targets, board behavior, or EL2/VM
  mechanisms.
- Root documents must preserve the Core/Arch/SoC/Board separation and state
  that QEMU is a reference platform, never a reason for a Core special case.
- A repository-local default must not require `/home/...`, a particular editor,
  a pre-existing generated directory, a private file, or an external sibling
  repository.  Personal local tools are ignored, not documented as project
  entry points.
- Licensing is mandatory repository metadata, but this design does not have
  authority to choose a license or copyright holder.

| Requirement | Detailed-design location | Acceptance |
|---|---|---|
| P0-W01 repository conventions and entry points | [artifact contract](01-artifact-contract.md) | P0-V01, P0-V09 |
| Git repository creation and empty-clone layout | [implementation workflow](02-implementation-and-review.md) | P0-V01 |
| Link, text, ignore, and scope review | [implementation workflow](02-implementation-and-review.md) | P0-V09 |

## Work breakdown and loading order

1. Read [Artifact contract](01-artifact-contract.md), inventory the current
   root against it, and resolve the license decision blocker with the project
   owner if it remains unset.
2. Apply the root-document, placeholder, and configuration changes in the
   order stated in [Implementation workflow](02-implementation-and-review.md).
3. Create the Git repository and initial committed baseline, then run the
   review matrix in that same document and store actual commands,
   output, environment, and result in
   `../../verification/p0-w01-repository-baseline-verification.md`; record what
   changed and any local decision in
   `../p0-w01-repository-baseline-record.md` (outside this design directory)
   only when implementation begins.

No Rust public API, ABI, wire format, persistent data layout, dependency, or
`unsafe` code is designed or authorized by W01.  The only callable interface
in scope is the optional developer shell helper contract described in the
artifact contract; it is internal, non-ABI, and must remain optional.

## Design-level state and lifecycle

The authoritative state is the Git-tracked repository tree.  There is no registry,
global manager, runtime state machine, lock, queue, allocation, interrupt
context, or telemetry subsystem in W01.  A checkout moves through this simple
documentary lifecycle:

```text
unversioned P0 scaffold
  -> Git repository initialized on main
  -> root baseline committed
  -> clone
  -> root entry read (README + AGENTS + docs/README)
  -> tracked placeholders and configuration present
  -> later package-specific setup/build work
```

Failure to find a required tracked path or to follow a root link stops at the
entry-read step; it must be fixed in the repository, not worked around by a
developer-local directory or undocumented command.  The artifact contract
defines the owner of each statement so two documents do not become competing
sources of truth.

## Open decision / completion blocker

The current authority does not select a software license.  The implementing
agent must ask the project owner for the approved license text and copyright
notice before adding root `LICENSE`.  It may prepare every other W01 artifact,
but cannot claim P0-V01/P0-V09 or close W01 while the required license location
is absent or contains an invented choice.  Record the answer and exact file in
the W01 implementation record.

## Downstream handoff

W02, W05, W19, and W20 may rely only on the final root navigation, tracked
placeholder policy, portable-helper contract, and documentation ownership
defined here.  They must define their own toolchain, documentation taxonomy,
workflow, and CI semantics rather than silently expanding W01.
