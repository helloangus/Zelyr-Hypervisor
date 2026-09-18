# P2-W06 Validation, Error Model, and Handoff Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P2-W06 detailed design](README.md).

## 1. Scope of validation for this package

W06's own evidence is host-side: composition, checks, and rendering are
pure logic over source values ([01 §2](01-scope-and-foundations.md)).
QEMU observation of rendered inspection during real boots is W09 evidence
(P2-V11); the no-hard-coded-view property is partly a review (W06-DV07).
Planning those here supplies neither. No row below may be reported as
proving more than its stated boundary.

## 2. Error, security, and observability model

- **Error model.** Two failure shapes only: `ConsistencyBroken{check,
  detail}` (fatal invariant stop at boot per P0-W14; assertable value in
  host tests) and `RenderError` (sink failure; report remains valid).
  Stateless failure: nothing is published on composition error, and a
  failed render leaves the report untouched. Inherited capacity failures
  surface as `CapacityInherited` rather than new truncation behavior.
- **Security model.** Inspection renders no untrusted content (no DT
  strings, no property values, no blob bytes — [01 §3](01-scope-and-foundations.md)
  rule 4), reads only through published queries, and cannot mutate any
  source. It adds no new trust boundary; its only security obligation is
  not to become a content-echo channel and not to mask divergence.
- **Observability.** W06 *is* observability for the P2 state: one
  deterministic text render per inspection point, stable section order for
  cross-boot comparison, named check IDs so a failure says exactly which
  cross-source invariant broke. No telemetry events are emitted (Reserved);
  the render through the P0 diagnostic channel is the entire output surface.

## 3. Validation matrix

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W06-DV01 → P2-V08 (P2-H01) | Platform section live-derived | Host test over W02 fixture facts, mutated | Change a fact state / CPU count / counter → section changes correspondingly | Section equals expectation computed from the mutated facts; no constant section content | Projection correctness; not W02's discovery correctness |
| W06-DV02 → P2-V08 (P2-H02) | Map section live-derived | Host test over W03 sealed-map fixtures, mutated | Change a clip/protected class → map section and C1/C2 inputs track | Section matches `summary()` and span queries exactly | Map rendering; not W03's map construction |
| W06-DV03 → P2-V08 (P2-H03) | Page-allocator stats live-derived | Host test driving real W04 allocate/free over fixture map | After each op, section matches `stats()` | Every rendered number equals the query result at composition time | Allocator rendering; not allocator safety (W04's evidence) |
| W06-DV04 → P2-V08 (P2-H03) | Heap stats live-derived | Host test driving real W05 alloc/dealloc | After each op, heap section matches `HeapStats` | Same as DV03 for heap fields | Heap rendering; not heap safety |
| W06-DV05 → P2-V08 (P2-H04) | Consistency checks | Host unit tests with injected divergence per check | Hand-build divergent sections (C1 mismatch, C2 equation break, C3 domain break, C4 ownership break) | Each check fires with its own `CheckId`; C1–C4 pass on consistent fixtures; first-failure ordering holds | Divergence detection at inspection points; not continuous monitoring |
| W06-DV06 → P2-V08 | Render determinism | Host test | Render the same report twice and across process runs; compare bytes | Identical bytes; fixed section order | Stable evidence format; not target font/log transport |
| W06-DV07 → P2-V08 (P2-H04) | No hard-coded view | Design + code review | Mechanical scan: no section field has a literal initializer independent of sources; every field assignment names a source or a check-derived value | Review passes with recorded method | The P2-H04 core property at review strength; not runtime proof (DV01–DV05 are the runtime half) |
| W06-DV08 → P2-V08 (P2-H01) | All-state rendering | Host test | Reports containing every `FactState` variant, `NotRecorded` boot context, empty/absent collections | Renders without panic; states remain visually distinct marks | Robustness across honest states; not hostile-input safety (W06 reads no untrusted input) |
| W06-DV09 → P2-V08 | Bounded, allocation-free render | Construction review + host bound test | Maximum-size fixture report (all capacities full); allocation-failure hooks if the P0 baseline provides them | Output bounded by the capacity product; no allocation call | Rendering cannot OOM a diagnostic path; not line-level log performance |
| W06-DV10 → W06 closure | Consumer walkthrough | Design review | Read as W09 (can I set QEMU expectations from the section schema?), W08 (can I assert check IDs?), W10/P3/P4-reviewer (can I assess readiness without a control API?) | Each consumer proceeds without new W06 work | Handoff readiness; not consumer implementations |

Evidence statuses are passed / failed / blocked / not run, recorded with
command, input, environment, timestamp in the verification record. Host
validation does not prove QEMU boot rendering, real-platform accounting
magnitudes, or the A5/A6 P1/P0 integrations; those appear as explicit
not-run entries here and as W09's scope.

## 4. Handoff checklist

Before handing W06 to review, provide:

- the changed-module list; confirmation of zero `unsafe`, zero allocation
  paths, zero locks, zero board/platform names, zero new dependencies;
- W06-DV01–DV10 evidence paths and statuses, including not-run entries
  (QEMU render → W09; boot wiring on target → P1/P0 integration; boot
  context → A5);
- confirmed consumer readiness: W09 (section schema + check IDs as
  integration expectations), W08 (check-ID assertions), W10 (inspection as
  a review input, explicitly not a control API);
- open items recorded, not resolved: physical module placement (P0-W03
  workspace), A5 boot-context record pending P1, `hv-platform-inspect`
  mode Reserved, render text format stage-local (W09 expectation files
  co-revise if it changes);
- explicit statement that W06 adds no public API beyond
  [03](03-code-contracts-inspection.md) and modifies no W02–W05 contract.
