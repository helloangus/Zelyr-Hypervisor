# P0-W05 Taxonomy and Baseline-Document Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W05 detailed design](README.md).

## 1. Logical artifact groups and ownership

W05 is governance-documentation work, so its logical modules are authoritative
artifact groups, not Rust modules.

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Documentation baseline document | `docs/development/documentation-baseline.md` | `docs/README.md` mandate, task book §3, this design | the sole normative home of the class inventory, metadata rules, stage separation, and change thresholds; it does not define ADR lifecycle detail (W06), workflow admission rules (W21), templates, or any future contract content |
| Coherence sweep edits | the existing directory READMEs and entry documents found inconsistent | the baseline document, the sweep findings | minimal corrections that remove contradictions and missing required statements; no style rewrites and no content authoring for future packages |
| Documentation routing | one row in `docs/README.md` routing table | baseline document location | discoverability of the baseline document; it does not restate policy |
| Stage implementation index | `docs/stages/p0/implementation/README.md` | design status | a truthful status row; it never claims completion |
| Implementation record | `docs/stages/p0/implementation/p0-w05-documentation-baseline-record.md` (created when work starts) | actual decisions and sweep findings | changed artifacts, findings, follow-ups, deviations; no command logs |
| Verification record | `docs/stages/p0/verification/p0-w05-documentation-baseline-verification.md` (created when evidence exists) | actual review and walkthrough evidence | run/not-run evidence per the validation matrix; not part of the design |

The artifact named in the second column is the sole authoritative home for the
statement in its row. Other documents may link to it but must not duplicate or
contradict it.

## 2. Prerequisite state

W01 is completed: the root navigation and the tracked documentation tree
exist. The ADR baseline exists. No other prerequisite gates W05. The baseline
document must carry the status header `docs/README.md` already mandates
(status, scope, version `v0.1`, owner/change context, supersedes: none) — the
header convention therefore exists independently of W05's own delivery, and
W05 documents are subject to the rules they define from the moment they are
written.

## 3. Class inventory (required document content)

The baseline document must contain this inventory as its normative class
table. Status is the review authority: a document in a normative class
constrains work; an informative document explains. "Conditional" means
normative when approved and routed, informative otherwise.

| Class | Location | Status | Authority role |
|---|---|---|---|
| Root navigation | root `README.md`, `AGENTS.md` | normative | mandatory entry points and routing; owned by W01's baseline |
| Documentation governance | `docs/README.md`, `docs/development/documentation-baseline.md` | normative | routing, precedence, taxonomy, metadata rules (this package) |
| Architecture decisions (ADR) | `docs/adr/` | normative | accepted architecture decisions and their change process; lifecycle detail reserved to W06 |
| Architecture descriptions | `docs/architecture/` | conditional | cross-cutting architecture descriptions when approved |
| ABI / wire-format contracts | `docs/abi/` | normative when present | versioned external contracts; none exist in P0 |
| Machine-type contracts | `docs/machine-types/` | normative when present | versioned guest machine model; none exist in P0 |
| Platform contracts | `docs/platform/` | conditional | PlatformInfo/support-tier/BSP/quirk contracts when approved |
| Testing | `docs/testing/` | conditional | test strategy, environments, evidence rules when approved |
| Security | `docs/security/` | conditional | threat model, safety, unsafe-audit records when approved |
| Development guidance | `docs/development/` | normative | the mandatory concise agent guides, routed detailed references, and per-topic governance baselines (toolchain, targets, build choices, documentation itself) |
| Templates | `docs/templates/` | normative as a rule, informative as content | the gating rule (approved template before a recurring type) and the template bodies |
| Stage documents | `docs/stages/<id>/` | normative | task book (what), plans (bounded packages), implementation (designs + records), verification (evidence) — see §7 |
| Implementation records | `docs/stages/<id>/implementation/<slug>-record.md` | stage-internal factual | decisions taken, changed artifacts, deviations; never evidence logs |
| Verification records | `docs/stages/<id>/verification/<slug>-verification.md` | stage-internal factual | evidence, run/not-run status, completion claims live only here |

