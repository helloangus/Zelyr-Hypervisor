# P4-W04 Code Contracts — World-Switch Boundary

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W04 detailed design](README.md).  
**Companion:** register-ownership contract and phase orders in
[02-architecture-and-state.md](02-architecture-and-state.md) §4–§5.

This file fixes the assembly boundary contracts. Register-ownership citations
(§ numbers below) refer to the table in
[02 §5](02-architecture-and-state.md). All names are P4-internal,
unstable-by-declaration. Pseudocode is register-transfer outline, not
runnable code; instruction spellings follow the pinned architecture revision
and are verified at implementation.

## 1. Sysreg helper layer (module `ws-switch`)

- **Name and stability:** small read/write helpers, e.g.
  `read_esr_el2() -> u64`, `read_elr_el2()`, `write_elr_el2(v)`,
  `write_spsr_el2(v)`, `read_far_el2()`, `read_hpfar_el2()`, plus the W02-
  owned activation writes for `VTTBR_EL2`/`VTCR_EL2` (W02 §3.6 contract —
  implemented there, invoked here). Internal.
- **Purpose and caller:** the only places raw system-register names appear;
  called by `vcpu-run`, `vcpu-ctx`, and the stubs' C-visible shims.
- **Preconditions:** caller is executing at EL2; register side effects are
  documented per helper (some reads have SideEffects — the helper list names
  them).
- **Postconditions:** value moved uninterpreted; interpretation belongs to
  typed decoders in `vcpu-run`/W06.
- **`unsafe` boundary:** each helper is a minimal inline-assembly `unsafe`
  block; SAFETY = "executing at EL2 enforced by caller context; no memory
  effects."
- **Validation:** encoding review; on-target read-back sanity (P4-V04 path).

## 2. Frame layout contract (module `ws-frame`)

- **Name and stability:** `HostArea` and `GuestExitFrame` — fixed-layout
  structures with explicit fields and offsets:

```text
HostArea {
  saved_sp_el2, saved_daif, host_callee_saved[?], current_vcpu: *mut Vcpu,
  phase_marker: u32            // diagnostics only, never authoritative
}
GuestExitFrame {
  gp[31], sp_el1, guest_pc, guest_pstate,
  esr: u64,                    // full syndrome
  far: u64, hpfar: u64,        // captured when architecturally provided
}
```

- **Purpose and caller:** the stub writes, Rust reads. The frame is the only
  Guest-data channel across the boundary.
- **Preconditions:** located via the P3 per-CPU mechanism (M5); storage is
  per-pCPU, Host Stage-1 mapped, and never freed while the pCPU is online.
- **Postconditions:** after stub step 2 ([02 §4.2](02-architecture-and-state.md)),
  every field is valid regardless of Guest behavior (unconditional capture).
- **Concurrency:** single writer (the stub on this pCPU); the interrupt-
  masked segment excludes re-entrancy.
- **Security:** the frame never influences control flow directly; Rust
  validates/decodes before use (Guest-influenced PC/IPA are data, never
  dereferenced as host addresses).
- **Representation rule:** explicit field layout per the Coding Guidelines;
  no raw-memory reinterpretation across the boundary beyond the one reviewed
  stub write path.
- **Validation:** layout review against the register-ownership table; on-target
  frame-consistency assertions in the exit handler ( Guest PC within Guest
  IPA RAM for the positive scenarios).

## 3. Guest-entry stub contract

- **Name and stability:** `ws_guest_enter(frame: &HostArea, ctx: &VcpuContext)
  -> !` (naked function; returns only via a later Guest exit through
  `ws_guest_exit`). Internal.
- **Purpose and caller:** perform [02 §4.1](02-architecture-and-state.md)
  steps 4–6 atomically: restore Guest context, save host state, `ERET` to
  EL1. Called by `vcpu-run` as the last Rust statement of an entry.
- **Inputs/outputs:** prepared context and the pCPU host area; no return
  value (control reaches Rust again only via the exit path).
- **Preconditions (SAFETY obligations, each cited to §5):**
  1. Stage-2 activation complete (W02 §3.6) and EL1 baseline restored (D5)
     — done by the caller before this stub;
  2. `ctx` satisfies the EL1-only invariant (type-carried);
  3. the host area is this pCPU's frame (located via the P3 mechanism), and
     this pCPU is the vCPU's bound pCPU (state-checked by caller);
  4. the function is a naked function: no compiler-generated stack use after
     the handoff; all host state saved before Guest registers are loaded
     (order in the outline below is normative).
- **Postconditions:** Guest executes at EL1h with DAIF masked; host SP,
  callee-saved set, and DAIF are recoverable from `HostArea`; `ERET` target
  is `ctx.pc` with `ctx.pstate`.
- **State/ownership change:** register ownership transfers to the Guest per
  §5; host state ownership retained in the frame.
- **Errors:** none recoverable after ERET — every precondition is checked
  before the stub runs; a precondition failure is a fatal invariant, never a
  Guest event.
