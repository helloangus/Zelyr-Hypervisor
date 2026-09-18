# P2-W07 Readiness Report Model

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W07 detailed design](README.md).  
**Contract notation:** implementation-design checklist §3 where applicable;
names are stage-local design freedom owned by this design.

## 1. Design goal of the report

The report answers exactly one question objectively: *if this blob were the
boot DTB, what would P2 intake and discovery conclude?* Every row is a
projection of a boot-code outcome; nothing in the report is an independent
judgment. This is the offline analogue of W06's no-hard-coded-view rule and
is reviewed the same way (W07-DV06).

## 2. Report schema

```text
ReadinessReport {
  meta:     { fixture_label, blob_len, blob_sha256_class (truncated digest),
              checker_semantics_marker,   # identifies the W01/W02 code state
              disclaimer: SUPPORT_DISCLAIMER },   # fixed string, §5
  intake:   [ IntakeRow; .. ],     # one per executed or skipped intake rule class
  facts:    [ FactRow; .. ],       # one per required-set fact group (§4)
  devices:  UnrelatedDevicesRow,   # aggregate §4.3
  deltas:   [ DeltaRow; .. ],      # vs the fixture's expectation file, §4.4
  verdict:  Verdict,               # §5
}
```

All lists fixed-capacity (corpus sizes are bounded by the manifest);
the report is a value; rendering to text is a pure function with stable
row order (fixture-independent ordering: intake rule order, then fact-group
order, then delta order).

## 3. `check` — the single entry

```text
Name and stability: offline::check(image: &DtbImage, params:
  &OfflineParams) -> Result<ReadinessReport, CheckerError>. Internal to the
  host tooling; called by the fixture harness and by W08's harness.
Purpose and caller: run W01 intake (offline parameterization per
  [01 §3](01-scope-and-foundations.md)) and, if intake yields a handle, W02
  normalize; map all outcomes to rows.
Inputs / outputs: image bytes + offline params (max_dtb_size override;
  skipped-rule markers). Output: report; `CheckerError` only for checker-
  internal misuse (capacity), never for blob content — bad blobs are FAIL
  rows, not errors.
Preconditions / postconditions: pre — none beyond a byte image; post —
  deterministic: identical inputs ⇒ identical report; no mutation of
  inputs; no environment or time input.
State and ownership: owns a fixed-capacity report under construction; the
  input image is only borrowed.
Concurrency/allocation context: host, single-threaded harness; no
  allocation beyond what the reused W01/W02 boot-storage policy already
  forbids (they are no-heap; the report is fixed-capacity).
Errors and failure guarantee: CheckerError::{Capacity} — stateless.
Security/authorization checks: the blob is untrusted exactly as at boot;
  all W01 untrusted-input rules apply unchanged because the code is
  unchanged; the report renders classes and counts, never blob content
  (strings shorter than the W02 fact caps are still rendered only as
  presence/length/state).
Logic (pseudocode):
    intake_out = w01::intake_offline(image, params)
    rows += map_intake(intake_out)            # §3 mapping incl. skipped rules
    match intake_out:
      Err(diag) -> facts_rows = []            # nothing else can run
      Ok(handle):
        match w02::normalize(handle):
          Err(fatal) -> rows += FAIL(fatal)
          Ok(info)   -> rows += map_facts(info) # §4 mapping
    devices = aggregate_unrelated(counters)
    deltas = expect::compare(rows, manifest.expectation(fixture_label))
    verdict = derive(rows, deltas)             # §5
    return Ok(ReadinessReport { .. })
Validation: W07-DV01–DV03, DV06.
```

### 3.1 Intake-row mapping

