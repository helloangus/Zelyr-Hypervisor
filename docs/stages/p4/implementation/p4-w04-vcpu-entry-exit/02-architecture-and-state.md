# P4-W04 Architecture, Objects, and State Model

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W04 detailed design](README.md).

## 1. Logical modules

| Module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| `vcpu-ctx` | Guest register context value type; initial-state construction (pure); save-from-frame / restore-to-frame | none persistent (value type) | GuestInput, exit frames | `VcpuContext` values | execution, scheduling |
| `vcpu-obj` | The vCPU object: identity, run state, context storage, exit budget | context copy, run state, counters | construction params, exit outcomes | state transitions, stop results | pCPU storage, Stage-2 |
| `ws-frame` | Per-pCPU host-context and exit-frame layout; location via the P3 CPU-local mechanism | frame storage (per pCPU) | stub writes, Rust reads | host-state preservation guarantees | Guest contents |
| `ws-switch` | The assembly boundary: guest-entry stub, guest-exit stub, sysreg helpers | none | prepared context/frame | entered Guest; captured exit frame | classification policy |
| `vcpu-run` | Run control: preconditions, activation call (W02), entry, exit interpretation, actions, stop/teardown sequencing | run-loop locals | vCPU object, W02 space, exit frames | Enter/Stop outcomes, events | fault diagnostics (W06) |

Layering: all modules are Arch-layer (register names appear in `ws-switch`
and `vcpu-ctx` only); Core-visible output is the P4-local outcome vocabulary
(enter result, stop cause) — the final generic `ExitReason` API is explicitly
not designed here. No board/SoC/QEMU names.

## 2. Core objects and ownership

### 2.1 `Vcpu` (module `vcpu-obj`)

- **Owned state:** saved `VcpuContext` (authoritative while not running),
  run state, exit-budget counter, vCPU identity.
- **Immutable after construct:** identity, initial-context derivation inputs
  (retained for repeat evidence).
- **Not owned:** Stage-2 space (owned by the P4 minimal VM setup, activated
  per entry), Guest RAM (W03), the per-pCPU frame (pCPU-owned), any pCPU.
- **Destruction:** only from a terminal run state (Stopped); sequence owned
  by the setup/teardown coordinator (W07 repeat and W09 record consume it).

### 2.2 `VcpuContext` (module `vcpu-ctx`)

- Value type: GP registers x0–x30, SP (SP_EL1 value), PC, PSTATE value
  (mode/flags/DAIF fields explicit), reserved alignment. No EL1 sysregs are
  part of the saved context in P4 (D5 of [01](01-scope-and-foundations.md);
  baseline-restore strategy) — the limitation is recorded, not hidden.
- Construction is total and pure; the type cannot represent a Guest at any
  exception level other than EL1 (constructor enforces mode/DAIF fields), so
  the "Guest is EL1, never virtual EL2" invariant is type-carried (ADR-022).

### 2.3 Per-pCPU world-switch frame (module `ws-frame`)

- **Host area:** host SP_EL2 value, host callee-saved register set,
  host PSTATE (DAIF) value, current-vCPU pointer (capacity reserved by P3,
  now used — W01 R14), and a switch-phase marker used only for diagnostics.
- **Exit frame (D6 of [01](01-scope-and-foundations.md)):** guest GP
  registers, SP_EL1, PC, PSTATE, syndrome fields (ESR class and
  qualification), fault-address fields captured unconditionally when
  architecturally provided.
- Located through the P3-declared per-CPU mechanism (M5) — typically a
  CPU-pointer register maintained by P3; the stub never trusts a
  Guest-clobbered general register for this purpose.
- **Ownership:** the pCPU owns the storage; `ws-switch` and `vcpu-run` are
  the only readers/writers, in the fixed phase order of
  [03](03-code-contracts-world-switch.md).

## 3. vCPU run-state machine (P4 subset)

