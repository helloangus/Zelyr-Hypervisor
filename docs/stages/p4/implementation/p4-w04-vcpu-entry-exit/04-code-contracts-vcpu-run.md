# P4-W04 Code Contracts — vCPU Object and Run Control

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W04 detailed design](README.md).  
**Companion:** state machine in
[02-architecture-and-state.md](02-architecture-and-state.md) §3; switch
boundary in [03-code-contracts-world-switch.md](03-code-contracts-world-switch.md).

All names are P4-internal and unstable-by-declaration; the P4 exit classes
are temporary vocabulary, not the final `ExitReason` API. Pseudocode is an
outline; the Coding Guidelines govern final Rust shape.

## 1. Types

### 1.1 `VcpuContext` (module `vcpu-ctx`)

- **Name and stability:** `VcpuContext { gp: [u64; 31], sp: u64, pc: u64,
  pstate: VcpuPstate }` with `VcpuPstate` a structured value (EL fixed EL1h,
  DAIF masked at construction, flags zero). Internal.
- **Purpose and caller:** the authoritative saved Guest state; produced by
  construction (§2.1) and updated from exit frames (§3.2).
- **Contract notes:** the type cannot represent non-EL1 execution
  (constructor enforces mode/DAIF fields); conversions from
  exit-frame captures validate the captured PSTATE before acceptance — a
  Guest cannot manufacture a context the type rejects (untrusted-capture
  discipline: validate syndrome-adjacent fields before they become state).

### 1.2 `RunState` (module `vcpu-obj`)

- **Name and stability:** `RunState` — `Ready | Running | Stopped(StopCause)`.
  Internal; P4 subset of the ADR vCPU lifecycle
  ([02 §3](02-architecture-and-state.md) records the mapping for W09/P7).
- **Contract notes:** explicit enum; no boolean phases; transitions only via
  §3/§4 operations, each emitting its event.

### 1.3 `ExitClass` and `ExitAction` (module `vcpu-run`)

- **Name and stability:** `ExitClass` — `Stage2Translation`,
  `Stage2Permission`, `Wfi`, `Wfe`, `UnknownSync(EsrValue)`,
  `IllegalExecution`, `BudgetExhausted`. `ExitAction` — `Reenter` |
  `Stop(StopCause)`. `StopCause` — `Controlled | GuestFault(ExitClass) |
  BudgetExhausted`. Internal, P4-temporary.
- **Purpose and caller:** minimal vocabulary for W04's run decisions; W06
  extends toward full classification/diagnostics; W07 counts; W08 matches
  markers. Not a frozen ABI (README excluded interfaces).
- **Contract notes:** decoding from raw ESR values lives here only as far as
  W04's action decision needs (class assignment); full qualification decode,
  IPA extraction, and diagnostic context are W06's consumption of the frame.

## 2. Construction and teardown contracts

### 2.1 `Vcpu::construct`

- **Name and stability:** `fn construct(input: GuestInput, scenario:
  ScenarioId) -> Result<Vcpu, VcpuError>`. Internal.
- **Purpose and caller:** reproducible single-vCPU initial state (P4-C01,
  P4-C03); called by the P4 minimal VM setup.
- **Inputs/outputs:** W03's validated `GuestInput` (entry IPA, stack top,
  boot-info location) and a scenario id validated against the W05-owned
  scenario table; returns the vCPU in `Ready` or an error.
- **Preconditions:** scenario id valid (unknown ids are rejected here — the
  untrusted-configuration discipline applied to host-authored input); entry
  and stack values inside the Guest RAM bounds per the layout (validated
  again here against the `GuestInput` invariants — defense in depth, not
  trust).
- **Postconditions:** context = pure function of inputs (zeroed GP except
  the scenario register; PC = entry; SP = stack top; PSTATE = EL1h/DAIF
  masked); state `Ready`; budget counter initialized.