| W01 outcome | Report class | Row content |
|---|---|---|
| Handle produced (all executed rules passed) | PASS | rules executed, blob size, version |
| Any executed-rule diagnostic (`DtbAbsent`, `DtbSizeInvalid`, `DtbHeaderInvalid`, `DtbStructureInvalid`, `DtbReservationInvalid`, …) | FAIL | diagnostic class + structured detail class (no content) |
| Skipped offline-inapplicable rules (alignment-of-base, reachability, image overlap) | NOT-P2 informational | rule name + "offline-not-applicable" |
| W01 anomaly counters (`StructureAnomaly`) | WARN | counter values |

## 4. Fact-row mapping

### 4.1 Required-set rows

One row per fact group — CPUs, boot-CPU relation, RAM, reservations,
GIC, timer, PSCI, chosen/console, bootargs — mapped from W02's model:

| W02 outcome | Report class | Notes |
|---|---|---|
| `Usable(payload)` | PASS | payload summarized (counts, method, path-presence) — not raw strings |
| `Unsupported(reason)` | WARN | e.g. GICv2, PSCI 0.1 — boot continues; later-stage owner decides |
| `Unusable(diagnostic)` | WARN | malformed-but-tolerated fact; the diagnostic class is the row detail |
| `Absent` | WARN | the platform does not describe it; P2 continues by design |
| `NotDiscovered` | NOT-P2 | areas P2 never examines (PCI, SMMU, ACPI) — always informational |
| W02 fatal diagnostic (`CpuInventoryEmpty`, `BootCpuUnmatched`, `NoMemoryBanks`, `CapacityExhausted`) | FAIL | boot would stop here |
| W02 skip/anomaly counters | WARN (aggregate) | observability of ignored content |

Rationale anchor: FAIL ⇔ boot-fatal, WARN ⇔ boot-continues, PASS ⇔ usable,
NOT-P2 ⇔ not examined — the mapping is total over W01/W02 outcomes, so
"objective" (P2-I03) is mechanical, reviewable, and stable.

### 4.2 Binding expectations

Each fixture's expectation file (schema in
[03 §3](03-fixture-and-expectation-matrix.md)) marks rows binding or
informational. A binding row whose class (or marked payload fact) differs
from the report is a `DeltaRow{row, expected, actual, binding=true}`.

### 4.3 Unrelated devices row

```text
UnrelatedDevicesRow { skipped_node_count, skipped_property_count,
  anomaly_count, class: WARN-always }
```

Always WARN, never FAIL (README Decision 5): presence of devices outside
the P2 required set is normal on real boards and says nothing about P2
readiness; the row exists so the ignorance is visible and countable.

### 4.4 Delta rows

`DeltaRow { row_id, expected_class/payload, actual_class/payload, binding:
bool }` — binding deltas feed the verdict; informational deltas are
recorded findings for platform planners.

## 5. Verdict and the no-support boundary

```text
derive():
  verdict = P2Discoverable   if no FAIL row and no binding delta
          = NotP2Discoverable otherwise
```

`SUPPORT_DISCLAIMER` is a fixed report line, exact wording fixed by this
design (implementation renders it verbatim): "Readiness describes P2
boot-description discovery only. It does not claim or imply EL2 runtime
support, board support, BSP support, or driver support for any platform."
Rules:

- The disclaimer is part of every report and every rendered artifact;
  removing it is a review failure.
- No verdict, class, or row may use the words "supported"/"support" about a
  platform (the disclaimer excepted); class names are the fixed set above.
  Mechanical review: W07-DV05 scans the report vocabulary.
- A PASS verdict on the RK3566 fixture is a statement about the *fixture
  blob*, not the board: the fixture may be a vendor DTB variant, and QEMU
  is the only platform with runtime claims (via W09, and even there only
  for the reference platform).

## 6. Explicitly unauthorized interfaces

No mutation API on the report; no re-render that re-derives rows (render
reads stored rows only); no comparison against boot logs inside the checker
(W09 records that comparison); no exit codes (no CLI); no allowlist of
"good boards"; no per-board code path. Checker behavior differs between
fixtures only through their data.
