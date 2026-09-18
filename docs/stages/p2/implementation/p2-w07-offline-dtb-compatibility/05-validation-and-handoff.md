# P2-W07 Validation, Error Model, and Handoff Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P2-W07 detailed design](README.md).

## 1. Scope of validation for this package

W07's own evidence is host-side: the checker runs fixtures offline and
records objective reports (P2-V09). The negative-regression *harness* over
malformed inputs is W08 (P2-V10) consuming this package's format and entry;
QEMU boot-time cross-check of the `virt` expectation is W09 (P2-V11).
Planning those here supplies neither, and no row below proves board
runtime behavior — by design ([02 §5](02-readiness-report-model.md)).

## 2. Error, security, and observability model

- **Error model.** Bad blobs are never checker errors: every intake
  diagnostic and W02 fatal becomes a FAIL row (that is the product).
  `CheckerError` exists only for harness misuse (capacity). Fixture-test
  failure means: a binding delta, a schema mismatch, or a non-terminating/
  crashing run — each distinct and recorded. A crash or hang on a fixture
  is a defect against W01's bounded-validation guarantee and blocks closure.
- **Security model.** The blob is untrusted and the checker inherits W01's
  enforcement by reusing its code (bounds-before-reads, checked arithmetic,
  caps, no allocation). The report renders classes, counts, and truncated
  digests only — no blob content, no property strings. Provenance notes
  record artifact origin so the corpus cannot silently substitute blobs.
  The no-support boundary ([02 §5](02-readiness-report-model.md)) is itself
  a security-adjacent honesty control: it prevents fixture outcomes from
  being cited as porting evidence.
- **Observability.** Every fixture run yields a rendered report (stable row
  order), a delta list, and a verdict; the corpus run yields a one-line
  summary per fixture. Evidence statuses passed / failed / blocked / not
  run are per fixture and per validation row, with command, environment,
  and timestamp in the verification record.

## 3. Validation matrix

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W07-DV01 → P2-V09 (P2-I04) | QEMU `virt` fixture report | Harness run over the `virt` triple | `check` + expectation comparison | Report produced; all binding rows match; verdict P2-discoverable; disclaimer present | The reference blob is P2-discoverable offline; not that a live boot succeeds (W09) |
| W07-DV02 → P2-V09 (P2-I05) | RK3566 fixture report | Harness run over the RK3566 triple | Same technique | Report produced; binding rows per [03 §5](03-fixture-and-expectation-matrix.md) match; verdict recorded | Shared semantics produce coherent results on a non-QEMU DT shape; not board support (P15) |
| W07-DV03 → P2-V09 (P2-I03) | Mapping objectivity | Unit tests over synthetic outcomes | Inject every W01 diagnostic class, W02 fact state, and W02 fatal; assert mapped class | Mapping table holds with no default-guess arm; FAIL ⇔ boot-fatal, WARN ⇔ boot-continues | The report is a mechanical projection; not that boot outcomes are correct (W01/W02's evidence) |
| W07-DV04 → P2-V09 (P2-I04) | Expectation drift detection | Mutation test on expectations | Flip one binding expectation; re-run | Binding delta reported; verdict flips to not-P2-discoverable; informational flips do not affect verdict | Expectations are load-bearing; not QEMU behavior |
| W07-DV05 → P2-V09 | No-support-claim boundary | Vocabulary + artifact review | Scan report vocabulary and verdict wording; confirm fixed disclaimer; confirm no checker code names a platform | Review passes with recorded method | The honesty boundary; not downstream quoting discipline |
| W07-DV06 → P2-V09 (P2-I01) | Single semantics | Code-identity review | Confirm checker contains no DT parsing/matching and calls W01/W02 entries unmodified; confirm skipped-rule rows | Review passes; NOT-P2 rows list exactly the offline-skipped rules | Offline result predicts boot behavior for the same blob; not a substitute for boot evidence |
| W07-DV07 → P2-V09/P2-V10 | Corpus robustness + W08 substrate | Harness run with corrupted/synthetic fixtures | Truncated/mutated corpus blobs | FAIL rows with classes; no panic, no hang; W08 can add mutation fixtures in-format | Bounded offline rejection; W08 owns deeper negative matrices |
| W07-DV08 → W07 closure | Consumer walkthrough | Design review | Read as W08 (can I extend the corpus and harness?), W10 (can I record the deliverable and boundary?), platform planners (can I read a candidate DTB's P2 gaps?) | Each consumer proceeds without new W07 work | Handoff readiness; not consumer implementations |

Evidence statuses are passed / failed / blocked / not run, per fixture and
per row, with command, input, environment, timestamp. Offline checking
does not prove EL2 boot success, hardware behavior, or fixture-board
runtime support; the QEMU boot-time cross-check is W09's and real-board
work is P15's — both appear as explicit not-run entries here.

## 4. Handoff checklist

Before handing W07 to review, provide:

- the changed-file list (checker modules, fixture triples, manifest) with
  the confirmation that W01/W02 code was not modified and no dependency,
  `unsafe`, CLI, or CI wiring was added;
- fixture provenance notes (origin, commands, tool versions, dates) for
  every tracked artifact, and explicit blocked/not-run entries for any
  unobtained fixture;
- W07-DV01–DV08 evidence paths and statuses, including the corrupted-
  fixture robustness rows;
- confirmed consumer readiness: W08 (format + entry + mapping),
  W10 (deliverable record with the verbatim disclaimer boundary), W09
  (the `virt` expectation file for boot-time cross-check), platform
  planners (readiness signal, not a BSP contract);
- open items recorded, not resolved: physical placement of host tooling
  (P0-W03), QEMU-version drift policy for future dumps
  ([03 §7](03-fixture-and-expectation-matrix.md)), Reserved CLI/mode.
