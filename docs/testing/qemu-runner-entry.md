# Zelyr QEMU Runner Entry Contract

**Status:** Normative interface contract; P1-W10 supplies its first executable
foundation (see §8); integrated boot validation remains pending.  
**Scope:** The single automated-QEMU execution entry: single-entry rule,
responsibility boundary, invocation grammar, reserved parameter classes,
runtime behavior requirements, exit-status taxonomy, evidence content set,
and placeholder status. It does not implement a runner, define CI checks, or
fix QEMU flag recipes, machine options, profiles, images, or timeout
defaults — those are owned by the implementing test designs.  
**Version:** v0.1 (the entry contract's own version)  
**Owner/change context:** P0-W09 QEMU automation entry baseline; grammar,
parameter-class, and taxonomy changes bump this version with a compatibility
statement (§4, §6).  
**Supersedes:** The absence of a QEMU automation entry contract.

Reference platform: the ADR baseline names QEMU `virt` as the reference
platform; the runner launches an emulator supporting that machine
([ADR-000](../adr/adr-000-architecture-baseline-v0.1.md)).

## 1. Single-entry rule and responsibility boundary

**Single-entry rule:** every automated, repeatable QEMU execution whose
output is used as evidence must go through the runner entry defined here. No
tracked script, document, or CI configuration may embed its own QEMU command
line for automated runs. Human-investigation QEMU commands (boot-recipe
exploration, debugging) are not automation entries and stay permissible, but
must never be promoted into scripts or CI outside the entry.

**The runner is responsible for:** launching the emulator with a declared,
reproducible invocation; capturing the target's serial output completely;
enforcing a mandatory timeout; terminating the target cleanly on timeout;
reporting the outcome in the fixed exit taxonomy (§5); and preserving the
run's evidence set (§6).

**The runner is not responsible for:** deciding what a test's success
condition is (supplied by the calling test design); interpreting guest or
EL2 semantics; defining pass/fail policy for stages; providing artifacts (it
records where they are); or serving as a development QEMU helper for humans.

## 2. Invocation grammar

Usage sketch (documentary outline, not code):

```text
<runner> run --profile <name>
             [--param <class>=<value> ...]
             [--timeout <duration>]
             [--evidence-dir <path>]
<runner> --version
```

- One primary verb (`run`) plus a version query. New verbs require an entry
  contract version bump.
- `--profile` selects a named invocation profile defined by the calling test
  design (for example a P1 boot-smoke profile). Profiles carry the concrete
  QEMU invocation and defaults; the runner carries the mechanics. Profile
  definitions are **not** part of this contract.
- `--param` carries per-run overrides from the reserved parameter classes
  (§3); unknown classes are a usage error (§5, status `1`), never silently
  ignored.
- `--timeout` is always honored; when omitted, the profile's default applies
  (§4). An infinite or absent effective timeout is invalid.
- `--evidence-dir` selects the evidence root; when omitted, the profile
  default applies. The runner never writes evidence outside the selected
  root.
- The runner's program name, language, and location are owned by the
  implementing design (expected P1-W10); this contract fixes the interface
  shape only.

## 3. Reserved parameter classes

Exactly these parameter classes are reserved; each becomes usable when an
owning test design defines its value domain:

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

## 4. Runtime behavior requirements

- **Launch:** the runner launches a QEMU system emulator supporting the virt
  machine, headless, with the serial output connected to a capturable
  channel. The concrete flag set is the implementing design's, constrained
  by: reproducibility from the invocation record alone, no reliance on
  interactive input, and serial capture completeness.
- **Serial capture:** the target's serial output is captured byte-completely
  from launch to termination into the evidence set; silent truncation,
  sampling, or loss is prohibited. If the emulator itself drops output, the
  run's evidence must say so.
- **Timeout:** every invocation has an effective finite timeout. On expiry
  the runner terminates the emulator, preserves the partial serial capture
  and a timeout record, and reports the timeout outcome class (§5). Timeouts
  are never treated as success.
- **Termination and cleanup:** after any outcome the runner terminates the
  emulator, releases its resources, and finalizes the evidence set before
  exiting. A crash of the runner itself is reported by whatever survives in
  the evidence set (partial capture plus launch record), which consumers must
  treat as an internal-error outcome, not as target failure.

## 5. Exit-status taxonomy

These six are the only statuses; any unexpected situation maps to `5`.
Consumers branch on the taxonomy, not on raw emulator exit codes, which are
preserved only inside the evidence set. Changing the taxonomy is a contract
version bump.

| Status | Meaning | Attributed to |
|---|---|---|
| `0` | target-under-test met the success condition supplied by the caller | target outcome |
| `1` | usage/contract error: invalid grammar, unknown parameter class or profile, invalid timeout | caller |
| `2` | launch failure: emulator not found, failed to start, or exited before producing its first observable output | environment / runner health |
| `3` | timeout: effective timeout expired; target terminated; partial evidence preserved | target outcome |
| `4` | target failure: target ran but did not meet the caller's success condition (including crash/panic detection logic owned by the calling design) | target outcome |
| `5` | internal runner error: the runner could not fulfill its contract (evidence not writable, unexpected internal state) | runner health |

## 6. Evidence requirements

Every run's evidence set must contain, regardless of outcome:

- an **invocation record:** entry contract version, grammar-level arguments
  (profile name, parameters, effective timeout, evidence root), runner
  identity (version if implemented), emulator identity (program name/version
  string), machine model used, host platform identifier, and start/end
  timestamps;
- the **complete serial capture** (§4);
- the **outcome record:** exit-status class with the caller-defined success
  condition reference and any target-emitted failure details the calling
  design specifies; and
- **pointers (not copies)** to any larger artifacts the caller declares.

Naming and identity of evidence files follow the artifact-naming baseline
delivered by P0-W17. Until that baseline exists, the placeholder rule
applies: evidence paths must be machine-processable, unique per run, and
deterministic given the invocation record — and W17's delivery supersedes
the placeholder without a version bump if it is compatible, or with one if
it is not. Retention locations follow the stage verification conventions
(documentation baseline); the runner never defines retention policy itself.

## 7. Consumability constraints for downstream packages

- **P0-W19:** documents the placeholder boundary and the pinned-toolchain
  prerequisite for the future implementing environment; it must not describe
  the entry as runnable.
- **P0-W20:** classifies the entry as a future/non-P0 check; absence of a
  QEMU check in CI is the designed state, never a silent skip of a required
  gate.
- **P1-W10 (expected first implementation):** its reference invocation,
  verdict logic, and evidence set are implemented against this contract;
  semantic renegotiation goes through a version bump, not a local rewrite.
- **P0-W17:** delivers the naming baseline that supersedes §6's placeholder
  naming rule.
- **P0-W07:** the entry's promotion path into the gate register is a
  future-class promotion per the quality-gates thresholds.

## 8. P0 placeholder status

**Current implementation:** [`scripts/qemu-runner`](../../scripts/qemu-runner)
implements this unchanged v0.1 interface. Its P1 profiles, defaults, dependencies
and evidence behavior are recorded in the
[W10 implementation record](../stages/p1/implementation/p1-w10-qemu-boot-regression-record.md).
The [verification record](../stages/p1/verification/p1-w10-qemu-boot-regression-verification.md)
distinguishes mechanism checks from pending integrated execution. This does
not promote a QEMU check into CI. The paragraph below records P0's historical
boundary; it does not describe the current executable state.

**In P0 this entry is an interface-only placeholder. No runner program
exists; the entry cannot be invoked; no QEMU execution has occurred under
it; and nothing in P0 proves EL2, guest, or hardware behavior.** The entry's
first concrete implementation is owned by the first design that executes
QEMU (expected P1-W10), subject to this contract. Until then, CI must treat
the entry as a future/non-P0 check, and documents must not describe it as a
passing or runnable test.
