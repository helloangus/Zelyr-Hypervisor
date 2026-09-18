# P4-W04 Scope, Foundations, and Resolved Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W04 detailed design](README.md).

## 1. Scope classification detail

### Required (P4-C01–C04, P4-E01, E03–E05)

- vCPU initial-state construction: pure function of (GuestInput from W03,
  validated scenario id) producing Guest PC/SP/PSTATE/registers.
- Guest entry by real `ERET` to EL1 (EL1h, DAIF masked), after Stage-2
  activation (W02) and EL1 baseline restore.
- Guest-exit capture: any exception from Guest EL1 lands in the Guest-exit
  stub; guest context saved; EL2 state restored; Rust exit handler runs with
  EL2 control intact.
- Re-entry: resume from saved context (at least one successful re-entry).
- Defined stop: terminal vCPU state with named causes and preserved
  diagnostics.
- P4-local exit-class vocabulary sufficient for W06 to extend: Stage-2
  translation, Stage-2 permission, WFI, WFE, unknown-synchronous,
  illegal-execution, budget-exhausted.

### Reserved (must not be precluded; not implemented in P4)

- Interruptible Guest execution and injected virtual interrupts (re-entry:
  P6; the DAIF-masked run segment is a recorded temporary policy).
- Full EL1 system-register save/restore and Guest-visible EL1 persistence
  across exits (re-entry: first Guest that legitimately needs it — P6 timer,
  P8 Linux).
- Multi-vCPU and cross-vCPU world-switch reuse (re-entry: P6+/P7; the
  per-pCPU frame design is already object-based).
- Blocked/Paused vCPU states and scheduler integration (re-entry: P7).

### Out of Scope

Scheduler policy and run queues, vGIC/timer virtualization, PSCI/hypercall
ABI (P5/P8), the final generic `ExitReason` API, VM lifecycle beyond the P4
stop/teardown used by W07/W09, Guest SMP, performance tuning of the switch,
and any QEMU-conditional behavior.

## 2. Assumed upstream contracts and failure boundaries

Per the [P4-W01](../p4-w01-entry-contract-reconciliation/README.md) entry
review; divergence between assumption and delivery becomes a recorded
conflict (W01 §4) and the affected W04 step stops.

| ID | Assumed contract | Source | Relied-on behavior | Failure boundary if delivered differently |
|---|---|---|---|---|
| M1 | Typed identifiers/addresses; error classification split (GuestFault vs InvariantViolation) | P0-W15/W14; W01 R19 | exit classes and error values typed; guest-caused failures are values | absent → W04 blocks (Coding-Guidelines shape) |
| M2 | Stable Non-secure EL2 runtime with stack, panic route, stable idle; ordered init lifecycle | P1-W02/W09; W01 R01/R05 | entry runs from the stable state; stop path returns there or to a defined halted state | unstable host state → P4-V04 evidence unsound; stop at mutation-level work |
| M3 | EL2 vector table with synchronous/IRQ/FIQ/SError entries and bounded context capture; crash diagnostics non-recursive | P1-W05/W07; W01 R02 | Guest exits are taken at EL2 through an extensible path; fatal host faults stay diagnosable | no extensible vector path → Guest-exit stub cannot be built; record blocker |
| M4 | Known EL2 baseline: routing/trap controls host-owned; EL1/EL0-preparation state defined; timer access baseline | P1-W04; W01 R03 | entry protocol programs only deltas over the baseline; EL1 baseline restore values exist | baseline undefined → D5 strategy has no source; block and reconcile |
| M5 | CPU-local mechanism: per-pCPU context discoverable (identity, local storage), current-vCPU reservation capacity | P3-W04/W14; W01 R14 | exit stub locates host state without trusting any Guest-clobbered register | discovery mechanism absent/different → adapt the stub to the delivered mechanism (seam), record change |
| M6 | Synchronization primitives (spin lock, IRQ masking rules) with documented semantics | P3-W06; W01 R15 | run-state transitions and stop path use them | absent → block; do not invent parallel primitives |
| M7 | Stage-2 activation/invalidation contracts | [P4-W02](../p4-w02-stage2-address-space/README.md) | entry calls activation; exit does not mutate Stage-2 | contract drift → joint design note (both designs), not local patching |
| M8 | GuestInput (entry IPA, stack top, boot info, validated scenario id) from W03; console page convention with EL2 console-quiet duty | [P4-W03](../p4-w03-guest-memory-image/README.md) | initial state is a pure function of these; console handoff honored during the run segment | input drift → joint design note; never informal parameters |
| M9 | Logging/trace baseline + event namespace; QEMU reference environment with automation entry | P0-W12/W13; P1-W10; W01 R06/R20 | enter/exit/stop events emitted; evidence runs on the reference environment | degrade per logging boundary; automation boundary is W08's |

Entry-order boundary: steps through exit-stub unit-level development can
proceed against assumed signatures, but P4-V04-class on-target evidence
requires M2/M3/M4 delivered; the workflow marks the gate.

