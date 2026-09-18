# P0-W12 Diagnostic Channels and Levels Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W12 detailed design](README.md).

## 1. Logical artifact groups and ownership

W12 is governance work, so its logical modules are authoritative artifact
groups, not Rust modules.

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Diagnostics baseline document | `docs/development/diagnostics-baseline.md` | ADR principle 10, ADR-048, §12, W04/W13/W14/W16 boundaries by subject, this design | the sole normative home of channel semantics, level semantics, visibility/trimming classes, fatal-information minimums, identity association, and consumer constraints; it does not implement or name any code surface |
| Documentation routing | one row in `docs/README.md` routing table | baseline document location | discoverability; it does not restate policy |
| Stage implementation index | `docs/stages/p0/implementation/README.md` | design status | a truthful status row; it never claims completion |
| Implementation record | `docs/stages/p0/implementation/p0-w12-logging-diagnostic-baseline-record.md` (created when work starts) | actual decisions taken | changed artifacts, deviations; no command logs |
| Verification record | `docs/stages/p0/verification/p0-w12-logging-diagnostic-baseline-verification.md` (created when evidence exists) | actual commands and review output | run/not-run evidence per the validation matrix; not part of the design |

The artifact named in the second column is the sole authoritative home for
the statement in its row.

## 2. Channel taxonomy contract

The baseline document must define exactly four channels. Each channel section
states purpose, audience, required content semantics, and a non-substitution
list. Channel semantics are binding on P1+; the mechanism that realizes a
channel is the owning design's.

### 2.1 Human-readable log

- **Purpose:** operator/bring-up narrative — what the system is doing and
  what an operator must know, in prose sentences.
- **Audience:** humans at a console or log capture; never a machine parser.
- **Required content semantics:** level-tagged prose; each statement
  self-describing enough to be read without source context; no field that a
  tool is expected to extract.
- **Must not be used to:** emit machine-consumed facts (that is the trace
  channel); carry aggregates (metrics channel); report a fatal exit (fatal
  channel); substitute for an error return — a logged error never replaces
  the operation's own failure handling.

### 2.2 Structured trace

- **Purpose:** machine-consumable facts about mechanism behavior, for
  analysis and regression comparison (ADR-048's coverage areas).
- **Audience:** tooling; offline analysis; never required to be human prose.
- **Required content semantics:** every event has a canonical identity under
  [W13](../p0-w13-trace-event-namespace-baseline/README.md)'s namespace
  (by subject); payload semantics are owned by the introducing design.
- **Must not be used to:** replace the human narrative for bring-up-critical
  milestones; carry unclassified failure exits (fatal channel); emit
  high-frequency events that cannot satisfy the trimming classes of §4.

### 2.3 Metrics

- **Purpose:** aggregable quantities — counts, rates, latencies, depths —
  for capacity and performance observation.
- **Audience:** aggregation and comparison tooling.
- **Required content semantics:** a quantity with a unit and a source
  identity; no prose; no one-off events (trace channel).
- **Must not be used to:** describe a single failure (that is trace or log
  depending on audience); hide a quantity that the ADR-048 coverage list
  expects (lock contention, IRQ latency, VM-exit distribution, and the rest
  are metric-or-trace subjects of their owning designs, never omissions).

### 2.4 Fatal/crash diagnostics

- **Purpose:** preserve the minimum state and attribution of a fatal exit
  (failure-class semantics owned by
  [P0-W14](../p0-w14-panic-failure-classification/README.md) by subject).
- **Audience:** post-mortem analysis by the crash design's consumers.
- **Required content semantics:** the minimum of
  [the crash contract](02-crash-identity-and-consumer-constraints.md) §1.
- **Must not be used to:** report guest-caused or recoverable failures
  (VM-scoped diagnostics instead); terminate execution — the channel records
  a fatal exit, the classification (W14) decides that the exit is fatal.

## 3. Log level contract

Five levels, fixed set and semantics. The future API surface names its own
items; these are semantic levels, not code identifiers.

| Level | Meaning | Retention default |
|---|---|---|
| Error | An operation failed such that the affected subsystem cannot continue correctly; the statement names what failed and the handling taken | always-retained class |
| Warn | Anomalous but survivable condition: degradation, retry, or unexpected state that does not stop the subsystem | removable-by-build-selection class |
| Info | Lifecycle and progress milestones an operator needs (bring-up stages, major object lifecycle transitions) | removable-by-build-selection class |
| Debug | Development-facing mechanism detail; useful without a live debugger, not needed in production | compiled-out-by-default class |
| Trace | High-frequency hot-path detail; correctness never depends on it | compiled-out-by-default class |

Cross-level rules the baseline must state:

- Levels measure impact on the *system's* operation; they are not the
  [W14](../p0-w14-panic-failure-classification/README.md) failure classes. A
  guest-caused fault is recorded at the level its system impact warrants and
  classified separately; a hypervisor-invariant fatal exit is never softened
  into an Error log line — it leaves the log channel for the fatal channel.
- Level choice is a design-review property: a P1+ design states the level of
  each diagnostic it introduces and may not promote/demote levels locally at
  implementation time to make output quieter or louder.
- No level may carry unvalidated guest-controlled content as if factual;
  guest-derived values are labeled as guest-provided and pass the
  untrusted-data rule of [the crash contract](02-crash-identity-and-consumer-constraints.md)
  §1.

## 4. Visibility and trimming contract

Three visibility classes, binding on every P1+ diagnostic design:

1. **Always-retained:** the fatal channel's minimum output and Error-level
   log statements. Present in every build where the channel exists at all.
2. **Removable-by-build-selection:** Warn and Info. A build selection may
   remove them; the selection must be classified under
   [W04](../p0-w04-build-profile-feature-governance/README.md) semantics — a
   binary capability or profile selection — never an ad-hoc
   conditional-compilation name invented at the call site.
3. **Compiled-out-by-default:** Debug, Trace, and any high-overhead trace
   event. Absent from production and benchmark builds unless a build
   selection explicitly includes them; when present, runtime filtering must
   be able to silence them without rebuild (ADR §12). The filter mechanism is
   the future subsystem design's; the filterability requirement is this
   baseline's and is not waivable per call site.

Additional required statements:

- Trimming decisions are per class and per channel, declared in the owning
  design — never per call site.
- A diagnostic removed by a build selection leaves no behavioral trace in the
  trimmed build beyond its absence (no half-formatted output, no side
  effects); arguments must not be evaluated when the class is compiled out —
  stated as a semantic requirement the future API design must satisfy.
- Benchmark/profile builds must be able to disable high-overhead events
  (§12) through the same classified selections, not through separate
  conventions.

## 5. Change thresholds for this contract's subject

- **Routine:** adding informative examples; clarifying wording without
  changing semantics.
- **Policy decision** (recorded issue and owner decision): adding or removing
  a channel; changing the level set or a level's meaning; reclassifying a
  level's visibility class.
- **ADR required:** dropping compile-time trimmability or runtime
  filterability for trace events; making any ADR-048 coverage area
  unobservable by design — each contradicts an accepted decision and follows
  `docs/adr/README.md`'s process.
