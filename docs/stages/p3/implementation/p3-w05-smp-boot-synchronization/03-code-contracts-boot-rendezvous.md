# P3-W05 Code Contracts — Boot Rendezvous

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W05 detailed design](README.md).

Contracts follow the project function/type template. Names are design-level
identifiers; concrete Rust paths are reserved to the workspace-owning
design. The whole surface is one-shot boot machinery: no type here offers
reset, reuse, or general-purpose waiting.

## 1. `BootPhase`

```text
Name and stability: BootPhase — enum { Bootstrap, GlobalInitPublished,
    SmpReady }; internal; stored in one AtomicU8 with fixed discriminants.
Purpose and caller: the boot-ordering authority; read by every rule in
    the architecture file §5; written only by the two transitions below.
Preconditions / postconditions: monotonic; exactly one instance per
    boot; starts at Bootstrap.
State and ownership change: only via publish_global_init and
    declare_smp_ready.
Concurrency/allocation context: CAS transitions (AcqRel/Acquire);
    acquire-loads for gate checks; no allocation.
Errors and failure guarantee: unknown encodings rejected on decode
    (corruption is a named failure, fatal path).
Security/authorization checks: none (not guest-reachable).
Logic: plain enum + atomic word.
Validation: W05-DV01.
```

## 2. `publish_global_init` and `require_published`

```text
Name and stability: publish_global_init() — internal; called once by the
    boot CPU when every global-init step (W01 intake, W03 build, W04
    allocate_all, W02 provisioning) has completed.
Purpose and caller: the once-only publication that permits CPU_ON
    dispatch; caller is the boot sequence.
Inputs / outputs: none; Ok or Err(PhaseViolation).
Preconditions / postconditions: precondition — the caller is the boot
    CPU and the phase is Bootstrap (verified by CAS). Postcondition —
    phase is GlobalInitPublished with release semantics; the phase gate
    is open for dispatch and secondary checks.
State and ownership change: the one transition.
Concurrency/allocation context: single CAS; no allocation; no locks.
Errors and failure guarantee: a CAS failure means the boot sequencer
    called twice — fatal invariant diagnostic (architecture §5 table),
    not a recoverable error.
Security/authorization checks: coordinator-only by contract.
Logic: CAS(Bootstrap -> GlobalInitPublished, Release); emit phase event.
Validation: W05-DV01.
```

```text
Name and stability: require_published() -> Result<(), NotPublished> —
    internal gate.
Purpose and caller: fail-closed check that global init is published;
    callers: W02 dispatch (before the first CPU_ON), secondary local-init
    start, any boot-time consumer of global structures.
Inputs / outputs: none; Ok when phase >= GlobalInitPublished.
Preconditions / postconditions: read-only; a false result means the
    caller is running out of order and must not proceed.
Concurrency/allocation context: one acquire-load.
Errors and failure guarantee: returns the refusal; the caller's contract
    (park/fail-closed) applies — the gate never blocks.
Security/authorization checks: this is the structural defense for
    P3-V05's "secondaries never use incomplete global resources".
Logic: phase.load(Acquire) >= GlobalInitPublished.
Validation: W05-DV02.
```

## 3. `signal_local_init_complete` and the ready gate

```text
Name and stability: signal_local_init_complete(logical: LogicalCpuId) —
    internal; once per CPU per boot.
Purpose and caller: records that this CPU finished local initialization
    (W04 install is complete); caller is W04's install tail only.
Inputs / outputs: the CPU's logical id (cross-checked against the
    calling CPU's installed identity — a mismatch is an invariant
    violation).
Preconditions / postconditions: precondition — the caller's W04 area is
    Installed and its W03 record is Initializing (the boot CPU also
    signals while Initializing; admission to Online happens afterwards,
    coordinator-side). Postcondition — the CPU's ready flag is set
    with release semantics; exactly once.
State and ownership change: ready flag false -> true.
Concurrency/allocation context: store-release on the flag; the
    coordinator acquire-loads. A repeated signal is detected via the
    flag's prior value and is a fatal invariant (it would mean install
    ran twice — contradicting W04's exactly-once).
Security/authorization checks: the logical id must match the calling
    CPU's installed identity; a CPU cannot signal for another.
Logic: flags[logical].swap(READY, AcqRel) != READY else fatal; emit event.
Validation: W05-DV03.
```

## 4. `coordinate_rendezvous` (coordinator)

