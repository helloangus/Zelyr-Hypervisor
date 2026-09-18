# P8-W05 Code Contracts — Classification Decision Path

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P8-W05 detailed design](README.md).

## 0. Standing rules for every contract in this file

- These are design-level logical contracts. Exact types, signatures, and
  home crates/modules are fixed at implementation time within the
  architecture layering (ADR §13: syndrome decode and register access are
  Arch-side; classification policy and containment are Core-side; no Core
  dependency on Arch register encodings leaks upward). No contract here
  freezes a public API.
- All names are internal to the hypervisor. None is a Guest-visible or
  management ABI.
- The environment is the established `no_std` EL2 runtime with the P0–P7
  contracts as assumed inputs; every contract states the boundary that
  applies if an assumed predecessor delivers a different shape.
- Pseudocode is logic outline, not runnable production code. The Coding
  Guidelines (checked arithmetic, no `unwrap` on guest-influenced paths,
  `SAFETY` discipline around the Arch boundary) govern the realization.

## 1. Type contract — `CpuBehaviorClass`

```text
Name and stability: CpuBehaviorClass — internal enum { Direct, Emulate,
  Reject, Hidden, Unsupported }; internal, stable within P8.
Purpose and caller: names the resolution of one Guest CPU operation;
  produced by the classification registry, consumed by the decision point
  and the rejection path.
Inputs / outputs: none (data only).
Preconditions / postconditions: total — exactly five values; no wildcard.
State and ownership change: none (value type).
Concurrency/allocation context: Copy, no allocation; safe to hold in exit
  context.
Errors and failure guarantee: not error-bearing; "unknown" is represented
  by Reject at the registry layer, never by a sixth value.
Security/authorization checks: none (pure name).
Logic: n/a (enumeration per [01-classification-model.md](01-classification-model.md) §2).
Validation: exhaustiveness review; classification totality tests (V08).
```

## 2. Type contract — `ClassificationEntry` and `BehaviorRoute`

```text
Name and stability: ClassificationEntry — internal record { behavior:
  BehaviorArea, outcome: CpuBehaviorClass, route: Option<BehaviorRoute>,
  diagnostic: DiagnosticTemplate }; BehaviorRoute — opaque token handed to
  a domain handler; both internal, stable within P8.
Purpose and caller: registry rows produced at initialization by the CPU
  posture and by domain packages (W06/W07/W08/W09) registering their
  routes; consumed only by the decision point.
Inputs / outputs: registration inputs are compile-time posture data, not
  Guest data.
Preconditions / postconditions: outcome = Emulate ⇔ route = Some; Reject /
  Unsupported / Hidden rows carry a diagnostic template; Direct rows carry
  neither.
State and ownership change: rows live in the ClassificationRegistry
  (§3); immutable after init.
Concurrency/allocation context: constructed before Guest execution;
  read-only afterward; no allocation in lookup.
Errors and failure guarantee: a registration that violates a precondition
  fails initialization (fatal, pre-Guest) — a malformed registry must not
  survive to Guest time.
Security/authorization checks: routes are registered only by the
  initialization path; no Guest-writable influence on any field.
Logic: n/a (data).
Validation: registry invariant review; V03/V08 rows.
```

## 3. Type contract — `ClassificationRegistry`

```text
Name and stability: ClassificationRegistry — internal, read-only-after-init
  collection of ClassificationEntry keyed by behavior identity (Arch-decoded
  syndrome class / operation identity); one per hypervisor image.
Purpose and caller: the single authority for class resolution; consulted by
  the exit-path decision point (§4); populated by the CPU posture and domain
  registrations at initialization.
Inputs / outputs: lookup takes an Arch-decoded trapped-operation identity
  (see §4 input note); yields Option<&ClassificationEntry>.
Preconditions / postconditions: lookup is total, side-effect-free, and
  bounded (no probing loops); a miss returns None (the decision point turns
  that into Reject — the registry itself never guesses).
State and ownership change: none at runtime; owned by the posture
  subsystem, not by any VM or vCPU.
Concurrency/allocation context: immutable during Guest execution ⇒
  lock-free reads; construction is single-threaded initialization.
  Failure boundary (assumed contract): if the P3 concurrency substrate
  delivers different guarantees, the immutability claim must be re-reviewed
  before implementation relies on lock-free reads.
Errors and failure guarantee: no error path at lookup.
Security/authorization checks: contains no Guest-controlled data; keys are
  Arch-decoded identities, never raw Guest pointers or lengths.
Logic:

  lookup(op_identity):
    entry <- table.find(op_identity)
    return entry          # None is handled by the decision point

Validation: totality/fuzz of the decoder→lookup path (V08); invariant
review (no runtime mutation path exists).
```

