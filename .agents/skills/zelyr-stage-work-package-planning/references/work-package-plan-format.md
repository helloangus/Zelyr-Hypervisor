# Work-package plan format and review checklist

Use this format for a single P<stage>-Wxx plan. It intentionally stops before
implementation-level module, API, and function design.

    # P<stage>-Wxx — Short package name

    Status: Planned work package; implementation not claimed
    Parent: link to the stage task book
    Prerequisites and consumers: link to the plans index

    ## Goal
    A single observable package outcome.

    ## Scope
    The artifacts, governance, or capability boundary this package must establish.

    ## Out of scope
    Nearby work that belongs to another package or a later stage.

    ## Work sequence
    1. Establish or inspect the package baseline.
    2. Define/produce the required package deliverable.
    3. Integrate it with prerequisite or shared-stage contracts.
    4. Review it against governing constraints.
    5. Run the applicable test, acceptance review, or evidence collection.
    6. Record status and hand off the resulting contract.

    ## Acceptance and closure
    Applicable validation IDs, evidence expected, and the objective passing
    condition. State when review rather than an executable test is appropriate.

    ## Handoff
    Which later package/stage consumes the output, what it can rely on, and
    what remains explicitly unimplemented.

## Review checklist

- The package has one coherent outcome and stable ID.
- Required, reserved, and out-of-scope work remain distinct.
- The plan links to its task book and index, and the index names its
  prerequisites and consumers.
- Work steps are substantial outcomes, not commands, code edits, function names,
  or package-manager instructions.
- Acceptance references a task-book validation ID or defines a new, objective
  condition that is added to the task book.
- The document distinguishes plan, implementation record, verification evidence,
  and completion claim.
- Dependencies are justified, acyclic, and sufficient for the stated handoff.
- No plan silently changes an ADR, frozen contract, layering rule, build/runtime
  boundary, security boundary, or future-stage scope.
- Validation scope says what it proves and what it does not prove.
- Any unresolved decision is classified as Implementation Choice, ADR Required,
  Platform Investigation, or Specification Investigation.

