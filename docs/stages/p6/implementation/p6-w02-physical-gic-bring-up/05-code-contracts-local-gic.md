# P6-W02 Code Contracts — Local GIC Bring-up (Phase B)

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P6-W02 detailed design](README.md).  
**Convention:** checklist §3 contract template. Phase B runs per pCPU on
that pCPU, at the P3 local-initialization point, after the distributor is
`Enabled` (boot CPU: after its own Phase A; secondaries: after rendezvous
release). All register operations go through
[03](03-code-contracts-register-access.md).

## 1. `bring_up_local_gic`

```text
Name and stability: fn bring_up_local_gic(expected: &ExpectedGicIdentity,
  pcpu: PcpuId) -> Result<LocalGicReadiness, LocalGicFailure>
  (P6-internal; the only Phase-B entry point; invoked by the P3
  local-initialization owner — the invocation-point wiring is the P3
  integration step of the [workflow](06-implementation-workflow.md))
Purpose and caller: establish independent Redistributor + CPU-interface
  readiness for the calling pCPU (P6-V03), publishing to the readiness
  ledger
Inputs / outputs: expected identity + calling pCPU identity → readiness
Preconditions / postconditions:
  - executing on the owning pCPU; interrupts not enabled at the CPU;
    distributor ledger slot is DistributorReady (ORD-1 of
    [02](02-architecture-and-state.md) §3.2; boot CPU's Phase B runs after
    its Phase A, secondaries after rendezvous + DistributorReady)
  - success: own GICR confirmed and awake; all SGI/PPI disabled with
    lowest default priority and Group1 NS assignment; residuals cleared
    and logged; ICC SRE verified; ICC_CTLR posture set (EOImode=0); PMR
    allow-all; Group1 enabled last with ISB; LocalReady published once
  - failure: LocalFailed(reason) published once; GICR left in the
    documented safe sub-state (ProcessorSleep cleared, SGI/PPI disabled)
State and ownership change: LocalGicContext transitions
  Unprobed → Confirmed → Waking → LocalConfigured → InterfaceReady →
  LocalReady (or LocalFailed from any state), exactly once per pCPU
Concurrency/allocation context: single writer (owning pCPU); no lock;
  ledger publication is release-ordered; allocation bounded (fixed-size
  records)
Errors and failure guarantee: named reasons — IdentityMismatch,
  FrameUnmapped, WakeTimeout, InterfaceFailure; failure never panics and
  never retries (P3 owns the consequence: CPU excluded from eligibility)
Security/authorization checks: GICR frame selected by affinity match to
  the calling pCPU (never by index into the frame list); Secure
  configuration untouched
Validation: W02-DV04/DV05 (BSP-local and AP-local evidence), DV06
  (ledger), DV08 (residuals)
```

### Pseudocode

```text
if ledger.local_decided(pcpu) -> return Err(already-decided)   # INV-4
frame = select_gicr_frame_by_affinity(own MPIDR affinity, expected)
  none/unmapped -> publish LocalFailed(FrameUnmapped); Err(...)
probe_redistributor_identity(frame, expected)      # W01 contract; read-only
  affinity mismatch / missing Last consistency
    -> publish LocalFailed(IdentityMismatch); Err(...)
waker_handshake(frame):                            # §2, bounded
  write GICR_WAKER ProcessorSleep=1
  poll ChildrenAsleep==1 (bounded) -> on timeout:
      write ProcessorSleep=0 (best-effort restore)
      publish LocalFailed(WakeTimeout); Err(...)
configure_local_baseline(frame):                   # §3
  survey+clear: read GICR_ISPENDR0/ISACTIVER0 -> log -> write CPEN/ICACTIVE
  disable all SGI+PPI: write GICR_ICENABLER0 = all-ones
  defaults: GICR_IPRIORITYR (lowest), GICR_IGROUPR0 = Group1
  poll GICR_CTLR completion (bounded)
cpu_interface_sequence():                          # §4 order, with ISB
  enable_and_verify_sre()  -> on failure: restore WAKER, publish
      LocalFailed(InterfaceFailure), Err(...)
  write ICC_CTLR_EL1 posture (EOImode=0, defaults); isb()
  write ICC_PMR_EL1 allow-all; isb()
enable_group1(): write ICC_IGRPEN1_EL1 = 1; isb()  # last action
publish LocalReady (release); emit gic.local_phase; Ok(readiness)
```

