# P2-W07 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P2-W07 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the
documents named in the parent README and inspects the tree: W01 and W02
implementations must exist with host-runnable entries and their own
verification records. If either is missing or diverges from its published
contracts ([01 §2](01-scope-and-foundations.md) A1/A2), the work is blocked
— record the upstream defect per task book §2; do not fork, stub, or
re-implement any validator or walker.

Stop and obtain direction instead of guessing when:

- the offline parameterization of a boot-only intake rule cannot be
  expressed without changing W01's signature — that is a W01 contract
  revision (sibling-design conflict), not a local adapter hack;
- a fixture artifact cannot be obtained with lawful, recorded provenance —
  record that fixture blocked; do not substitute an untracked or
  hand-edited blob;
- an expectation cannot be justified from the fixture's own provenance —
  mark the row informational; do not invent board facts;
- support for a CLI, extra SoC analysis, or a new board fixture is
  requested — Reserved (parent README); route to a new design.

## 2. Ordered implementation steps

### Step 1 — offline parameterization and `check` entry

Target: `offline::check` + the W01/W02 call adapters.

Work: implement the entry per
[02 §3](02-readiness-report-model.md): intake over image bytes with the
offline parameter table ([01 §3](01-scope-and-foundations.md)), then W02
normalize on success. Rationale: the entry is the single-semantics seam —
everything else is mapping and data.

**Acceptance:** running `check` over a W01/W02 test-corpus blob produces
intake and fact rows identical in class to what the boot-path tests
produce for the same bytes; skipped rules appear as NOT-P2 rows.  
**Failure/blocker:** any need to special-case checker behavior per fixture
is a design violation — checker behavior varies only through data.

### Step 2 — report model and mappings

Target: `ReadinessReport`, intake-row and fact-row mappings, unrelated-
devices aggregate, delta rows.

Work: implement per [02](02-readiness-report-model.md) §2–§4 with the
total W01/W02-outcome mapping table. Rationale: the mapping's totality is
what makes the report objective; it is one reviewable table, not scattered
conditionals.

**Acceptance:** for every diagnostic class in W01's taxonomy and every
fact state and fatal diagnostic in W02's model, a unit demonstration maps
it to the specified class; the mapping is exhaustive (no default arm that
guesses).  
**Failure/blocker:** an outcome with no specified class is a gap in this
design — stop and record a design issue; do not pick a class locally.

### Step 3 — expectation comparison and verdict

Target: delta comparison + `derive`.

Work: implement per [02 §4.2](02-readiness-report-model.md) and
[02 §5](02-readiness-report-model.md), including the fixed disclaimer
string and the vocabulary rule.

**Acceptance:** binding deltas fail the verdict; informational deltas do
not; the disclaimer appears in every report and render; no report text
contains support-claims about platforms (vocabulary check).  
**Failure/blocker:** an expectation file failing schema validation fails
the harness with a schema diagnostic, not a silent skip.

### Step 4 — fixture corpus

Target: fixture area — QEMU `virt` triple, RK3566/Orange Pi 3B triple,
manifest.

Work: acquire artifacts and record provenance per
[03 §2](03-fixture-and-expectation-matrix.md); author expectation files
per [03 §3–§5](03-fixture-and-expectation-matrix.md) from the artifacts
themselves (classes fixed by the design; payload marks from the blobs).

**Acceptance:** each triple is complete (image + expectation + provenance),
the manifest iterates them in fixed order, and expectation values cite
their provenance-derived authority in notes.  
**Failure/blocker:** an unobtainable artifact blocks its fixture row only
([03 §2](03-fixture-and-expectation-matrix.md)); record and continue with
synthetic-fixture validation.

### Step 5 — fixture harness and evidence

Target: host test target over the manifest.

Work: run `check` per fixture, compare expectations, render reports to the
evidence files. Rationale: the harness is the P2 invocation surface (no
CLI, parent README Decision 7).

**Acceptance:** one command runs the corpus and reports per-fixture
verdicts; each fixture's rendered report and delta list is captured for
the verification record.  
**Failure/blocker:** a binding delta on the QEMU fixture is a *finding*:
either QEMU drift (new dump version needed, [03 §7](03-fixture-and-expectation-matrix.md))
or a semantics regression — diagnose before touching anything; never
loosen an expectation to pass.

### Step 6 — validation and closure

Work: run the matrix in
[05-validation-and-handoff.md](05-validation-and-handoff.md), complete the
handoff checklist, and record decisions/deviations in
`../p2-w07-offline-dtb-compatibility-record.md` and evidence in
`../../verification/p2-w07-offline-dtb-compatibility-verification.md`
(created when the work starts). Completion is claimed only in the
verification record, only for what actually ran.

## 3. Evidence destinations

| Evidence | Destination | Created when |
|---|---|---|
| Implementation decisions, deviations, changed files | `../p2-w07-offline-dtb-compatibility-record.md` | implementation starts |
| Fixture provenance notes | beside the fixtures (fixture area) | artifact acquisition |
| Per-fixture rendered reports, deltas, run/not-run rows | `../../verification/p2-w07-offline-dtb-compatibility-verification.md` | validation runs |
| Cross-check vs QEMU boot-time DTB | W09's verification record, cross-referenced | W09 executes |

Nothing in this design creates or pre-fills those files.