```text
                construct()             enter()                 exit captured
  [absent] ----------------> Ready ----------------> Running ---------------+
                                ^  |                                        |
                                |  +----------- reenter() <----(action=Reenter)
                                |                                            |
                                |              action=Stop(cause)            |
                                +--------------------------------------------+
                                |                       
                            Stopped (cause: Controlled | GuestFault | BudgetExhausted)
                                |
                            destroy()   [teardown sequencing: vCPU stopped ->
                                         space destroy (W02) -> GuestRam release (W03)]
```

Rules:

- `Ready → Running` happens only through the entry protocol with all
  preconditions checked ([04 §3.1](04-code-contracts-vcpu-run.md)); the
  transition is recorded by an event (W01 A8).
- `Running` is exclusive to the bound pCPU; a second concurrent entry attempt
  is a programming error escalated as a fatal invariant (no legitimate
  concurrent caller exists in P4).
- `Stopped` records its cause and preserves the last exit frame (diagnostics
  remain readable after stop — EL2 must stay "live and diagnosable" per the
  task book).
- The ADR vCPU lifecycle (`Offline → Runnable → Running → Blocked → Paused →
  Stopped → Faulted`) is **not** implemented here; this P4 subset maps onto
  it (`Ready≈Runnable`, `Stopped` union of Stopped/Faulted outcomes) and the
  mapping note travels to W09 so P7 can supersede without re-derivation.
- No hidden boolean state: phase is an explicit enum; the frame's diagnostic
  marker is derived data, never authoritative.

## 4. Entry protocol and Guest-execution controls

### 4.1 Entry sequence (normative order)

```text
1. run preconditions validated (state, space, budget)     [vcpu-run]
2. Stage-2 space activated for this pCPU                  [W02 activate]
3. EL1 architectural baseline restored                    [D5]
4. Guest context restored into the live register file     [ws-switch entry stub]
5. host PSTATE (DAIF) masked and saved; host SP saved     [ws-switch]
6. ERET to Guest EL1                                      [ws-switch]
```

Steps 4–6 are one assembly block — no Rust may run between context restore
and `ERET` (any Rust step could clobber Guest registers or fault into a
confused state).

### 4.2 Exit sequence (normative order)

```text
1. Guest exception taken at EL2 (vector entry)            [P1 vector path]
2. Guest-exit stub: store GP regs, SP_EL1, ELR/SPSR/ESR,
   fault addresses into the per-pCPU exit frame           [ws-switch]
3. restore host SP, callee-saved regs, host DAIF          [ws-switch]
4. Rust exit handler runs with EL2 control intact         [vcpu-run]
5. classification/diagnostics                             [W06 consumes frame]
6. action: Reenter (goto entry steps 3–6) or Stop          [D7]
```

Steps 1–3 contain no memory accesses through Guest-influenced addresses;
the stub writes only to the pCPU-owned frame (Host Stage-1 mapped).

### 4.3 Guest-execution control intents

Required routing outcomes while the Guest runs (exact bit encodings verified
against the pinned architecture revision at implementation; QEMU divergences
become Specification Investigation records, W01 A7):

| Guest behavior | Required outcome |
|---|---|
| WFI | synchronous trap to EL2, classified `Wfi` |
| WFE | synchronous trap to EL2, classified `Wfe` |
| HVC | does not reach a Guest-handled path; produces a classified synchronous exit (routing intent: disabled/undefined for the Guest) |
| SMC | synchronous trap to EL2, classified (EL2 owns SMC proxying per ADR-008; P4 only classifies and stops) |
| Access to unmapped/permission-violating IPA | Stage-2 fault with syndrome + faulting IPA available (W02 mechanics; W06 diagnosis) |
| EL1 system-register writes | contained by D5 baseline restore; not individually trapped in P4 (documented limitation) |
| Any other synchronous exception | classified `UnknownSync` with full syndrome |

