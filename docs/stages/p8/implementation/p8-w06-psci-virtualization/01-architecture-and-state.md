# P8-W06 Architecture and State

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W06 detailed design](README.md).

## 1. Logical modules

PSCI virtualization is a firmware-interface layer between the unified Guest
exception path and existing VM/vCPU lifecycle mechanisms. Five logical modules
cover the plan scope; none introduces a new subsystem boundary beyond them.

| Module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| M1 Conduit dispatch (PSD) | Recognize a PSCI call on the HVC path, decode the function ID, route to the handler, write response registers | None (stateless per call) | Exception frame (x0–x3), calling vCPU, VM config | Handler outcome → x0 (and x1–x3 where a function defines them) | Does not classify non-PSCI HVC traffic (P5 dispatcher owns that); does not validate parameters |
| M2 Request validation and query (PSV) | Validate parameters; answer PSCI_VERSION and PSCI_FEATURES from machine-contract facts | Read-only view of VM PSCI config (frozen subset, version) | Function ID + decoded arguments | Validated request or spec error code | Does not own lifecycle transitions; does not own machine values |
| M3 vCPU lifecycle bridge (PSC) | Bind CPU_ON/CPU_OFF/AFFINITY_INFO to vCPU lifecycle transitions and P7 scheduler seams | None of its own (mutates vCPU state through owned lifecycle APIs only) | Validated target/context; P7 admission/stop results | Transition outcome → PSCI status | Does not implement scheduler policy, placement, or entry register contents |
| M4 VM system lifecycle bridge (PSS) | Bind SYSTEM_OFF/SYSTEM_RESET to the VM lifecycle | None of its own | Calling vCPU, VM handle | VM transition request to the VM lifecycle owner; disposition of the calling pCPU | Does not implement VM destroy/reload mechanics; does not touch Host power state |
| M5 Diagnostics and telemetry (PST) | Emit structured events for every PSCI outcome | Sequence counters per VM (bounded, wrap-tolerant) | Handler outcomes, rejection reasons | Telemetry events, VM-facing diagnostic context for W13 | Not a logging facility; no unbounded buffers |

M1–M5 are contract names (decision D8 of the README). Physical placement in
crates follows the approved workspace decision; the layering rule is fixed:
M1–M3 logic is architecture-specific (AArch64 PSCI semantics) but must not
depend on any board, SoC, or QEMU constant; every Guest-visible value enters
through machine-contract facts.

## 2. Core objects and ownership

No new long-lived object class is introduced. The design adds fields to
existing planned objects and one per-VM read-only view:

- **VM PSCI configuration view** (per VM, created at VM creation, immutable
  after the W02-frozen machine facts are applied): reported PSCI version,
  implemented-function table, conduit selection. Owner: the VM object. This is
  the only state M2 reads; it is machine-contract data, not runtime state.
- **Per-vCPU PSCI pending-target state** (per vCPU, exactly one of
  {absent, pending-on}): records that a validated CPU_ON has committed a vCPU
  to enter with the W03 entry state. Owner: the vCPU object; written only by
  the M3 transition, read by the arch entry path when the vCPU first runs.
  Cleared at first entry; absent again after CPU_OFF.
- **Per-VM PSCI telemetry counters** (per VM): accepted/rejected counts by
  function ID, bounded, wrap-tolerant. Owner: M5 via the VM's telemetry
  registry (P0 trace-event namespace). Never guest-readable.

No mutable state is shared directly between Guest and Hypervisor; every PSCI
effect flows through existing lifecycle owners. There are no guest-writable
hypervisor structures in this design.

## 3. Lifecycle and state machines

### 3.1 PSCI call lifecycle (per call, stateless)

```text
exception entry (HVC, from NS EL1)
  -> classify: PSCI function-ID namespace?  no -> P5 hypercall route (out of W06 scope)
  -> decode + validate (M2)          invalid -> spec error code -> response -> done
  -> execute handler (M3/M4)         internal failure -> VM-scoped fault path (W13) -> done
  -> write response registers (M1)   telemetry event (M5) -> done
```

### 3.2 Target-vCPU lifecycle as seen from CPU_ON/CPU_OFF

Mapped onto the P7 vCPU lifecycle states; W06 introduces no new state:

```text
Offline / Stopped (stopped by CPU_OFF or never run)
  --CPU_ON (validated target)-->  pending-on (Runnable, entry = W03 secondary state)
  --scheduler runs it-->          Running (entry state consumed once, pending cleared)
  --CPU_OFF (self)-->             Stopped (runs-to-completion of the call; never returns to Guest)
```

Transitions not listed are refused by the lifecycle owner and surface as PSCI
errors: CPU_ON of an online or already-pending vCPU → `ALREADY_ON`;
CPU_OFF while the vCPU cannot legally stop → `DENIED`. Repeated cycles
(OFF→ON→OFF) must return to identical state — this is a validation scenario
(S3b), not an implementation detail.

### 3.3 VM system lifecycle as seen from SYSTEM_OFF/SYSTEM_RESET

