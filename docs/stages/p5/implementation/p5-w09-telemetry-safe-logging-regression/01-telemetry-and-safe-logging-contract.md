# P5-W09 Telemetry and Safe-Logging Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P5-W09 detailed design](README.md).

## 1. Assumed prerequisite contracts and failure boundaries

| Assumed contract | Source | What W09 assumes | Failure boundary if delivered differently |
|---|---|---|---|
| One result category per dispatched call, per the W06 taxonomy | W06 ([dispatch design](../p5-w06-dispatch-permission-containment/README.md), its containment file §4) | the category set and per-call assignment exist at S8 | a divergent category set is a W06 conflict — stop and record; W09 must not define a second taxonomy |
| Stable Guest markers and scenario inventory | W07 ([suite design](../p5-w07-validation-guest-isolation-suite/README.md)) | marker grammar, inventory, and non-success classes as delivered | markers are consumed read-only; divergence blocks the regression rows concerned |
| Fixed fuzz-smoke configuration and oracle classes | W08 ([baseline design](../p5-w08-host-fuzz-stress-smp-baseline/README.md)) | seeds, bounds, and oracle classes as delivered | the smoke row mirrors the delivered config; divergence blocks the row |
| P4 regression set and automation entry point | P4-W08/P4-W09 (assigned prerequisites) | an implemented, evidenced P4 regression exists to inherit | absence is a blocked prerequisite for P5-V16's inherited rows — record; never reconstruct P4 cases from their plan text |
| Diagnostics semantics (levels, visibility, trimming, crash info) | P0-W12 | delivered as planned | a gap blocks classification of outputs — record; no private convention |
| Event namespace and review rules | P0-W13 | the capability-domain namespace rules exist | a gap blocks event registration — record; no off-namespace names |

## 2. Counter contract

### 2.1 `HypercallResultCounters` (type, internal)

```text
Name and stability: HypercallResultCounters; internal to the P5 telemetry
integration; not API, not ABI.
Purpose and caller: per-VM accumulation of dispatch result categories;
instantiated per VM/security context at VM creation by the VM's owning
lifetime code; read by debug/telemetry readers and by the regression
record collection.
Inputs / outputs: record(category) on the dispatch path; read() producing
per-category counts.
Preconditions / postconditions: one counter set per VM; counts only
increase during the VM's life; read() is consistent enough for diagnostics
(approximate under concurrency is acceptable and stated at read).
State and ownership change: owned by the VM; no global registry of counts.
Concurrency/allocation context: record() runs in VM-exit context: fixed
size, no allocation, no locks that can block — per-CPU or atomic-slot
layout per the delivered P3 per-CPU contract; aggregation at read.
Errors and failure guarantee: counter failure (e.g., exhausted slot) is
recorded as a telemetry-diagnostic event and never alters the dispatch
outcome — observability must not change core semantics.
Security/authorization checks: counters carry only categories and VM
identity — never addresses, buffer data, or capability state (see §4).
Logic: plain per-category counters with a read-time aggregate.
Validation: W09-DV02/DV03.
```

### 2.2 Category set

Exactly the W06 taxonomy: `completed`, `denied-structural`,
`denied-reference`, `denied-authority`, `denied-address`, `denied-state`,
`resource`, `invariant`. The `invariant` counter is expected to remain zero
in all Guest-facing evidence; a nonzero value is itself a recorded failure
condition for the regression suite. No category may be added, merged, or
renamed by W09.

## 3. Trace-event contract

### 3.1 `HypercallResult` event (namespace-registered)

```text
Name and stability: event family per the P0-W13 namespace rules for the
capability domain; the concrete event name is assigned at implementation
under those rules and recorded; the family is versioned per the namespace's
version/compatibility rule. P5-experimental; not a public contract.
Purpose and caller: structured, machine-readable record of one dispatched
call's outcome for diagnostics and regression tooling; emitted at dispatch
S8 by the same step that records the category.
Inputs / outputs (typed fields, fixed set):
  vm identifier; operation identity; outcome class; result category;
  dispatch stage reached (for non-completed outcomes).
  Explicitly excluded fields: Host virtual/physical addresses; Host object
  pointers; Guest buffer contents; raw handle values beyond what the Guest
  itself supplied (permitted: the Guest-known reference value where W07's
  discipline allows it); capability internal state.
Preconditions / postconditions: at most one event per dispatched call,
emitted with or after the category record; never before the outcome exists.
State and ownership change: none.
Concurrency/allocation context: VM-exit context — bounded, fixed-size
fields; no formatting of variable data; emission is skipped, never blocked,
if the diagnostics path is disabled or cannot keep up (loss is acceptable
for this event family and stated; the invariant event of §2.2's `invariant`
category is the P0-W12-classified crash path's concern, not this family's).
Errors and failure guarantee: emission failure never propagates to the
dispatch outcome.
Security/authorization checks: field set is closed; §4 redaction applies.
Logic: field population from the dispatch outcome; no computation.
Validation: W09-DV03 (categories observable), W09-DV04 (no disclosure).
```

### 3.2 Log-channel classification (per P0-W12, cited not restated)

| Output | P0-W12 channel | W09 rule |
|---|---|---|
| per-call result category/event | structured trace | default path; §3.1 fields only |
| aggregate counters view | metrics-style structured output | read-time aggregation; no per-call flood |
| unusual-condition notes (e.g., repeated resource outcomes) | rate-limited human log | rate-limited; counts and categories only; release/debug visibility per P0-W12 |
| invariant-path context | crash diagnostics | owned by the P0-W14-classified fatal path; W09 adds nothing |

## 4. Safe-debug context and redaction rules

**Default diagnostics** = every output reachable in the release/production
configuration and every regression-run artifact. **Developer-debug
context** = builds/configuration gated per P0-W12's debug visibility and
trimming rules.

| Content | Default diagnostics | Developer-debug context |
|---|---|---|
| VM/security-context identifier | allowed (identity, not authority) | allowed |
| outcome class, result category, dispatch stage | allowed | allowed |
| operation identity within the delivered set | allowed | allowed |
| Guest-known reference (handle) value | only where W07's marker discipline allows | allowed with care; never presented as authority |
| Guest buffer contents | forbidden | forbidden (no diagnostic needs buffer contents; redaction of raw sensitive buffers is unconditional) |
| Host virtual/physical addresses | forbidden | only as P0-W12's debug rules permit, never in regression artifacts |
| Host object pointers / capability internal state | forbidden | only via the owning designs' explicit debug facilities; never in regression artifacts |
| credentials/secrets | out of scope by architecture (EL2 stores none); any appearance is an invariant finding | same |

The redaction rules bind every W09-visible path: counters, events, rate-
limited logs, and the regression record's collection step. The reviewer-
facing check is W09-DV04; the automated backstop is the regression suite's
`leaked-sensitive-information` class
([02 §4](02-regression-matrix-and-workflow.md)).

## 5. Lifecycle

Counter sets are created with the VM and destroyed with it; no counter
outlives its VM. Events exist only per dispatched call. The regression
expectation set is versioned by the composed rows
([02 §2](02-regression-matrix-and-workflow.md)); its history lives in the
records, not in the code. Telemetry additions for P6+ mechanisms are
future families under P0-W13's namespace rules, explicitly out of scope
here.
