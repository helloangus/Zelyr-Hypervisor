# P1-W12 Contract Set and Review Design

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W12 detailed design](README.md).

## 1. Authoritative document groups

All documents live under `docs/stages/p1/contracts/` (parent README decision
1), carry the normative status header, and state accepted boundaries with
pointers to their owning sources. Content rules below bound each document;
informative detail is allowed but must not contradict a required statement or
introduce a new decision (assembly rule).

| # | Document | Required content (source boundaries it assembles) | Non-responsibility |
|---|---|---|---|
| 1 | `aarch64-boot-contract.md` | canonical QEMU `virt` boot path and invocation; entry/image/loading assumptions; Non-secure EL2 requirement and boot-CPU statement; MMU/cache assumptions at entry; boot parameters and supplied-DTB treatment (raw passthrough position, no discovery claim); minimum memory; rejection boundary and its diagnostics (from W01, W03 capability classification, W09 `entry`/`runtime` phases) | no platform discovery design; no board support; no guest protocol |
| 2 | `el2-initialization-contract.md` | the ordered lifecycle: phases, prerequisites, legal transitions, deferred-marker model, failure-routing matrix, stable state and its entry conditions; the unowned pre-vector window limitation (from W09) | no mechanism internals of W01–W08; no general service lifecycle |
| 3 | `host-address-space.md` | Host Stage-1 mapping classes with permission/execution/memory attributes; the controlled transition and post-MMU behavior; explicit temporary assumptions and the no-identity-map-promise statement (from W08) | no dynamic memory manager, allocator, or Stage-2 content |
| 4 | `exception-diagnostics-contract.md` | vector coverage by category; origin/context classification; recoverable/fatal boundary; diagnostic field list (build identity, CPU/EL, PC, syndrome, fault address, phase, register context); fatal terminal behavior and non-recursion statement (from W05, W07; phase attribution from W09) | no GIC/IRQ subsystem; no recovery policy; no crash storage |
| 5 | `reference-qemu-environment.md` | reference invocation via the P0-W09 runner entry; the W01 canonical recipe reference; frozen marker rules and verdict semantics; timeout/exit classification; evidence layout and the 100-cycle procedure's acceptance rule; environment facts recorded by W10 (from W01, W10, P0-W09 contract) | no CI configuration; no performance data; no matrix beyond the canonical boot |
| 6 | `known-limitations.md` | consolidated limitations with source pointers: temporary reference-console and static-boot assumptions; unowned pre-vector window; environment-only negative coverage; single-boot-CPU scope; absence of allocator/discovery/GIC/SMP/Guest; unsafe/API/dependency reporting status and where those reports live; any open finding recorded by W01–W11 | no new limitation invented; no mitigation promised |
| 7 | `p2-handoff.md` | the stable-assumptions list P2 may rely on; the explicit non-goals; the deliverable-to-consumer mapping of the parent README's handoff table with document pointers; the statement that P2 owns discovery, memory map, and allocation (from the parent README table; P2 task book §2 as named inputs) | no P2 design content; no API or module freeze |
| 8 | `stage-gate-evidence-map.md` | P1-V01–P1-V21 mapped to evidence locations (existing records or future `docs/stages/p1/verification/` paths); task-book exit criteria 1–7 mapped to their evidence rows; the completion-review questions from task book §8 restated with where each answer must be recorded; the rule that completion claims live only in verification evidence | no evidence produced here; no completion status asserted |

Each document is the single authoritative home of its statements; the plans,
designs, and records it cites remain the authority of their own content.
Cross-documents link instead of restating.

## 2. Review workflow

### Step 1 — collect accepted sources

Target: implementation record
(`../p1-w12-p1-documentation-handoff-record.md`, created in this step).

Work: inventory the accepted state of W01–W11 — designs, implementation
records, verification records — and their recorded open issues, findings, and
limitations. Record per work package: which sources are accepted, which
boundaries they fix, and which open items exist. Identify contradictions
between sources now.

Suggested observation: read-only inventory (`git ls-files`; read the records
that exist).

**Acceptance:** the record lists, for every package, its accepted sources and
open items; every contradiction is named.  
**Failure/blocker:** a package with no accepted boundary leaves its contract
document blocked, recorded as such; it is not drafted from plans alone.

### Step 2 — author the contract set

Target: the eight documents of §1.

Work: write each document to its content contract, citing sources for every
normative statement. Apply decision 4's header and claim rules. Build the
evidence map with real paths for existing records and explicit future paths
for missing evidence.

