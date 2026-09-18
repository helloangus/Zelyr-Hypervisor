# P3-W03 Code Contracts — CPU Lifecycle State Machine

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W03 detailed design](README.md).

Contracts follow the project function/type template. Names are design-level
identifiers; concrete Rust paths are reserved to the workspace-owning
design.

## 1. `CpuLifecycleState`

```text
Name and stability: CpuLifecycleState — enum { Present, Starting,
    Initializing, Online, Failed, Offline, Stopping, Suspended };
    internal; W03-owned runtime vocabulary.
Purpose and caller: the total lifecycle vocabulary for physical CPUs;
    every P3 package and P4 (via W14) reads it; only W03 transitions it.
Inputs / outputs: stored packed in one byte per record (`as_u8`,
    `try_from_u8`); `Offline`, `Stopping`, `Suspended` are representable
    but unreachable at P3 (no edges).
Preconditions / postconditions: a state value is always one of the eight;
    a state read reflects a transition that was accepted, or the initial
    build value (Present for Present-class records; Failed/… never initial).
State and ownership change: only via §3 operations.
Concurrency/allocation context: Copy; reads are acquire-loads; the packed
    representation is fixed here so later designs never renumber it.
Errors and failure guarantee: `try_from_u8` rejects unknown encodings
    (corruption surfaces as a named error, not a wrong state).
Security/authorization checks: none (not guest-reachable).
Logic: plain enum + explicit discriminants.
Validation: W03-DV01 round-trip and unreachable-state tests.
```

## 2. Initial states at registry build

```text
Name and stability: build-time initial mapping — rule, not a function.
Purpose and caller: seeds each record's state from W01's input class.
Inputs / outputs: TopologyClass -> CpuLifecycleState.
Preconditions / postconditions:
    Present      -> Present     (bring-up candidate, not yet usable)
    Possible     -> Failed      (excluded: terminal non-member; cause
                                   recorded as `excluded_class` — see
                                   note below)
    Unavailable  -> Failed      (excluded, cause `excluded_class`,
                                   reason from W01 preserved in the
                                   record's exclusion note)
Boot CPU: the boot CPU's record also starts at Present; its
    Present→Initializing edge is the boot variant.
State and ownership change: set at build, before publication.
```

Note on the mapping: `Failed` is reused as the terminal *non-member*
state for excluded classes because P3's reachable set has no other
terminal non-member; the record's immutable exclusion note (class +
reason) preserves *why* it can never be online, so a Failed CPU that was
attempted and one that was never a candidate remain distinguishable in
diagnostics and evidence. The attempted/never-attempted distinction is
also derivable from whether a start request was ever recorded (W02's
outcome stream). If a reviewer finds this conflation misleading in
practice, splitting `Excluded` out of `Failed` is a reviewed vocabulary
change — recorded here as the preferred follow-up rather than silently
shipped in implementation.

## 3. Transition operations

All five operations share one template; the differences are the edge,
owner, and cause vocabulary.

```text
Name and stability: request_start(logical) — Present→Starting;
    enter_initializing(logical) — Starting→Initializing (secondary self)
    and Present→Initializing (boot variant, boot CPU only);
    report_failure(logical, phase, cause) — Starting→Failed;
    admit_online(logical) — Initializing→Online (rendezvous coordinator
    only). All internal; callable only through the registry.
Purpose and caller: the only mutation paths for state words; owners per
    the architecture §2 table.
Inputs / outputs: logical id + cause payload; Ok(()) or
    Err(TransitionError).
Preconditions / postconditions: precondition — the record's current
    state matches the edge's source and the caller is the designated
    owner (per contract); postcondition — state equals the edge's
    target, exactly once, with one event emitted.
State and ownership change: one atomic compare-exchange per call;
    release semantics on success.
Concurrency/allocation context: lock-free; acquire-release CAS; no
    allocation; callable in any context where atomics are legal (the
    registry is built before interrupts matter at P3).
Errors and failure guarantee: Err(TransitionError::IllegalEdge |
    UnknownCpu) without mutation; duplicate admission contention (§4 of
    the architecture file) does not return — it takes the fatal path.
Security/authorization checks: ownership checks are semantic (documented
    caller), enforced by call-site review and, for admit_online, by the
    coordinator-only access path.
Logic (per operation):

    transition(record, expected, target, cause):
        cur = record.state.load(Acquire)
        if cur != expected: return Err(IllegalEdge(cur))
        if record.state.compare_exchange(expected, target, AcqRel, Acquire).is_err():
            if edge is admit_online: fatal_invariant(duplicate_admission)
            return Err(IllegalEdge)          # raced edge other than admission
        emit_event(logical, expected, target, cause)   # ADR-048
        return Ok(())

Validation: W03-DV02 table tests (every legal edge accepted exactly once;
    every illegal edge refused with the current state named);
    W03-DV03 admission tests.
```

## 4. `TransitionError`

```text
Name and stability: enum { IllegalEdge(CpuLifecycleState), UnknownCpu }
    — internal.
Purpose and caller: refusal diagnostics; callers are W02 (reported
    failure paths), W05 (coordinator), and tests.
Preconditions / postconditions: carries the observed state for
    attribution; no mutation ever accompanies an Err.
Errors and failure guarantee: n/a (is the error type).
Logic: plain enum.
Validation: exhaustive-match review.
```

## 5. Transition events

```text
Name and stability: lifecycle transition event — content record per the
    P0 trace governance; catalog registration owned by P3-W11.
Purpose and caller: ADR-048 observability of lifecycle activity; emitted
    by every accepted transition and (as refusal diagnostics) by every
    refusal.
Inputs / outputs: fields — logical id, hardware id, from-state, to-state,
    cause, caller role (requester/secondary/coordinator/boot).
Preconditions / postconditions: one event per accepted edge; refusals
    emit at diagnostic level with the same attribution.
Concurrency/allocation context: emission must not allocate (governance
    permitting) and must be safe from the secondary's early context;
    exact transport per P0-W12.
Errors and failure guarantee: emission failure must never alter the
    transition result (observability is not load-bearing for state).
Validation: W03-DV05 capture review.
```
