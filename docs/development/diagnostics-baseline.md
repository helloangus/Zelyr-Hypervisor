# Zelyr Diagnostics Baseline

**Status:** Normative diagnostics governance.  
**Scope:** The four diagnostic channels, the five log levels and their
retention classes, the three visibility/trimming classes, the minimum fatal
information, identity association, and the binding constraint lists for P1+
diagnostic designs. It implements and names **no code surface** — the
mechanism realizing a channel is the owning design's; trace-event naming is
P0-W13's; failure classes are P0-W14's; identity fields are P0-W16's.  
**Version:** v0.1  
**Owner/change context:** P0-W12 logging/diagnostic baseline;
operationalizes the ADR's first-class telemetry principle, ADR-048, and the
§12 observability invariants.  
**Supersedes:** The absence of a diagnostics policy.

## 1. Channel taxonomy

Exactly four channels exist. Channel semantics are binding on P1+; each
section states purpose, audience, required content semantics, and a
non-substitution list.

### 1.1 Human-readable log

- **Purpose:** operator/bring-up narrative — what the system is doing and
  what an operator must know, in prose sentences.
- **Audience:** humans at a console or log capture; never a machine parser.
- **Required content semantics:** level-tagged prose; each statement
  self-describing enough to be read without source context; no field that a
  tool is expected to extract.
- **Must not be used to:** emit machine-consumed facts (trace channel);
  carry aggregates (metrics channel); report a fatal exit (fatal channel);
  substitute for an error return — a logged error never replaces the
  operation's own failure handling.

### 1.2 Structured trace

