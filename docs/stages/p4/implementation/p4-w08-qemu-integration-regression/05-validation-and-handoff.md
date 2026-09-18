# P4-W08 Validation, Error/Security Model, and Handoff

**Status:** Proposed design; implementation and validation are not claimed.  
**Parent:** [P4-W08 detailed design](README.md).

## 1. Validation matrix

The regression is validated in two layers: the tooling against synthetic
evidence (always available), and the scenarios against real runs (blocked
until W02–W07 deliver). This matrix defines what W08's evidence must show;
it is a plan until the verification record exists.

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W08-DV01 | Single entry, declared build/image/boot boundary | review + host-side integration test | run the command against a scripted fake QEMU stream | one entry produces build→boot→collect→verdict→evidence; no second QEMU path; parameters ride the P0 surface | boundary coherence; not real-boot correctness |
| W08-DV02 | Collection rules exact | golden-stream unit tests | well-formed, truncated, noisy, version-mismatched streams | line-exact extraction; unknown versions → `INCOMPLETE` named | parser correctness; not parser completeness for future grammars |
| W08-DV03 → P4-V13 | Positive path determinate (SM-01) | real run (gated) | execute RS-D SM-01 | verdict `PASS` with exact marker/record match and EL2 liveness | automated positive determination in the declared QEMU environment; not hardware behavior |
| W08-DV04 → P4-V14 | Fault/survival determinate (SM-02–SM-08) | real runs (gated) | execute RS-D fault rows | each row `PASS` with named fault class, IPA predicate, `Match`, `HV-DIAG` present; any deviation determinate | translation/permission/illegal negatives and EL2 survival are objectively detected; not isolation completeness or hardware fault semantics |
| W08-DV05 → P4-V15 | Repeat stability determinate (RS-C) | real runs (gated) | ≥ 5 independent boots, per-iteration records compared | all iterations `PASS`; records equal on S1–S5 surfaces per declared build | repeat stability in the declared environment; not cross-build or cross-environment stability, not hardware |
| W08-DV06 | Non-success outcomes diagnosable | synthetic + real (when available) | force timeout (stalled fake), truncated capture, missing identity | `TIMEOUT`/`INCOMPLETE`/`UNSUPPORTED` each produced with rule id and evidence; none can masquerade as `PASS` | taxonomy determinacy; not that every future failure mode is enumerated |
| W08-DV07 | Set aggregation all-or-nothing | unit tests + real set runs | inject one failing iteration into a set | set verdict non-`PASS` with the iteration named | verdict aggregation integrity; not per-iteration correctness (DV03–DV05) |
| W08-DV08 | Evidence layout complete and immutable | review of produced evidence | inspect a real run directory against [03 §6](03-automation-contract.md) | all files present, verbatim logs, verdicts with rule ids; no post-capture edits | evidence traceability; not evidence authenticity beyond tooling integrity |
| W08-DV09 | Grammar/table version coherence | joint review with W05/W06/W07 | `SM-T1` vs `VG-T<n>` vs `P4RR:VER v1` | all versions aligned; change rules acknowledged | seam stability; not scenario correctness (owned upstream) |

Record each row as **passed / failed / blocked / not run** with command or
review input, environment (including QEMU version when run), date, and
reason. Rows DV03–DV05, DV07 (real part), and DV08 are blocked until the
M1–M6 upstream evidence exists; blocked rows name the waiting W01 row or
open item.

## 2. Error model

- The tooling's failure modes are the verdict taxonomy itself: every run
  ends in exactly one determinate verdict; there is no "error" that escapes
  classification.
- Tooling defects (parser panic, crash, unwritable evidence) are defects of
  W08's own tooling — they fail the affected run as `INCOMPLETE` with the
  tooling error recorded, and are fixed like any code defect; they never
  alter scenario expectations.
- Missing upstream evidence or surfaces are blocked prerequisites recorded
  per W01 §4 with the waiting row named.
- No telemetry or guest data flows through W08 other than captured console
  streams and derived records; W08 transforms nothing into contract.

## 3. Security model

- W08 adds no privileged surface, no Guest-reachable interface, no hypervisor
  code, and no `unsafe`; its entire surface is host-side tooling and data.
- Captured streams and evidence may contain addresses and diagnostic text;
  they are retained under the stage verification area and never fed back
  into builds or expectations automatically (no auto-tuning loop).
- The manifest is the only configuration authority for a run; ad-hoc
  command-line QEMU overrides are not part of the contract and their output
  is not acceptable evidence (single-entry rule, D1).
- Standing scope boundaries: no CI policy (P0 package), no hardware runs
  (P15), no Linux scenarios (P8), no performance capture — pressure to add
  any is a stage-boundary violation to record.

## 4. Observability model

- The regression's outputs are exactly: per-iteration evidence files, the
  verification record's summary table, and the defect reports to owning
  packages. Verbose console chatter from the tooling is not evidence and
  must not substitute for the §6 layout.
- Environment facts (QEMU version, machine options, accelerator) are part
  of every run's identity (open item O2) so results are attributable.
- Verification claims live only in
  `../../verification/p4-w08-qemu-integration-regression-verification.md`
  and the per-run evidence directories; design and plan documents carry no
  results.

## 5. Handoff checklist

Before handing W08 work to a reviewer:

- exact changed-file list (tooling + manifest + record paths) and
  implementation-record path
  (`../p4-w08-qemu-integration-regression-record.md`);
- DV01–DV09 statuses with explicit not-run/blocked entries and the waiting
  upstream rows/open items each blocked item waits on;
- confirmation: zero hypervisor/Guest code changes, zero new `unsafe`, no
  changes to VG/IS/P4-RR grammars, no second QEMU entry;
- matrix/grammar versions recorded (`SM-T1`, consumed `VG-T<n>`,
  `P4RR:VER v1`) with open items O1/O2 status;
- defect reports raised against owning packages for any real-run failures,
  with links;
- handoff to consumers: entry-point usage, environment facts, scenario-set
  identity, and evidence locations to
  [P4-W09](../p4-w09-closeout-p5-handoff/README.md); preserved-scenario
  extension rules to P5 regression users
  ([P5-W09](../../../p5/plans/p5-w09-telemetry-safe-logging-regression.md),
  [P5-W10](../../../p5/plans/p5-w10-closeout-p6-handoff.md));
- open items carried forward: O1 runner parameter surface, O2 declared
  environment facts, P2-ACR-01 unchanged and unresolved.
