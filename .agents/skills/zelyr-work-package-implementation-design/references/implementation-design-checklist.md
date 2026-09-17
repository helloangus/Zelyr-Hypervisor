# Work-package implementation-design checklist

Use this reference after the mandatory project documents and before handing a
detailed design to an implementing agent.

## 1. Goal-to-baseline ledger

Create this ledger before deciding files, functions, or commands:

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable or prerequisite | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|

Rules:

- Inspect the real state: repository metadata, tracked files, generated versus
  source artifacts, configuration, existing records, and dependency outputs.
- A plan's goal and acceptance wording are requirements, even when their
  enabling artifact is not enumerated as a work-sequence bullet.
- A missing foundation is a design item when it is the only bounded way to make
  the stated result true.  A missing remote URL, license decision, security
  policy, ABI choice, or architecture decision is instead a blocker to record.
- Do not assume an empty directory proves a tracked placeholder, a local file
  proves clone availability, or a passing syntax check proves the goal.

## 2. Recommended detailed-design shape

Use only sections needed by the package, but make the entry document easy to
load and preserve this information:

1. Status, scope, owner/change context, supersession, and parent plan.
2. Purpose, authority order, ADR constraints, Required/Reserved/Out-of-Scope.
3. Existing dependencies, consumers, and the goal-to-baseline ledger.
4. Logical modules or authoritative artifact groups: responsibility, owned
   state/artifact, inputs, outputs, and non-responsibility.
5. Core objects, ownership, lifecycle/state machines, and concurrency/security
   model when code or stateful behavior is in scope.
6. Exact interfaces: names and contracts for functions/types/data structures;
   pseudocode/logic; or an explicit statement that none are authorized.
7. Ordered implementation workflow.  Every step has target, work, acceptance,
   failure/blocker handling, and evidence destination.
8. Requirement-to-validation matrix, handoff checklist, extension points, and
   ADR-required/open questions.

Split large material into linked files by logical unit or implementation mode;
do not split a short design merely to create folders.  The entry document links
to the Coding Guidelines and tells the agent which supporting file to load for
each assigned step.

## 3. Step and interface templates

### Implementation step

```text
Step N — <outcome-oriented name>
Target: <artifact/module>
Work: <what to inspect/change and why>
Suggested observation: <optional command/tool/result>
Acceptance: <observable condition>
Failure/blocker: <rollback, stop condition, or decision owner>
Evidence: <implementation or verification record path>
```

### Function/type contract

```text
Name and stability: <exact name; public/internal/temporary>
Purpose and caller: <semantic operation and users>
Inputs / outputs: <types and meaning>
Preconditions / postconditions: <invariants>
State and ownership change: <authoritative owner>
Concurrency/allocation context: <lock/IRQ/blocking/allocation rules>
Errors and failure guarantee: <named cases and retained state>
Security/authorization checks: <untrusted inputs and checks>
Logic: <pseudocode or algorithm outline, not runnable code>
Validation: <focused tests/reviews>
```

## 4. Handoff self-review

Before declaring the design ready, verify all of the following:

- The goal-to-baseline ledger has no unexamined goal requirement.
- Every missing essential artifact is a bounded step, an assigned prerequisite,
  or an explicit decision blocker.
- No inferred file tree, crate, API, target, board behavior, or future runtime
  mechanism exceeds source authority.
- Each designed mutable state has one owner; stateful objects have lifecycle,
  failure, and destruction/recovery behavior where applicable.
- Each important interface has a complete contract; no code interface is
  silently implied by prose.
- Every validation entry has a success condition and proof boundary; planned,
  run, blocked, and failed evidence are distinct.
- The design has no completion claim and points to separate implementation and
  verification record locations.
- A basic coding agent can start from the entry document and Coding Guidelines
  without rediscovering the package goal, required artifacts, or acceptance.