## 4. Function contract — exit-path decision point
(`classify_and_dispatch`)

```text
Name and stability: classify_and_dispatch — internal; the single resolution
  point for Guest-originated synchronous traps (and the classification of
  WFI/WFE and MMIO consorts) on the vCPU exit path. The function name is
  the design name; the implementation binds it into the established P4 exit
  path.
Purpose and caller: called by the P4-established vCPU exit/return flow for
  every exit whose syndrome indicates a Guest-originated operation needing
  resolution; returns the disposition the exit flow executes.
Inputs / outputs: input — an exit context: Arch-decoded syndrome (EC/IL/
  ISS as the Arch layer presents it), faulting PC/PSTATE, FAR/HPFAR validity
  and value when architecturally valid, current vCPU identity, current
  ExitReason (per the P4 exit contract). Output — an ExitDisposition, one
  of: Resume (return to Guest), ResumeWithEffect (handler completed an
  Emulate effect; resume), VcpuStop(diagnostic), VmFault(diagnostic).
Preconditions / postconditions: on return, exactly one disposition is
  produced; the Guest-visible architectural state is either untouched
  (Resume) or advanced by exactly one architecturally correct effect
  (ResumeWithEffect) or the vCPU/VM is contained per §VcpuStop/VmFault
  rules ([03](03-controlled-failure-and-diagnostics.md) §3). The function
  never returns "continue and ignore".
State and ownership change: Direct/Emulate — no persistent state beyond
  handler-owned domain state and telemetry counters; Reject — vCPU (or VM)
  lifecycle state transitions per the P7 run-state contract, and one
  diagnostic record emitted.
Concurrency/allocation context: runs in vCPU-exit context on the owning
  pCPU (P3 per-CPU facts): bounded work, no blocking, no unbounded loops,
  no allocation on the Direct/Emulate path; the diagnostic record is
  fixed-capacity and written in place (V-telemetry path may enqueue
  references per the P0 trace governance assumed contract).
Errors and failure guarantee: registry miss, undecodable syndrome, missing
  handler, or handler error ⇒ Reject path (fail-closed). A failure *inside*
  the rejection path (cannot contain, cannot record) is an
  InvariantViolation per the P5 failure-classification assumed contract —
  the one path that may escalate beyond the VM, by the established
  invariant-failure rules, and it must be as narrow as the P5 contract
  defines.
Security/authorization checks: all Guest-derived values (syndrome fields,
  PC, FAR) are treated as untrusted: decoded via the Arch boundary only,
  range-checked before use in any address-like role, and never dereferenced.
  No Guest value selects a handler; only the registry maps identity→route.
Logic:

  classify_and_dispatch(exit):
    op <- arch.decode(exit)                      # total, checked decode
    entry <- registry.lookup(op.identity)
    match entry:
      None                -> return reject(exit, Reject, template_unknown)
      Direct              -> return Resume                       # no effect needed
      Emulate:
        handler <- entry.route.handler()
        if handler is None -> return reject(exit, Reject, template_missing_handler)
        outcome <- handler.handle(op, exit.vcpu)  # bounded; may itself fail
        match outcome:
          Ok(ResumeWithEffect) -> return ResumeWithEffect
          Ok(VcpuStop / VmFault as d) -> return d           # domain-owned containment
          Err(Recoverable)     -> return reject(exit, Reject, entry.diagnostic)
          Err(Invariant)       -> propagate per P5 invariant rules
      Reject / Unsupported -> return reject(exit, class, entry.diagnostic)
      Hidden               -> return reject(exit, Reject, template_hidden_use)
        # Hidden resolves at discovery; reaching dispatch means the Guest
        # used a concealed feature anyway.

Validation: V07 (declared Direct/Emulate paths exercised by the Linux
run), V08 (Reject path); exhaustive-class fuzz at the decode boundary.
```

## 5. Interface contract — `TrappedOperationHandler` (domain route)

