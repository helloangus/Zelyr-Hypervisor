# P2-W09 Validation, Evidence Mapping, and Handoff Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P2-W09 detailed design](README.md).

## 1. Two validation layers

As with W08, two surfaces exist and must not be conflated:

1. **The integration scenarios** ([03](03-scenarios-and-evidence.md)) —
   these produce the P2-V11 evidence when executed against real boots.
   This design plans them; it runs nothing and contains no result.
2. **The integration design itself** — reviewed before and after execution
   so the evidence chain is trustworthy. The matrix below covers layer 2.

## 2. Package validation matrix (layer 2)

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W09-DV01 → P2-V11 readiness | Matrix completeness | Review [02 §3](02-configuration-matrix.md) against P2-K01–K05 | Map each K requirement to configurations/scenarios | Every K group covered; multi-bank cell resolved (present or recorded fallback per [02 §2](02-configuration-matrix.md)) | Designed coverage; not executed coverage |
| W09-DV02 → P2-V11 readiness | Expectation integrity | Review expectation files vs dumps | Check provenance, binding flags, domain declarations ([02 §5](02-configuration-matrix.md)) | Values pinned from artifacts with provenance; no invented platform facts | Expectations are honest; not that boots pass |
| W09-DV03 → P2-V11 | Execution completeness | Evidence-record review after runs | Check every configuration × repetition has a run row | Full repetition counts passed; C4/C6 on every boot; statuses are the fixed four | P2-V11 satisfaction when all passed; partial runs leave it unsatisfied |
| W09-DV04 → boundary honesty | Proof-boundary review | Review evidence and W10 drafts for scope language | Confirm no hardware/real-board claims; every record carries the §3 boundary of [01](01-scope-and-foundations.md) | Review passes | QEMU evidence stays QEMU evidence |
| W09-DV05 → drift handling | Revision discipline | Review any expectation revisions | Old evidence intact; new version + rationale recorded | No overwrite-style revision exists | Longitudinal comparability; not product correctness |
| W09-DV06 → closure | Consumer walkthrough | Design review | Read as W10 (can I fill the gate map and limitations?), P3/P4 planners (can I plan on this baseline?), W07 (did the cross-check land?) | Each consumer proceeds without new W09 work | Handoff readiness; not consumer implementations |

Layer-1 execution evidence lives only in the verification record per
[03 §5](03-scenarios-and-evidence.md). Until a full passed run exists,
P2-V11 is unsatisfied; P2-V01–V10 remain separately required and are never
claimed by W09 rows.

## 3. Error and observability model

- **Run failures** are product evidence: scenario ID, configuration,
  expectation version, observed vs expected with both inspection renders
  attached on drift ([03 §2](03-scenarios-and-evidence.md) W09-C5 rule).
- **Blocked runs** name the upstream owner (runner, P1, environment) and
  are distinct from `not run`; W10's gate map depends on this distinction.
- **Observability of the suite itself:** the matrix summary's totals and
  the not-run list make partial integration states visible at stage-gate
  review without reading every log.

## 4. Handoff checklist

Before handing W09 to review, provide:

- the changed-file list (expectation files, dumps, evidence record);
  confirmation that no runner/CI/script artifact was created, no product
  code was modified, and no expectation was loosened to pass;
- W09-DV01–DV06 records with statuses;
- the executed-run evidence (or explicit blocked/not-run per configuration
  with upstream owners) including QEMU version pinning per run;
- the C7 cross-check result delivered to W07's record (or its not-run
  entry);
- confirmed consumer readiness: W10 (evidence locations, run/not-run
  distinction, boundary wording), P3/P4 planners (baseline facts and
  runner usage pattern via W10);
- open items recorded, not resolved: QEMU version matrix expansion
  (Reserved), SMMU/machine variants (Reserved, P14+), hardware evidence
  (P15), 8-CPU matrix extension (P3).