```text
Running
  --SYSTEM_OFF-->  all vCPUs requested to stop -> VM moves toward Stopped (ADR §4.1)
  --SYSTEM_RESET-> all vCPUs requested to stop -> VM moves toward a re-loadable state
                   (Host may re-establish W03 boot inputs; mechanism is implementation choice)
```

The calling vCPU does not resume Guest execution after either call. Other-VM
and Host state are untouched by construction: the bridge calls the VM-scoped
transition only.

## 4. Runtime flow (CPU_ON end to end)

```text
Guest EL1 (Linux cpu_up)         Hypervisor EL2
  HVC #imm  x0=CPU_ON             M1: classify, decode target MPIDR, entry IPA, context_id
       x1=target MPIDR            M2: target in VM topology? reserved bits zero? entry IPA within
       x2=entry IPA                  Guest RAM and Stage-2-mappable (P4 contracts)?
       x3=context_id              M3: lifecycle transition Offline/Stopped -> pending-on (P7 admission)
                                  entry state = W03 secondary boot state (PC=entry IPA, x0=context_id)
                                  M5: telemetry
  x0=SUCCESS (caller resumes) <--- response written; target scheduled independently
```

## 5. Concurrency and security model

- **Lock discipline:** every PSCI handler runs in the calling vCPU's VM-exit
  context. It may take bounded VM/vCPU lifecycle locks (order and scope follow
  the P3/P7 lock rules); it must not poll, sleep, or wait for another vCPU.
  In particular, CPU_ON never waits for the target to run: it commits the
  transition and returns.
- **Cross-CPU transitions:** CPU_ON affects a target that may be assigned to
  another pCPU. The transition is expressed as a lifecycle request consumed by
  the target's scheduler domain (P7 seams) plus, where required, the P3
  notification transport for a remote kick. W06 specifies the request, not the
  transport.
- **Allocation:** no unbounded allocation is permitted in the VM-exit path.
  All PSCI handling uses pre-provisioned objects (topology vCPUs, per-VM
  config view); the design admits no dynamic object creation on the call path.
- **Untrusted-input validation (ADR §19):** function IDs outside the frozen
  subset; non-zero reserved parameter bits; target MPIDRs outside the VM's
  virtual topology; entry IPAs outside Guest RAM or pointing at non-mappable
  IPA — each yields a spec error code and a telemetry event, never a Host
  fault. `context_id` is Guest-owned data passed through unmodified; it never
  influences Hypervisor control flow.
- **Authorization:** the calling vCPU implicitly acts on its own VM only.
  Target resolution is scoped to the calling VM's topology, so no PSCI call
  can name another VM's vCPU. No capability, role, or VM-ID-based grant exists
  on this path (ADR-051).
- **Non-leakage:** response values derive from virtual state only. AFFINITY_INFO
  reports virtual-topology status, not pCPU placement or Host PSCI state
  (decision D6).

## 6. Failure boundaries of assumed prerequisite contracts

W06 consumes planned upstream contracts. If a prerequisite delivers
differently from the assumption below, the affected step stops and records an
issue per the README's authority order; it does not adapt silently.

| Prerequisite | Assumed contract (cite) | Failure boundary if different |
|---|---|---|
| P7 admission/stop seams | `docs/stages/p7/plans/p7-w02-scheduler-admission-lifecycle.md`, `p7-w07-pause-stop-fault.md`, handoff `p7-w14-documentation-p8-handoff.md` | If admission cannot express "commit a stopped vCPU to Runnable with pending entry state", [03](03-code-contracts-cpu-lifecycle.md) §2 blocks; an `Architecture Change Request` against the P7 seam is recorded, not a local scheduler patch |
| P4 Stage-2/IPA validation helpers | `docs/stages/p4/plans/p4-w09-closeout-p5-handoff.md` entry/Stage-2 facts | If Guest RAM bounds or IPA validation are unavailable, M2's entry-IPA check has no basis; CPU_ON blocks and records the gap |
| P5 hypercall numbering | `docs/stages/p5/plans/p5-w02-hypercall-abi-error-boundary.md` | If P5 numbers overlap the PSCI SMC function-ID namespace, M1 classification is ambiguous; resolution requires an ABI coordination decision (README decision D5) — flag as `ADR Required` if it forces either numbering to change |
| W03 boot contract | `docs/stages/p8/plans/p8-w03-linux-boot-contract.md` secondary boot state | If no secondary entry state exists, M3 cannot commit pending-on state; block at the W03/W06 seam |
| W04 DTB facts | `docs/stages/p8/plans/p8-w04-guest-dtb-contract.md` PSCI node | If the node disagrees with the frozen subset/conduit, the machine-contract review (P8-V05) fails; fix is in W04's contract, not in W06 code |
| W02 machine gate | `docs/stages/p8/plans/p8-w02-machine-contract-governance.md` | If the gate has not approved the PSCI rows, no implementation may treat the subset, conduit, or version as frozen (README decision D2) |
| W05 classification | `docs/stages/p8/plans/p8-w05-linux-cpu-virtualization.md` | If a PSCI-adjacent operation is classified differently (e.g. SMC trapping), M1's classification table follows W05's categories; conflicts stop the affected choice |
