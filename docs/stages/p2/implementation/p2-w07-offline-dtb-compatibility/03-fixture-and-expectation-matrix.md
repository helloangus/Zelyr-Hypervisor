# P2-W07 Fixture and Expectation Matrix

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W07 detailed design](README.md).

## 1. Fixture contract

A fixture is a triple, tracked in the W07 fixture area (physical location
pending P0-W03's workspace; logical grouping is fixed here):

| Part | Form | Rules |
|---|---|---|
| Image | `.dtb` byte file | Exactly the blob as obtained; no post-editing without a provenance note |
| Expectation | expectation file, schema §3 | Data only; binding flags explicit; versioned with the image |
| Provenance | provenance note | Origin, acquisition command/source, date, tool versions, and any transformation |

The manifest lists fixtures and their labels; the harness iterates the
manifest in fixed order. Fixture labels may name platforms — as data; no
checker code does ([01 §4](01-scope-and-foundations.md)).

## 2. Provenance acquisition (implementation-time)

- **QEMU `virt`:** dump the DTB that QEMU supplies to a guest at the P1
  boot recipe's configuration, using the documented QEMU mechanism (e.g.
  dumping the generated DTB via the QEMU monitor or `-machine dumpdtb=`),
  recording the exact QEMU version and machine options in the provenance
  note. The dump command is recorded as evidence, not embedded in the
  repository as a script (P0-W09 owns runner scripting; W07 owns the
  artifact and its provenance).
- **Orange Pi 3B / RK3566:** obtain a vendor/mainline DTB for the board
  (source repository and commit recorded), compiled to DTB with the tool
  and version recorded. The board remains an *offline fixture only* (task
  book Reserved list); the provenance note states this boundary.

Blocked handling: if either artifact cannot be obtained lawfully and
reproducibly at implementation time, that fixture is recorded
blocked in the verification record and W07-DV01/DV02 take not-run entries;
the mechanism (checker + format) is still validated on synthetic fixtures
from the W01/W02 test corpora.

## 3. Expectation file schema

```text
expectation ::=
  meta:        { fixture_label, expectation_version, rationale_ref }
  rows:        [ { row_id,             # intake rule class or fact-group id
                   binding: bool,      # binding deltas fail; informational record findings
                   expected_class,     # PASS | WARN | FAIL | NOT-P2
                   expected_payload?,  # optional marked facts (e.g. cpu_count = 4,
                                       #   psci_method = hvc, bank_count = 1)
                   note? } ]
```

Rules:

- Rows correspond 1:1 to report row IDs; an expectation row with no report
  row (or vice versa) is itself a binding delta (schema drift).
- `binding = true` requires a note stating the authority for the
  expectation (for QEMU `virt`: the P1 boot-recipe configuration and
  QEMU's documented generation; for RK3566: the fixture's own vendor DTB
  content, i.e. the expectation records what this blob expresses, not what
  the board "should" express).
- Expected payload facts are compared only when marked; unmarked payload
  variation is not a delta.
- Expectation files change only with a rationale update; the harness fails
  on version mismatch with the recorded report marker.

## 4. QEMU `virt` fixture — binding expectation set

The reference platform (ADR-003). Binding rows (values to be fixed in the
expectation file from the actual dumped blob at implementation time; the
classes below are fixed by this design because they follow from the P1
boot recipe and QEMU `virt`'s documented generation):

| Row | Expected class | Binding payload marks |
|---|---|---|
| Intake executed rules | PASS | — |
| CPUs | PASS | `cpu_count` = the P1 recipe's `-smp` value |
| Boot-CPU relation | PASS | matched |
| RAM | PASS | `bank_count` per the dumped blob |
| Reservations | PASS or WARN (per blob) | count recorded |
| GIC | PASS | GICv3 class |
| Timer | PASS | — |
| PSCI | PASS | method per blob (`hvc` expected on `virt`) |
| Chosen/console | PASS | `stdout-path` present |
| Unrelated devices | WARN | count recorded |

The exact payload values are written into the expectation file from the
dumped blob (not from this document) so the fixture always agrees with its
own provenance; this design fixes only the classes and the binding flags.
Rationale: expectations derived from the artifact cannot be falsified by
this design, and deltas then genuinely indicate QEMU-version drift or
semantics regressions.

## 5. RK3566 / Orange Pi 3B fixture — shared-semantics expectation set

The fixture exercises that the *same* walker code produces coherent
results on a non-QEMU DT shape. Binding rows are limited to structure and
class-level outcomes; payload marks are informational unless the vendor
blob itself makes them certain:

| Row | Expected class | Rationale |
|---|---|---|
| Intake executed rules | PASS | a shippable vendor DTB must pass structural validation; failure is a finding to record, and binding so it cannot be ignored |
| CPUs | PASS | the fixture blob describes its CPUs; count informational |
| Boot-CPU relation | expected per blob (PASS or binding WARN/FAIL if the blob lacks the relation) — fixed from the blob at implementation time | vendor DTBs may omit `boot_cpuid_phys` agreement; the report must state it objectively |
| RAM | PASS | bank layout informational |
| Reservations | PASS/WARN per blob | `/reserved-memory` children expected on RK3566; counts informational |
| GIC | PASS (GICv3 class expected for RK3566) — binding at class level only if the blob's compatible says so | the checker reports the blob; this design does not certify the SoC |
| Timer / PSCI / console | per blob, fixed at implementation time with provenance notes | same rule |
| Unrelated devices | WARN | expected to be non-zero on a real SoC; the row exercises the aggregate-ignorance policy |

Design intent: the RK3566 fixture proves semantic *portability* of the
fact model (P2-V09's wording), i.e. no QEMU-specific assumptions leak into
walkers — not that any board is supported. If the fixture surfaces a real
gap (e.g. a fact the model cannot express), the finding is recorded for
platform planners; widening the fact model is a design change, not a
fixture edit.

## 6. Negative-fixture handoff to W08

W08 owns malformed-input harnesses. The shared substrate W07 fixes:

- the fixture triple format and manifest iteration order (W08 adds
  mutation-generated fixtures as additional triples with
  `expected_class = FAIL` binding rows);
- the `check` entry and `CheckerError` semantics (bad blobs are rows, not
  harness crashes);
- the rule that a corrupted fixture must never panic the checker — the
  W01 untrusted-input guarantee evaluated offline.

W07 itself tracks only the two real-platform fixtures plus, optionally,
synthetic fixtures reused from W01/W02 test corpora (labeled synthetic in
provenance).

## 7. Fixture regression stability

A fixture's image bytes are immutable once provenance is recorded; any
revision (e.g. a new QEMU version's dump) is a *new fixture version* with
its own expectation file and provenance, keeping the old one until the new
one has a recorded run. Rationale: regression value comes from comparing
like with like; silently replacing a fixture destroys the baseline.
