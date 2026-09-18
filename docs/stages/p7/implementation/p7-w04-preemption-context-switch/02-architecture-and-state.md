# P7-W04 Architecture and State

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P7-W04 detailed design](README.md).

## 1. Logical modules

| Module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| `slice` | slice type, validation, policy-source seam, expiry determination | none (pure + policy hook) | policy query, monotonic time | `TimeSlice` values, expiry verdicts | timer programming (P6), pick order (W05) |
| `deadline` | per-pCPU deadline bookkeeping around the P6 mechanism | pCPU deadline state (` Armed{at} / Disarmed`), expiry flags | P6 arm/cancel results, IRQ arrival | armed/cancelled confirmations, `DeadlineExpired` triggers | timer registers, time source (P6) |
| `intent` | pCPU-local reschedule intent | `reschedule_pending` flag | IRQ handler, remote request surfacing (W08) | consume/peek for the control loop | transport (P3/W08), loop policy (W05) |
| `switchseq` | the vCPU-switch sequence orchestrating predecessor mechanisms | none (sequence owner) | picker decision (W05), exit outcomes (W02 boundary), P4/P6 mechanism calls | completed switch or typed failure | queue membership (W05), lifecycle authority (W02), state transfer (P4/P6) |

Dependency direction (normative): `switchseq` → `deadline`/`intent`/`slice`
+ W02 contracts + P4/P6 mechanism seams; the control loop (W05) →
`intent`/triggers + `switchseq`; `deadline` → P6 mechanism seam only.
Nothing in W04 depends on queue internals or policy internals.

## 2. Preemption-points model

```text
 Guest running on pcpu (state Running, deadline Armed{now+slice})
   |
   |-- deadline IRQ fires (P6) --> Guest exits (IRQ-class)
   |      IRQ context: record expiry; intent.set(); mark trace; return
   |      control loop (schedulable ctx): sees DeadlineExpired trigger
   |
   |-- any other Guest exit (HVC, WFI->Block, fault, ...) --> exit boundary
   |      control loop sees GuestExit(exit-kind) trigger
   |
   |-- remote/local reschedule request (W08 semantics; P3 transport)
   |      surfaced on target pcpu: intent.set()
   |      control loop sees RescheduleRequest trigger at a safe point

 Control loop at any trigger: reconsider (W05 policy) using
   intent.consume() + exit outcome -> continue current | switch | idle
```

Closed trigger set: `DeadlineExpired`, `GuestExit(ExitKind)`,
`RescheduleRequest`. Adding a trigger is a design change. Note the
architectural property (README decision 2): preemption happens only at
these EL2 control points; the Guest always reaches an exit before losing
the pCPU; the pCPU is lost only by the owner-context sequence, never by
another CPU directly.

## 3. Switch ownership table (normative)

Per ADR §5, every vCPU switch must explicitly handle each class below. The
owner column is exclusive: exactly one module/layer writes each class.

| State class | Owner | Transfer point in the switch | W04's role |
|---|---|---|---|
| GPRs, PC/PSTATE/SP, EL1 sysreg state | P4 arch context (`VcpuContext`) | saved by P4 at Guest exit; restored by P4 at entry | assert saved-before-quiesce; call restore at activation |
| lazy FP/SIMD | P4 (Reserved per ADR §5) | not switched in P7 | require the P4 contract to state its P7 behavior explicitly |
| Stage-2 / VMID address space | P4 | `activate_address_space(vcpu)` call at activation step | order the call before entry; require TLB correctness statement from P4 |
| virtual timer state | P6 vCPU timer | P6 save (at quiesce) / restore (at activation) | order the calls; require monotonicity statement |
| vGIC/LR state, pending vIRQ, pending events | P6 (LR/maintenance; pending queue) | preserved per P6 contracts; LR save/restore per P6 virtualization contract | require preservation statement; no LR access |
| `current_vcpu` slot | W02 admission module | cleared at quiesce step; set inside `admit_for_entry` | orchestrate ordering |
| run-state | W02 lifecycle engine | `Deschedule`/`Block`/... at quiesce; `Dispatch` inside gate | orchestrate ordering |
| run-queue membership | W05 | enqueue after deschedule (per W05 discipline) | call W05 operations only |
| per-pCPU deadline + intent | W04 | cancel at quiesce; arm at activation end | own |
| switch accounting/trace | W04 semantics → W09 rendering | emission points in the sequence | own semantics |

