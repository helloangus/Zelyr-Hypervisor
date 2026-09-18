# P0-W05 Documentation Baseline — Verification Evidence

**Status:** Complete evidence recorded; W05 closure claimed.
**Date:** 2026-09-18 (Asia/Shanghai)
**Environment:** Documentation review against branch
`p0/w05-documentation-baseline` (baseline: merge of PR #15). Link checks run
with a local Markdown-link resolution pass; no build/gate/CI execution is
applicable.

## Results

| ID | Review | Result | Evidence / reason |
|---|---|---|---|
| W05-DV01 → P0-V09 | Class-inventory review | **passed** | Every existing `docs/` directory maps to exactly one §1 class with a status; the two unclassified locations found by the sweep (`docs/development/`, `docs/stages/` — missing READMEs, not missing classes) were brought under existing classes; no phantom class was created. |
| W05-DV02 → P0-V09 | Metadata-rules review | **passed** | §3 applied to the sample (concise guides, integration workflow, toolchain/build-target/build-profile baselines, `docs/README.md`, `docs/adr/README.md`): all carry Status/Scope/Version/Owner-change context/Supersedes after the sweep edits; no rule contradicts the `docs/README.md` mandate. |
| W05-DV03 → P0-V09 | Stage-separation review | **passed** | `docs/stages/p0/` placements conform to the §5 table (task book, plans, implementation designs/records, verification evidence each in its sublocation; p1–p8 hold task books/plans/designs only — no evidence or completion claims); W06 and W21 pointers present in the baseline. |
| W05-DV04 → P0-V09 | Entry/link review | **passed** | Link-resolution pass over `docs/`: all entry-document links resolve from a fresh checkout; the one broken entry-document link (toolchain baseline → manifest) was fixed; every `docs/` directory now has a conforming role statement. Remaining unresolved links are exclusively forward references inside P8 proposed designs (see not-exercised note). |
| W05-DV05 → P0-V09 | Bounded-edit scope review | **passed** | The implementation record maps every edit to a specific baseline rule (findings 1–5, 7); finding 6 was recorded, not executed; no style rewrite; diff contains only sweep-justified changes plus the new baseline document and routing row. |
| W05-DV06 → P0-V09/P0-V15 | Referencability walkthrough | **passed** | Three author roles walked (evidence below); each reached a complete answer from repository documents alone; no inventory gap surfaced. |
| W05-DV07 → W05 closure | Consumability review | **passed** | Read as W06 (ADR class slot + header conventions ready), W07/W21 (separation and coherence rules available to gate on / flow pointers), and a P1 planner (§1 names a home for every output before the package starts); each consumer can act without inventing policy. |

## Referencability walkthrough (W05-DV06 evidence)

1. **W06 author (ADR governance document + template):** routing row
   "Any architecture-affecting work" → ADR baseline; documentation questions
   route to the documentation baseline; the output's class slot is the ADR
   class (`docs/adr/`, normative) per §1; header rules §3 apply (Status =
   lifecycle position; Supersedes names any replaced guidance); template
   gating follows the templates class rule. Complete answer; no invented
   location.
2. **P1 Plan Agent (EL2 design, then verification evidence):** plans
   index → stage task book → design goes to `docs/stages/p1/implementation/`
   (§5: designs and records only), verification evidence to
   `docs/stages/p1/verification/` (§5: evidence and completion claims only);
   metadata rules §3 apply; no mixing permitted between the sublocations.
   Complete answer.
3. **Contributor adding a new policy baseline:** routing table
   ("Creating, versioning, or classifying documentation") → this baseline;
   class slot: Development guidance (`docs/development/`, normative) per §1;
   required header per §3; change thresholds §4 (a new class or a directory
   reclassification would be policy-decision territory, but a per-topic
   baseline inside the existing class is routine review). Complete answer.

## Not run / not exercised

- **ADR lifecycle authored:** not in W05; W06 owns it (the baseline's ADR row
  and §3 pointer deliberately pre-empt nothing).
- **Workflow admission rules:** not in W05; W21 owns them (§5 fixes locations
  and contents only).
- **Templates authored:** none created; the templates class rule is recorded.
- **Mechanical/CI coherence checks:** none added; W07/W20 own enforcement
  wiring.
- **P8 forward-reference links:** intentionally unresolved (five links in
  proposed P8 designs point at records their packages will create); not entry
  documents, so the §6 link rule does not classify them as defects. Recorded
  as an observation for the P8 implementing packages.
