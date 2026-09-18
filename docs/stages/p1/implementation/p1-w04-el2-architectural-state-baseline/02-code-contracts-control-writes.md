# P1-W04 Control-Write Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W04 detailed design](README.md).

Pseudocode is an outline, not runnable production code. Values are recorded
as mask/write pairs per parent README decision 2; exact bit positions follow
the architecture revision recorded in the implementation record (same
discipline as W03's decode references). Every raw `msr`/`mrs` pair belongs
to the audited-`unsafe` boundary listed in §3.

## 1. Establishment entry

```text
Name and stability: fn establish_el2_baseline(); internal; stable within P1.
Purpose and caller: the `el2-baseline` phase's entire mechanism — assert,
  write (guarded), verify, declare. Caller: run_init_sequence via the W09
  adapter (W09 §8: "EL2 baseline mechanism").
Inputs / outputs: none; on normal return every C1–C8 category is
  Established and declared; on failure it does not return.
Preconditions / postconditions: W09 phase prerequisites (capabilities
  complete; CAPABILITIES published); execution state per C1. Postcondition:
  the recorded baseline values hold and are declared.
State and ownership change: the C2–C8 registers per §2; the declaration
  static of [03-code-contracts-readback-and-declaration.md](03-code-contracts-readback-and-declaration.md)
  §4; nothing else.
Concurrency/allocation context: boot context; DAIF masked; no allocation.
Errors and failure guarantee: any verify mismatch or guard inconsistency
  routes via BaselineError -> panic route, phase-attributed `el2-baseline`
  (W09 matrix row); categories after the failure point stay
  NotEstablished; no retry, no rollback.
Security/authorization checks: the baseline is itself the security posture
  (deny-by-default); it adds no runtime authorization decision.
Logic:
  assert_preconditions()                       # C1 read-back asserts
  for spec in BASELINE_SPECS (C2..C8 order):
    if spec.guard is Some(fact)
       and CAPABILITIES.query(fact).observation != Present:
      record_skip(spec); continue              # guard decision per §3 of the
                                               # read-back contracts
    value = spec.apply(read(spec.reg))         # RMW-with-mask or constant
    write(spec.reg, value)
    observed = read(spec.reg) & spec.mask
    if observed != (value & spec.mask):
      fail_baseline(spec.control, expected, observed)   # diverges
  declare_established()
Validation: W04-DV02 (values/masks), W04-DV03 (routes), W04-DV05 (order).
```

## 2. Write specifications

Each specification: control, form (RMW mask / full constant), recorded
value, rationale, consumer. The C1 entries are asserts (read-back only,
owner W02).

| # | Control | Form | P1 value (recorded) | Rationale | Consumer |
|---|---|---|---|---|---|
| C1a | `SPSel` | assert | 1 | EL2-stack discipline (W02 stage 2) | W05 (exception stacks), W08 |
| C1b | `DAIF` | assert | all-masked | no interrupt delivery in P1 | W05, W09 (unowned window boundary) |
| C2 | `HCR_EL2` | full constant | `RW=1`, all other bits 0 (IMO/FMO/AMO=0: no interrupt routing/trapping; VM=0: Stage-2 off; TGE=0) | one write covers C2, C3, C7 (RW), C8 (VM) posture | W05 (routing context), W08 (VM=0 premise), P4+ (supersedes) |
| C4a | `CPTR_EL2` | RMW mask | `TFP=1` (trap FP/SIMD at EL2) | deny-by-default; loud fault over silent residue | W05 (save areas must not assume FP); the no-FP build guarantee is its complement |
| C4b | `CPACR_EL1` | RMW mask | `FPEN=00` (deny FP/SIMD at EL1/EL0) | EL1/EL0 preparation; residue removal | future EL1-entry stage |
| C5a | `MDCR_EL2` | RMW mask | `TDA=1, TDOSA=1, TDE=1, TPM=1, TPMCR=1` (trap debug/OS-lock/PMU accesses from lower ELs; route EL0/EL1 debug exceptions to EL2) | debug/performance deny-by-default | W05 (these faults become diagnosable), P6+ (PMU policy supersedes) |
| C5b | `MDSCR_EL1` | full constant | 0 (within RES1 constraints — written as the recorded constant) | software debug stateless; residue removal | future debug work |
| C6a | `CNTHCTL_EL2` | RMW mask | EL1 physical-timer access gates = 0 (EL1 timer accesses trap) | timer access deny-by-default at EL1 | P6 (supersedes with virtualization policy) |
| C6b | `CNTKCTL_EL1` | RMW mask | 0 within mask (EL0 virtual-timer/event access denied) | EL0 preparation; residue removal | P6 |
| C6c | `CNTHP_CTL_EL2` | full constant | `ENABLE=0, IMASK=1` (EL2 physical timer disabled and masked — it must not fire into an interrupt-less stage) | the EL2 timer must never surprise P1 | P6 |
| C6d | `CNTHV_CTL_EL2` | full constant, **guarded** by `El2VirtualTimer` | `ENABLE=0, IMASK=1` | same as C6c for the optional timer | P6; skipped with `SkippedAbsent` when the fact is Absent |
| C7 | `SCTLR_EL1` | RMW mask | `M=0, C=0, I=0` (no EL1 MMU/cache residue); all other bits preserved | EL1/EL0 preparation without inventing EL1 policy | future EL1-entry stage |
| C8a | `SCTLR_EL2` | RMW mask | `M=0, C=0, I=0` (MMU and caches off at EL2); other bits preserved | the pre-MMU premise W08 requires; no mapping promise | W08 (enables through its design) |
| C8b | `VTCR_EL2` | full constant | 0 (within RES1 constraints) | Stage-2 translation residue removal | P4 (supersedes) |
| C8c | `VTTBR_EL2` | full constant | 0 | Stage-2 residue removal | P4 |

Notes: (a) C2's single `HCR_EL2` write intentionally carries the posture of
three categories — the matrix of
[01-architecture-and-state.md](01-architecture-and-state.md) §2 records that
mapping; (b) EL1-register writes (C4b, C5b, C6b, C7) are direct EL2 writes
with no trap expected under C3's no-trap policy — a trap there would itself
be a diagnosable baseline failure inside the unowned window, recorded as a
known limitation, not mitigated; (c) no barrier is required between these
writes and their read-backs (context-synchronizing `msr`/`mrs` pairs),
recorded as an implementation note per
[01-architecture-and-state.md](01-architecture-and-state.md) §7.

## 3. The audited `unsafe` boundary

One `unsafe` primitive per direction, called by the specification engine:

```text
Name and stability: fn read_sysreg(reg: ControlId) -> u64 and
  fn write_sysreg(reg: ControlId, value: u64); internal; stable within P1.
Purpose and caller: the only architectural system-register access in W04.
  Caller: the specification engine of §1.
Preconditions / postconditions: executing at EL2 (phase precondition);
  ControlId enumerates exactly the §2 controls — no other register is
  nameable, so no other access is expressible.
State and ownership change: the written register only.
Concurrency/allocation context: boot context; no allocation.
Errors and failure guarantee: cannot fail; the verification layer (§1)
  owns failure semantics.
Security/authorization checks: none; posture-setting is the baseline's
  purpose.
Safety justification (P0 unsafe inventory): the compiler cannot type
  `msr`/`mrs`; the ControlId set restricts access to the recorded baseline
  controls; executed only inside the establishment body's preconditions.
Validation: W04-DV04 boundary review — the inventory contains exactly these
  two primitives.
```

## 4. Precondition assertion

```text
Name and stability: fn assert_preconditions(); internal; stable within P1.
Purpose and caller: verify C1 (W02-owned execution state) and the W09
  phase prerequisites before the first write. Caller: establishment entry.
Inputs / outputs: none; returns or routes.
Preconditions / postconditions: none beyond its own checks; on success the
  C1 read-back values are recorded with the write log.
State and ownership change: none (asserts only).
Concurrency/allocation context: boot context; no allocation.
Errors and failure guarantee: mismatch routes via fail_baseline with the
  C1 control identity — phase-attributed `el2-baseline`.
Security/authorization checks: none.
Logic: read SPSel and DAIF; compare against the recorded W02 values
  (1; all-masked); mismatch -> fail_baseline.
Validation: W04-DV03; demonstrates the one-owner rule operationally.
```
