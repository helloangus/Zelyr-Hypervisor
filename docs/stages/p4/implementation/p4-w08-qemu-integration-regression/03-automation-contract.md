# P4-W08 Automation Contract

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W08 detailed design](README.md).  
**Companion:** scenario rows in [02-scenario-matrix.md](02-scenario-matrix.md).

W08's logical modules are authoritative automation artifact groups, not Rust
modules: the regression is host-side tooling plus versioned data. The named
owner is the sole authoritative home for its group's content; other
documents link but do not duplicate.

## 1. Artifact groups and ownership

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Scenario manifest (`SM-T1` data) | the automation tooling's manifest file(s), schema per §2 | matrix rows, W07 minimums, environment label | machine-readable scenario/repeat configuration; it does not carry verdict logic |
| Automation entry point | the single P4 regression command extending the P0-W09 runner | manifest, built artifacts, evidence directory | orchestration of build→boot→collect→verdict; it does not implement the runner or QEMU itself |
| Verdict rules | the tooling's verdict module, rules per §4 | captured serial stream, P4-RR record lines, manifest expectations | determinate verdicts per iteration and set; it does not alter expectations to fit observations |
| Evidence store | `docs/stages/p4/verification/` layout per §6 | raw run outputs | retained, identifiable evidence; it does not interpret results (interpretation lives in the verification record) |
| Implementation record | `../p4-w08-qemu-integration-regression-record.md` (created when work starts) | actual tooling decisions, environment facts | entry-point usage, manifest version, declared environment, limitations; no run results |

## 2. Build and image boundary (P4-K01)

- **Declared build:** the Hypervisor and the Validation Guest are built by
  the repository's documented build commands under the pinned toolchain
  (assumed M4); the automation invokes those commands, it does not define
  new ones. The Guest reaches the Hypervisor exclusively through the W03
  embedding route — no side-channel image loading exists in P4.
- **Build identity:** the automation records the build identity (P0 version
  baseline) with every run; runs without an obtainable identity are
  `UNSUPPORTED` for comparison-bearing sets (RS-C), because stability is
  defined per build (W07 open item O2).
- **Image preparation:** "prepare the image" means verify the embedded
  Guest image is present and hash it into the evidence (integrity of
  evidence, not a Guest feature). No image transformation is performed.

## 3. Boot and collection boundary (P4-K02)

- **Boot:** exactly one QEMU invocation per iteration through the P0-W09
  entry with the P4 parameter set (open item O1: scenario set, repeat
  counts, evidence directory). The automation owns no QEMU flags of its own
  beyond what the entry's parameters expose.
- **Serial capture:** the entry's capture facility provides the stream; W08
  requires it retained verbatim per iteration (evidence, not filter input).
- **Collection rules:** from the stream, the tooling extracts: (a) VG marker
  lines (grammar per W05 `VG-T<n>`, version echoed in the banner); (b) the
  complete P4-RR record (first line `P4RR:VER v1`, last line `P4RR:END`);
  (c) `HV-DIAG` blocks (report presence check only — content is human
  evidence). Version mismatches (`VG-T?` unknown, `P4RR:VER` unknown) yield
  `INCOMPLETE` with the version named — never fuzzy parsing.
- **Panic/fatal detection:** a Host fatal path (P1 crash diagnostics output,
  assumed M5-adjacent convention from P1-W07) maps to `FAIL` with the crash
  evidence retained; a Guest `VG-PANIC` marker maps per its scenario row
  (unexpected in all current rows → `FAIL`).

## 4. Verdict taxonomy and determination rules (determinate outcomes)

| Verdict | Meaning | Determination rule |
|---|---|---|
| `PASS` | expected observable obtained exactly | all row conditions of [02](02-scenario-matrix.md) hold for the iteration |
| `FAIL` | ran, but evidence contradicts expectations | any marker mismatch, unexpected fault class, missing expected fault, `Mismatch` verdict, accounting/record anomaly, or Host fatal path |
| `TIMEOUT` | ran, did not reach the expected end within the budget | no expected terminal marker/record end within the scenario's timeout budget (§5); partial evidence retained |
| `INCOMPLETE` | could not determine; evidence missing or unparsable | missing P4-RR record, unknown grammar/version, truncated capture, missing mandatory lines |
| `UNSUPPORTED` | environment cannot produce a valid run | missing build identity, undeclared environment label (open item O2), runner parameter surface unavailable (open item O1) |

Rules:

- Exactly one verdict per iteration and per set; every non-`PASS` names its
  rule and retains evidence. "Determinate" means the verdict plus evidence
  answer "what happened" without human guesswork (plan item 5).
- `TIMEOUT` budgets are per-scenario constants in the manifest, derived from
  the scenario's expected work (clean episodes get the base budget; the
  budget is an environment-failure bound, never tuned to make slow runs
  pass).
- A `FAIL` never mutates the manifest or the matrix; defects route to owning
  packages (D2/D8).

## 5. Repetition and seed rules (P4-V15)

- Iteration counts: RS-C ≥ 5 (W07 minimum); the manifest may raise counts,
  never lower them (W07 D7). RS-A/RS-B repeat counts follow W07's episode
  minimums inside one boot.
- Independence: fresh QEMU process per iteration; no carried state; the only
  shared inputs are the declared build artifacts and the manifest.
- No seeds: the matrix contains no randomness (W05 Guest has no timing
  loops or randomness; W07's plan is fixed), so stability needs no
  statistical treatment — exact equality per W07 D3.
- Stability: RS-C passes iff every iteration's P4-RR record equals the
  first's on the S1–S5 surfaces and all verdicts are `PASS`; RS-D passes
  iff all its rows pass. Set verdicts follow D7 (all-or-nothing).

## 6. Evidence destinations

Per iteration `i` of scenario `s` in run `r`, under the stage verification
area (paths recorded when evidence exists; this design creates none):

```text
docs/stages/p4/verification/
  p4-w08-qemu-integration-regression-verification.md   # run summary table,
                                                       # verdicts, environment,
                                                       # links to raw runs
  p4-w08-runs/<run-id>/
    manifest.json            # manifest hash + parameters for this run
    build-identity.txt       # P0 version baseline identity
    <scenario>-<iter>.serial.log   # verbatim capture
    <scenario>-<iter>.p4rr.txt     # extracted P4-RR record
    <scenario>-<iter>.verdict      # verdict + matched rule id
    <scenario>-<iter>.hv-diag.txt  # extracted report (fault rows)
```

Rules: raw evidence is never edited after capture; corrections appear in the
verification record, not the logs; the verification record's summary table
distinguishes **passed / failed / blocked / not run** per matrix row, so
planned, run, and missing evidence stay distinct (checklist rule).

## 7. Extensibility for P5 (preserved regression)

- New scenarios are added as new manifest rows alongside P4 rows; P4 rows
  are preserved verbatim (the P5 task book's preserved-P4-regression
  requirement, P5-V16). Removing or weakening a P4 row requires the owning
  designs' agreement and is a recorded change, never a silent edit.
- The manifest carries `SM-T<n>` and grammar versions; a P5 consumer must
  handle version rejection explicitly (same discipline as P4RR versioning).
