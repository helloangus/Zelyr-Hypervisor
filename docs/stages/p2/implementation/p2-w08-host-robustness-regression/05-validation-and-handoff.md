# P2-W08 Validation, Evidence Mapping, and Handoff Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P2-W08 detailed design](README.md).

## 1. Two validation layers

W08 has two distinct validation surfaces, which must not be conflated:

1. **The suite's scenarios** ([02](02-matrices-input-and-map.md),
   [03](03-matrices-allocator-stress-determinism.md)) — these produce the
   P2-V10 evidence when run. This design plans them; it runs nothing.
2. **The regression design itself** — reviewed before handoff so a flawed
   suite cannot manufacture false assurance. The matrix below covers layer
   2; its "evidence" is review records, not scenario results.

## 2. Package validation matrix (layer 2)

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W08-DV01 → P2-V10 readiness | Assertion fidelity | Matrix review against sibling contract files | For each row: does the expected observable name a published contract outcome verbatim? | Every expected observable traceable to a contract file section; zero re-derived semantics | The suite tests the real contracts; not that contracts hold (scenario runs do that) |
| W08-DV02 → P2-J coverage | Group coverage | Coverage review of matrices vs P2-J01–J05 wording | Map each J requirement to its rows | Every J group has rows; hard gate has the soak (S308); determinism has end-to-end rows (S503/504) | Designed coverage; not executed coverage |
| W08-DV03 → reproducibility | Hermeticity review | Review harness design for wall-clock, network, filesystem, parallelism, unseeded randomness | Design walk-through with checklist | No such input exists; fixed order; seeds recorded | Suite determinism by construction; not a substitute for S505's replay check |
| W08-DV04 → P2-V10 | Suite executability | Dry-run of the harness on a minimal fixture set (execution belongs to implementation) | Invoke the suite entry; check per-row records produced | Every row yields a status row per [04 §5](04-automation-and-evidence.md); aggregation computes | The machinery works; not that P2-J properties hold |
| W08-DV05 → evidence quality | Evidence format review | Inspect produced record skeleton against [04 §5](04-automation-and-evidence.md) schema | Field-by-field check | Schema complete; statuses are the fixed four; seeds present where required | Evidence usability for P2-V10/P2-V13 review; not evidence existence |
| W08-DV06 → closure | Consumer walkthrough | Design review | Read as W09 (can I build integration expectations on this baseline?), W10 (can I fill the evidence map?), owning packages (are my contracts exercised fairly?) | Each consumer proceeds without new W08 work | Handoff readiness; not consumer implementations |

Layer-1 execution evidence (the S-matrix rows themselves) is recorded only
in the verification record per
[04 §5](04-automation-and-evidence.md), with statuses passed / failed /
blocked / not run per row. Until a real run exists, P2-V10 is unsatisfied —
this design creates no result, and no document may claim otherwise.

## 3. Error and observability model for the harness

- **Harness failures** (schema mismatch, missing fixture, capability gap)
  are `blocked` rows naming the upstream owner — distinguished from
  scenario `failed` rows, which are always product evidence.
- **Assertion output** on failure prints scenario ID, expected observable,
  actual observable, and the owning package ID; no internal state dumps
  beyond what published query APIs expose.
- **The suite is its own observability target:** S505's replay rows make
  suite-level nondeterminism detectable by the suite itself.

## 4. Handoff checklist

Before handing W08 to review, provide:

- the harness/scenario file list; confirmation that no production module
  of W01–W06 was modified, no new dependency added, no `unsafe` introduced
  (beyond what the host-test baseline already governs), no QEMU/CI artifact
  created, and no timing assertion written;
- W08-DV01–DV06 review records with statuses;
- the scenario-run evidence (when execution happened) or the explicit
  blocked/not-run statement per group with reasons;
- confirmed consumer readiness: W09 (safety baseline + scenario IDs),
  W10 (evidence map rows + not-prove boundaries verbatim), owning packages
  (scenario-to-contract traceability);
- open items recorded, not resolved: P0-W08 entry point (A8), physical
  placement (P0-W03), Reserved fuzzing/formal extensions, seed/scale
  revision policy ([04 §3](04-automation-and-evidence.md)).