## 3. Authority analysis for contested areas

- **Interrupts during Guest execution:** neither the ADR nor the task book
  mandates a delivery mechanism in P4 (vGIC/timer are P6; physical IRQ
  dispatch is not required before P6). Masking host interrupts for the Guest
  run segment is the smallest policy that keeps the world-switch sound
  (no re-entrant switch, no IRQ-during-save race). It is recorded as a
  temporary limitation feeding W07/W09, with the Reserved re-entry point.
- **EL1 register persistence:** full save/restore of Guest-modified EL1
  state is real virtualization machinery whose scope belongs with timer/vGIC
  consumers (P6/P8). Restoring a fixed baseline per entry contains an
  untrusted Guest safely (it cannot persist EL1 MMU/VBAR changes across
  exits) and matches the P4 Guest's needs; the limitation is explicit and
  re-owned later.
- **WFI/WFE result:** P4 defines the result as "classified exit + defined
  action" (`Stop` for the P4 scenarios; `Reenter` available for future use)
  — a scheduler blocking policy (P7) would be out of scope here. This keeps
  P4-V09's "defined diagnosable result" honest.
- **HCR_EL2 and trap bit intents:** the design fixes the *required routing
  outcomes* (table in [02 §4.3](02-architecture-and-state.md)); exact bit
  encodings are verified against the pinned architecture revision during
  implementation, and QEMU divergences are Specification Investigation items
  (W01 A7), never silently-adopted semantics.

## 4. Resolved design decisions

| ID | Decision | Rationale | Authority basis |
|---|---|---|---|
| D1 | Initial vCPU state = pure function `initial_state(GuestInput, ScenarioId) -> VcpuContext`; zeroed GP registers except the scenario register; PC = entry IPA; SP = stack top; PSTATE = EL1h with DAIF masked | P4-C03 reproducibility; no firmware residue; deterministic repeat evidence | task book P4-C03; W03 GuestInput contract |
| D2 | One vCPU object owns: arch context (saved on exits), run state, exit budget counter, identity; the per-pCPU world-switch frame is separate pCPU-owned storage | keeps vCPU schedulable-shaped (P7-ready) and pCPU state pCPU-owned (W01 A3); no God object | Plan Agent guardrails; ADR §4/§5 |
| D3 | Entry protocol is fixed-order: validate run preconditions → W02 activation → restore EL1 baseline → restore Guest context → save host PSTATE (masked) → switch stacks → `ERET`. Exit is the mirror via exception entry. The Guest can never fall through into EL2 flow: EL2 PC lives only in host-owned storage and Guest mode (EL1h) cannot address it | containment by construction (ADR §19); reviewable symmetric protocol | ADR-022; ADR §19 |
| D4 | Host interrupts masked across the whole Guest run segment; WFI/WFE trap to EL2 (routing intents: WFI trap enabled, WFE trap enabled, Guest HVC disabled/undefined, SMC trapped); recorded limitation with Reserved re-entry (P6) | smallest sound policy without vGIC/timer; deterministic evidence | task book scope; ADR §7 |
| D5 | EL1 architectural baseline (SCTLR_EL1, TCR_EL1, TTBR0/1_EL1, VBAR_EL1, CPACR_EL1-access policy, SPSR_EL1/ELR_EL1, DAIF) restored from P1-declared values on every entry; Guest EL1-state modifications do not survive an exit | containment of untrusted Guest state manipulation with bounded complexity; persistence is Reserved | P1-W04 baseline (M4); ADR-007 |
| D6 | Exit frame = fixed-layout per-pCPU record (guest GP x0–x30, SP_EL1, PC from ELR_EL2, PSTATE from SPSR_EL2, syndrome set from ESR_EL2, fault address registers as applicable) written entirely by the exit stub before any Rust runs | syndrome capture must precede anything that could fault in EL2; W06 needs the frame verbatim | P1-W05 capture discipline (M3); W06 consumption |
| D7 | P4 exit actions: `Reenter` (resume saved context) or `Stop(cause)`; a bounded consecutive-exit budget (D8) forces `Stop(BudgetExhausted)`; WFI/WFE scenarios use `Stop` as their defined result | P4-E05 defined stop; runaway-Guest guard for automated runs; scheduler blocking is P7 | task book P4-E05; P4-V09 |
| D8 | Exit budget is a fixed small constant recorded in the implementation record; it is a test-guardrail, not a scheduling quantum, and is never advertised as policy | prevents infinite re-enter loops from wedging automated runs (P4-V13 dependency) | W08 automation need; W07 repeat evidence |
| D9 | All `unsafe` is concentrated in: the two naked switch stubs, the host-context save/restore block, and sysreg read/write helpers; each carries structured SAFETY justifications tied to the register-ownership contract ([02 §5](02-architecture-and-state.md)) | auditable minimal unsafe (ADR-006); the ownership contract is the justification source | Coding Guidelines; P0-W10 governance |
