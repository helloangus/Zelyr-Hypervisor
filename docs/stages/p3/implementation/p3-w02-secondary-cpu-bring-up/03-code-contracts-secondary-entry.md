# P3-W02 Code Contracts — Secondary Entry Path

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W02 detailed design](README.md).

Contracts follow the project function/type template. Names are design-level
identifiers; concrete Rust paths and the entry symbol's final link address
are reserved to the approved build/layout design. This file covers the
secondary side only; the requester is in
[04-code-contracts-start-requester.md](04-code-contracts-start-requester.md).

## 1. Entry parameters and positioning

```text
Name and stability: secondary_entry — assembly entry symbol, referenced by
    address from the requester's CPU_ON call; internal to the hypervisor
    image; stability: fixed for the stage, renaming is a reviewed change.
Purpose and caller: the instruction address handed to PSCI CPU_ON; entered
    by a started secondary under the conditions PSCI and the platform
    firmware define.
Inputs / outputs: on entry the architecture-defined CPU_ON entry state
    holds; x0/w0 carries the context id the requester chose, which this
    design fixes to the target's LogicalCpuId (decision rationale in the
    requester file §3). No value is returned; the function never returns
    into firmware on any path.
Preconditions / postconditions: the CPU executes in the exception level and
    security state PSCI defines for the platform (expected: Non-secure
    EL2, matching the P1 boot contract); nothing else may be assumed about
    register contents beyond the PSCI-specified x0.
State and ownership change: switches the CPU from firmware context to a
    provisional W02 environment, then into W03's Initializing state, then
    (on success) hands over to the W05 readiness gate.
Concurrency/allocation context: exactly one CPU executes this per start;
    the stack it installs is reserved for it; no allocation happens on the
    secondary in W02's scope.
Errors and failure guarantee: any failed check parks the CPU forever in a
    local loop after recording its failure; a parked CPU writes no further
    shared state and is never a bring-up candidate again in P3.
Security/authorization checks: the entry symbol address is an internal
    constant; a CPU that arrives without a corresponding request is still
    safely parked by the identity/registration checks (the mailbox it
    would write is only written after W03 accepts Initializing).
Logic: see §2.
Validation: W02-DV03 (identity/entry revalidation behavior at the QEMU
    level); reviewed against the P1 entry contract line by line.
```

## 2. `secondary_entry` logic (pseudocode, assembly + Rust boundary)

```text
secondary_entry:                      # assembly
    load sp, [entry_params + current_slot_stack_top]
        # entry_params is a per-CPU parameter block the requester filled
        # before CPU_ON; located by the fixed layout of the block base +
        # logical id (dense ids per W01 make this an index, not a search)
    b secondary_entry_rust            # never returns

secondary_entry_rust(logical_from_context):
    phase = EnteredEl2
    if !entry_state_matches_p1_contract():        # CurrentEL, security state
        return park_failed(phase, EntryStateMismatch)
    phase = EntryStateValidated
    mpidr = MpidrValue::read_current()            # unsafe: arch boundary
    hw = mpidr.to_hardware_id()
    logical = topology.logical_of(hw)             # Option; None = unknown CPU
    if logical.is_none() || logical != logical_from_context:
        return park_failed(phase=IdentityConfirmed, IdentityMismatch)
    phase = IdentityConfirmed
    # W03 transition call: this CPU becomes Initializing
    if !lifecycle.enter_initializing(logical):    # exactly-once inside W03
        return park_failed(phase, AdmissionConflict)
    apply_p1_per_cpu_baseline()                   # bounded, per P1-W04 contract
    phase = LocalEnvironmentReady
    report_arrival(logical, Success{stack_token}) # release-store mailbox
    boot_gate.signal_local_init_complete(logical) # W05 contract call
    enter_secondary_idle()                        # P3 stable per-CPU idle
```

`entry_state_matches_p1_contract` checks the exact items the approved P1
entry contract requires and nothing more; extending the checklist is a
P1-contract change, not a local edit.

## 3. Identity confirmation rule (binding)

- The authoritative identity is the CPU's own `MPIDR_EL1` read through
  `MpidrValue` (W01 type); the context id is supporting evidence only.