**Acceptance:** every required-content item of §1 is present with a source
pointer; no statement lacks an authority; the map covers all 21 validation
IDs.  
**Failure/blocker:** a contradiction found while writing is a finding
returned to the owning package (parent README decision 3); the affected
document section is drafted as "conflicting sources, finding recorded", never
silently resolved.

### Step 3 — link and governance review

Work: resolve every relative link from a fresh checkout; verify one plan per
package across the stage, an objective condition for every validation ID,
acyclic dependencies in the stage map, and that W12 documents added no new
decision.

**Acceptance:** no broken links; the P1-V21 checklist items each pass with a
pointer.  
**Failure/blocker:** a structural planning defect (missing plan, cyclic
dependency) is a stage-governance finding for the coordinator — not fixable
inside W12.

### Step 4 — no-claim review

Work: search every W12 document for completion, validation, and performance
language; verify each evidence reference says "located at" (or cites the
record's own truthful status), and every status header is truthful.

**Acceptance:** zero claim violations (W12-DV06).  
**Failure/blocker:** any violation is corrected in W12's own documents or
reported where the claim originated.

### Step 5 — closure and handoff

Work: confirm the handoff checklist (§5), record review outcomes in the
verification record as required by the plan's acceptance, and state the
P1-V20/P1-V21 status truthfully. Completion of the stage is claimed only
through the completion review the evidence map points to — never in W12
documents.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W12-DV01 → P1-V20 | Contract-consistency review | read the eight documents against each other and their sources | no document contradicts a source or sibling; every normative statement has an authority; status headers truthful | internal consistency of the assembled set; not the correctness of the underlying contracts |
| W12-DV02 → P1-V20 | Coverage review | map §1 required content and W01–W11 boundaries to document sections | every task-book W12 scope item and every accepted boundary is represented or its absence recorded with reason | completeness of the set; not that boundaries were correctly established (that is W01–W11 evidence) |
| W12-DV03 → P1-V21 | Governance review | check one-plan-per-package, objective conditions per ID, acyclic dependencies, link resolution | all structural checks pass with pointers; any defect is a recorded coordinator finding | stage governance as documented; not future compliance |
| W12-DV04 → P1-V21 | Evidence-map completeness review | enumerate P1-V01–P1-V21 against the map | all 21 IDs mapped; existing evidence cited with real paths; missing evidence marked as future locations, never as present | the map is complete and honest; not that evidence exists |
| W12-DV05 → P1-V20 | P2 consumability review | read `p2-handoff.md` and the contract set as each named P2 consumer (W01, W02, W03, W04/W05, W06, W08, W09, W10) | each can locate its assumed inputs and non-goals without inferring P1 implementation detail | handoff usability; not P2 design correctness |
| W12-DV06 → W12 closure | No-claim review | Step 4 search across all W12 documents and the record | zero completion/validation claims; run/not-run statements truthful | governance cleanliness; nothing else |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with method, scope, timestamp, and reason. Reviewing documents satisfies only
P1-V20/P1-V21 review conditions; it never substitutes for the execution
evidence other IDs require.

## 4. Error, observability, and conflict-handling model

**Errors.** W12's failure mode is documentation defect: a missing required
section, a broken link, an unsupported statement, or a claim violation. Each
maps to the validation matrix row that catches it and is recorded as failed
until corrected.

**Conflicts.** Source contradictions are first-class findings: recorded in
the implementation record with both sources, returned to the owning package,
and reflected in the affected document as an explicit open item. W12 never
resolves an architectural or ownership conflict editorially; a conflict that
looks architectural is labeled for the ADR/change process per AGENTS.md.

**Observability.** The evidence map is the stage's observability surface:
every validation ID's evidence status is findable from one document. W12 adds
no telemetry, logging, or runtime behavior.

## 5. Handoff checklist

Before handing W12 to a reviewer, provide:

- the exact created-file list (the eight documents plus W12's own record
  paths; nothing else in the repository touched);
- the source inventory from step 1, with contradictions and their owners;
- W12-DV01..DV06 outcomes with pointers, including blocked items and the
  packages they block on;
- confirmation that no new technical decision, no ADR change, no P2 design
  content, and no completion claim was introduced;
- confirmation that all links resolve from a fresh checkout and the evidence
  map distinguishes existing evidence from future locations;
- open items: contract-set location ratification by the documentation-taxonomy
  authority; any unresolved source finding; the stage completion review's
  dependency on evidence still to be produced by W01–W11 — recorded, not
  resolved here.