- **Security:** the Guest cannot influence any input (all host-authored);
  mode/DAIF fields are constructed, not copied from Guest-reachable state
  (P4-C02 containment).
- **Logic:**

```text
ws_guest_enter(frame, ctx):                 // naked
    // 1. save host state first (§5 host rows)
    frame.saved_sp_el2  = sp_el2
    frame.saved_daif    = daif ; mask_daif()
    frame.callee_saved  = x19..x30
    // 2. load guest context (§5 guest rows)
    x0..x30 = ctx.gp ;  sp_el1 = ctx.sp
    elr_el2 = ctx.pc ;  spsr_el2 = ctx.pstate        // EL1h, DAIF masked
    // 3. switch to ERET
    isb()                       // context sync after sysreg writes
    eret()                      // target EL1; no EL2 code runs beyond this
```

- **Validation:** on-target first-entry evidence (P4-V04); review that no
  instruction between host-save and guest-load clobbers either set;
  fault-injection review (deliberately bad target IPA must exit via the
  exit stub, never by EL2 recursion).

## 4. Guest-exit stub contract

- **Name and stability:** `ws_guest_exit() -> ExitTicket` conceptually —
  implemented as the vector-path tail: the Guest-exit vector target performs
  frame capture and host restore, then branches to a Rust handler with a
  pointer/episode reference to the frame. Internal.
- **Purpose and caller:** perform [02 §4.2](02-architecture-and-state.md)
  steps 2–3; hand control to `vcpu-run` with EL2 fully recovered. It is the
  only path by which the Guest returns control to EL2.
- **Inputs/outputs:** hardware exception state (ESR/ELR/SPSR/FAR family);
  outputs the populated `GuestExitFrame` reference.
- **Preconditions (SAFETY obligations):**
  1. vector entry established a valid EL2 stack (P1 vector baseline — this
     stub may switch SP from the interrupt stack to the saved host stack as
     its first memory-safe act);
  2. the pCPU frame is locatable without Guest-clobbered registers (uses
     the CPU-pointer mechanism only, M5);
  3. capture order is unconditional and complete before any branch that can
     fault (no memory walk, no stack-dependent code before capture).
- **Postconditions:** host SP/callee-saved/DAIF restored; Rust executes
  normal EL2 code; EL2 control intact (P4-E03).
- **State/ownership change:** register ownership transfers back to the Host;
  the captured frame becomes read-only input for classification.
- **Errors:** none in the stub; faults during stub execution are fatal host
  invariants (P1 crash path — non-recursive diagnostics, M3).
- **Security:** the Guest chooses *when* to exit but not *where* control
  goes: the stub's control flow is fixed code; all Guest-derived values are
  data in the frame (ADR §19).
- **Logic:**

```text
guest_exit_vector_tail:                     // entered from P1 vector path
    frame = per_cpu_frame()                 // CPU-pointer mechanism only
    frame.gp[0..31] = x0..x30
    frame.sp_el1     = sp_el1
    frame.guest_pc   = elr_el2
    frame.guest_pstate = spsr_el2
    frame.esr        = esr_el2
    frame.far, frame.hpfar = far_el2, hpfar_el2
    sp_el2   = frame.host_area.saved_sp_el2
    x19..x30 = frame.host_area.callee_saved
    daif     = frame.host_area.saved_daif
    isb()
    b rust_exit_handler(&frame.guest)       // EL2 C/Rust domain, control intact
```

- **Validation:** every P4-V04/V06/V07/V08 exit evidence passes through this
  path; stub-level review is part of each; dedicated review: recursive-entry
  check (a fault during capture must terminate in the P1 fatal path, not
  re-enter the Guest).

## 5. Host-context integrity review requirements

Because this boundary is the package's primary risk, review (not test alone)
carries specific duties:

- every `unsafe` block cites the register-ownership rows it relies on, by
  §5 reference, in its SAFETY note;
- the unsafe inventory (W01 R20) records: two naked stubs, the host
  save/restore sequences within them, and the sysreg helpers — total surface
  must match the inventory;
- no Rust code executes between context restore and `ERET`, and none between
  exception entry and frame capture (verified by review of generated code for
  the two stubs);
- interrupt masking discipline (D4) is asserted at entry and checked at exit
  handling (a masked-state mismatch is a fatal invariant).

## 6. Validation summary for this boundary

| Check | Technique | Passes when |
|---|---|---|
| Entry reaches EL1 | on-target marker via W05 scenario (VG-001 class) | Guest code runs and reports CurrentEL = EL1 |
| Exit retains EL2 | on-target controlled exit (VG-004 class fault) | Rust handler runs after capture with usable EL2 state |
| Frame fidelity | exit-handler assertions + W06 cross-checks | captured PC matches the Guest scenario's expected fault site |
| Host-state integrity | post-exit diagnostics + repeat runs | EL2 console, allocators, logging behave normally after each exit |
| Non-recursion | deliberate fault-during-capture review/injection | termination in the P1 fatal path, no Guest re-entry |