## 2. GICR wake handshake contract

```text
Name and stability: fn waker_handshake(frame) -> Result<(), WakeTimeout>
  (P6-internal; Phase B only)
Purpose and caller: quiesce the Redistributor's children before local
  configuration per the architectural WAKER protocol
Preconditions: frame confirmed; single local writer
Postconditions: success — ChildrenAsleep observed set, configuration done
  while asleep by the caller, ProcessorSleep cleared afterwards by the
  caller's restore step (this function performs the sleep-entry half; the
  caller sequence performs configure + wake-exit and verifies
  ChildrenAsleep cleared, bounded)
Errors: timeout → caller publishes LocalFailed(WakeTimeout); the restore
  path (§1 pseudocode) never leaves ProcessorSleep set on any exit
Concurrency: none beyond the single-writer rule
Validation: W02-DV05; sequence-level host test of the handshake state
  machine on the abstracted surface
```

The sleep-exit is a separate bounded step (`waker_release`, same contract
shape, reverse direction) so failure handling can call it independently —
no exit path leaves the Redistributor asleep.

## 3. Local baseline contract

```text
Name and stability: fn configure_local_baseline(frame) -> ResidualReport
  (P6-internal; Phase B only, while ProcessorSleep quiesced)
Purpose and caller: INV-1 ([01](01-scope-and-foundations.md)): after local
  bring-up every SGI and PPI is disabled, lowest priority, Group1 NS;
  residual pending/active state is cleared and logged
Postconditions: ISENABLER0 read-back == 0 across SGI+PPI; priorities
  default; IGROUPR0 set; pending/active clear; report logged
Non-responsibility: does NOT enable the maintenance PPI or timer PPIs;
  those are enabled later by their owning consumers through the W03
  registration surface — W02's disabled baseline is exactly what makes
  that ownership clean
Validation: W02-DV08 + local read-back verification in DV05 evidence
```

## 4. CPU-interface sequencing contract (fixed order)

```text
Name and stability: fn cpu_interface_sequence(local) ->
  Result<(), InterfaceFailure> (P6-internal; Phase B only)
Purpose and caller: establish the system-register interface posture in the
  one authorized order; consumers (W03 dispatch) rely on this posture
Order (fixed; reordering is a design change, not a tuning knob):
  1. SRE write-and-verify ([03](03-code-contracts-register-access.md) §5.1)
  2. ICC_CTLR_EL1: EOImode=0, no preemption grouping, implemented
     defaults; ISB
  3. ICC_PMR_EL1: allow-all (lowest masking); ISB
  4. (later, after distributor Enabled is re-asserted by ledger) ICC_
     IGRPEN1_EL1 enable; ISB — written by enable_group1() as the final
     local action
Rationale: SRE must precede any ICC access; PMR must not mask the
  interface the moment group-enable makes interrupts visible; group
  enable last guarantees no interrupt is taken against a half-configured
  interface (consumer registration happens only after readiness)
Errors: InterfaceFailure at any step; no partial posture is published
Validation: W02-DV04; W03 consumes the posture (combined-EOI rule)
```

## 5. Ledger publication contract

```text
Name and stability: fn publish_local(readiness | failure) (P6-internal;
  write-once per pCPU)
Purpose and caller: the only cross-CPU visibility of Phase-B results
Preconditions: called exactly once per pCPU; caller is the owning pCPU
Postconditions: release-ordered slot write; subsequent queries by W03/W04
  observe the final value with acquire semantics
Errors: double publication is an invariant violation (debug assert +
  diagnostic), never a silent overwrite
```

## 6. Non-responsibilities restated

Phase B does not: touch GICD registers (Phase A owns them); enable any
consumer interrupt; run in IRQ context; decide P3 consequences of failure;
perform FIQ handling; or read another pCPU's GICR frame. A pCPU may not
run Phase B twice; a "retry" is a reboot-level event owned elsewhere.