The P4-local exit-class vocabulary (`Stage2Translation`,
`Stage2Permission`, `Wfi`, `Wfe`, `UnknownSync(ESR)`, `IllegalExecution`,
`BudgetExhausted`) is temporary (README excluded interfaces); W06 extends it
toward classification detail and W07 consumes counts.

## 5. Register-ownership contract

This table is the authoritative division used by every `unsafe` SAFETY
justification in [03](03-code-contracts-world-switch.md); any code touching
these registers outside this division is a review failure.

| Register group | While Guest runs | On exit | On entry |
|---|---|---|---|
| x0–x30 (GP) | Guest-owned; arbitrary values permitted | saved verbatim to exit frame by stub | loaded from saved/initial context |
| SP_EL1 | Guest-owned | saved to exit frame | loaded from context |
| PC (ELR_EL2 sense) | not host-visible; Guest executes freely | captured as Guest PC | loaded into ELR_EL2 before ERET |
| PSTATE (SPSR_EL2 sense) | Guest-owned (EL1 domain) | captured | constructed (EL1h, DAIF masked) |
| Host SP_EL2, host callee-saved x19–x30, host LR/return | Host-owned; stored in pCPU frame; untouchable by Guest (Guest has no architectural route to EL2's stack) | restored by stub before Rust | saved by entry stub after last Rust step |
| EL2 stack memory | Host-owned; idle while Guest runs; remains valid | in use by stub/handler | in use until stub hands off |
| ELR_EL2/SPSR_EL2 | reserved by hardware for exception return; not Guest-accessible at EL1 | consumed by exception entry to host's benefit | written with Guest PC/PSTATE |
| ESR_EL2, FAR_EL2/HPFAR_EL2 (fault address family) | written by hardware on exit only | copied to exit frame | not Guest-relevant |
| EL1 architectural registers (SCTLR/TCR/TTBR/VBAR/SPSR_EL1/ELR_EL1/DAIF-EL1) | Guest-writable (untrusted) | not saved (D5 limitation) | restored from baseline before ERET |
| VTTBR_EL2/VTCR_EL2/VMID | Host-owned; never Guest-visible at EL1 | unchanged (no exit-time Stage-2 mutation) | programmed by W02 activation in step 2 |
| HCR_EL2 and routing controls | Host-owned | unchanged | programmed once during setup (P1 baseline + P4 deltas) |
| CPU-pointer/per-CPU base register | Host-owned (P3 mechanism) | used by stub to locate frame | used likewise |

Security reading of the table: the Guest owns no host-recoverable storage;
every register the Guest can influence is either captured into the frame or
deliberately reset (D5). The Guest has no architectural ability to observe or
alter host EL2 state, and ERET target EL/PSTATE fields are constructed
host-side (P4-C01/C02 evidence basis).

## 6. Concurrency model

- One Guest run path per pCPU; `Running` exclusivity enforced by explicit
  state checks (fatal on violation — no legitimate concurrent entry exists).
- The world-switch contains no lock acquisitions; correctness comes from the
  interrupt-masked segment (D4) and single-owner frame storage (P3 M5), not
  from locking — locks in an entry/exit path would be a design violation
  (Coding Guidelines: never block in VM-exit paths).
- Run-state transitions and budget updates happen in the Rust handler after
  the stub has restored host state; they use the P3-era primitives (M6) and
  never run in interrupt context.
- AP interactions: none in P4 (no cross-CPU operations on the running vCPU);
  the Reserved multi-pCPU invalidation seam lives in W02, not here.
- The console-quiet duty (W03 D5) is honored structurally: EL2 does not
  write the console between entry and the exit handler's diagnostics point.

## 7. Telemetry points (W04-scope)

`vcpu.construct`, `vcpu.enter`, `vcpu.exit` (with P4 class and Guest PC),
`vcpu.reenter`, `vcpu.stop` (with cause), `vcpu.destroy` — routed through
the P0 baseline (W01 A8). W06 correlates frames; W07 counts; W08 matches
markers. Events never carry Guest data content beyond PC/IPA-class
addresses already required for diagnosis.
