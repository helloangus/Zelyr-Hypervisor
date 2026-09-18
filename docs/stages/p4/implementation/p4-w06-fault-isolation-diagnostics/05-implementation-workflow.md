# P4-W06 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W06 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the
documents named in the parent README and inspects the current tree:
`git ls-files` confirms what exists; the sibling implementation records for
W02/W04/W05 (paths `../p4-w02-stage2-address-space-record.md` and alike,
created when those packages start) are the evidence sources for the assumed
contracts M1–M6 in [01 §2](01-scope-and-foundations.md). Entry-order rule:
decode, matcher, report, and dump logic with host-side unit tests can proceed
against assumed signatures, but any on-target evidence (DV04–DV09) requires
the W02/W04/W05 paths delivered; a missing upstream is a recorded blocked
prerequisite per W01 §4 — never repaired inside W06.

Stop and obtain direction instead of guessing when any of the following
occurs:

- the delivered W04 frame differs from the assumed field set (M2) — joint
  design note with W04; do not re-capture or reinterpret registers locally;
- the delivered W02 `QueryResult` differs from the assumed shape (M4) — joint
  design note; do not walk descriptors as a workaround;
- the W03/W05 artifacts cannot carry the three IS probe configurations
  (open item O1) — joint change under their recorded rules; do not invent a
  private encoding channel;
- an `unsafe` need appears anywhere in W06 — stop; add a contract here first
  and an inventory entry with SAFETY justification (D10);
- QEMU syndrome behavior contradicts the pinned architecture revision —
  record a Specification Investigation item; do not bend decode tables to
  the emulator (W01 A7).

## 2. Ordered implementation steps

### Step 1 — decode layer with host-side proof

Target: `exit-decode` module (per [03 §1–§2](03-code-contracts-exit-classification.md)).

Work: implement `decode_esr` and `reconstruct_fault_ipa` against the pinned
architecture revision; build the host-side test tables covering every W04
class plus unknown/reserved sweeps. Why first: everything else consumes these
pure functions, and they carry the only architecture-revision risk.

Suggested observation: host-side unit test run reporting table coverage.

**Acceptance:** every synthesized class decodes to the documented detail;
unknown encodings are represented, never rejected; IPA composition matches
the revision's rule on boundary-width cases.  
**Failure/blocker:** a revision mismatch is a Specification Investigation
record plus a decode-table correction — never a QEMU-driven silent fix.  
**Evidence:** `../../verification/p4-w06-fault-isolation-diagnostics-verification.md`
(DV01/DV02 rows).

### Step 2 — diagnosis record and agreement classification

Target: `fault-diag` module ([03 §3](03-code-contracts-exit-classification.md)).

Work: implement `ExitDiagnostic`, `diagnose`, and `classify_agreement`;
wire the W02 `query` call behind the assumed M4 signature; add the W04-class
cross-check assertion ([03 §4](03-code-contracts-exit-classification.md)).

**Acceptance:** field-completeness and totality tests pass for every class;
agreement classification passes the exhaustive pair matrix; cross-check
assertion holds on identical frames.  
**Failure/blocker:** a W02/W04 signature mismatch stops the step (joint
design note), it is not adapted to silently.  
**Evidence:** verification record DV03.

### Step 3 — expectation table, probe membership, matcher

Target: `isolation-expect` module
([04 §1–§3](04-code-contracts-diagnostics-isolation.md)).

Work: encode the `IS-T1` matrix as static versioned data; implement
`probe_membership` against the W03 layout record and P2 fact signatures;
implement `check_expectation`.

**Acceptance:** every IS row has a well-formed expectation; membership
rejections fire on the named error cases; matcher tests cover every reason
dimension positively and negatively.  
**Failure/blocker:** P2 fact signature absent (`FactsUnavailable`) is a
blocked prerequisite recorded against the P2 rows (W01 R07/R08), not worked
around with hardcoded ranges.  
**Evidence:** verification record DV04/DV05 rows and the review note for
open item O2.

### Step 4 — report and dump

Target: `diag-report` module ([04 §4–§5](04-code-contracts-diagnostics-isolation.md)).

Work: implement `format_diagnostic` and `dump_mapping_state` over
fixed-capacity buffers; wire the truncation-annotated fallback path.

**Acceptance:** golden-text tests pass; truncation marks; dump agrees with
the diagnostic snapshot on stub ledgers; no allocation anywhere (review).  
**Failure/blocker:** a buffer budget that proves too small in practice is
fixed by a recorded budget change in the workflow, not by switching to
unbounded output.  
**Evidence:** verification record DV06.

### Step 5 — integration with W04's stop path and joint reviews

Target: the W04 exit handler's post-stop diagnosis step; W05/W07/W08
consumer notes.

Work: integrate `diagnose → check_expectation → events → report/dump` after
W04's stop decision; run the three joint reviews: (a) W05 §6 rule — IS-series
refinements of VG expected outcomes acknowledged (open item O2); (b) W04 —
diagnosis ordering and the re-entry silence rule confirmed; (c) W07 —
`diag.*` event field list for counters/correlation agreed.

**Acceptance:** integration compiles against the delivered W04 path; the
three review notes exist; no action-policy change was made.  
**Failure/blocker:** a review disagreement re-opens the affected contract in
this design; it is never resolved by local improvisation.  
**Evidence:** implementation record notes; verification record DV10/DV11
review rows.

### Step 6 — on-target isolation evidence (gated on M2–M6 delivered)

Target: on-target runs through the W05 scenarios on the QEMU reference
platform, recorded by W08 automation.

Work: execute the IS rows: VG-004 with the three probe configurations
(IS-01/02/03), VG-005 (IS-04), VG-006 (IS-05), VG-010 (IS-06), VG-011
(IS-07), VG-007/VG-012 (IS-08); for each, verify the expected
`ExitDiagnostic`, the `Match` verdict, EL2 liveness after stop, and the
report/dump presence.

**Acceptance:** each mandatory IS row produces `Match` with the post-stop
EL2-live condition; any `Mismatch` (including `StaleTranslation`) fails the
run and is diagnosed before proceeding.  
**Failure/blocker:** a failing row is failed evidence — record, diagnose,
correct the design or the implementation, and re-run; a pass is never
declared from a partially matching row.  
**Evidence:** verification record DV05–DV09 rows; raw run data under the
W08 evidence layout.

### Step 7 — closure review and handoff

Work: run the review matrix in
[06-validation-and-handoff.md](06-validation-and-handoff.md), confirm the
handoff checklist, record implementation facts (decode coverage, matrix
version, limitations) in `../p4-w06-fault-isolation-diagnostics-record.md`
when work starts. Completion is claimed only in the verification record,
with evidence, only for what actually ran.

## 3. Validation matrix

See [06-validation-and-handoff.md](06-validation-and-handoff.md) §1 (W06-DV01
through W06-DV12). Each row is recorded as **passed / failed / blocked /
not run** with command or review input, environment, date, and reason. No
row here proves P4-V01–V05 or P4-V10–V16, and none may be reported as doing
so.
