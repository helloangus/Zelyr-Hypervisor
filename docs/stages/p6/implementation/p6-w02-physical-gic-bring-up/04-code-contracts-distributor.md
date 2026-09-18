# P6-W02 Code Contracts — Distributor Bring-up (Phase A)

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P6-W02 detailed design](README.md).  
**Convention:** checklist §3 contract template. Phase A runs once on the
boot pCPU, before secondary release, with interrupts not yet enabled at the
CPU. All register operations go through
[03](03-code-contracts-register-access.md).

## 1. `bring_up_distributor`

```text
Name and stability: fn bring_up_distributor(decision: &GicCapabilityDecision,
  frames: &GicFrames) -> Result<DistributorReadiness, DistributorFailure>
  (P6-internal; sole writer of the global distributor lifecycle state;
  called by the established global-initialization owner after platform
  discovery and before secondary release)
Purpose and caller: produce the safe global Distributor state required by
  P6-V02; the initialization owner records the outcome
Inputs / outputs: W01 decision (grade + ExpectedGicIdentity) + mapped
  frames → readiness (or failure with reason)
Preconditions / postconditions:
  - decision grade ∈ {ReadyForP6, PhysicalOnly} (Rejected never reaches
    here — the initialization owner stops earlier)
  - executing on the boot pCPU in the pre-SMP-release window; interrupts
    not enabled at the CPU
  - success: state = Enabled and published; every supported SPI disabled,
    lowest default priority, Group1 NS assignment, determinate routing to
    the boot-pCPU affinity; GICD enabled with ARE per posture; completion
    observed after the final write
  - failure: state = DistributorFailed(reason); no interrupt enabled;
    quiesce performed if the failure occurred after the first write (§3)
State and ownership change: global distributor state transitions
  Unconfigured → Confirmed → Quiesced → Configured → Enabled per
  [02](02-architecture-and-state.md) §3.1; exactly once per boot
Concurrency/allocation context: single-threaded by boot rendezvous (P3);
  no lock required during Phase A; allocation bounded to the fixed SPI
  range bookkeeping
Errors and failure guarantee: named reasons — ProbeMismatch, Coverage,
  QuiesceTimeout, EnableTimeout, ConfigurationReject; the retained state is
  the last confirmed state; on any failure no consumer registration
  surface may be used (W03 consults the ledger)
Security/authorization checks: frame descriptors already validated; every
  write targets the confirmed frame; Secure-view registers never written
Validation: W02-DV02/DV03 (BSP bring-up evidence), W02-DV08 (residuals)
```

### Pseudocode

```text
probe_distributor_identity(frames)                    # W01 contract; read-only
  mismatch -> Err(ProbeMismatch{expected, observed})  # stop before 1st write
verify_gicr_coverage(frames, cpus.possible())         # GICR_TYPER reads only
  any possible-pCPU affinity without a frame -> Err(Coverage{affinity})
                                                      # Incomplete finding
write GICD_CTLR: clear enable fields                  # disable first
poll GICD_CTLR completion bit (bounded) -> Err(QuiesceTimeout) on timeout
survey_residuals(spi_range):                          # §2
  pending = read ISPENDR over supported range  -> log(count, sample)
  clear  = write ICPENDR over same range
  active = read ISACTIVER over supported range -> log; write ICACTIVER
configure_spi_range(spi_range, boot_affinity):        # §3
  disable: write ICENABLER over supported range
  priority defaults: write IPRIORITYR across range (lowest)
  group: IGROUPR=Group1, IGRPMODR=0 over range
  routing: for each supported SPI: write IROUTR = affinity(boot pCPU),
           IRM=0, reserved fields zero
enable: write GICD_CTLR ARE + EnableGrp1NS per pinned revision
poll completion (bounded) -> Err(EnableTimeout)
dsb()
verify_posture(): read back CTLR/one IROUTR/one priority; compare with
  expected identity -> Err(ConfigurationReject) on mismatch
publish DistributorReady (release); Ok(readiness)
```

## 2. Residual survey-and-clear

```text
Name and stability: fn survey_and_clear_spi_residuals(frames, spi_range)
  -> ResidualReport (P6-internal; Phase A only, while GICD disabled)
Purpose and caller: satisfy the residual-state acceptance scenario; stale
  firmware pending/active state must not survive into an enabled state
  with zero registered consumers (INV-1 of [01](01-scope-and-foundations.md))
Inputs / outputs: supported SPI range → bounded report (counts per class
  plus a bounded sample of IDs)
Preconditions: GICD disabled and completion observed
Postconditions: pending and active bits clear across the supported range
Concurrency: single-threaded (pre-SMP)
Errors: none (reads/writes only); unexplained repeated reappearance would
  be a hardware-anomaly diagnostic, not a loop (single pass only)
Logic: per register-word across the range: read ISPENDR → mask to
  supported IDs → accumulate → write ICPENDR with the same mask; repeat
  for ISACTIVER/ICACTIVER; log the report via gic-telemetry
Validation: W02-DV08 (QEMU with a firmware-residual fixture is not
  constructible; the scenario is validated by sequence-level host tests on
  the abstracted surface plus QEMU observation of a clean boot)
```

Rationale (recorded decision): clearing residual *pending* state before any
consumer exists is safe — an interrupt whose state is cleared here was
asserted before the hypervisor owned the platform and no in-flight consumer
exists to lose; the report preserves the diagnostic.

## 3. Failure-path contract (quiesce-on-failure)

```text
Name and stability: fn quiesce_on_failure(frames) (P6-internal; Phase A
  error path only)
Purpose and caller: on a failure after the first write, leave the GICD
  disabled and SPI-disabled so a later attempt (reboot) starts from the
  documented safe state
Preconditions: Phase-A failure reason already recorded
Postconditions: GICD disabled (best effort, bounded); no enable published
Errors: quiesce failure during failure handling is logged and swallowed
  (never masks the original failure reason)
Validation: covered by the failure-recovery row of the matrix
  ([07](07-validation-and-handoff.md) §2)
```

## 4. `DistributorReadiness` / `DistributorFailure`

```text
Name and stability: DistributorReadiness { spi_range: SupportedRange,
  default_route: Affinity, config_record: ConfigFacts }
  DistributorFailure { reason: named enum, detail }
  (P6-internal; readiness fact mirrored into the ledger global slot)
Purpose: W03 reads the enabled/quiet boundary (spi_range defines the
  classify space; W04 reads default_route as the state it may change)
Postconditions on success: facts match the W01 ExpectedGicIdentity
Validation: DV02/DV03 review; consistency check against W01 record
```

## 5. Non-responsibilities restated

Phase A does not: enable any interrupt; configure any GICR; touch any ICC
system register; implement routing *policy* (the initial determinate route
is a fixed initial state, changeable only by W04); handle FIQ (P1 entry
diagnostics own the unexpected-FIQ path); or decide boot behavior on
`Rejected` capability grades (the initialization owner does, per the W01
entry contract).
