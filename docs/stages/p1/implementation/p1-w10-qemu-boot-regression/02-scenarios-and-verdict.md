# P1-W10 Scenarios and Verdict Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P1-W10 detailed design](README.md).

## 1. Reading the matrix

Each scenario fixes input/precondition, expected observable, pass condition,
and proof boundary **before** execution. Scenarios R2–R5 verify the harness
itself (it must detect failures, not merely record them); R1 and R7 produce
the package's headline evidence. Techniques marked "suggested" are the
expected way to induce a condition; the contract is the observable outcome,
not the flag spelling. All scenarios run through the
[automation contract](01-automation-contract.md); evidence destinations are
stated per scenario and summarized in §4.

## 2. Scenario matrix

### R1 — Canonical single boot (positive control)

- Input/precondition: W01 canonical invocation, canonical reference image,
  default timeout, one cycle.
- Expected observable: boot-start markers, then the `stable` token; no
  forbidden class; runner exit in the expected set.
- Pass condition: outcome `PASS` per the classification table; evidence per
  [01-automation-contract.md](01-automation-contract.md) §6.
- Proves: the canonical boot reaches the stable state under the fixed rules,
  repeatably enough to serve as the regression baseline. Does **not** prove:
  hardware behavior, absence of latent faults on unexercised paths, EL2
  semantics beyond QEMU's model, or that the image is bug-free.
- Evidence: verification document R1 section; raw capture referenced.

### R2 — Panic-bearing boot (detection control)

- Input/precondition: image built with the
  [P1-W11](../p1-w11-negative-fault-validation/README.md) panic scenario
  selection (or, until W11 lands, any recorded panic-inducing variant; the
  technique is W11's).
- Expected observable: panic-class marker in the capture.
- Pass condition: outcome `FAIL-PANIC`; full capture retained; driver exits
  nonzero. A `PASS` here is a harness defect.
- Proves: the verdict detects the panic class. Does not prove: anything about
  W07's report content (W11's scenarios do that).
- Evidence: harness-validation section of the verification document.

### R3 — Timeout behavior (bounded-run control)

- Input/precondition: canonical invocation with a timeout intentionally below
  the measured boot completion (suggested technique; recorded in evidence).
- Expected observable: no completion within the bound.
- Pass condition: outcome `FAIL-TIMEOUT`; QEMU terminated per the runner
  contract; partial capture preserved; driver exits nonzero within bounded
  wall-clock time.
- Proves: runs are bounded and unbounded hangs become explicit failures. Does
  not prove: the default timeout's adequacy (that is R1's measured evidence).
- Evidence: harness-validation section.

### R4 — Missing stable marker (marker control)

- Input/precondition: a boot that stops before the stable state (suggested
  techniques: a W11 scenario image that exits the lifecycle early, or an
  image built from a deliberately truncated boot path; the technique is
  recorded).
- Expected observable: boot starts; stable token absent; no forbidden class
  required either way.
- Pass condition: outcome `FAIL-MARKER`; capture retained; driver exits
  nonzero.
- Proves: absence of the success marker is detected as failure, not
  leniency. Does not prove: the truncated path's own diagnostics (W11).
- Evidence: harness-validation section.

### R5 — Abnormal exit classification

- Input/precondition: an invocation the runner cannot complete as specified
  (suggested technique: invalid image path or malformed runner arguments).
- Expected observable: runner failure before or during launch.
- Pass condition: outcome `ERROR-INVOCATION`; no cycle counted; distinct
  record from boot failures; driver exits with the invocation-error code.
- Proves: environment errors are distinguishable from boot failures, so a
  broken environment cannot masquerade as a passed or failed boot.
- Evidence: harness-validation section.

### R6 — Order-independence review (static)

- Input/precondition: the matcher implementation and the R1 capture.
- Expected observable: verdict computed on permuted line orderings and on
  benign textual variations (extra blank lines, interleaved QEMU noise).
- Pass condition: verdict unchanged under permutation and benign variation;
  matching code review shows no order, timing, count, or full-text predicate.
- Proves: the plan's no-order-only-criteria requirement is actually met. Does
  not prove: marker-token uniqueness against future output (reviewed each
  time tokens change).
- Evidence: review record in the verification document.

## 3. The 100-cycle procedure

1. Preconditions: R1 passing on the exact image identity under test; marker
   rules frozen; evidence directory fresh; no other load on the host that the
   implementation record identifies as a flakiness source (recorded, not
   policed).
2. Run `p1-boot-regression --cycles 100` with the recorded default timeout.
3. Acceptance (P1-V17): all 100 cycles `PASS`, reaching the same stable token,
   zero forbidden-class occurrences, zero `FAIL-*` outcomes. The first failure
   stops the run (parent README decision 5); the stopped run is the evidence
   of non-repeatability and P1-V17 is not met by re-running.
4. The run is valid only with its `meta.txt` (image identity, environment,
   timeout) and `summary.txt`; a summary without retrievable raw-evidence
   references is a review failure.

What 100 clean boots prove: startup is not randomly failing under the
reference environment — no race between boot steps, no unstable
firmware/QEMU interaction, no order-dependent luck in the lifecycle. What
they do **not** prove: correctness beyond the marker rules, behavior under
host load or different QEMU versions (recorded environment only), hardware
behavior of any kind, or freedom from faults on paths the marker rules do not
observe.

## 4. Evidence destinations

| Scenario / run | Destination |
|---|---|
| R1 | `../../verification/p1-w10-qemu-boot-regression-verification.md`, R1 section; raw capture per [01-automation-contract.md](01-automation-contract.md) §6 |
| R2–R5 | same document, harness-validation section; one entry per scenario with outcome and retention path |
| R6 | same document, review section |
| 100-cycle run | same document, repetition section: procedure, `meta`/`summary` content, stop reason if any, and the explicit statement of what is not proven |

Every entry records run status (**passed**, **failed**, **blocked**,
**not run**) with command, environment, and timestamp. Writing the driver
without R1/R6 does not satisfy P1-V16; the 100-cycle run is the only evidence
for P1-V17 and nothing else may be reported as providing it.

## 5. Reserved extensions (not designed here)

RAM/CPU-size variations, SMP counts, additional machine types, parallel
execution, statistical flakiness metrics, and CI scheduling are Reserved for
the stages that own them (P2-W09 for integration matrices; P0-W20 for CI).
Adding any of them to this package is a scope conflict.
