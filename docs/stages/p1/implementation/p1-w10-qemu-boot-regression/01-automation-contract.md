# P1-W10 Automation Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W10 detailed design](README.md).

## 1. Position in the tool chain

```text
P0-W09 runner entry (assumed contract: serial capture, timeout, exit status,
  evidence collection)  <- invoked by ->  p1-boot-regression driver (W10)
W01 canonical invocation (assumed contract: the one reference boot recipe)
W09 lifecycle (assumed contract: stable state + marker vocabulary)
W07 fatal path (assumed contract: crash/panic report marker classes)
```

W10 implements the first P0-contract runner and the driver. The driver contains no QEMU
invocation details of its own beyond what the W01 canonical path and the P0
runner entry already fix; duplicating them would recreate the runner drift
P0-W09 exists to prevent.

### 1.1 Runner foundation

The runner alone owns QEMU process creation, serial capture, termination,
cleanup and per-run evidence. Its public grammar remains P0 v0.1:
`run --profile NAME [--param CLASS=VALUE ...] [--timeout DURATION]
[--evidence-dir PATH]`, plus `--version`. Record the executable path,
profile/value domains and defaults before execution. Unknown or unsupported
values are usage errors. The driver passes image selection through a
profile-defined `boot-smoke` value domain; no guest interpretation is implied.
W11 scenario profiles use this same execution entry.

Lifecycle: validate → create evidence → launch → observe → terminate/reap →
finalize evidence. Profiles supply the W01 recipe and caller-owned success
condition; the runner supplies mechanics. P0 statuses remain 0 success,
1 usage, 2 launch, 3 timeout, 4 target failure, 5 internal error. Preserve the
raw child exit status separately. Failed evidence creation is status 5;
failures retain whatever partial evidence can be written.

The target idles indefinitely at stable. The caller fixes and records a finite
post-marker observation interval before verdict-bearing runs. Continue capture
through that interval to catch subsequent panic/fatal output, then terminate
and reap QEMU and classify the final capture. Completion must precede the
independent hard timeout: timeout is always status 3, even after stable was
seen. Negative profiles similarly observe after the expected terminal
diagnostic. Unexpected early child exit is classified explicitly and never
accepted solely because a marker was printed.

Stream complete serial bytes to disk through termination and final drain;
bound matcher memory, individual lines and total output with recorded limits.
Crossing a limit terminates the child with an explicit failure while retaining
all bytes received through cleanup. Never silently truncate or sample capture.
Pass process arguments as an argument vector; serial content must never be
executed or used to derive paths. Host allocation and bounded blocking are
allowed. This adds no target ABI or runtime interface.

W10-DV03/DV04 validate grammar, all six statuses, unwritable evidence, cleanup,
complete capture, output-limit handling and late fatal detection. Controlled
child fixtures may test runner mechanics; only QEMU supplies boot evidence.

## 2. Regression entry contract

```text
Name and stability: p1-boot-regression; the canonical command name for the
  P1 boot regression. Owned by this design (parent README decision 3);
  implemented as a tracked tooling entry under scripts/ in whatever form the
  P0 runner baseline makes natural. Renames follow decision 3's authority rule.
Purpose and caller: run the canonical boot (and, on request, the 100-cycle
  repetition), decide the verdict per the fixed rules, and preserve evidence.
  Callers: developers, W11 scenarios, the P1 completion review, and any CI
  job P0-W20 wires to it.
Inputs / outputs:
  --cycles N        1 (default) or 100 for the stage-gate repetition
  --timeout T       per-boot wall-clock bound; required to be set or defaulted
                    to the implementation-recorded value (decision 6)
  --image PATH      defaults to the canonical reference image per the P0
                    build baseline; explicit override is recorded in evidence
  --evidence DIR    run-local artifact directory (default: implementation-
                    recorded path outside docs/)
  Output: machine-checkable exit code, one verdict line per cycle, and the
  evidence set of §6.
Preconditions / postconditions: the P0 runner entry exists and behaves per
  its contract; the image is built from the tracked source state; the marker
  rules of §3 are fixed in the driver, not supplied at run time. On exit,
  evidence for every attempted cycle exists (§6), whatever the verdict.
State and ownership change: creates only evidence artifacts; touches no
  repository state beyond its own tracked implementation.
Concurrency/allocation context: sequential cycles; one QEMU at a time; no
  parallel-cycle semantics are defined in P1 (Reserved for later stages).
Errors and failure guarantee: every failure mode maps to §5; evidence is
  written before the driver exits in all cases; a driver crash itself is a
  recorded ERROR outcome with whatever partial evidence exists.
Security/authorization checks: serial output is untrusted input to the
  matcher — bounded line length, bounded total capture, no shell
  interpretation of output, no path derived from output content.
Logic: pseudocode in §5.
Validation: W10-DV01/DV03/DV04.
```