Inventory rules the baseline must state: a class is added or removed only
through the §8 thresholds; a document must not create a new implicit class by
inventing a location; informative content inside a normative document must be
labeled informative at the point of use (existing rule, made checkable).

## 4. Normative/informative discipline (required document content)

The baseline document must operationalize the existing informative-labeling
rule: any statement that does not constrain work says so in place; a
normative document's informative sections are marked; an informative document
must not use normative verbs ("must", "only", "prohibited") about project
behavior. Review implication: mixing without labeling is a coherence defect
found by the sweep, not a style issue.

## 5. Metadata rules (required document content)

The baseline document must fix, for every normative document (and recommended
for informative ones):

- **Required header fields:** Status, Scope, Version, Owner/change context,
  Supersedes (where applicable) — the fields `docs/README.md` already
  mandates, now with semantics: *Status* names the lifecycle position
  (for example proposed/accepted/superseded for contracts); *Scope* states
  what the document governs and what it deliberately does not; *Owner/change
  context* names the owning package or role and why the document may change;
  *Supersedes* names replaced documents or none.
- **Version format:** `v<major>.<minor>`. Bump the minor for normative
  content additions or refinements that keep prior guidance valid; bump the
  major for a rewrite that invalidates prior guidance.
- **Supersession mechanics:** a replaced document is not silently edited or
  deleted; a successor names it in Supersedes, and the predecessor's status
  moves to superseded with a pointer to the successor. (ADR-specific
  lifecycle rules are W06's; the baseline must point there rather than
  define them.)
- **Same-change rule:** documentation that a change invalidates is updated in
  the same change as the change itself (existing rule); a review that finds a
  contract lagging its implementation is a defect, not a TODO.

## 6. Version and change thresholds (required document content)

- **Routine maintenance** (ordinary PR review): content additions within a
  class; metadata corrections; broken-link fixes; informative clarifications.
- **Policy decision required** (recorded issue and owner decision before the
  change): adding, removing, or re-homing a class; reclassifying a
  directory's status; relocating an accepted contract (executed only with
  the owning package's consent — the W02 re-homing reservation is the
  standing example).
- **ADR required:** a change that would alter an accepted architecture
  decision or the ADR change process itself (routed to `docs/adr/README.md`
  and W06's lifecycle).

## 7. Stage-document separation (required document content)

The baseline document must fix, per stage `docs/stages/<id>/`, exactly:

| Sublocation | May contain | Must not contain |
|---|---|---|
| `task-book-v*.md` | required outcomes, constraints, validation matrix, exit criteria | detailed designs, function/module specifications, implementation notes, evidence |
| `plans/` | bounded work-package plans and the index | designs claiming approval status they lack; implementation notes; evidence |
| `implementation/` | detailed designs and implementation records | verification evidence or completion claims; policy that belongs to a normative class outside the stage |
| `verification/` | evidence and completion reports per package | designs, records of decisions, policy |

The baseline must also state the separation rationale in one paragraph (plans
provide granularity, not implementation prescriptions; designs authorize
implementation; records trace it; verification proves it) and must defer the
*responsibility flow and admission criteria between layers* to W21 by
explicit pointer.

## 8. Entries, links, and referencability (required document content)

The baseline document must fix the entry rules the sweep checks:

- every directory under `docs/` has a README stating the directory's role and
  status class per the inventory;
- all Markdown links in entry documents resolve from a fresh checkout;
- a document referencing another's normative statement links to it instead of
  restating it (single-home rule);
- referencability: for every output a P0 package or a P1 planner produces,
  the inventory names its home before the package starts.

## 9. Explicitly excluded interfaces

There are no Rust types, functions, traits, modules, crates, APIs, build
artifacts, or tool behaviors in this design. Also excluded by scope: ADR
lifecycle states and transition rules (W06 owns them; the baseline's ADR row
and §5 pointer must not pre-empt them), workflow admission rules (W21),
template authoring (owning packages), and any task-book/plan rewrite. Adding
any of these under W05 is a scope conflict requiring the owning package's
design and must be stopped at review.
