# P3-W15 Workflow, Review, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W15 detailed design](README.md).

## 1. Preconditions and failure boundary

Before any work, the implementer verifies it has loaded the parent README
and [02-acceptance-artifacts-and-checks.md](02-acceptance-artifacts-and-checks.md),
and inspects the actual tree (`git ls-files` under `docs/stages/p3/`;
which designs, records, and stage files exist). Stop and record instead
of improvising when:

- a check would fail against an upstream document — record a finding
  with owner; do not fix the upstream document here;
- assembling the closure package before the gate condition holds — the
  package stays a template and the deferral is the recorded status;
- a governance question (layer ownership, index rows, status wording)
  has no rule in `docs/README.md` or the task book — record the question
  for the governance owner instead of inventing a rule;
- W15's own artifacts would need to claim something unevidenced — they
  state absence instead.

## 2. Ordered workflow

### Step 1 — baseline the document set

Target: W15's checks log (opened in the implementation-record area).

Work: enumerate every P3 document with path, owner, and status header;
verify each path resolves. This is the input snapshot for all checks.

**Acceptance:** a complete, dated snapshot; unresolved paths listed as
findings.  
**Failure/blocker:** a moving tree (parallel designs landing) means the
snapshot is re-taken; the log keeps each snapshot dated.

### Step 2 — run C1, C2, C5, C6 (truthfulness and separation)

Work: status scan (C1), link resolution (C2), evidence-location map
population (C5), separation scan (C6). Log results and findings with
owners.

**Acceptance:** four checks logged with dated results; the map exists
with real paths or dated `ABSENT` markers.  
**Failure/blocker:** findings are reported, never fixed upstream here.

### Step 3 — construct the traceability matrix

Work: build the §3 matrix from the task book §3/§5 and the plans index;
run C3 (exactly-one coverage) and C4 (acyclicity) against it.

**Acceptance:** matrix complete; C3/C4 logged; defects are findings to
the planning authority.  
**Failure/blocker:** a task-book/plans mismatch is a finding — the matrix
records the discrepancy rather than choosing a side.

### Step 4 — run C7, C8 (boundary and claim wording)

Work: boundary-wording scan (C7) and unperformed-work scan (C8) over the
document set; classify any architecture-level breach for escalation
labeling.

**Acceptance:** checks logged; escalations labeled `ADR Required` /
`Architecture Change Request` with owners.  
**Failure/blocker:** a breach in W15's own artifacts is fixed here; a
breach upstream is a finding.

### Step 5 — assemble the open-issue register

Work: merge upstream open items (including P2-ACR-01 by reference), the
W14 register (or its dated absence), and this run's findings.

**Acceptance:** the register is the single visible list, each row with
source, owner, status.  
**Failure/blocker:** an ownerless issue is itself a finding routed to
the stage planning authority.

### Step 6 — evaluate the closure gate (deferred phase)

Work: check the §8 gate: every P3-V01–V15 verification record present
with real evidence, findings resolved or registered. If open — assemble
the closure package per the template; if not — record the deferral with
the specific missing rows.

**Acceptance:** either the package exists with complete evidence
pointers, or the deferral names exactly which rows are missing.  
**Failure/blocker:** no partial assembly "to be ready early" — a package
with placeholders is a C6 violation of W15's own making.

### Step 7 — record W15's evidence and closure

Work: produce W15's verification record (P3-V15 documentation-review
evidence: which checks ran, results, findings, register state, gate
status) and the implementation record; run the validation matrix and
handoff checklist.

**Acceptance:** records honest; P3-V15 satisfied only by the actual
review evidence, and stage closure still requires P3-V01–V14
independently per the task book.  
**Failure/blocker:** honest statuses only — `planned`/`blocked` are
legal states for this package today.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W15-DV01 → P3-V15 | separation and ownership review | C6 + ownership-table audit | every statement's home is unique and layer-correct | the separation norm holds; not that P3 mechanisms work |
| W15-DV02 → P3-V15 | navigation and links review | C2 + map walk from stage entry points | a newcomer reaches any P3 artifact from the stage README/plans index; all links resolve; map paths real or dated-absent | coherence of navigation; not evidence existence |
| W15-DV03 → P3-V15 | coverage review | C3 + C4 over the matrix | exactly-one outcome→plan mapping; all validation IDs mapped; dependency graph acyclic and index-consistent | traceability completeness; not that the plans are satisfied |
| W15-DV04 → P3-V15 | status and wording review | C1 + C7 + C8 | no status lie, no boundary breach, no implied claim anywhere in the set | unperformed-work wording is clean |
| W15-DV05 → P3-V15 | evidence-location and register review | C5 + register audit | map truthful; register complete with owners (P2-ACR-01 included by reference) | evidence locations and open issues are visible and honest |
| W15-DV06 → P3-V15 | closure-package gate review | §8 gate evaluation | package assembled only with gate open; otherwise the deferral names the missing rows exactly | the closure apparatus respects the evidence precondition |
| W15-DV07 → W15 closure | W15 self-closure review | handoff checklist below | W15's own records truthful; no self-certification of stage closure | W15 package closure; stage closure remains the reviewer's decision per task book §7 |

Record each validation as **passed**, **failed**, **blocked**, or
**not run** with input snapshot, date, and reason. The design phase can
honestly complete DV01–DV05 against the current document set (as designs
land) and must record DV06 as gated/deferred until the stage's evidence
exists.

## 4. Error, security, and observability model

- **Failure mode of documentation governance is drift**: links rot,
  statuses lag, registers go stale. The controls are dated snapshots,
  re-runable checks, and regeneration-not-patch for the matrix and map.
- **Security posture**: the package handles no secrets and creates no
  runtime surface; its boundary role is C7 — keeping guest/Stage-2/
  hardware claims out of P3 documents is a scope-security function.
- **Observability**: the checks log, map, matrix, and register are the
  stage's self-description; a reviewer can reconstruct the stage's state
  at any date from the log's snapshots.

## 5. Handoff checklist

Before handing W15 to a reviewer, provide:

- the dated document-set snapshot and checks log with per-check results;
- the traceability matrix and evidence-location map (regeneration dates);
- the open-issue register with owners and the escalation labels carried;
- the closure-gate status: open (package path) or the exact missing rows;
- W15's own implementation and verification records;
- confirmation that no upstream document, record, stage index row owned
  by others, or evidence file was created or edited, and no completion
  claim was made anywhere;
- open items for the stage closure reviewer (task book §7 questions and
  where each is answered) and for P4-W01 (where to enter the P3
  document set).

## 6. Future record paths

Implementation record:
`../p3-w15-stage-documentation-acceptance-record.md` (created only when
implementation begins). Verification record:
`../../verification/p3-w15-stage-documentation-acceptance-verification.md`
(created only when evidence exists). The closure-review package, when
the gate opens, is assembled under the stage's verification area per the
naming convention fixed in the implementation record. None of these
exist today; this design claims nothing.