## 3. Marker protocol (verdict inputs)

### 3.1 Marker classes

| Class | Source contract | W10 rule |
|---|---|---|
| `stable` token | emitted at W09 `stable` entry; token selected by W10 (decision 2) | required: at least one occurrence in a passing boot |
| fatal/crash class | W07 fatal report markers | forbidden: zero occurrences in a passing boot |
| panic class | W02/P0 panic route markers as fixed by W07 | forbidden: zero occurrences in a passing boot |
| boot-start markers | W09 `entry`/`runtime` events via W06 | required as sanity context, not verdict-critical alone |

The exact token strings are fixed in the implementation record before the
first verdict-bearing run and are never adjusted after evidence exists; a
token change invalidates prior comparisons and is recorded as such.

### 3.2 What is matched

Match is by fixed tokens against the bounded serial capture. Not matched and
never sufficient: line ordering, inter-line timing, total line count, checksum
of the full output, or absence of *unexpected* text beyond the forbidden
classes.

### 3.3 Order-independence rule

The verdict function is a conjunction of set-membership and set-emptiness
predicates over the capture (§5), plus the exit and timeout predicates. It
must remain true under any permutation of capture lines; this property is
checked by review and by scenario R6
([02-scenarios-and-verdict.md](02-scenarios-and-verdict.md)).

## 4. Outcome classification

Every cycle ends in exactly one outcome:

| Outcome | Condition | Verdict | Driver behavior |
|---|---|---|---|
| `PASS` | stable token present; forbidden classes absent; runner exit in expected set; completed within timeout | pass | record evidence; continue |
| `FAIL-MARKER` | timeout not hit and exit expected, but stable token missing or forbidden class present | fail | preserve full capture; stop cycle loop |
| `FAIL-TIMEOUT` | no completion within the bound | fail | terminate QEMU per runner contract; preserve partial capture; stop |
| `FAIL-EXIT` | runner/QEMU exit outside the expected set | fail | preserve capture and exit status; stop |
| `FAIL-PANIC` | panic/fatal class present (regardless of other markers) | fail | preserve full capture; stop |
| `ERROR-INVOCATION` | driver/runner could not start or arguments invalid | not a boot verdict | record as environment error; no cycle counted |

`FAIL-PANIC` takes precedence over `FAIL-MARKER` (a boot that panics after
printing stable content did not boot cleanly). The expected exit set for the
reference runner is fixed in the implementation record from the P0-W09
contract; until that contract exists, the driver is blocked (upstream defect
per the parent README), not defaulted.

## 5. Driver logic (pseudocode)

```text
for cycle in 1..=cycles:
    start runner(canonical invocation, image, timeout T, capture file)
    wait for exit or timeout
    capture <- bounded read of serial capture (line-length and size caps)
    outcome <- classify(capture, exit_status, elapsed)        # §4 table
    write evidence(cycle, outcome, capture or per-policy excerpt, meta)  # §6
    if outcome is fail or error: break                        # stop-on-failure
verdict_line(per-cycle outcomes, totals)
exit(0 iff all attempted cycles PASS and cycles == requested count)
```

Classification is a pure function of (capture, exit status, elapsed) so it can
be reviewed and, where the P0 host-test baseline permits, unit-tested without
QEMU. Matching uses fixed-token search with bounded windows; no regular
expression sourced from output, no recursion, no unbounded buffering.

## 6. Evidence layout and retention

Per run (the driver creates; the verification document references):

```text
<evidence-dir>/
  meta.txt            command line, image identity, host, QEMU/runner versions,
                      timestamps, timeout value used
  cycle-NNN/
    serial.log        full bounded capture (always for failures; per policy
                      below for passes)
    outcome.txt       outcome, predicate results, elapsed, exit status
  summary.txt         per-cycle outcome table, totals, stop reason if any
```

Retention policy for passing cycles in a 100-cycle run: full captures for the
first and last cycle, outcome records for all; intermediate full captures may
be summarized to keep evidence bounded — the policy chosen is recorded in
`meta.txt`. Failures always keep full captures. The committed verification
document (`../../verification/p1-w10-qemu-boot-regression-verification.md`)
embeds: the summary, the environment, the marker rules in force, failure
captures or their location, and explicit not-run entries. Raw evidence outside
`docs/` is referenced by path and identity; "location not recorded" is a
review failure.

## 7. Non-responsibilities

The driver does not build the image, patch the boot recipe, interpret crash
reports (it only detects their marker class), retry failed cycles, compare
runs against each other statistically, or validate output *content* beyond the
§3 classes. Those belong to the supplying contracts and to later stages.