- **Purpose:** machine-consumable facts about mechanism behavior, for
  analysis and regression comparison (ADR-048's coverage areas).
- **Audience:** tooling; offline analysis; never required to be human prose.
- **Required content semantics:** every event has a canonical identity under
  the [trace namespace](trace-event-namespace.md) (P0-W13 subject); payload
  semantics are owned by the introducing design.
- **Must not be used to:** replace the human narrative for bring-up-critical
  milestones; carry unclassified failure exits (fatal channel); emit
  high-frequency events that cannot satisfy §3's trimming classes.

### 1.3 Metrics

- **Purpose:** aggregable quantities — counts, rates, latencies, depths —
  for capacity and performance observation.
- **Audience:** aggregation and comparison tooling.
- **Required content semantics:** a quantity with a unit and a source
  identity; no prose; no one-off events (trace channel).
- **Must not be used to:** describe a single failure (trace or log,
  depending on audience); hide a quantity the ADR-048 coverage list expects
  (lock contention, IRQ latency, VM-exit distribution, and the rest are
  metric-or-trace subjects of their owning designs, never omissions).

### 1.4 Fatal/crash diagnostics

- **Purpose:** preserve the minimum state and attribution of a fatal exit
  (failure-class semantics owned by the [failure
  classification](../security/failure-classification.md), by subject).
- **Audience:** post-mortem analysis by the crash design's consumers.
- **Required content semantics:** the minimum of §2.
- **Must not be used to:** report guest-caused or recoverable failures
  (VM-scoped diagnostics instead); terminate execution — the channel records
  a fatal exit; the classification decides that the exit is fatal.

## 2. Log levels

Five levels, fixed set and semantics. The future API surface names its own
items; these are semantic levels, not code identifiers.

| Level | Meaning | Retention default |
|---|---|---|
| Error | An operation failed such that the affected subsystem cannot continue correctly; the statement names what failed and the handling taken | always-retained class |
| Warn | Anomalous but survivable condition: degradation, retry, or unexpected state that does not stop the subsystem | removable-by-build-selection class |
| Info | Lifecycle and progress milestones an operator needs (bring-up stages, major object lifecycle transitions) | removable-by-build-selection class |
| Debug | Development-facing mechanism detail; useful without a live debugger, not needed in production | compiled-out-by-default class |
| Trace | High-frequency hot-path detail; correctness never depends on it | compiled-out-by-default class |

Cross-level rules:

- Levels measure impact on the *system's* operation; they are not the
  failure classes of the [failure
  classification](../security/failure-classification.md). A guest-caused
  fault is recorded at the level its system impact warrants and classified
  separately; a hypervisor-invariant fatal exit is never softened into an
  Error log line — it leaves the log channel for the fatal channel.
- Level choice is a design-review property: a P1+ design states the level of
  each diagnostic it introduces and may not promote/demote levels locally at
  implementation time to make output quieter or louder.
- No level may carry unvalidated guest-controlled content as if factual;
  guest-derived values are labeled as guest-provided and pass §2.3's
  untrusted-data rule.

## 3. Visibility and trimming

Three visibility classes, binding on every P1+ diagnostic design:

1. **Always-retained:** the fatal channel's minimum output and Error-level
   log statements. Present in every build where the channel exists at all.
2. **Removable-by-build-selection:** Warn and Info. A build selection may
   remove them; the selection must be classified under the [build-choice
   governance](build-profile-governance.md) semantics — a binary capability
   or profile selection — never an ad-hoc conditional-compilation name
   invented at the call site.
3. **Compiled-out-by-default:** Debug, Trace, and any high-overhead trace
   event. Absent from production and benchmark builds unless a build
   selection explicitly includes them; when present, runtime filtering must
   be able to silence them without rebuild (ADR §12). The filter mechanism is
   the future subsystem design's; the filterability requirement is this
   baseline's and is not waivable per call site.

Additional required statements:

- Trimming decisions are per class and per channel, declared in the owning
  design — never per call site.
- A diagnostic removed by a build selection leaves no behavioral trace in
  the trimmed build beyond its absence (no half-formatted output, no side
  effects); arguments must not be evaluated when the class is compiled out —
  a semantic requirement the future API design must satisfy.
- Benchmark/profile builds must be able to disable high-overhead events
  (§12) through the same classified selections, not through separate
  conventions.

## 4. Minimum fatal information

Semantic requirements on the P1 crash/panic designs (P1-W06 early console
logging; P1-W07 fatal crash diagnostics); the owning design decides
representations, registers, and formats within them.

### 4.1 Panic message minimum

Every panic/fatal-exit message must permit attributing:

- **Failure classification:** which failure class the exit belongs to, per
  the [failure classification](../security/failure-classification.md)
  taxonomy (by subject; until that taxonomy was delivered, the statement was
  written in W14-agnostic terms — the taxonomy is now delivered and binds).
- **Site identification:** where the exit was raised — source location or an
  equivalent site identity the owning design defines.
- **Build identity:** a pointer to §5's identity minimum — inline where the
  owning design can, otherwise by a stated artifact association.

### 4.2 Crash dump minimum

A crash dump must additionally preserve, as applicable to the site and named
by the owning design:

- the machine state needed to diagnose the exit class (for an EL2 exception
  site, the architectural state the P1-W07 design names; stated here as a
  requirement, never as a register list);
- the identity set of §5;
- enough of the last diagnostic context (channel, level, or trace
  identities) to correlate the exit with preceding behavior, when the
  logging design makes that correlable.

### 4.3 Untrusted-data rule

Guest, device, firmware, or management-domain values must never appear in a
panic message or crash dump as trusted facts. They are labeled by source and
included only as bounded, validated displays; a value that cannot be
validated at dump time is shown as unvalidated. Reason: the dump is consumed
as ground truth for post-mortem work (ADR-007's posture applied to
diagnostics).

## 5. Identity association

- **Property:** every diagnostic record must be associable with the
  build/version identity minimum owned by P0-W16 (by reference; this
  baseline restates no field). Cross-review record: the delivered
  [version/build metadata baseline](version-build-metadata.md) carries an
  identity minimum set that satisfies this property; the mutual cross-review
  is recorded in both verification records.
- **Inline versus associable:** fatal/crash output must carry identity
  inline. Other channels must be associable via the artifact or session that
  produced them; the mechanism (embedding, sidecar, session record) is owned
  by the producing design, consistent with W16's deferral of association
  mechanics.
- **Scope boundary:** this baseline requires associability of diagnostics;
  it does not define artifact naming (P0-W17 subject) or any metadata field.

## 6. Constraints on P1+ diagnostic designs

A P1+ design that cannot satisfy an item raises the conflict at design
review instead of deviating silently.

### 6.1 Must

- **M1:** Define the concrete logging/trace/metrics surface (names, macros,
  or items) within the level set and visibility classes of §§2–3; the
  surface's namespace follows the consuming design's naming governance.
- **M2:** Bind the P1 early console as the first transport for the human log
  and fatal channels; the transport is the P1-W06 (early console logging)
  subject.
- **M3:** Implement runtime filtering for the compiled-out-by-default class
  wherever that class exists in the build.
- **M4:** Keep channel boundaries: machine-consumed facts to trace/metrics,
  narrative to the log, fatal exits only through the fatal channel.
- **M5:** Satisfy §4's fatal minimums for every fatal exit path the design
  introduces.
- **M6:** State, per introduced diagnostic, its channel, level, and
  visibility class.

### 6.2 Must not

- **N1:** Create a parallel logging path (direct console writes as a
  competing channel) after the logging design's adoption point.
- **N2:** Exit fatally outside the [failure
  classification](../security/failure-classification.md)'s rules — in
  particular never on guest-caused input.
- **N3:** Encode machine-parsed fields in the human log, or prose in
  trace/metrics.
- **N4:** Route around trimming with unclassified switches or call-site
  conditional compilation (violates the [build-choice
  governance](build-profile-governance.md) too).
- **N5:** Log unvalidated guest-controlled data as trusted fact (violates
  §4.3).

### 6.3 Transitional early-console rule

Before a logging subsystem exists (P1 start), bring-up output via the early
console is permitted as the boot-time transport, provided it is designed as
the M2 transport binding — not as a competing channel. The P1-W06 design
owns the transition point at which early-console output becomes the logging
surface's transport; from that point N1 applies. This rule gives P1
governance from day one without pre-deciding its logging design.

## 7. Boundaries with sibling contracts

| Subject | Owner | This baseline's statement |
|---|---|---|
| Trace-event names, domain registry, compatibility | [trace namespace](trace-event-namespace.md) (P0-W13) | channels reference canonical names; no name is defined here |
| Failure classes, fatal vs recoverable, containment | [failure classification](../security/failure-classification.md) (P0-W14) | the fatal channel is reserved for invariant-fatal exits; other classes get VM-scoped or metric treatments |
| Identity fields, formats, timestamp policy | [version/build metadata](version-build-metadata.md) (P0-W16) | the association property consumes W16's minimum set by reference |
| Channel semantics, levels, visibility classes, fatal minimums | this baseline (P0-W12) | others reference; none restate |

A conflict between delivered sibling contracts and this one is reconciled in
the same change where possible; otherwise raised as a cross-package design
conflict, and where an ADR-level property is touched, labelled
`ADR Required`.

## 8. Change thresholds

- **Routine:** adding informative examples; clarifying wording without
  changing semantics.
- **Policy decision** (recorded issue and owner decision): adding or
  removing a channel; changing the level set or a level's meaning;
  reclassifying a level's visibility class.
- **ADR required:** dropping compile-time trimmability or runtime
  filterability for trace events; making any ADR-048 coverage area
  unobservable by design — each contradicts an accepted decision and follows
  the [ADR process](../adr/README.md).
