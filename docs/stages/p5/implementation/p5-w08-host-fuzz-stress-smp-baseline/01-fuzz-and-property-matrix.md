# P5-W08 Fuzz and Property Matrix

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P5-W08 detailed design](README.md).

## 1. Seam model

Each seam is a host-callable, hardware-independent entry to one delivered
validation boundary, exercised over plain inputs with an injected test
environment (fake object table, fake capability state, fake address-space
descriptor — as provided by the boundary's own design, not invented here).
Seam names below are logical; the delivered designs' names govern.

| Seam | Boundary / owner | Input model | Deliverable consumed |
|---|---|---|---|
| S-DECODE | request decode (W02, as exposed through W06's pipeline) | raw request register words | typed request or structural class |
| S-REF | reference resolution (W04) | arbitrary handle bit patterns over a fake table | live reference or controlled class |
| S-AUTH | authority check (W05) | caller + reference + right class over fake capability state | grant or denial class |
| S-RANGE | Guest-data validation (W03) | address/length/direction over a fake address-space descriptor | accepted access or controlled failure |
| S-DISP | dispatch pipeline composition (W06) | full synthetic requests driving S-REF/S-AUTH/S-RANGE | one outcome + category |

A boundary delivered without a host-callable seam is a blocked prerequisite
(README decision 1): record it against the owning package; do not approximate
the boundary inside the harness.

## 2. Fuzz matrix

Each row is a fuzz target over one seam. "Oracle" lists the failure
conditions per run; all oracles of README decision 2 apply to every row
additionally.

| ID | Target | Input generation | Expected observable | Oracle (failure iff) | Proves / does not prove |
|---|---|---|---|---|---|
| FZ-01 | S-DECODE | uniformly random 4–8 register words; plus structured mutations of valid requests | every input yields exactly one outcome in the delivered vocabulary | any panic/abort/hang; any outcome outside the vocabulary; any non-structural class for structurally invalid input | the decoder cannot be crashed or confused into an unclassified answer; not that unknown protocols are safe |
| FZ-02 | S-REF | arbitrary 64-bit patterns; plus bit-flips around table-present values; plus stale-generation neighbors | every input yields `live` or a controlled invalid class; `live` only for currently-valid entries | any acceptance of a forged/stale/wrong-generation value (accepted-invalid); panic/hang | stale/forged references cannot resolve (INV-P5-03) under random attack; not the table's concurrency correctness (scenario SMP-*) |
| FZ-03 | S-AUTH | random caller/reference/right-class combinations over fake state, incl. revoked and cross-object pairs | success iff a live grant covers the exact required right; otherwise the precise denial class | any unauthorized success; any denial-class conflation beyond the delivered vocabulary; panic/hang | rights enforcement resists random authority probing (INV-P5-04); not W05's internal representation |
| FZ-04 | S-RANGE | random (address, length, direction) incl. boundary values: 0, 1, max, max-1, overflow neighbors, page edges | accepted iff the fake descriptor maps the whole range with the required permission; otherwise a controlled failure | any acceptance of an unmapped/partial/wrong-direction range; any arithmetic wrap producing acceptance (accepted-invalid); panic/hang | the range check never accepts a wrap or a partial mapping (INV-P5-02); not real Stage-2 behavior |
| FZ-05 | S-DISP | compositions of the above driving the full pipeline, incl. valid request shapes with hostile payloads | exactly one outcome + one category per call; Guest-result buffer untouched on S1–S4 denials (observed via the fake environment) | any panic from Guest-controlled values; any missing/duplicate category; any state mutation on read-only denials | end-to-end containment under randomized input (W06-DV05's host side); not Guest-side observable behavior (W07) |

## 3. Property tests

Deterministic property suites complement the randomized rows; each property
is checked exhaustively over generated cases with recorded seeds.

| ID | Property | Model | Pass condition | Proves / does not prove |
|---|---|---|---|---|
| PR-01 | create → destroy → recreate never reactivates a stale reference | fake W04 table | after each cycle, all pre-destroy references fail; only the new reference resolves | lifecycle correctness over the generated space; not concurrency |
| PR-02 | grant → check → revoke → check | fake W05 state | check succeeds only between grant and revoke; identical inputs flip result exactly at revoke | revoke enforcement (INV-P5-06); not revocation trees |
| PR-03 | accepted iff fully mapped and permitted | generated fake address spaces incl. adversarial edge descriptors | S-RANGE predicate equals the descriptor predicate for every case, with no wrap | range-check completeness; not hardware page-walk equivalence |
| PR-04 | decode totality | exhaustive small field spaces + sampled large ones | every input maps to exactly one class; mapping is stable across repeated calls | determinism and totality of the boundary; not compatibility policy |
| PR-05 | quota exhaustion is clean | fake table/capability state at capacity limits | exhaustion yields the delivered resource outcome; no partial allocation; release restores capacity | determinate resource semantics (Plan-Agent §57); not real allocator behavior |

## 4. Oracle definitions (normative)

These apply to every randomized and property run; they implement README
decision 2:

- **O1 panic/abort:** the run terminates by panic, abort, or
  trap-like failure. Guest-caused input must never trigger this.
- **O2 hang:** the run exceeds the declared per-run watchpoint.
- **O3 unclassified outcome:** a result outside the delivered boundary's
  result vocabulary (including "success" shapes the boundary never defines).
- **O4 accepted-invalid:** the seam accepts an input its contract rejects —
  stale/forged reference, unauthorized success, unmapped/wrapped range,
  wrong-direction write.
- **O5 audit mismatch:** the post-run invariant audit of
  [02 §4](02-stress-smp-scenarios.md) does not hold.
- **O6 disclosure:** any evidence artifact (log, counter dump) contains a
  Host pointer or Guest buffer content.

A run with any oracle hit is a **failed** run: the finding is recorded with
the reproducing seed and minimal input, fixed, and the run repeated. Oracle
hits are never waived to make a suite pass.

## 5. Seed, duration, and repetition rules

- **Seeds:** every randomized run takes an explicit, recorded seed (or
  seed-file) and derives all randomness from it; a rerun with the same seed
  and configuration must reproduce the same coverage and result.
- **Bounds:** every run declares an iteration bound and a wall-clock cap in
  its record; both are recorded with the result. A run that stops on its cap
  is recorded as cap-limited, not exhaustive.
- **Smoke versus deep:** the **smoke configuration** is a fixed, small
  per-seam seed/bound set recorded in this package's record and handed to
  W09 as the regression row; **deep runs** are larger local evidence and are
  never required for regression. Smoke must include, at minimum: FZ-01–FZ-05
  with recorded seeds and PR-01–PR-05 over their declared case counts.
- **Repetition:** regression reruns use identical smoke configuration;
  changing a smoke seed or bound is a record-level change owned by this
  package's maintenance rule, visible to W09 — never a silent harness edit.
- **Persistent corpora:** Reserved. If a corpus is later introduced, its
  format, location, and minimization rule require a new approved decision;
  this design creates none (the working tree constraint against pre-created
  corpora holds for design time; the implementing agent may only produce
  run-local inputs as recorded evidence).
