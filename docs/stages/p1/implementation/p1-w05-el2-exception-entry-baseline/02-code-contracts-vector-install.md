# P1-W05 Vector Install Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W05 detailed design](README.md).

Pseudocode is an outline, not runnable production code. Exact bit positions
and instruction encodings follow the architecture revision recorded in the
implementation record (the discipline of
[W04's write contracts](../p1-w04-el2-architectural-state-baseline/02-code-contracts-control-writes.md)).
All names are internal boot-scope items owned by this design; they are not
ABI and may be renamed only by a recorded design change. No allocation
anywhere; all fixed-size data.

## 1. Phase body — mechanism entry

```text
Name and stability: fn install_el2_exception_entry(); internal; stable
  within P1. (Naming rule: W09 owns the `exceptions_step` adapter name; the
  supplying mechanism deliberately does not reuse it — W03 §6 precedent.)
Purpose and caller: the `exceptions` phase's entire mechanism — assert,
  install, verify, declare. Caller: run_init_sequence via the W09
  `exceptions_step` adapter (W09 §8: "Vector installation;
  unexpected-event classification").
Inputs / outputs: none; on normal return vectors are installed, verified,
  and declared; on failure it does not return.
Preconditions / postconditions: W09 phase prerequisites (el2-baseline
  complete; EL2_BASELINE declared). Postcondition: VBAR_EL2 holds the table
  base (read-back verified); VECTOR_STATE reports Established; the table is
  live for every category and origin.
State and ownership change: VBAR_EL2 (owner: W05, taken here); the
  declaration static; nothing else.
Concurrency/allocation context: boot context; DAIF masked; no allocation.
Errors and failure guarantee: assertion or read-back failure routes via
  VectorError -> panic route, phase-attributed `exceptions` (W09 matrix
  row); no retry, no rollback (W09 T2/T3).
Security/authorization checks: none as a runtime decision; the install is
  itself the completion of the stage's fault-containment posture.
Logic:
  assert_preconditions()                    # §3 below
  base = &p1_el2_vector_table as address    # table of [vector install] §4
  vbar_write(base)                          # audited boundary, §5
  observed = vbar_read()                    # read-back, W04 decision 6 pattern
  if observed != base: fail_vector(VectorError { control: VBAR, expected, observed })
  isb()                                     # synchronize vector fetch path
  declare_established()
Validation: W05-DV04 (baseline integration), W05-DV02 (coverage review).
```

## 2. `VBAR_EL2` write and read-back

```text
Name and stability: the VBAR_EL2 install step of §1; internal; stable
  within P1.
Purpose and caller: take explicit ownership of the vector base per the W04
  ownership matrix. Caller: the phase body, exactly once per boot.
Inputs / outputs: the table base address; no return value (failure routes).
Preconditions / postconditions: executing at EL2; table is 2 KiB aligned
  (the architectural VBAR_EL2 alignment: low bits RES0 — enforced by the
  alignment attribute of the table symbol and checked at build/link review).
  Postcondition: VBAR_EL2 == table base.
State and ownership change: VBAR_EL2 only.
Concurrency/allocation context: boot context; no allocation; the write is
  context-synchronizing; `isb` after the write before any exception is
  expected to vector through the new base (recorded implementation note —
  the same no-extra-barrier discipline as W04 §7, with the one `isb` this
  fetch-path change requires).
Errors and failure guarantee: mismatch -> fail_vector; the write itself
  cannot fail.
Security/authorization checks: none; the value written is the linked table.
Safety justification (P0 unsafe inventory): VBAR_EL2 has no typed accessor
  the compiler can check; the primitive writes exactly one named register
  with a caller-supplied table base whose alignment is a reviewed invariant.
Validation: W05-DV04; ownership review (W05-DV05) confirms no second writer.
```

## 3. Baseline assertions (W04 integration — plan work seq 3)

```text
Name and stability: fn assert_preconditions(); internal; stable within P1.
Purpose and caller: verify the W04-declared categories the vector path
  builds on before the first write. Caller: the phase body, first step.
Inputs / outputs: none; returns or routes.
Preconditions / postconditions: none beyond its checks; on success the
  asserted statuses are recorded in the implementation evidence.
State and ownership change: none (reads the declaration API only — never
  re-reads control registers; W04 decision 8: recorded status, not live
  re-reads).
Concurrency/allocation context: boot context; no allocation.
Errors and failure guarantee: any NotEstablished category routes via
  fail_vector with the category identity — phase-attributed `exceptions`.
Security/authorization checks: none.
Asserted set (and why each is load-bearing here):
  C1 (SPSel=1, DAIF all-masked)   — the SPx-origin premise of §2 of the
                                    architecture file; masks premise of the
                                    IRQ/FIQ classification.
  C2 (HCR IMO/FMO/AMO=0)          — "no routing surprise" premise of the
                                    unexpected-event classification.
  C3 (HCR_EL2 = RW only)          — no trap groups: synchronous faults are
                                    genuinely unexpected, not emulated traps.
  C4 (CPTR_EL2.TFP=1)             — the FP-free capture premise.
Logic: for each category in the set: baseline_status(cat) == Established,
  else fail_vector.
Validation: W05-DV04; W04-DV07 consumability reads the same point.
```

## 4. Vector table region and symbols

```text
Name and stability: p1_el2_vector_table (region symbol), __p1_vectors_start
  / __p1_vectors_end (boundary symbols for the W08 region inventory);
  assembly + linker placement; internal; stable within P1.
Purpose and caller: the 16-entry table the hardware fetches through
  VBAR_EL2; the region identity W08 maps. Callers: hardware (fetch); the
  phase body (base); W08 (region inventory).
Inputs / outputs: none; a placed, read-only-at-runtime region.
Preconditions / postconditions: exactly 0x800 bytes (16 entries × 32
  instructions... layout per the architecture: 16 entries of 0x80 bytes);
  2 KiB aligned; contents fixed at link time; no absolute long addresses
  that assume a specific VA (position-local branches only, so the table is
  correct under any mapped VA W08 chooses).
State and ownership change: none at definition.
Concurrency/allocation context: read-only after install; no allocation.
Errors and failure guarantee: a layout/alignment violation is a link-time
  defect caught by review (W05-DV01), not a runtime path.
Security/authorization checks: none.
Layout: entry i (i = 0..15) encodes origin = i / 4 and category = i % 4 per
  the architectural offset rule; each entry is a single branch to the shared
  capture entry with its origin/category constants materialized (see
  [entry capture](03-code-contracts-entry-capture.md) §1).
Validation: W05-DV01 (16 entries, alignment, no fall-through); region
  symbols cross-checked against W08's inventory (W05-DV07).
```

## 5. Audited `unsafe` boundary

```text
Name and stability: fn vbar_write(base), fn vbar_read() -> u64,
  fn exception_state_read(Reg) -> u64 (Reg ∈ { ESR_EL2, ELR_EL2, SPSR_EL2,
  FAR_EL2, HPFAR_EL2 }), fn exception_barrier_isb(), and the minimal
  branch/stop asm helpers used by the entry path; internal; stable within
  P1.
Purpose and caller: the only architectural system-register and barrier
  access in W05. Callers: the phase body (§1–§2); the capture entry (§2 of
  the capture contracts); routing (isb not required there — listed for
  completeness of the boundary).
Inputs / outputs: register-scoped reads/writes only; the Reg set is closed
  — no other register is nameable, so no other access is expressible (the
  W04 ControlId precedent, scoped to the exception set).
Preconditions / postconditions: executing at EL2 (phase precondition for
  the install primitives; the exception primitives execute only inside the
  exception path after install).
State and ownership change: the accessed register only.
Concurrency/allocation context: boot or exception context; no allocation.
Errors and failure guarantee: cannot fail; failure semantics belong to the
  layers above (§1, capture, routing).
Security/authorization checks: none; reads reflect machine facts.
Safety justification (P0 unsafe inventory): the compiler cannot type
  `msr`/`mrs`/`isb`; the closed register sets restrict access to the
  recorded controls; each primitive's callers are the contracted paths of
  this design. HPFAR_EL2 is read only when the syndrome table marks it
  valid (see [classification](04-code-contracts-classification-routing.md)
  §2) — reading it unconditionally would be architecturally meaningless in
  P1 (Stage-2 disabled) and is prohibited by the capture contract.
Validation: W05-DV05 boundary review — the inventory contains exactly these
  primitives and no raw asm elsewhere.
```

## 6. Declaration API

```text
Name and stability: VectorStatus (Unestablished | Established); static
  VECTOR_STATE in a once-write cell; fn vector_status() -> VectorStatus;
  fn vector_base() -> Option<u64>; all internal; stable within P1.
Purpose and caller: the recorded fact that vectors are live — the seam
  W06 (exception-context callability premise), W07 (arming premise), W08
  (region premise), W09 (unowned-window narrowing), and W11 (scenario
  windows) consume. Callers: the phase body (writer); the consumers
  (readers).
Inputs / outputs: status queries only; no control re-reads.
Preconditions / postconditions: reads before install return
  Unestablished / None (honest default, never a fabricated Established).
  The Established transition happens exactly once.
State and ownership change: the declaration static only; monotone within a
  boot.
Concurrency/allocation context: the audited once-write cell (the shared
  pattern family: W02 BootContext, W03 CAPABILITIES, W04 EL2_BASELINE;
  SAFETY: single boot CPU, DAIF masked, one boot path); readers from boot
  or exception context; no allocation.
Errors and failure guarantee: cannot fail; misuse observes the honest
  unestablished state.
Security/authorization checks: none; declaration is knowledge, not
  authority.
Logic: establish() sets the status inside the audited cell after §2's
  read-back succeeds; queries read the cell.
Validation: W05-DV04; consumer reviews (W05-DV07) read the API, not the
  registers.
```

What the declaration does **not** promise: no return path, no IRQ dispatch,
no recovery, no statement about pre-install exceptions (the W09 unowned
window stands until this phase completes), and no statement that the table
base is a permanent VA (W08's transition contract governs its continuity).
