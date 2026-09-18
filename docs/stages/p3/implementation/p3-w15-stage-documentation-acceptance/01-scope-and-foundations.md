# P3-W15 Scope, Foundations, and Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W15 detailed design](README.md).

## 1. Goal-to-baseline ledger

The entry README carries the goal-to-baseline ledger; this section records
the foundation reasoning and the assumed-contract failure boundaries.

### 1.1 What must concretely exist for the plan goal to be true

"Planning, implementation traceability, verification evidence, completion
review, and P4 handoff coherent and separate" requires five concrete
artifacts:

1. **The artifact-group ownership table** —
   [02-acceptance-artifacts-and-checks.md](02-acceptance-artifacts-and-checks.md)
   §2.
2. **The traceability matrix** and **evidence-location map** — same file
   §3, §5.
3. **The checks C1–C8 with their logged runs** — same file §4, §6.
4. **The open-issue register** — same file §7.
5. **The gated closure-review package** — template and assembly gate —
   same file §8.

### 1.2 Prerequisites treated as assumed contracts

| Prerequisite | Source | Assumed content | Failure boundary if delivered differently |
|---|---|---|---|
| Task book outcome and validation sets | [P3 task book](../../task-book-v0.1.md) §3–§5 | Fifteen outcomes, fifteen validation IDs, exit criteria §6, review questions §7 | A task-book change re-bases the matrix; W15 reports the delta rather than adapting silently |
| Plans and index | [P3 plans index](../../plans/README.md) | One plan per package; dependency map | A coverage defect (missing/duplicate plan) is a finding for the planning authority — W15 never patches it |
| Sibling designs, records | W01–W14 artifacts | Statuses, links, and claims each document owns | A defect is a finding with owner; W15 checks and reports, never edits others' artifacts |
| Stage-level files | Stage README, implementation index, verification directory | Navigation roles; the implementation index is coordinator-owned | W15 writes only rows that present W15's own artifacts where those are its to write; all else is checked/reported |
| W14 contract | [P3-W14](../p3-w14-p4-smp-handoff/README.md) | The P4-facing summary and its unresolved register | Until W14 produces it, the register row is `ABSENT (dated)` and the closure gate cannot open — honestly recorded |
| Documentation governance | [`docs/README.md`](../../../../README.md) | Layer separation and status-header norms | A governance change re-bases C6; findings cite the rule violated |

### 1.3 Why no hidden essential deliverable remains

- Plan step 1 ("inspect W01–W14 and the task-book validation/exit
  criteria") is realized by the ownership table and matrix sources.
- Plan step 2 ("assemble required stage documents, plan links,
  implementation/verification locations, and P4 handoff references") is
  realized by the evidence-location map plus the handoff pointer row.
- Plan step 3 ("reconcile every required task-book outcome with exactly
  one plan and validation path") is realized by the matrix construction
  rules.
- Plan step 4 ("review statuses, scope boundaries, dependency acyclicity,
  and unperformed-work wording") is realized by C1, C4, C7, C8 (with C2,
  C3, C5, C6 covering the coherence remainder).
- Plan step 5 ("collect documentation-review evidence and record open
  issues") is realized by the evidence rules and the register.
- Plan step 6 ("produce the closure-review package only when underlying
  evidence actually exists") is realized by the §8 gate; until it opens,
  W15's deliverable is the apparatus plus honest deferral status.

## 2. Scope classification

### 2.1 Required

- Artifact-group ownership table; checks C1–C8 with log format;
  traceability matrix and its construction rules; evidence-location map;
  open-issue register; closure-package template and gate.
- W15's own truthful status rows and links (where W15-owned).
- The documentation-review evidence for P3-V15 (its own verification
  record), produced when the review runs.

### 2.2 Reserved (must not block a future design; not implemented now)

- Closure-package assembly and any closure recommendation; trigger: the
  §8 gate (P3-V01–V15 evidenced).
- Post-closure documentation maintenance; trigger: P4 handoff feedback
  through the W14 contract lifecycle.
- Machine-readable governance formats (automated link/matrix checks);
  trigger: a tooling decision — the checks are manual-method designs
  until then.

### 2.3 Out of Scope

- Creating or editing any verification/implementation record, upstream
  plan, task book, stage README rows owned by others, or the coordinator-
  owned stage index.
- Declaring P3 complete; certification of any validation.
- Designing mechanisms or re-designing packages.
- The P4 handoff contract's content (W14) and P4's own documentation.
- Guest/P4+ documentation; any new architecture content.
