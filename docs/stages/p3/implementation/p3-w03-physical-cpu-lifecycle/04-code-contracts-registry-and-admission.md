# P3-W03 Code Contracts — Registry, Admission, and Online Set

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W03 detailed design](README.md).

Contracts follow the project function/type template. Names are
design-level identifiers; concrete Rust paths are reserved to the
workspace-owning design.

## 1. `PhysicalCpuRecord`

```text
Name and stability: PhysicalCpuRecord — registry-owned record; internal.
Purpose and caller: the authoritative per-CPU object (ADR object model);
    accessed by W03 operations and read-only by consumers.
Inputs / outputs: fields — logical: LogicalCpuId; hardware:
    HardwareCpuId; class_note: immutable copy of the W01 class + exclusion
    reason; boot: bool; state: AtomicU8 (packed CpuLifecycleState);
    exclusion_note: Option<ExclusionNote> (class/why for never-candidates).
Preconditions / postconditions: identity fields immutable after build;
    state changed only through §3 of the state-machine file; exactly one
    record per discovered CPU (dense logical ids per W01).
State and ownership change: owned by the registry for the whole boot;
    never freed.
Concurrency/allocation context: allocated once at build (P2-W05 small
    allocation, boot CPU, pre-release); state word is the only atomic.
Errors and failure guarantee: n/a.
Security/authorization checks: none (not guest-reachable).
Logic: plain data + atomic word.
Validation: covered by registry tests (W03-DV02..DV04).
```

## 2. `CpuRegistry`

```text
Name and stability: CpuRegistry — the single lifecycle authority;
    internal; exactly one instance per boot.
Purpose and caller: owns all records and the transition operations;
    queried by every P3 package; seeded by the boot sequence.
Inputs / outputs: built by `CpuRegistry::build(topology)`; accessors:
    `record(logical) -> Option<&PhysicalCpuRecord>`,
    `hardware_of(logical)`, `logical_of(hardware)`,
    `state_of(logical) -> Result<CpuLifecycleState, UnknownCpu>`,
    `online_set() -> OnlineSet`, `eligibility(logical) -> Eligibility`,
    `count()`, the five transition operations, and
    `render_state_dump() -> diagnostic lines`.
Preconditions / postconditions: after build — record set equals the
    discovered inventory (dense), initial states per the build mapping,
    boot record flagged; no record is ever added or removed at P3.
State and ownership change: build-time only, then frozen except state
    words.
Concurrency/allocation context: build on the boot CPU pre-release
    (allocation per P2-W05; failure fatal); reads lock-free.
Errors and failure guarantee: build failure is fatal boot-critical; no
    partial registry is published.
Security/authorization checks: n/a.
Logic: fixed-capacity table indexed by logical id (density invariant from
    W01 makes this an index, not a search); capacity = topology count
    (≤ 8).
Validation: W03-DV02.
```

## 3. `admit_online` — exactly-once admission

```text
Name and stability: admit_online(logical: LogicalCpuId) -> Result<(),
    TransitionError> — the Initializing→Online edge; coordinator-only
    access path.
Purpose and caller: the P3-V03 online-admission invariant; caller is the
    P3-W05 rendezvous completion, once per CPU whose readiness condition
    holds.
Inputs / outputs: logical id; Ok on first admission, Err(IllegalEdge)
    if the record is not Initializing.
Preconditions / postconditions: precondition — the caller is the
    rendezvous coordinator and the CPU's declared readiness condition
    holds (W05's contract, not re-checked here). Postcondition — at most
    one success per record per boot; state is Online; event emitted.
State and ownership change: single CAS; release-store makes Online
    visible; the W05 publication gate then publishes the rendezvous
    result cross-CPU.
Concurrency/allocation context: lock-free CAS (AcqRel/Acquire). On CAS
    failure the operation does NOT return an error to the caller: a
    contender that already changed the word indicates a duplicate
    admission attempt, which is an internal invariant violation — take
    the fatal diagnostic path with both the logical id and the call site
    (architecture §3; README decision 4).
Errors and failure guarantee: Err only for a record not in Initializing;
    never mutates on Err.
Security/authorization checks: coordinator-only path documented; a
    non-coordinator call is a review-visible contract violation.
Logic: as `transition(...)` with the admission-specific failure rule.
Validation: W03-DV03 (first admission succeeds; second attempt is
    detected; non-Initializing sources are refused).
```

## 4. `Eligibility`

```text
Name and stability: enum Eligibility { Eligible, EligibleForLocalInstall,
    NotEligible(NotEligibleCause) } with NotEligibleCause {
    NotInitialized, Failed, ExcludedByClass, Unknown } — internal.
Purpose and caller: the structural gate preventing use-before-init and
    failed-CPU use; callers: W04 (install-time), W05, W07/W08
    (targeting), tests.
Inputs / outputs: computed from the record's current state and class
    note per the architecture §4 table.
Preconditions / postconditions: pure function of the state at read time;
    a snapshot taken once must be re-checked after any yielding wait
    (callers' obligation).
State and ownership change: none.
Concurrency/allocation context: acquire-load of the state word; no
    allocation.
Errors and failure guarantee: n/a.
Security/authorization checks: this is the authorization boundary for
    per-CPU resources at P3: gate answers are the only sanctioned "may I"
    source.
Logic: match per architecture §4.
Validation: W03-DV04 table tests over all states × queries.
```

## 5. `OnlineSet` snapshot

```text
Name and stability: OnlineSet — immutable snapshot value { logical_ids:
    bit set over dense range, count: usize }; internal.
Purpose and caller: the authoritative answer to "which CPUs are online";
    callers: W05 (terminal-state checks), W07/W08 (targeting universe),
    W12/W13 (assertions), diagnostics.
Inputs / outputs: produced by `online_set()`; supports `contains`,
    `count`, iteration in logical order.
Preconditions / postconditions: consistent with the registry as of the
    scan; becomes stale as transitions occur (documented: consumers
    needing freshness re-query; rendezvous-time consumers read after the
    publication gate).
State and ownership change: none (derived value).
Concurrency/allocation context: fixed-capacity bit set (≤ 8), stack
    constructible, no allocation.
Errors and failure guarantee: n/a.
Security/authorization checks: the targeting universe boundary for W07 —
    a CPU absent here must be untargetable by construction.
Logic: scan records; collect Online states.
Validation: W03-DV04; capture review in W03-DV05.
```

## 6. `render_state_dump`

```text
Name and stability: render_state_dump() — diagnostic output per the P0
    logging governance.
Purpose and caller: the SMP-ready registry dump surface; caller is the
    boot sequence at the point W05's design designates (after the
    rendezvous completes), and the P1-W07 fatal path when available.
Inputs / outputs: one line per record — logical id, hardware id, class
    note, boot flag, state, exclusion note if any — plus the online
    count.
Preconditions / postconditions: read-only; safe at any point after build.
Errors and failure guarantee: none.
Validation: W03-DV05 capture review (P3-V03 evidence surface).
```
