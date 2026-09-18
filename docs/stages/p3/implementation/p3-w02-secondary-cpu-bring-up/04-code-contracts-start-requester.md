# P3-W02 Code Contracts — Start Requester

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W02 detailed design](README.md).

Contracts follow the project function/type template. The requester side
runs only on the boot CPU during the P3 bring-up phase. Names are
design-level identifiers; concrete Rust paths are reserved to the
workspace-owning design.

## 1. `StartPhase`

```text
Name and stability: StartPhase — enum { RequestDispatch, EnteredEl2,
    EntryStateValidated, IdentityConfirmed, LocalEnvironmentReady };
    internal; shared vocabulary with the entry path.
Purpose and caller: phase attribution for P3-V02; produced by both sides
    (RequestDispatch by the requester, the rest by the secondary).
Preconditions / postconditions: phases are ordered; a result records the
    last completed phase, never a set.
Logic: plain enum, ordered as listed.
Validation: exhaustive-match review; DV04 unit tests name phases in
    failure results.
```

## 2. `psci_cpu_on`

```text
Name and stability: psci_cpu_on(target_mpidr: MpidrValue, entry_addr:
    EntryPointAddress, context_id: u64) -> Result<(), PsciCallError> —
    internal; the package's single `unsafe` firmware-call boundary.
Purpose and caller: issues the PSCI CPU_ON call; caller is the bring-up
    sequencer (§5) only.
Inputs / outputs: target identity as `MpidrValue` (W01 type — the start
    request is MPIDR-typed by guardrail); entry address of
    `secondary_entry`; context id fixed to the target's `LogicalCpuId`
    (u64 representation). Returns the PSCI-denial mapping of §3.
Preconditions / postconditions: precondition — conduit and CPU_ON function
    id are present in `StartCapabilityFacts` (checked by the caller,
    §5); the boot CPU executes under the P1 runtime contract. The call is
    issued at most once per target per boot.
State and ownership change: none in hypervisor state; firmware-side CPU
    state changes are the PSCI contract's domain.
Concurrency/allocation context: boot CPU only; no allocation; no locks.
    No interrupt enabling is required or performed by this function.
Errors and failure guarantee: on error the target CPU state is whatever
    the firmware reports (denied/invalid/unsupported); no hypervisor state
    was mutated, so the caller's only duty is to report and never
    re-dispatch. The function has no retry loop and no timeout (the
    watcher owns waiting).
Security/authorization checks: the conduit (HVC vs SMC) and function id
    come exclusively from recorded P2 facts; no literal identifiers or
    platform names may appear (ADR-044/ADR-052). The instruction sequence
    is the audited `unsafe` block with a `SAFETY` comment covering:
    register clobber list, exception-level/conduit contract inherited from
    the P1 baseline, and the assumption that the firmware honors the PSCI
    calling convention recorded by P2.
Logic (pseudocode):

    psci_cpu_on(target, entry, ctx):
        fid = start_capability.cpu_on_function_id   # checked by caller
        match start_capability.conduit:
            Hvc -> x0=fid; x1=target.raw(); x2=entry; x3=ctx; `hvc #0`
            Smc -> x0=fid; x1=target.raw(); x2=entry; x3=ctx; `smc #0`
        return map_psci_status(x0)

Validation: DV01 review (single boundary, no platform constants); DV02
    return-mapping unit tests; DV06/DV07 QEMU evidence.
```

## 3. `PsciCallError`

```text
Name and stability: enum { NotSupported, InvalidParameters, Denied,
    AlreadyOn, UnrecognizedStatus(i32) } — internal mapping of the PSCI
    return codes recorded by P2's PSCI facts (PSCI 0.2/1.x status set).
Purpose and caller: makes firmware denials nameable in diagnostics and
    outcomes; caller is §2 and the outcome mapper.
Preconditions / postconditions: exhaustive over the recorded status set;
    unknown values land in UnrecognizedStatus and are treated as failures.
Errors and failure guarantee: n/a (this is the error type).
Security/authorization checks: values are compared against the recorded
    PSCI version's status definitions, not magic numbers inlined at call
    sites.
Validation: DV02 table tests over every recorded status value.
```

## 4. `SecondaryStartOutcome`

```text
Name and stability: enum SecondaryStartOutcome { NotAttempted,
    RequestRejected(PsciCallError), TimedOut, FailedBySecondary { phase:
    StartPhase, cause: FailureCause }, Started } — internal; terminal
    per-CPU result.
Purpose and caller: the requester's report to W03 (failure transitions),
    W05 (attempted/terminal accounting), W11 (event content), and W13
    (regression assertions).
Preconditions / postconditions: exactly one terminal outcome per candidate
    CPU per boot; `NotAttempted` applies only to Present-class CPUs the
    sequencer never dispatched (possible only if bring-up aborts early —
    a state P3 avoids by failing closed before dispatch).
State and ownership change: recorded once in the outcome map; never
    mutated after recording.