- The cross-check fails closed: unknown identity, context-id mismatch, or a
  topology lookup that returns a non-candidate all park the CPU with phase
  `IdentityConfirmed` recorded. A CPU that cannot prove which CPU it is
  must not write to any per-CPU structure.
- The mailbox write is authorized by W03's accepted
  `enter_initializing(logical)` transition; if that transition is refused
  (duplicate, unknown, or non-candidate), the CPU parks without writing
  the mailbox — the requester's timeout then reports the anomaly with CPU
  attribution from the registry side.

## 4. Provisional environment contract

```text
Name and stability: entry_params per-CPU parameter block and provisional
    stack — internal scaffolding; layout fixed by this design; replaced
    (or retained) by the P3-W04 design on success.
Purpose and caller: gives a starting secondary a valid stack and its
    per-CPU parameters before any W04 runtime exists.
Inputs / outputs: block fields — stack_top (physical/VirtualAddr per the
    approved build design), logical id, mailbox/result addresses (or the
    fixed offsets from a bring-up base).
Preconditions / postconditions: allocated and populated by the requester
    before CPU_ON; page-aligned per the P2-W04 allocation contract;
    constant size `PROVISIONAL_STACK_SIZE` (stage-local constant, bounded
    to entry validation + local init; recorded value with rationale in the
    implementation record; not a runtime-service stack).
State and ownership change: boot CPU owns until CPU_ON success; on
    reported success ownership transfers to the W04 runtime design; on
    failure or timeout the allocation is quarantined (never freed, never
    reused in P3).
Concurrency/allocation context: allocated single-threaded pre-release;
    used by exactly one CPU afterwards.
Errors and failure guarantee: allocation failure is fatal boot-critical
    (P0 panic policy); no partial bring-up proceeds.
Security/authorization checks: n/a (internal scaffolding, no guest reach).
Logic: allocation loop per candidate in requester order; block population
    immediately after allocation so a CPU_ON is never issued against an
    uninitialized block.
Validation: W02-DV01 ownership review; stack distinctness asserted by
    W02-DV06 capture (distinct stack addresses per CPU in diagnostics).
```

## 5. `report_arrival` and `park_failed`

```text
Name and stability: report_arrival(logical: LogicalCpuId, result:
    SecondaryArrivalResult) — internal; single call site (entry success
    tail).
Purpose and caller: publishes the secondary's terminal result to the
    requester; caller is secondary_entry_rust only.
Inputs / outputs: writes the per-CPU result record (phase reached, cause
    if failure, stack token if success), then Release-stores the mailbox
    sentinel value.
Preconditions / postconditions: called at most once per CPU per boot
    (enforced by reaching it only after W03's exactly-once transition).
    After the store, the secondary owns nothing shared.
State and ownership change: mailbox: NotArrived -> Arrived; the result
    record becomes requester-readable.
Concurrency/allocation context: Release store pairs with the requester's
    Acquire load (architecture §4); no other synchronization exists or is
    permitted.
Errors and failure guarantee: none (cannot fail).
Security/authorization checks: authorized by the W03 transition (§3).
Logic: store result record fields; `AtomicUsize::store(ARRIVED, Release)`.
Validation: W02-DV04 unit tests on the protocol ordering; QEMU arrival
    evidence in W02-DV06.
```

```text
Name and stability: park_failed(phase: StartPhase, cause: FailureCause) —
    internal; diverges (never returns).
Purpose and caller: the terminal failure path of the entry stub; caller is
    any failed check in §2.
Inputs / outputs: attempts (best effort, ordered) — W03 report-failure
    transition request (if identity is confirmed), a CPU-attributed fatal
    diagnostic through the P1-W07 path if the console is usable, then a
    local infinite parked loop (WFE Reserved for P3-W07 integration).
Preconditions / postconditions: postcondition — the CPU writes no shared
    state after parking and is permanently outside every online
    computation.
Errors and failure guarantee: this is the failure guarantee.
Security/authorization checks: a CPU that failed before identity
    confirmation does not write the registry; only its own park state.
Logic: ordered attempts as above; loop { } tail.
Validation: W02-DV07 (absent-CPU rejection path exercised from the
    requester side); review that every check in §2 fails into this path.
```