- **State/ownership:** new object; owns its context copy.
- **Concurrency:** no shared state touched; callable during setup only.
- **Errors:** `InvalidScenario`, `EntryOutOfRange`, `StackOutOfRange` — all
  VM-facing recoverable setup failures.
- **Security:** no firmware/QEMU residue enters the state; the Guest cannot
  influence any input before its first instruction.
- **Logic:**

```text
construct(input, scenario):
    scenario_table.validate(scenario)?           // W05-owned table
    check input.entry within guest ram bounds?   // re-validation
    check input.stack_top within bounds?
    ctx = VcpuContext { gp: zeroed with gp[0] = scenario.id(),
                        sp: input.stack_top, pc: input.entry,
                        pstate: EL1h | DAIF_MASKED }
    return Vcpu { ctx, state: Ready, budget: EXIT_BUDGET }
```

- **Validation:** pure-function determinism tests (same inputs → identical
  context bytes); rejection tests; on-target PC/SP correctness via W05
  marker evidence (P4-V04).

### 2.2 `Vcpu::destroy`

- **Name and stability:** `fn destroy(self) -> Result<(), VcpuError>`
  (consuming). Internal.
- **Purpose and caller:** terminal teardown (P4 stop path completion);
  sequencing coordinator calls it after `Stopped`.
- **Preconditions:** state is `Stopped` (destroy from any other state is a
  type/state error, not a runtime surprise); Guest not executing.
- **Postconditions:** object consumed; last exit frame remains in the
  per-pCPU frame for post-mortem reading until overwritten by a later
  episode (documented retention).
- **Errors:** none expected; misuse escalates as invariant.
- **Validation:** lifecycle test with W07's same-session restart (P4-V10
  consumption).

## 3. Run-control contracts (module `vcpu-run`)

### 3.1 `run_entry`

- **Name and stability:** `fn run_entry(vcpu: &mut Vcpu, space: &mut
  GuestAddressSpace) -> Result<EnterOutcome, VcpuError>`. Internal.
- **Purpose and caller:** the guarded transition `Ready → Running → (exit)
  → action`; called by the P4 minimal VM setup / W07 repeat driver.
- **Inputs/outputs:** the vCPU and its Stage-2 space; returns the outcome of
  the first exit episode (`Exited(ExitInfo)` or `Stopped(cause)`).
- **Preconditions:** vCPU `Ready` (or re-entering from a saved state per
  §3.3); space constructed; not running elsewhere; exit budget > 0 for
  re-entry; setup-phase context (no locks held that the exit handler needs).
- **Postconditions:** on return, EL2 control intact; vCPU state updated
  (`Running` transiently, then per action); exit frame preserved for
  diagnostics.
- **Concurrency:** single Guest path; `Running` exclusivity asserted (fatal
  on violation).
- **Errors:** setup-class errors only; Guest-caused conditions are outcomes,
  not errors of this function (W01 A2).
- **Logic:**

```text
run_entry(vcpu, space):
    assert vcpu.state == Ready else InvariantViolation
    space.activate(current_pcpu())?                // W02 §3.6
    emit vcpu.enter
    vcpu.state = Running
    loop:                                          // bounded by budget (D7/D8)
        ws_guest_enter(host_area(), &vcpu.ctx)     // [03 §3]; no return
        // — control resumes here via the exit stub —
        frame = take_exit_frame()                  // [03 §4]
        info = classify_minimal(frame)             // §3.2
        emit vcpu.exit(info.class, info.guest_pc)
        action = decide(info)                      // §3.2 policy
        match action:
            Stop(cause) => { vcpu.state = Stopped(cause); return Stopped(cause) }
            Reenter      => { budget--; if budget == 0 { stop(BudgetExhausted) }
                              else { vcpu.ctx = context_from(frame); continue } }
```

- **Validation:** P4-V04 (entry, classified exit, re-entry, stop); budget
  exhaustion test; fatal-invariant test for concurrent-entry misuse.

### 3.2 `classify_minimal` and `decide`