Concurrency/allocation context: produced on the boot CPU; map is
    published like other boot-phase data.
Logic: mapping rules — call error → RequestRejected; no arrival within
    bound → TimedOut; arrival with failure → FailedBySecondary (secondary
    phase wins over requester guesses); arrival with success → Started.
Validation: DV04 exhaustive mapping tests.
```

## 5. `bring_up_secondaries`

```text
Name and stability: bring_up_secondaries(topology: &TopologyInputs) ->
    SecondaryStartMap — internal; called once per boot from the boot
    sequence after global initialization is published.
Purpose and caller: the ordered bring-up of every Present-class non-boot
    CPU; caller is the boot sequencing owner under the P3-W05 phase
    contract.
Inputs / outputs: the frozen W01 topology; returns the per-CPU outcome
    map (dense, logical-id ordered).
Preconditions / postconditions: precondition — boot phase is
    GlobalInitPublished (W05's gate; asserted, not implemented here); all
    provisional stacks and parameter blocks are allocated and populated;
    W03 registry is seeded and functional. Postcondition — every
    candidate CPU has a terminal outcome; every non-boot Present CPU was
    dispatched exactly once; no CPU is left in a non-terminal,
    non-registry state.
State and ownership change: requests W03 transitions (Present→Starting by
    request, Starting→Failed on terminal failure); on success leaves the
    CPU in Starting for the secondary's own Initializing transition;
    transfers stack ownership per the environment contract.
Concurrency/allocation context: boot CPU only; allocation is complete
    before entry (none inside the loop); the poll phase uses
    acquire-loads only.
Errors and failure guarantee: never aborts the loop on an individual
    failure — a failed candidate is terminal and bring-up continues with
    the remaining candidates, because per-CPU isolation (not
    all-or-nothing) is the diagnosability requirement. A systemic failure
    (missing start capability, phase-gate violation) fails closed before
    any dispatch.
Security/authorization checks: candidate set = topology Present-class,
    excluding boot — no runtime notion of "eligible" beyond W01's
    classification (eligibility authority is W03's).
Logic (pseudocode):

    bring_up_secondaries(topology):
        assert boot_gate.require_published().is_ok()  # W05 phase gate
        if !start_capability.usable(): fail closed (StartUnavailable)
        outcomes = {}
        for cpu in topology.present_non_boot_in_logical_order():
            stack = take_preallocated_stack(cpu.logical)
            lifecycle.request_start(cpu.logical)          # W03: ->Starting
            r = psci_cpu_on(cpu.hardware, secondary_entry, cpu.logical)
            match r:
                Err(e) -> outcomes[cpu] = RequestRejected(e)
                          lifecycle.report_failure(cpu, RequestDispatch, e)
                Ok(_)  -> armed.insert(cpu)
        # poll phase: bounded, per armed CPU
        remaining_bound = POLL_BOUND
        while armed not empty and remaining_bound > 0:
            for cpu in armed snapshot:
                if mailbox[cpu].load(Acquire) == ARRIVED:
                    record outcome from result record          # §4
                    armed.remove(cpu)
            remaining_bound -= 1
        for cpu still armed: outcomes[cpu] = TimedOut
            lifecycle.report_failure(cpu, Timeout, NoArrival)
        emit per-CPU bring-up diagnostics (P0 governance)
        return outcomes

Validation: DV02 (sequencer unit tests with a mocked call boundary and
    fake mailboxes: error, success, late-failure, timeout paths), DV04,
    DV06/DV07 (QEMU).
```

## 6. Timeout watcher rule

```text
Name and stability: POLL_BOUND — stage-local constant (value fixed in the
    implementation record with its rationale); applies to the requester's
    poll pass per boot.
Purpose and caller: bounds the requester's wait; caller is §5's poll loop.
Semantics and limitation: "no arrival within POLL_BOUND acquisitions" — a
    scheduling/execution bound, not a wall-clock interval. Recorded
    limitation: a real timeout measurement requires the P6 timer baseline
    (Reserved). The value must be large enough that a healthy reference
    platform arrives with wide margin (verified by DV06) and small enough
    that a failed boot still terminates.
Failure guarantee: after the bound, the outcome is terminal; no path
    waits forever.
Validation: DV05 watcher unit tests (deterministic fake progress); DV06
    margin evidence on QEMU.
```

## 7. Bring-up diagnostics

```text
Name and stability: per-CPU bring-up event emission — content and level per
    the P0 logging/trace governance; event catalog ownership is P3-W11's.
Purpose and caller: makes every terminal outcome observable with CPU,
    phase, and cause; caller is §5's tail and §3 of the entry file (parked
    failures, via the P1-W07 fatal path).
Preconditions / postconditions: every candidate CPU yields at least one
    diagnostic line; request-phase and timeout failures are emitted by the
    requester, secondary-phase failures by the secondary (before parking).
Validation: DV06/DV07 capture review: diagnostics present for success,
    rejected-request, and (host-side) timeout paths.
```