```text
Name and stability: TrappedOperationHandler — internal trait/interface
  implemented by domain packages' mechanisms (PSCI: W06; vGIC: W07; timer:
  W08; console MMIO device: W09; feature presentation: the CPU posture).
  The interface shape is design-level; the established trait lives with its
  owning mechanism.
Purpose and caller: performs the architecturally correct effect for one
  class of trapped operation; called only by the decision point.
Inputs / outputs: input — decoded operation, vCPU context accessors
  (register read/write via the Arch boundary); output — HandlerOutcome:
  ResumeWithEffect, VcpuStop(diagnostic), VmFault(diagnostic), or error.
Preconditions / postconditions: the handler either produces the
  architectural effect and resumes, or fails into an outcome the decision
  point can contain; it must not spin, allocate unboundedly, block, or
  emit a global panic for Guest-caused conditions (ADR §19). Emulate
  handlers are idempotent per operation and side-effect-accurate
  ([01](01-classification-model.md) §5.4).
State and ownership change: only domain-owned state (e.g., timer deadlines,
  vGIC state) and vCPU register context; handlers never touch the
  classification registry or another VM's state.
Concurrency/allocation context: vCPU-exit context, bounded work; domain
  state synchronization follows the owning contract (P6 interrupt/timer, P7
  lifecycle). Failure boundary: if a domain contract requires different
  context (e.g., blocking), the integration stops and the conflict is
  routed — the decision point cannot block.
Errors and failure guarantee: distinguished per the P5 error classes —
  GuestFault-recoverable (becomes Reject diagnostics), resource/invalid
  input (same), invariant (escalates per P5 rules). No silent partial
  effect: a failed handler leaves the Guest state either untouched or fully
  contained.
Security/authorization checks: the handler re-validates anything Guest-
  derived (the decision point's decode does not exempt handlers); PSCI
  handlers additionally operate strictly within the P5 hypercall contract's
  authority and input rules.
Logic: per-domain; owned by W06/W07/W08/W09 designs. This contract fixes
  only the interface obligations above.
Validation: per-domain V-rows; negative tests confirm failed handlers land
  in contained diagnostics (V08).
```

## 6. Function contract — controlled rejection (`reject`)

```text
Name and stability: reject — internal; the single implementation of the
  Reject outcome. Called only by the decision point.
Purpose and caller: converts one trapped, unprovided operation into the
  controlled Guest-visible failure plus diagnostic, containment, and
  telemetry ([03](03-controlled-failure-and-diagnostics.md) §2–§3).
Inputs / outputs: input — exit context, resolved class (Reject or
  Unsupported), diagnostic template; output — VcpuStop(diagnostic) or
  VmFault(diagnostic) per the containment rules.
Preconditions / postconditions: the Guest observes an architecturally
  legal fault at its own exception level (injected per the P6
  virtual-interrupt / exception contracts — cite, do not redesign here);
  the diagnostic record is complete (no placeholder fields); containment
  is applied exactly once (idempotent under the repeated-exit conditions
  the P7 lifecycle defines).
State and ownership change: vCPU run state per P7 lifecycle (and VM state
  on escalation per [03](03-controlled-failure-and-diagnostics.md) §3);
  one telemetry event.
Concurrency/allocation context: as §4 — exit context, fixed-capacity
  record, bounded work.
Errors and failure guarantee: a containment failure is an
  InvariantViolation (narrow, per P5) — recorded as such, never swallowed.
Security/authorization checks: diagnostic content is Host-side truth about
  the Guest (never the reverse channel); no Guest-controlled string or
  buffer is copied into the record.
Logic: per [03](03-controlled-failure-and-diagnostics.md) §2.
Validation: V08 negative matrix.
```

## 7. Type contract — `GuestDiagnosticRecord`

```text
Name and stability: GuestDiagnosticRecord — internal, fixed-capacity
  structured record; internal to P8, consumed by W13 diagnostics and the
  telemetry layer.
Purpose and caller: the VM-facing diagnostic of one Reject outcome (and the
  payload of domain-originated stops); produced by reject/domains, consumed
  by W13's diagnosability work and the structured trace.
Inputs / outputs: fields (all Host-derived): vCPU identity; VM identity;
  classification (class + behavior area); syndrome fields (EC, IL, ISS as
  applicable); PC and PSTATE at exit; FAR/HPFAR values with validity flags;
  exit reason; recent-trace correlation reference (per the P0 trace
  governance assumed contract); containment action taken.
Preconditions / postconditions: complete on delivery — a record with a
  validity flag cleared where data is architecturally unavailable (FAR
  invalid), never a missing field; timestamps/correlation follow the P0
  trace contract.
State and ownership change: written into the diagnostic sink; ownership
  transfers per the telemetry assumed contract.
Concurrency/allocation context: fixed capacity, written in exit context,
  no allocation; one record per Reject outcome.
Errors and failure guarantee: record-construction failure ⇒
  InvariantViolation path (narrow) — a Reject without a diagnostic is not an
  acceptable outcome (P8-V08).
Security/authorization checks: no Guest-controlled content is embedded
  verbatim (addresses appear as address-typed values, not copied buffers).
Logic: n/a (record).
Validation: V08 completeness review; W13 consumption review.
```
