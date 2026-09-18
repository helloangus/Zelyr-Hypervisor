# P0-W09 Runner Entry Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W09 detailed design](README.md).

## 1. Logical artifact groups and ownership

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Runner entry contract | `docs/testing/qemu-runner-entry.md` | this design, ADR-003 reference-platform decision, plan scope | the sole normative home of the single-entry rule, responsibility boundary, invocation grammar, parameter model, runtime behavior, exit taxonomy, evidence content set, and placeholder marking; it does not implement a runner, define CI checks, or fix QEMU flag recipes |
| Documentation routing | one row in `docs/README.md` (a pointer line in `docs/testing/README.md` is allowed only without restating policy) | contract location | discoverability; no policy duplication |
| Stage implementation index | `docs/stages/p0/implementation/README.md` | design status | a truthful status row; never a completion claim |
| Implementation record | `../p0-w09-qemu-automation-entry-baseline-record.md` (created when work starts) | decisions taken | changed artifacts and deviations; no command logs |
| Verification record | `../../verification/p0-w09-qemu-automation-entry-baseline-verification.md` (created when evidence exists) | actual review output | run/not-run evidence per the validation matrix; never a QEMU-run claim |

The record and verification paths are future locations; this design does not
create them. No runner program exists in P0; there is no runner artifact group.

## 2. Single-entry rule and responsibility boundary

**Single-entry rule (contract content):** every automated, repeatable QEMU
execution whose output is used as evidence must go through the runner entry
defined here. No tracked script, document, or CI configuration may embed its
own QEMU command line for automated runs. Human-investigation QEMU commands
(boot-recipe exploration, debugging) are not automation entries and stay
permissible, but must never be promoted into scripts or CI outside the entry.

**The runner is responsible for:** launching the emulator with a declared,
reproducible invocation; capturing the target's serial output completely;
enforcing a mandatory timeout; terminating the target cleanly on timeout;
reporting the outcome in the fixed exit taxonomy; and preserving the run's
evidence set.

**The runner is not responsible for:** deciding what a test's success
condition is (supplied by the calling test design); interpreting guest or EL2
semantics; defining pass/fail policy for stages; providing artifacts (it
records where they are); or serving as a development QEMU helper for humans.

## 3. Invocation grammar (contract outline)

The contract records the grammar as a usage sketch, not code:

```text
<runner> run --profile <name>
             [--param <class>=<value> ...]
             [--timeout <duration>]
             [--evidence-dir <path>]
<runner> --version
```

Grammar rules:

- One primary verb (`run`) plus a version query. New verbs require an entry
  contract version bump.
- `--profile` selects a named invocation profile defined by the calling test
  design (for example a P1 boot-smoke profile). Profiles carry the concrete
  QEMU invocation and defaults; the runner carries the mechanics. Profile
  definitions are **not** part of this contract.
- `--param` carries per-run overrides from the reserved parameter classes
  (§4); unknown classes are a usage error (§6), never silently ignored.
- `--timeout` is always honored; when omitted, the profile's default applies
  (§5). An infinite or absent effective timeout is invalid.
- `--evidence-dir` selects the evidence root; when omitted, the profile
  default applies. The runner never writes evidence outside the selected
  root.
- The runner's program name, language, and location are owned by the
  implementing design (expected P1-W10); this contract fixes the interface
  shape only.

## 4. Reserved parameter classes

The contract reserves exactly these parameter classes, matching the plan's
enumeration; each becomes usable when an owning test design defines its
value domain:

| Class | Purpose (future) | Owning design decides |
|---|---|---|
| `boot-smoke` | minimal boot-to-marker runs | marker, profile defaults (P1) |
| `smp` | CPU-count and topology variants | count range, topology encoding (P3+) |
| `memory` | RAM size / memory-map variants | size domain, layout knobs (P2/P4) |
| `gic` | interrupt-controller variants | GIC version/config domain (P6) |
| `smmu` | IOMMU-presence variants | SMMU configuration domain (P14) |
| `guest-image` | image under test selection | image identity, versioning (P4/P8) |
| `regression` | repeat-count / suite selection | repeat semantics (e.g. P1-W10's cycle target) |

Extension rule: adding a class, changing a class's meaning, or changing the
grammar bumps the entry contract version with a compatibility statement;
callers branch on the version, never on probing.

## 5. Runtime behavior requirements

- **Launch:** the runner launches a QEMU system emulator supporting the virt
  machine (ADR-003), headless, with the serial output connected to a
  capturable channel. The concrete flag set is the implementing design's,
  constrained by: reproducibility from the invocation record alone, no
  reliance on interactive input, and serial capture completeness (§7).
- **Serial capture:** the target's serial output is captured byte-completely
  from launch to termination into the evidence set; silent truncation,
  sampling, or loss is prohibited. If the emulator itself drops output, the
  run's evidence must say so.
- **Timeout:** every invocation has an effective finite timeout. On expiry
  the runner terminates the emulator, preserves the partial serial capture
  and a timeout record, and reports the timeout outcome class (§6). Timeouts
  are never treated as success.
- **Termination and cleanup:** after any outcome the runner terminates the
  emulator, releases its resources, and finalizes the evidence set before
  exiting. A crash of the runner itself is reported by whatever survives in
  the evidence set (partial capture plus launch record), which consumers must
  treat as an internal-error outcome, not as target failure.

## 6. Exit-status taxonomy (stable machine-facing surface)

| Status | Meaning | Attributed to |
|---|---|---|
| `0` | target-under-test met the success condition supplied by the caller | target outcome |
| `1` | usage/contract error: invalid grammar, unknown parameter class or profile, invalid timeout | caller |
| `2` | launch failure: emulator not found, failed to start, or exited before producing its first observable output | environment / runner health |
| `3` | timeout: effective timeout expired; target terminated; partial evidence preserved | target outcome |
| `4` | target failure: target ran but did not meet the caller's success condition (including crash/panic detection logic owned by the calling design) | target outcome |
| `5` | internal runner error: the runner could not fulfill its contract (evidence not writable, unexpected internal state) | runner health |

Rules: these six are the only statuses; any unexpected situation maps to `5`.
Consumers branch on the taxonomy, not on raw emulator exit codes, which are
preserved only inside the evidence set. Changing the taxonomy is a contract
version bump.

## 7. Evidence requirements

Every run's evidence set must contain, regardless of outcome:

- an invocation record: entry contract version, grammar-level arguments
  (profile name, parameters, effective timeout, evidence root), runner
  identity (version if implemented), emulator identity (program name/version
  string), machine model used, host platform identifier, and start/end
  timestamps;
- the complete serial capture (§5);
- the outcome record: exit-status class with the caller-defined success
  condition reference and any target-emitted failure details the calling
  design specifies; and
- pointers (not copies) to any larger artifacts the caller declares.

Naming and identity of the evidence files follow the artifact-naming baseline
delivered by [P0-W17](../p0-w17-artifact-naming-baseline/README.md). Until
that baseline exists, the contract's placeholder rule applies: evidence
paths must be machine-processable, unique per run, and deterministic given
the invocation record — and W17's delivery supersedes the placeholder
without a version bump if it is compatible, or with one if it is not.
Retention locations follow the stage verification conventions
([W05](../p0-w05-documentation-baseline/README.md)); the runner never
defines retention policy itself.

## 8. P0 placeholder status (mandatory content)

The contract must contain, verbatim in meaning:

- In P0 this entry is an interface-only placeholder. No runner program
  exists; the entry cannot be invoked; no QEMU execution has occurred under
  it; and nothing in P0 proves EL2, guest, or hardware behavior.
- The entry's first concrete implementation is owned by the first design
  that executes QEMU (expected P1-W10), subject to this contract.
- Until then, CI must treat the entry as a future/non-P0 check (W20) and
  documents must not describe it as a passing or runnable test.

## 9. Explicitly excluded interfaces

No script, program, binary, CI workflow, crate, target, or dependency is
authorized. The grammar sketch in §3, the taxonomy table in §6, and the
evidence content list in §7 are documentary contract outlines. Concrete QEMU
command lines, machine options, profiles, images, and timeout defaults are
owned by the implementing designs. If delivering W09 appears to require a
committed script or a QEMU run, that is a scope boundary: stop and record.