```text
Name and stability: coordinate_rendezvous(attempted: AttemptedSet) ->
    RendezvousResult — internal; called once by the boot CPU after
    dispatching secondaries and completing its own install.
Purpose and caller: drives the boot from publication to the rendezvous
    end; caller is the boot sequence.
Inputs / outputs: the attempted set — the boot CPU plus every secondary
    W02 dispatched, i.e. every candidate that must reach ready-or-
    terminal (boot CPU included; it signals like every other CPU);
    returns per-CPU outcome (Ready / TerminalFailed) and the ready count.
Preconditions / postconditions: precondition — phase is
    GlobalInitPublished; W02 outcomes are terminal for every attempted
    CPU (bounded poll guarantees this). Postcondition — every attempted
    CPU accounted; every ready CPU admitted online via W03; result
    consistent with the registry.
State and ownership change: online admissions (via W03); nothing in the
    phase word (declaration is a separate step, §5).
Concurrency/allocation context: acquire-loads only in the wait; no
    allocation; no timer — the wait is bounded by the terminal condition
    (entry README decision 5).
Errors and failure guarantee: an admission refusal for a ready CPU is a
    fatal invariant (W03/W05 conditions disagree — the designs' contracts
    are broken); the wait cannot hang because terminals are guaranteed.
Security/authorization checks: coordinator-only.
Logic (pseudocode):

    coordinate_rendezvous(attempted):
        loop:
            all_ready     = attempted.all(|c| flags[c].load(Acquire) == READY)
            all_terminal  = attempted.all(|c|
                flags[c] == READY or registry.state_of(c).is_terminal())
            if all_ready or all_terminal: break
        for c in attempted where flags[c] == READY:
            registry.admit_online(c)?    # refusal = fatal invariant
        for c in attempted where flags[c] != READY:
            emit degraded-participant diagnostic (c, registry state)
        return RendezvousResult { ready: counted, failed: listed }

Validation: W05-DV03/DV04 host-side tests with simulated progress.
```

## 5. `declare_smp_ready` and `smp_ready_state`

```text
Name and stability: declare_smp_ready(result: RendezvousResult) ->
    SmpReadyState — internal; called once by the boot CPU after
    coordinate_rendezvous.
Purpose and caller: the once-only SMP-ready publication; caller is the
    boot sequence.
Inputs / outputs: the rendezvous result; returns the declared
    SmpReadyState (Ready or Degraded — Pending is never declared).
Preconditions / postconditions: precondition — phase is
    GlobalInitPublished and the result accounts for every attempted CPU.
    Postcondition — degraded record written; `dmb ish` full barrier;
    phase CAS to SmpReady with release semantics; result diagnostics
    emitted.
State and ownership change: the second and final phase transition.
Concurrency/allocation context: full-barrier store-release per the
    architecture §4 rule 3; the barrier publishes registry, flags,
    outcomes coherently.
Errors and failure guarantee: a CAS failure (already declared) is a
    fatal invariant.
Security/authorization checks: coordinator-only.
Logic: record degraded set; execute the documented full-barrier fence
    (`dmb ish`); CAS(GlobalInitPublished -> SmpReady); emit SMP-ready
    event with the state.
Validation: W05-DV01 (once-only), W05-DV04 (degraded correctness).
```

```text
Name and stability: SmpReadyState — enum { Pending, Ready { online },
    Degraded { online, failed: FailedSet } }; FailedSet is a fixed-
    capacity list of LogicalCpuId (≤ 8); internal.
Purpose and caller: the testable SMP-ready condition; callers: W12/W13
    assertions, boot-integration policy, diagnostics.
Preconditions / postconditions: monotonic Pending -> (Ready|Degraded);
    `online` equals W03's admitted count at declaration; `failed` is
    derivable from the registry at any later time.
State and ownership change: written once at declaration; read-only after.
Concurrency/allocation context: acquire-load for readers; the fence of
    §5 makes the whole boot state coherent with it.
Errors and failure guarantee: n/a.
Logic: stored value + registry-derived consistency check available for
    diagnostics.
Validation: W05-DV04; W05-DV06 captures.
```

## 6. Diagnostic events

```text
Name and stability: phase-transition and rendezvous events — content per
    the P0 trace governance; catalog registration owned by P3-W11.
Purpose and caller: make every boot's ordering observable (P3-V05
    repeatability evidence); callers: the transitions above and the
    coordinator.
Inputs / outputs: phase from->to; ready count; degraded set; optional
    counter-based latency observation (informative only; the counter
    read is a diagnostic, not a timing authority — P6/W11 own real
    timing).
Validation: W05-DV06 capture review.
```
