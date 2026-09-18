# P6-W02 Code Contracts — Register Access Surface

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P6-W02 detailed design](README.md).  
**Convention:** checklist §3 contract template. This file defines the *only*
authorized `unsafe` surface for W02. The distributor and local-GIC contracts
([04](04-code-contracts-distributor.md),
[05](05-code-contracts-local-gic.md)) build on it and may add no other
hardware access. Pseudocode is design logic, not runnable code.

## 1. Hardware rules binding this surface (Coding Guidelines)

- Every MMIO access is volatile; the compiler may not elide, merge, or
  reorder the program-ordered access sequence.
- Reserved-bit handling per register class per
  [01 §4](01-scope-and-foundations.md): reads mask to implemented fields;
  RMWs preserve unknown bits; writes zero reserved-as-zero fields; nothing
  is assumed about implementation-defined fields.
- Barriers: system-register writes that affect instruction execution or
  ordering are followed by instruction synchronization (`ISB`);
  configuration whose effect must be visible before subsequent operations
  is separated by data synchronization (`DSB`) as the sequences state
  explicitly. The surface exposes barrier helpers; it does not insert
  hidden barriers — sequencing is explicit at the call sites.
- One owner per state transition: the surface provides mechanisms; the
  sequences of [04](04-code-contracts-distributor.md) /
  [05](05-code-contracts-local-gic.md) are the only callers authorized to
  perform state-changing GIC writes.
- Interrupt IDs are typed: P0 newtype baseline IDs (`PpiId`, `SgiId`,
  `SpiId`, and the W03 `PhysicalIntId` family); this surface never takes a
  naked `u32` interrupt number.
- All waits are bounded; no busy-wait without a timeout bound and a named
  timeout outcome.

## 2. `unsafe` boundary and inventory

```text
Name and stability: module gic-regaccess (the sole W02 unsafe surface;
  P6-internal, not a public API)
Purpose and caller: map GIC frames once; provide volatile, masked register
  operations to the W02 sequences (and, by contract extension, to W03/W04
  through the same surface — no second surface may be created)
Inputs / outputs: PlatformInfo frame descriptors → frame handles; typed
  register operations
Preconditions / postconditions: a handle is created only from a
  W01-confirmed, in-bounds frame descriptor; every operation's offset is
  frame-bounds-checked against the handle (a bug panics the hypervisor as
  an invariant violation in host builds and is unreachable by construction;
  the check is a debug + release invariant per the P0 failure-classification
  baseline)
State and ownership change: mappings live for boot lifetime; no unmap
Concurrency/allocation context: mappings created before SMP release; handles
  are Share; operations are stateless and interrupt-context-safe (single
  instruction volatile accesses)
Errors and failure guarantee: out-of-bounds offset / null mapping →
  invariant-violation diagnostic path (never silent)
Security/authorization checks: callers are the W02 sequences; the module is
  not exported beyond the interrupt-subsystem boundary
SAFETY requirements (each unsafe block carries a nearby SAFETY comment):
  address validity derives from the bounds-checked handle; volatile
  semantics per access; no reference formation, no static mut, no transmute
```

The `unsafe` inventory delta from this module is reported per the P0 unsafe
governance contract and reviewed in W02-DV09.

## 3. Register-operation contracts

### 3.1 `read_reg` / `write_reg`

```text
Name and stability: fn read_reg(frame: &FrameHandle, reg: RegOfs) -> u32;
  fn write_reg(frame: &FrameHandle, reg: RegOfs, value: RegValue)
  (P6-internal; volatile single access)
Purpose and caller: full-register volatile access for registers whose
  semantics are write-value or read-whole (e.g. WAKER, IAR-class, ISPENDR
  set/clear where the written value is the complete intent)
Inputs / outputs: frame handle + typed register offset → value / effect
Preconditions / postconditions: offset in frame bounds; value already
  masked by the caller per the register's reserved-bit class
Errors and failure guarantee: bounds violation → invariant path (§2)
Logic: volatile pointer read/write at handle.base + reg.ofs; alignment is
  guaranteed by the offset type (4-byte for word registers; the 64-bit
  IROUTER-class accessor of [W04](../p6-w04-smp-interrupt-routing-sgi/README.md)
  uses an 8-byte-aligned variant defined there, not here)
Validation: host-side unit tests with a fake frame backend verify masks and
  call ordering; hardware behavior validated only by W02-DV rows
```

### 3.2 `read_modify_write`

```text
Name and stability: fn read_modify_write(frame, reg, field_mask: u32,
  set_bits: u32) (P6-internal)
Purpose and caller: set/clear implemented fields while preserving unknown
  and reserved bits (e.g. CTLR-class configuration)
Preconditions / postconditions: set_bits ⊆ field_mask; RMW is a single
  read then single write; no retry loop (callers hold the transition
  ownership that makes the register single-writer)
Logic: v = read; v = (v & !field_mask) | set_bits; write(v)
Concurrency: NOT atomic against other masters — legal only under the
  single-writer rule of the owning sequence; W04's routed-change path
  holds the distributor lock across its RMW
Validation: mask-preservation unit tests with the fake backend
```

### 3.3 `poll_until` (bounded)

