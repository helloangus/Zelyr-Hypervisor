# P2-W08 Scenario Matrices — Malformed DTB and Memory Map

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W08 detailed design](README.md).  
**Row semantics:** each row is one scenario with a fixed ID. Input/
precondition, expected observable, pass condition, and proof boundary are
normative. "Technique" guides the implementer; the command spelling is not
the contract ([README](README.md) authority constraints).

## 1. Column key

- **Expected observable** names a published contract outcome (W01
  diagnostic class, W03 `MapFatal` variant, counter, or documented
  property).
- **Pass condition** is what the evidence row must show.
- **Proves / does not prove** restates the [01 §4](01-scope-and-foundations.md)
  boundary for that row.

## 2. W08-S1xx — malformed DTB (P2-J01)

Inputs are DTB byte images built with the W07 fixture format (mutation
fixtures labeled synthetic with generation rule in provenance;
[W07 §6](../p2-w07-offline-dtb-compatibility/03-fixture-and-expectation-matrix.md)).
Base blob: a known-valid fixture from the W01/W02 corpora.

| ID | Input / precondition | Expected observable | Pass condition | Technique | Proves / does not prove |
|---|---|---|---|---|---|
| W08-S101 | Empty/zero-length image | `DtbAbsent` | Err row with class; no reads | Direct call | Absence gate; not firmware behavior |
| W08-S102 | Length < 40 (header truncated at each boundary: 0, 20, 39) | `DtbSizeInvalid` (3 rows) | Class + detail per row | Parametric truncation | Size floor; not real-world truncation shapes |
| W08-S103 | Length > configured max; `total_size` ≠ supplied length | `DtbSizeInvalid` (2 rows) | Same | Field edits | Size ceiling and consistency check |
| W08-S104 | Magic corrupted (each byte flipped in turn) | `DtbHeaderInvalid` | Class; no cursor created | Byte sweep | Magic gate; not exhaustive bit-flip coverage |
| W08-S105 | `version` = 16; `last_comp_version` = 18 | `DtbHeaderInvalid` (2 rows) | Class | Field edits | Version policy enforced (W01 Decision 3) |
| W08-S106 | Each block offset/size pair: unaligned, out of range, or overflowing `total_size` | `DtbHeaderInvalid` | Class + offending field class | Parametric edits | Span arithmetic gate; not fuzz coverage |
| W08-S107 | Structure block: truncated mid-token, unknown token, unbalanced BEGIN/END, missing `FDT_END`, trailing token after `FDT_END`, `FDT_END` not last | `DtbStructureInvalid` (one row per case) | Class per case | Synthetic token streams | Token-stream gate; not semantic node checks |
| W08-S108 | Property `nameoff`/`len` out of strings/structure bounds; property name not NUL-terminated; node name with interior NUL or empty non-root | `DtbStructureInvalid` (one row per case) | Class per case | Synthetic streams | Encoding gates; not content validity |
| W08-S109 | Depth or node/property count above caps (caps lowered in test config) | `DtbStructureInvalid` with cap detail | Class + which cap | Config-driven oversize | Iteration bounds are real (W01-DV06 continuation); not production cap adequacy |
| W08-S110 | Reservation list: missing terminator, terminator beyond bounds, entry count above cap | `DtbReservationInvalid` (3 rows) | Class per row | Synthetic rsvmaps | Reservation gate; not W03 semantics |
| W08-S111 | Valid structure with anomaly shapes (depth exactly at cap, empty non-root node, non-printable property-name bytes) | Runs; anomaly counters incremented | `StructureAnomaly` counters recorded; not rejected | W01 §6 shapes | Anomaly observability; not rejection behavior |
| W08-S112 | Random single-byte mutations of the valid base (seeded sweep, N = 256) | For each mutant: a W01 diagnostic class or a valid handle — never a panic, hang, or out-of-bounds accessor result | Mutation loop completes; zero unexpected outcomes | Seeded mutation loop | Bounded-rejection robustness over a sample; not adversarial completeness |
| W08-S113 | Every S1xx rejection re-run through the W07 `check` entry | FAIL rows with the same classes; checker never errors | Class equality boot-path vs offline for each mutant | Cross-entry comparison | Single semantics offline/boot (W07-DV07 substrate); not W07's own fixture rows (its DV01/02) |

## 3. W08-S2xx — map conflicts and range overflow (P2-J02)

Inputs are W02-shaped fact records (synthetic, from the W03 test corpus)
fed to the W03 builder/seal contracts.

| ID | Input / precondition | Expected observable | Pass condition | Technique | Proves / does not prove |
|---|---|---|---|---|---|
| W08-S201 | RAM bank base or length not 4 KiB-aligned | `MapFatal::UnalignedRange` | Class; nothing published | Builder call | Alignment gate; not real firmware banks |
| W08-S202 | Bank span overflow (`base + len` wraps) | `MapFatal::RangeOverflow` | Class | Boundary constants | Checked arithmetic at boundary |
| W08-S203 | Two RAM banks overlapping (partial, contained, identical) | `MapFatal::RamOverlap` (3 rows) | Class with identities | Span pairs | RAM-RAM fatality (W03 R3) |
| W08-S204 | Adjacent/identical RAM banks | Merged entry + counter | Merge recorded, not duplicated | Span pairs | Normalization merge (W03 R4) |
| W08-S205 | Protected-protected overlap, distinct sources (image∩DTB, rsvmap∩reserved-node pairs) | `MapFatal::ProtectionConflict` | Class with both identities | Span pairs | Disagreeing protections are fatal (W03 R6) |
| W08-S206 | Exact duplicate protection from the same class | Deduped + counter | Single entry; counter recorded | Duplicated ranges | Dedup policy |
| W08-S207 | Protected range overlapping RAM edge/middle (clip cases: head, tail, hole) | Clipped allocatable spans + `ClipRecord`s | Spans = bank minus protected; clips recorded | Interval cases | Protected-wins clipping (W03 R5); the hard gate's construction basis |
| W08-S208 | Zero-size RAM bank; zero-size protection | Dropped with counters / recorded (2 rows) | Counters; boot-shape map | Degenerate spans | Zero-size policy (W03 R8/R9) |
| W08-S209 | Protection entirely outside RAM | Recorded warning, not fatal | `outside_ram_warnings` incremented | Disjoint spans | Out-of-RAM policy (W03 R10) |
| W08-S210 | Protected-range count beyond capacity (capacity lowered in test config) | `MapFatal::CapacityExhausted` | Class; nothing published | Oversized input set | No silent truncation of protections |
| W08-S211 | Seal: plan range misaligned; plan range outside allocatable; plan range overlapping protected | `MapFatal::SealRejected` (3 rows, distinct violations) | Class + violation; map still draft-shaped, no partial mutation | Tampered `MetadataPlan` | Seal validation (W03 R11) |
| W08-S212 | Valid draft+seal end-to-end | `BootMemoryMap` with equation `ram = allocatable + protected` exact | Equation asserted on summary | Golden-path seal | Accounting equation; not W04 behavior |
| W08-S213 | Repeated builder runs on identical S201–S212 inputs | Identical outcomes and entries | Bit-identical results | Double-run diff | Builder determinism (feeds S5xx) |