Rule: if an implementation step must write a class it does not own, the
step is wrong — the design is violated, not merely suboptimal.

## 4. Concurrency and context bounds

- **Deadline IRQ context:** record + flag + trace mark only; no locks
  beyond the P6 IRQ-safe primitives; no allocation; no queue/lifecycle
  operations. Bound: constant work, single trace mark.
- **Control-loop (schedulable) context:** the switch runs here with IRQs
  in the state the P3 baseline prescribes for scheduler critical sections;
  the sequence holds no lifecycle lock across mechanism calls (W02
  leaf-lock rule governs; the gate takes the cell lock internally).
- **Cross-CPU rule:** a switch always executes on the pCPU that will run
  the next vCPU; no cross-CPU switch exists in P7; migration is
  requeue-elsewhere (W05) plus remote reschedule (W08). A remote request
  never writes another pCPU's intent or deadline directly — it requests
  via the P3 transport and the owner surfaces it.
- **Re-arm discipline:** at most one armed deadline per pCPU at any time;
  every switch ends with either an armed deadline (dispatch) or a
  disarmed deadline (idle — idle wake handling is W08). Cancel-before-arm
  ordering is mandatory; P6's rearm semantics (P7-IN-07) are consumed, not
  redefined.

## 5. Isolation classes for P7-V09 (A→B→C→A rotation)

The plan requires: "A→B→C→A preserves registers, PC/PSTATE, required
system state, address space, timer, vIRQ and events." The checkable
isolation classes:

1. Architectural context (GPR/PC/PSTATE/SP + declared EL1 sysregs) — P4
   save/restore correctness under rotation.
2. Address space — each vCPU's Stage-2 identity re-activated per switch;
   no cross-VM address-space leakage (VMID correctness is P4's, invoked
   per switch).
3. Virtual timer — per-vCPU timer state and deadlines survive rotation.
4. vIRQ and events — pending state survives rotation and is delivered to
   the right vCPU after arbitrary interleaving.
5. Scheduler-owned facts — run state, slot agreement, queue membership
   (W02/W05 invariants sampled under rotation).

W04's obligation: the sequence makes each transfer point explicit and
instrumentable; W10's guest suite provides the workload evidence; W11
stresses the interleavings. W04 claims none of the evidence.

## 6. Failure model

| Condition | Classification | Behavior |
|---|---|---|
| Deadline IRQ with no running Guest (spurious/late) | recoverable, expected race | record expiry; intent set; loop consumes and idles or dispatches; no error surface |
| Gate rejects the switch candidate (`NotDispatchable` etc.) | recoverable | requeue/re-pick per W05; idle allowed (W08); bounded by picker progress rules |
| P4 address-space activation error | internal error, Guest-unrelated | candidate undispatched this round + diagnostic; repeat escalation to invariant investigation; never marked `Faulted`; fatal only if classified an invariant violation per P0 policy |
| P6 timer/vIRQ restore error | internal error | same containment as above for the timer/vIRQ class |
| Deadline arm fails (P6 error) | recoverable with degraded property | dispatch proceeds without preemption deadline is NOT permitted for shared pCPUs (a CPU-bound Guest could monopolize): arm failure on a shared dispatch is a recorded error and the dispatch aborts (candidate requeued); on a pinned/dedicated dispatch, arm failure degrades to no-preemption and is recorded — equivalence to static binding makes this safe |
| Invariant violation during switch (slot mismatch, state disagreement) | fatal | P0 panic policy; P1 diagnostics |

The arm-failure asymmetry (shared aborts, pinned degrades) preserves the
stage's headline guarantee (no monopolization) while keeping pinned
operation equivalent to the static baseline even under timer degradation.
