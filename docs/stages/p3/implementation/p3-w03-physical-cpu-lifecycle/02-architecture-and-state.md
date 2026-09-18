# P3-W03 Architecture and State

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W03 detailed design](README.md).

## 1. Objects and ownership

| Object | Count | Owner | Mutable parts | Immutability |
|---|---|---|---|---|
| `CpuRegistry` | exactly one per boot | W03 | none after build (the collection itself is fixed) | built once during global initialization from `TopologyInputs`; record set is boot-static |
| `PhysicalCpuRecord` | one per discovered CPU | W03 | the state word only | identity fields (logical id, hardware id, class, boot flag) immutable after build |
| `CpuLifecycleState` word | one per record | the record | via designated transition operations only | packed encoding; direct writes are a review failure |
| Transition event | one per attempted transition | emitted, not stored (telemetry/audit consume) | n/a | immutable once emitted |

There is exactly one registry and no shadow authority: consumers that need
"which CPUs are usable" call the snapshot; consumers that need "may I use
CPU N" call the gate. Cached or derived copies elsewhere (counters,
telemetry, diagnostics) are informative only and must be labeled as such.

## 2. Lifecycle state machine

Reachable states (P3):

```text
                    boot CPU only (no start request exists for it)
        Present ────────────────────────────┐
           │                                │
           │ requester (W02 flow)           │
           v                                v
        Starting ────> Initializing ────> Online
           │                                
           │ requester, on rejection or timeout            
           v                                
         Failed  (terminal; Initializing has no Failed exit at P3 —
                  see the refusal rules below)
```

Binding edge list (all others are refusals):

| Edge | Designated owner | Called from | Cause vocabulary |
|---|---|---|---|
| Present → Starting | W03 operation: request-start | W02 bring-up sequencer, once per candidate, before its CPU_ON | start-requested |
| Starting → Initializing | W03 operation: enter-initializing | the started secondary itself, after identity confirmation | self-initializing |
| Present → Initializing | W03 operation: enter-initializing (boot variant) | the boot CPU itself, during its local initialization | boot-self-initializing |
| Starting → Failed | W03 operation: report-failure | W02 requester on request rejection or timeout | request-rejected, timeout |
| Initializing → Online | W03 operation: admit-online | W05 rendezvous completion (boot CPU coordinating), once per CPU | rendezvous-admitted |

Refusal rules (diagnosed, never coerced):

- Any edge not in the table is refused with `TransitionError::IllegalEdge`.
- Initializing→Failed is deliberately **not** a P3 edge: a CPU that
  completes local initialization has passed the point where failure is
  W02's to report; a later fault on an Initializing/Online CPU is a fatal
  hypervisor event (P1-W07 path), not a lifecycle transition at P3.
  Rationale: at P3 a faulting online CPU indicates an invariant violation,
  and lifecycle must not silently absorb it into a recoverable-looking
  state.
- Failed is terminal; Starting and Initializing have no exit other than
  the listed edges.

Reserved, declared-unreachable states (vocabulary only): `Offline`
(runtime online→offline), `Stopping` (graceful teardown), `Suspended`
(low-power). They have no incoming or outgoing P3 edges; encoders must
still represent them so later designs do not renumber the vocabulary.

## 3. Transition ownership and concurrency

- One owner per edge (table above). The ownership is a *semantic*
  property: the operation checks nothing about which CPU invoked it beyond
  what its contract states (e.g., admit-online is callable only by the
  rendezvous coordinator; enter-initializing's boot variant only by the
  boot CPU). Hardware identity of the caller is verified where the
  contract requires it (the secondary's self-transition is made by that
  CPU for its own logical id — verified by W02's identity confirmation
  before the call, and by the transition targeting exactly that record).
- Each state word is a single atomic cell. Transitions use
  acquire-release compare-exchange against the expected source state;
  reads use acquire-loads. No lock is used anywhere in W03 (P3-W06 owns
  lock semantics; none is needed for single-word transitions).
- The admission edge (Initializing→Online) is the exactly-once critical
  section: a single compare-exchange. Success admits; failure means
  another actor attempted a duplicate admission — an invariant violation
  that takes the fatal diagnostic path (README decision 4).
- Publication: the online states reached during rendezvous become
  cross-CPU visible under W05's publication gate; W03's obligation is
  that the state word is release-stored by the transition itself, so the
  gate's acquire side observes a consistent value.

## 4. Eligibility and the online set

```text
Eligibility gate (consumed by W04/W05/W07 admission paths):
    Online               -> Eligible
    Initializing         -> EligibleForLocalInstall   (W04 only)
    Present/Starting     -> NotEligible (NotInitialized)
    Failed               -> NotEligible (Failed)
    Possible/Unavailable -> NotEligible (ExcludedByClass)
    no record            -> NotEligible (Unknown)

Online set (snapshot):
    the set of logical ids whose state word reads Online, plus the count;
    derived by a registry scan at call time; no cached authority.
```

`EligibleForLocalInstall` is the boundary that makes P3-V03's "no CPU
usable before local initialization" structural: W04 may install per-CPU
runtime only on a CPU whose state is `Initializing` (the boot CPU
included — it enters Initializing via its own boot-variant transition
before installing, and is admitted Online by the rendezvous like every
other CPU), and every *use* surface — notification targeting,
scheduler-adjacent queries, tests — sees only `Eligible`.

## 5. Failure model

- Transition refusals are diagnosable data (`TransitionError`), returned
  to the caller; the registry never mutates on refusal. A refusal is a
  caller bug or a protocol violation; callers either handle it as a
  fatal invariant violation (secondary side) or as a reported failure
  (requester side, before the CPU ever ran).
- Duplicate admission contention: fatal diagnostic with both the logical
  id and both contenders' call sites (the CAS loser knows its own).
- Registry build failure (allocation, malformed topology): fatal
  boot-critical before any CPU starts; no partial registry is published.

## 6. Consumer integration map

| Consumer | Surface consumed | Contract point |
|---|---|---|
| W04 (per-CPU runtime) | eligibility gate (install-time), online-set (rollout checks) | this file §4 |
| W05 (boot rendezvous) | admit-online operation; registry scan for terminal-state checks | §2 table; §4 snapshot |
| W07/W08 (notification, TLB transport) | online-set snapshot as targeting universe | §4 |
| W09 (exception/interrupt diagnostics) | read of own record state for attribution | §1 |
| W10 (SMP audit) | transition-event stream; classification of registry state words | §2, §3 |
| W11 (observability) | event content contract (catalog owner) | §2 causes |
| W12/W13 (stress, regression) | snapshot assertions; state dump at SMP-ready | §4 |
| W14/P4 (handoff) | availability semantics only | entry README |
