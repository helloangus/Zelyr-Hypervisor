# Zelyr Documentation Baseline

**Status:** Normative documentation governance.  
**Scope:** The document class inventory, normative/informative discipline,
document metadata rules, version and change thresholds, stage-document
separation, and entry/link/referencability rules for the whole `docs/` tree.
It does not define ADR lifecycle detail (owned by the ADR governance
document, P0-W06), workflow admission rules (P0-W21), templates, or any
future contract's content.  
**Version:** v0.1  
**Owner/change context:** P0-W05 documentation baseline; operationalizes the
[`docs/README.md`](../README.md) mandate and the P0 task book's delivery
hierarchy.  
**Supersedes:** The absence of an explicit documentation taxonomy (the
`docs/README.md` mandate predates this document and is preserved, not
replaced).

## 1. Class inventory

Every document in the repository belongs to exactly one class below. Status
is the review authority: a document in a normative class constrains work; an
informative document explains. "Conditional" means normative when approved
and routed, informative otherwise.

| Class | Location | Status | Authority role |
|---|---|---|---|
| Root navigation | root `README.md`, `AGENTS.md` | normative | mandatory entry points and routing |
| Documentation governance | `docs/README.md`, `docs/development/documentation-baseline.md` | normative | routing, precedence, taxonomy, metadata rules |
| Architecture decisions (ADR) | `docs/adr/` | normative | accepted architecture decisions and their change process; lifecycle detail reserved to the ADR governance document |
| Architecture descriptions | `docs/architecture/` | conditional | cross-cutting architecture descriptions when approved |
| ABI / wire-format contracts | `docs/abi/` | normative when present | versioned external contracts; none exist in P0 |
| Machine-type contracts | `docs/machine-types/` | normative when present | versioned guest machine model; none exist in P0 |
| Platform contracts | `docs/platform/` | conditional | PlatformInfo/support-tier/BSP/quirk contracts when approved |
| Testing | `docs/testing/` | conditional | test strategy, environments, evidence rules when approved |
| Security | `docs/security/` | conditional | threat model, safety, unsafe-audit records when approved |
| Development guidance | `docs/development/` | normative | the mandatory concise agent guides, routed detailed references, and per-topic governance baselines (toolchain, targets, build choices, documentation itself) |
| Templates | `docs/templates/` | normative as a rule, informative as content | the gating rule (approved template before a recurring type) and the template bodies |
| Stage documents | `docs/stages/<id>/` | normative | task book (what), plans (bounded packages), implementation (designs + records), verification (evidence) — see §5 |
| Implementation records | `docs/stages/<id>/implementation/<slug>-record.md` | stage-internal factual | decisions taken, changed artifacts, deviations; never evidence logs |
| Verification records | `docs/stages/<id>/verification/<slug>-verification.md` | stage-internal factual | evidence, run/not-run status; completion claims live only here |

Inventory rules:

- A class is added or removed only through the §4 thresholds; a document must
  not create a new implicit class by inventing a location.
- Informative content inside a normative document must be labeled informative
  at the point of use.

## 2. Normative/informative discipline

Any statement that does not constrain work says so in place. A normative
document's informative sections are marked; an informative document must not
use normative verbs ("must", "only", "prohibited") about project behavior.
Mixing without labeling is a coherence defect found by review, not a style
issue.

## 3. Document metadata rules

For every normative document (recommended for informative ones), the leading
header carries:

- **Status** — the lifecycle position (for example proposed / accepted /
  superseded for contracts).
- **Scope** — what the document governs and what it deliberately does not.
- **Version** — `v<major>.<minor>`. Bump the minor for normative content
  additions or refinements that keep prior guidance valid; bump the major for
  a rewrite that invalidates prior guidance.
- **Owner/change context** — the owning package or role and why the document
  may change.
- **Supersedes** — replaced documents, or none.

Supersession mechanics: a replaced document is not silently edited or
deleted; a successor names it in Supersedes, and the predecessor's status
moves to superseded with a pointer to the successor. ADR-specific lifecycle
rules are owned by the ADR governance document (`docs/adr/`); this baseline
points there rather than defining them.

Same-change rule: documentation that a change invalidates is updated in the
same change as the change itself; a review that finds a contract lagging its
implementation is a defect, not a TODO.

## 4. Version and change thresholds

- **Routine maintenance** (ordinary PR review): content additions within a
  class; metadata corrections; broken-link fixes; informative clarifications.
- **Policy decision required** (recorded issue and owner decision before the
  change): adding, removing, or re-homing a class; reclassifying a
  directory's status; relocating an accepted contract (executed only with the
  owning package's consent).
- **ADR required:** a change that would alter an accepted architecture
  decision or the ADR change process itself (routed to
  [`docs/adr/README.md`](../adr/README.md)).

## 5. Stage-document separation

Per stage `docs/stages/<id>/`, exactly:

| Sublocation | May contain | Must not contain |
|---|---|---|
| `task-book-v*.md` | required outcomes, constraints, validation matrix, exit criteria | detailed designs, function/module specifications, implementation notes, evidence |
| `plans/` | bounded work-package plans and the index | designs claiming approval status they lack; implementation notes; evidence |
| `implementation/` | detailed designs and implementation records | verification evidence or completion claims; policy that belongs to a normative class outside the stage |
| `verification/` | evidence and completion reports per package | designs, records of decisions, policy |

Rationale: plans provide granularity, not implementation prescriptions;
designs authorize implementation; records trace it; verification proves it.
The responsibility flow and admission criteria *between* these layers are
owned by the P0-W21 stage-workflow design; this baseline fixes only the
locations and their contents.

## 6. Entries, links, and referencability

- Every directory under `docs/` has a README stating the directory's role and
  its status class per the §1 inventory.
- All Markdown links in entry documents resolve from a fresh checkout.
- A document referencing another's normative statement links to it instead of
  restating it (single-home rule).
- Referencability: for every output a P0 package or a P1 planner produces,
  the §1 inventory names its home before the package starts.
