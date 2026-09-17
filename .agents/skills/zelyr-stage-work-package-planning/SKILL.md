---
name: zelyr-stage-work-package-planning
description: Turn a Zelyr stage task book into an authoritative full task book and bounded P0-Wxx-style work-package plans. Use when creating, reorganizing, or extending stage planning documents; do not use for code implementation or completion evidence alone.
---

# Zelyr stage work-package planning

Use this skill to convert a Zelyr Stage Task Book into a coherent, reviewable
planning set. The outcome is a stage-level source of truth plus one bounded
work-package plan per P<stage>-Wxx package. It is not an implementation-level
module/API design and it must not claim that the planned work has been completed.

## Required reading and authority

Before planning, read the repository AGENTS.md, docs/README.md, the Architecture
Baseline ADR, the applicable Stage Task Book, and the concise Plan Agent guide.
Follow this authority order:

    Architecture ADR
      > current Stage Task Book
      > frozen interface/ABI/machine-model contracts
      > established module contracts
      > stage-local planning freedom

If sources conflict, stop the affected decision and record Architecture Change
Request or ADR Required. Do not resolve it by changing the task book or a plan.

Read the detailed Plan Agent reference only when its routing trigger applies.
For a work-package plan without module/object/interface design, retain the
stage-scope, validation, handoff, and completion rules. If the request includes
an implementation-level detailed design, follow the concise guide's relevant
routing before drafting it.

## Separate the document layers

Preserve this hierarchy:

    ADR -> Stage Task Book -> Work-package plans -> Detailed design/implementation
        -> Verification evidence -> Stage completion report

- The task book defines the complete stage outcome, required/reserved/out-of-
  scope boundary, work-package requirements, stage validation, exit criteria,
  and handoff.
- A work-package plan describes only one small work package: its bounded
  deliverable, large work steps, acceptance/closure, and downstream handoff.
- A detailed design defines modules, objects, interfaces, state, and function
  contracts only when needed before code.
- Implementation and verification records contain actual changes, commands,
  logs, environments, results, and limitations.

Never use a task book or work-package plan to invent crate boundaries, module
trees, APIs, function signatures, data layouts, algorithms, or future-stage
runtime mechanisms unless an authoritative source already requires them.

## Workflow

1. Establish the source baseline. Inspect the current task book, relevant ADR
   constraints, existing plans, implementation records, verification evidence,
   and repository status. Preserve established terminology and report whether
   documents are planned, implemented, or evidenced.
2. Classify every stage requirement as Required, Reserved, or Out of Scope.
   Keep future-stage features reserved or excluded; do not promote them merely
   because directories or placeholders exist.
3. Define bounded work packages. Give each a stable ID such as P0-W01, a
   single coherent outcome, explicit prerequisites, and clear consumers. Split
   packages when independent acceptance or handoff would otherwise be unclear.
4. Write the task book as the complete stage map. Include source constraints,
   dependency/execution map, requirement-to-validation mapping, validation
   matrix, exit criteria, and next-stage handoff. It must be possible to see
   all required work without reading every plan.
5. Write one plan per package using the format in
   references/work-package-plan-format.md. Use four to seven outcome-oriented
   steps. Include planning, integration, review, test/acceptance, and handoff
   where applicable, but do not prescribe commands or implementation internals.
6. Create or update the plans index. It must provide a reading order, all
   package links, prerequisites, downstream consumers, and the location for
   later implementation and verification evidence.
7. Validate the planning set. Confirm every task-book package has exactly one
   plan; every plan has the required sections; each validation ID has a success
   condition; links resolve; dependencies have no unexplained cycle; and no
   planned document asserts unperformed validation.
8. Report changed files, validation run and not run, new unsafe, ABI/public API
   changes, dependencies, and unresolved architecture conflicts.

## Stage-specific guardrails

- Keep P0, P1, and later work separate. A P0 plan can prepare an entry point
  for EL2/QEMU work but cannot claim EL2 boot, guest execution, or hardware
  behavior.
- Preserve Core/Arch/SoC/Board layering. No plan may authorize board-name or
  QEMU-name behavior in generic Core.
- Keep build capability distinct from runtime policy.
- Do not treat role as authority, QEMU behavior as architecture, or a temporary
  limit as a permanent semantic contract.
- Validation plans may be written before implementation. Verification reports
  require real, reproducible evidence and must distinguish run, not run, blocked,
  and failed results.

## Deliverable checks

Use the reference's checklist before handoff. Keep long plans, raw validation
output, and completion evidence as repository files rather than relying on chat
history.