- **Name and stability:** `fn classify_minimal(frame: &GuestExitFrame) ->
  ExitInfo` and `fn decide(info: &ExitInfo) -> ExitAction`. Internal.
- **Purpose:** assign the P4 class and the P4 action; the minimum W04 needs;
  the *full* classification/diagnostic surface belongs to
  [P4-W06](../p4-w06-fault-isolation-diagnostics/README.md) (which consumes
  the same frame).
- **Decoding rules (E-class → P4 class):**

| Observed condition (frame fields) | P4 class |
|---|---|
| Stage-2 instruction/data abort syndrome with translation-fault status | `Stage2Translation` |
| Stage-2 abort syndrome with permission-fault status | `Stage2Permission` |
| WFI-trap syndrome | `Wfi` |
| WFE-trap syndrome | `Wfe` |
| Illegal/undefined-execution syndrome | `IllegalExecution` |
| Any other synchronous syndrome | `UnknownSync(ESR)` (kept raw for W06) |

- **Action policy (D7):** WFI/WFE → `Stop(Controlled)` (the P4-defined
  diagnosable result for P4-V09); `Stage2Translation`/`Stage2Permission`/
  `IllegalExecution` from W05's fault scenarios → `Stop(GuestFault(class))`;
  `UnknownSync` → `Stop(GuestFault(UnknownSync))`; budget exhaustion →
  `Stop(BudgetExhausted)`. `Reenter` exists for the re-entry proof
  ([plan P4-E04](../../plans/p4-w04-vcpu-entry-exit.md)) and is exercised by
  the controlled re-entry scenario (W05's planned VG-009 class), not as
  general policy.
- **Security:** classification consumes only frame data; raw ESR values are
  data, never executed or used as host addresses; unknown values fail safe
  to `UnknownSync` + stop (fail-closed), never to silent re-entry.
- **Validation:** table-driven unit tests over synthesized frame values
  covering every row (host-side); on-target agreement via W05/W06 scenarios.

### 3.3 Re-entry

- **Name and stability:** re-entry is `run_entry` continuing its loop with
  `vcpu.ctx = context_from(frame)` — no separate API; recorded as the
  P4-E04 proof path.
- **Contract notes:** the saved-context update validates the captured
  PSTATE (EL1, DAIF masked as constructed) before it becomes `VcpuContext`
  state; a capture that fails validation is a Guest-caused containment event
  → `Stop(GuestFault(...))`, never a corrupted resume.

## 4. Stop and teardown sequencing contract

- **Name and stability:** the P4 stop path = `Stop(cause)` → `Stopped` state
  → diagnostics retained → coordinated teardown: `vcpu.destroy()` →
  `space.destroy()` (W02 §3.8) → `GuestRam::release()` (W03 §3.4).
- **Purpose and caller:** P4-E05 defined stop; the ordering is the W02/W03
  sequencing duties applied from this side; the setup/teardown coordinator
  (W07 repeat driver, W09 record) executes it.
- **Preconditions:** vCPU not executing; no further `run_entry` permitted
  after `Stopped` (state machine enforcement).
- **Postconditions:** Host memory ownership fully restored; EL2 live and
  diagnosable (console, allocator, logging all functional — the repeat
  scenario's precondition).
- **Errors:** teardown errors surface in order with explicit leak reporting
  (never swallowed); a teardown failure stops the repeat driver — it never
  leaves a half-torn Guest for the next iteration.
- **Validation:** P4-V04 stop evidence; P4-V10 same-session restart (with
  W07); accounting restoration checks (with W03 DV07).

## 5. Error model recap

`VcpuError`: `InvalidScenario`, `EntryOutOfRange`, `StackOutOfRange`,
`NotReady`, `AlreadyRunning` (misuse class → fatal-invariant escalation),
`TeardownOrderViolation`. All Guest-caused conditions arrive as outcomes
(`ExitInfo`/`StopCause`), never as errors — the type split itself documents
the W01 A2 boundary.
