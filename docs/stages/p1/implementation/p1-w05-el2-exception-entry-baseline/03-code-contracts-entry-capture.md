# P1-W05 Entry Capture Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W05 detailed design](README.md).

Pseudocode is an outline, not runnable production code. The entry assembly
is the only assembly W05 authorizes besides the boundary helpers of
[vector install](02-code-contracts-vector-install.md) §5; every instruction
outside these contracts is a scope violation. No allocation; no stack
assumptions beyond the running boot stack (the path never grows it); no
FP/SIMD anywhere (the W04 C4 deny posture and the no-FP build guarantee
make any such state impossible in P1 code — the capture must not assume
more).

## 1. Vector stubs and the shared capture entry

```text
Name and stability: p1_el2_vector_table (16 stubs) branching to
  p1_exception_capture_entry; assembly, internal; stable within P1.
Purpose and caller: give every (origin, category) pair a valid entry path
  that lands in the capture entry with its coordinates materialized.
  Caller: the hardware, through VBAR_EL2.
Inputs / outputs: the taken exception (hardware state); no outputs except
  control reaching the capture entry.
Preconditions / postconditions: vectors installed and VBAR_EL2 pointing
  here (phase postcondition). Postcondition: the capture entry is entered
  with the stub's (origin, category) constants in agreed scratch registers;
  no stub executes more than its branch.
State and ownership change: none in the stubs.
Concurrency/allocation context: exception context; no allocation; no stack
  use before the guard decides (a stack-overflow fault must not deepen the
  problem before R1 of the state machine).
Errors and failure guarantee: no stub can fail; it is a branch.
Security/authorization checks: none; entry geometry is not a trust decision
  in P1 (nothing below EL2 executes).
Logic (per stub, 16 times — shown once):
  <stub origin=k, category=c>:
    mov x_scratch0, #k          # OriginClass constant
    mov x_scratch1, #c          # ExceptionCategory constant
    b   p1_exception_capture_entry
Validation: W05-DV01 coverage review — all 16 stubs present, each stamping
  its coordinates; no fall-through entry.
```

Origin/category constants follow the architectural vector-offset rule
(origin = offset / 0x200, category = (offset / 0x80) % 4); the mapping is
recorded in the implementation record against the recorded architecture
revision.

## 2. `ExceptionFrame` — bounded diagnostic context (work seq 2)

```text
Name and stability: ExceptionFrame; internal struct; fixed size; stable
  within P1. One static instance CAPTURED_FRAME in a plain static (guard-
  protected; not a once-cell — the slot is written only inside the guarded
  exception path).
Purpose and caller: the bounded capture P1-V09 requires — enough context to
  locate the failure, bounded by construction. Callers: the capture entry
  (writer); W07's report seam and W05's pre-arm summary (readers).
Inputs / outputs: written from live state at entry; fields:
  x: [u64; 31]        x0..x30 at entry
  sp: u64             SP_EL2 at entry
  pc: u64             ELR_EL2 (the faulting/return address)
  spsr: u64           SPSR_EL2 (return state)
  esr: u64            ESR_EL2 (syndrome: EC/IL/ISS raw)
  far: u64            FAR_EL2; valid only when frame.far_valid
  hpfar: u64          HPFAR_EL2; valid only when frame.hpfar_valid
  far_valid: bool     per the syndrome table's FAR-validity flag
  hpfar_valid: bool   per the syndrome table; in P1 expected never valid
                      (Stage-2 disabled — recorded, not assumed)
  origin: u8          OriginClass stamped by the stub
  category: u8        ExceptionCategory stamped by the stub
Preconditions / postconditions: written only by the capture entry with the
  guard held; readers run after routing hands the frame over (same-path
  code or W07's seam); no reader may observe a partially written frame
  because the writer does not return before the frame is complete and
  routing runs on the same CPU with masks set.
State and ownership change: the slot's contents; nothing else.
Concurrency/allocation context: exception context; no allocation; single
  CPU; guard-held.
Errors and failure guarantee: capture cannot fail (plain moves from
  registers); a fault *during* capture hits the guard (R1: bounded stop).
Security/authorization checks: the frame reflects machine state only; no
  memory is dereferenced to enrich it (no stack crawl, no symbolization).
Capture logic (capture entry):
  if guard already set: bounded_stop()                 # R1
  set guard
  save x0..x30, sp onto the frame                      # order fixed
  esr = read ESR_EL2; pc = read ELR_EL2; spsr = read SPSR_EL2
  cls = syndrome_class(esr)                            # §2 of classification
  far = read FAR_EL2 if cls.far_valid else 0
  hpfar = read HPFAR_EL2 if cls.hpfar_valid else 0     # never in P1's set
  frame.origin = scratch0; frame.category = scratch1
Validation: W05-DV03 (field/coverage review); NC3/NC6 observe the frame's
  fields through the reports (W11 execution, deferred).
```

Sizing note: the fixed field set is the whole capture — no growth path is
implicit. Later packages that need more (e.g. a return path needing caller
state) extend the frame through their own recorded design (Reserved
trigger, parent README).

## 3. Prohibited content (review explicitness)

The entry path must not contain: any MSR/MRS outside the §5 boundary set;
any ERET, DRPS, or exception-return instruction (decision 4: no return path
exists); any interrupt-controller (GIC) register access or acknowledge/EOI
sequence; any DAIF-clearing or mask-modifying instruction; any FP/SIMD
instruction; any load or store addressed by fault-controlled registers
(enrichment is prohibited); any branch outside the module except the
capture entry and the bounded stop; any second output path (routing owns
output through §4 of the classification contracts). W05-DV05 walks this
list.

## 4. The bounded stop

```text
Name and stability: bounded_stop() -> !; the terminal instruction sequence
  of the exception path; internal; stable within P1 (same discipline family
  as W02's panic-route stop and W01's rejection stop).
Purpose and caller: the defined terminal outcome of every disposition.
  Callers: routing (after the pre-arm summary or the W07 report returns
  control, which its contract says it does not — the stop is the
  belt-and-braces terminal); the guard-true recursion path.
Inputs / outputs: none; never returns.
Preconditions / postconditions: none; masks remain set; no further output.
State and ownership change: none.
Concurrency/allocation context: exception context; no allocation.
Errors and failure guarantee: cannot fail; execution halts observably for
  the harness (W10's timeout/outcome classification owns the wall-clock
  interpretation).
Security/authorization checks: none.
Logic: branch-to-self (the W01/W02 terminal discipline).
Validation: W05-DV05; observed by every executed fault scenario (W11).
```
