---
name: zelyr-work-package-implementation-design
description: Turn one approved Zelyr P-stage-Wxx work-package plan into an implementation-level design that a coding agent can execute. Use before code changes; do not use to create stage task books/plans or completion evidence.
---

# Zelyr work-package implementation design

Create a bounded, implementation-level design for exactly one approved Zelyr
work package.  The result lets a coding agent load one design entry point plus
the Coding Guidelines, then consult other sources only when a routed decision
requires them.  It is a proposed design, never an implementation or validation
claim.

## Required reading and authority

Before drafting, read `AGENTS.md`, `docs/README.md`, the Architecture Baseline
ADR, the applicable stage task book, selected P<stage>-Wxx plan, and concise
Plan Agent guide.  Follow its routed detailed-reference sections for the
actual design type.  Read the Coding Guidelines too when the design will direct
code changes, so the handoff does not prescribe forbidden implementation work.

Authority is:

```text
Architecture ADR > current stage task book > frozen contracts > work-package plan
> established module contract > stage-local design freedom
```

If sources conflict or a required choice is not authorized, label it
`ADR Required` or `Architecture Change Request`; do not make the choice in the
design.  Preserve the separation:

```text
ADR -> task book -> work-package plan -> detailed design -> implementation record
    -> verification evidence
```

Use `zelyr-stage-work-package-planning` instead when creating or reorganizing
the stage task book or work-package plans themselves.

## Workflow

1. **Establish the baseline.** Inspect the plan's prerequisites and consumers,
   existing designs/records/evidence, actual repository status, tracked versus
   merely local files, and relevant configuration.  Do not infer that a named
   directory, crate, repository, target, or command exists from its name.

2. **Derive the foundation deliverables before decomposing work.** For every
   stated outcome, compare the required end state with observable current
   state.  Record the missing prerequisite artifact, its owner, and the proof
   that it is actually needed to reach the outcome.  A prerequisite implied by
   the outcome is a required foundation deliverable, not merely a validation
   limitation.  For example, a clone-safe repository outcome requires an
   initialized tracked repository and a committed baseline when neither exists.
   It does not authorize inventing a remote URL, license, crate, or runtime
   policy; those remain decision blockers until an authority supplies them.

3. **Classify scope and boundaries.** Mark each item Required, Reserved, or Out
   of Scope.  Keep future-stage mechanisms and unapproved API/crate boundaries
   out.  For code-bearing work, define logical modules rather than assuming a
   file tree; for documentation/configuration work, define authoritative
   artifact groups instead.

4. **Design the execution units.** Give each unit a clear goal, target
   artifact/module, inputs, output, owner of mutable state, non-responsibility,
   and failure boundary.  Use an ordered workflow where each step says what to
   inspect/change, why, and its acceptance condition.  Suggest commands only
   when they help observe a result; do not make incidental command spelling the
   contract.

5. **Specify code contracts only when code is in scope.** For each important
   function, type, state machine, or shell interface, state its exact name,
   purpose, callers, inputs/outputs, preconditions, postconditions, state and
   ownership changes, synchronization/allocation context, errors, side effects,
   authorization/security checks, and failure-state guarantee.  Give
   implementation logic or pseudocode, not runnable production code.  State
   explicitly when no code interface is authorized.

6. **Close the validation loop.** Map each foundation and plan requirement to a
   review or test with a passing condition and a statement of what it does not
   prove.  Include normal, boundary, invalid-input, repeated-lifecycle,
   failure-recovery, concurrency, host, QEMU, guest, and hardware checks only
   when the package's scope makes them applicable.  Record actual results only
   in verification material.

7. **Write for handoff and self-review.** Put the design under the stage
   `implementation/` area, give it a concise entry README and split supporting
   files only when they allow selective loading.  The entry README must tell a
   coding agent what to read, what not to assume, where to record implementation
   and verification evidence, and which open decisions block completion.  Run
   the checklist in [the reference](references/implementation-design-checklist.md)
   before handoff.

## Non-negotiable foundation check

Do not translate a plan into a list of edits before asking: **what concrete
artifact or state must exist for its goal to be true?**  Check the actual
repository for that artifact/state.  When absent, do one of the following:

- add a bounded, traceable design step if the plan's goal necessarily requires
  it;
- mark it as a prerequisite supplied by another package if the dependency map
  assigns it there; or
- stop for an authorized decision if supplying it would select an architecture,
  legal, remote-hosting, security, ABI, or policy choice.

Never relabel a missing essential deliverable as merely "not run" or leave it
implicit because the plan names it only in its goal.

## Deliverables and report

Use the document structure and review tables in the reference.  Report changed
files, validation run/not run, new `unsafe`, ABI/public API changes,
dependencies, and unresolved design conflicts.  Do not claim completion from a
design document or from a planning-only inspection.