```text
Name and stability: fn poll_until(frame, reg, mask, expected: u32,
  budget: PollBudget) -> Result<(), PollTimeout> (P6-internal)
Purpose and caller: bounded completion waits (GICD enable/disable
  completion RWP-class bits, GICR WAKER handshake)
Preconditions / postconditions: budget is a design-fixed bounded count /
  time source chosen at implementation from the P1/P3 time-basis
  contracts; on timeout the caller executes its named failure outcome —
  the surface never decides success
Logic: loop { if (read & mask) == expected -> Ok; dec budget; on 0 ->
  Err; brief relax hint }
Validation: fake-backend timeout test; real behavior via DV rows
```

### 3.4 Barrier helpers

```text
Name and stability: fn dsb(), fn isb() (P6-internal wrappers around the
  established arch barrier primitives; no new barrier semantics invented)
Purpose and caller: explicit synchronization at sequence-declared points;
  the arch primitives themselves are owned by the P1 designs
Rule: this surface exposes them so sequences state barriers inline;
  sequences must not rely on implicit ordering between MMIO and system
  registers (none exists architecturally for cross-interface observation)
```

## 4. Register classes and reserved-bit table (Required content)

The implementing change must contain a register-class table (code-adjacent
documentation) enumerating, for every register this design touches:
architectural name, offset class, access width, reserved-bit rule,
reset-expectation, and the sequence(s) that may write it. Required rows at
minimum:

| Register (class) | Touched by | Reserved-bit / write rule highlights |
|---|---|---|
| GICD_CTLR | Phase A | RMW on implemented NS-view fields; completion polling on the RWP-class bit after writes |
| GICD_TYPER / PIDR2-class identity | Phase A probe | read-only |
| GICD_I{SEN,CEN,SPEN,CPEN}ABLER<E> / ISACTIVER / ICACTIVER | Phase A | set/clear-by-writing-1s registers; write only bits of the supported SPI range |
| GICD_IPRIORITYR<E> | Phase A | byte-lane writes within word accesses; default lowest priority |
| GICD_IGROUPR<E> / IGRPMODR<E> | Phase A | NS-view group assignment; never touch Secure-view-only configuration |
| GICD_IROUTR<L> (64-bit) | Phase A initial routing; W04 changes | 8-byte aligned; write full 64-bit value with reserved fields zero |
| GICR_CTLR | Phase B | RMW implemented fields; RWP-class completion poll |
| GICR_TYPER / GICR_PIDR-class | Phase B probe + Phase A coverage read | read-only |
| GICR_WAKER | Phase B | strict handshake order (ProcessorSleep → ChildrenAsleep poll → configure → reverse); never leave ProcessorSleep set on any exit path |
| GICR_I{SEN,CEN,SPEN,CPEN}ABLER0 / ISACTIVER0 / ICACTIVER0 | Phase B | IDs 0–31 (SGI+PPI) only; write-1s semantics |
| GICR_IPRIORITYR<S/P> | Phase B | defaults; byte-lane rule as GICD |
| GICR_IGROUPR0 | Phase B | Group1 assignment for SGI/PPI |
| ICC_SRE_EL2 | Phase B | write-and-verify (§5.1); failure is a local bring-up failure |
| ICC_CTLR_EL1 | Phase B | EOImode=0 posture ([README decision 5](README.md)); RMW preserved fields |
| ICC_PMR_EL1 | Phase B | allow-all initial value; W10/W05 adjust later through their own authority |
| ICC_IGRPEN1_EL1 | Phase B | enabled last in the local sequence; ISB after |

Numeric bit positions and reset values are transcribed from the pinned GIC
specification revision into the table at implementation (W01 step-1 record);
this design fixes names, ownership, and rules only.

## 5. System-register sequencing rules (Required)

- SRE-first rule: no ICC_CTLR/PMR/IGRPEN access is permitted before the
  SRE write-and-verify succeeds on that pCPU.
- Write-and-verify pattern: write intended bits → ISB → read back → compare
  against the posture requirement; mismatch is a failure, never ignored
  (firmware may hold some fields RAO/WI; the verify rule distinguishes
  "enabled as required" from "silently ignored").
- Group-enable-last rule: ICC_IGRPEN1_EL1 is written only after the
  distributor is `Enabled` and the local baseline is configured; it is the
  final local action before readiness publication, followed by ISB.
- No ICC register is accessed from a context other than its owning pCPU.

### 5.1 SRE enablement contract

```text
Name and stability: fn enable_and_verify_sre(local: &mut LocalGicContext)
  -> Result<(), InterfaceFailure> (P6-internal; Phase B only)
Purpose and caller: establish the system-register CPU interface
Preconditions: Phase A published DistributorReady; own GICR confirmed;
  executing on the owning pCPU with interrupts disabled
Postconditions: SRE posture verified per pinned revision, or
  InterfaceFailure (never a silent continuation on a memory-mapped-only
  interface — that configuration is Unsupported per the W01 posture)
Errors and failure guarantee: on failure the caller routes to
  LocalFailed(interface); no ICC register is further touched
Logic: write SRE+Enable bits; isb(); read back; verify required bits;
  record observation in the local context
Validation: W02-DV04 scenario (failure path exercised via probe-failure
  injection in host-side sequence tests where the surface is abstracted)
```

## 6. Layering rules

The surface depends on: P0 newtype/address baselines, the arch barrier
primitives, and platform frame descriptors. It must not depend on any board,
SoC, or QEMU constant; fixture frames in tests are constructed values. QEMU
bring-up success does not relax any rule in this file (Coding Guidelines:
QEMU is not evidence that volatile/barrier/reserved-bit rules can be
omitted).
