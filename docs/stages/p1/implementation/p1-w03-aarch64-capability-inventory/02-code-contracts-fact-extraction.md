# P1-W03 Fact Extraction Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W03 detailed design](README.md).

All names are internal boot-scope items owned by this design. Pseudocode is
an outline, not runnable production code. No allocation; every function is
pure over its inputs except the architectural reads themselves.

## 1. Raw register reads (the audited `unsafe` boundary)

```text
Name and stability: one function per source register, named
  read_<register>() -> u64 (read_mpidr, read_id_aa64pfr0, read_id_aa64mmfr0,
  read_id_aa64mmfr1, read_cntfrq, read_current_el); internal; stable within
  P1.
Purpose and caller: the only architectural-register access in W03. Callers:
  the typed extraction functions of §2.
Inputs / outputs: none; each returns the raw register value.
Preconditions / postconditions: executing at EL2 with DAIF masked (the W02
  establishment invariant); each source is a pure identification register —
  no read has a side effect, and none can trap at EL2.
State and ownership change: none.
Concurrency/allocation context: no allocation; no synchronization; callable
  from boot context only (the phase body's caller contract).
Errors and failure guarantee: cannot fail; an Unsupportable read would be an
  architecture violation outside P1's owned surface (W09 unowned-window
  limitation covers the pre-vector window).
Security/authorization checks: none; the sources self-describe the CPU.
Safety justification (per function, for the P0 unsafe inventory): the
  compiler cannot type an `mrs`; the register is architecturally accessible
  at EL2 (ARM ARM identification-register accessibility rules); side-effect-
  free per the same rules; executed only under the phase body's preconditions.
Logic: mrs x_dst, <register>; return.
Validation: W03-DV04 review of the boundary list; the inventory of these
  functions is exactly the W03 `unsafe` surface (nothing else is authorized).
```

## 2. Typed extraction functions

```text
Name and stability: one function per fact, named extract_<fact>(raw: u64)
  -> FactValue, plus the two composites below; internal; pure; stable
  within P1.
Purpose and caller: decode register fields into fact values with explicit
  reserved-value handling. Callers: the phase body's draft construction.
Inputs / outputs: raw register value (or nothing, for the composites);
  returns the fact's value.
Preconditions / postconditions: pure decode; no state; decoding a reserved
  value yields Absent (or the documented special value) — never a panic and
  never a fabricated "supported".
State and ownership change: none.
Concurrency/allocation context: no allocation; no synchronization.
Errors and failure guarantee: cannot fail; unknown encodings map to the
  documented Absent/unknown representation and are visible in the report.
Security/authorization checks: none (self-description data).
Key decodes (field semantics fixed here; exact bit ranges per the announced
  architecture revision recorded in the implementation record):
  ExecutionLevel  <- CurrentEL.EL field; demanded value EL2
  CpuAffinity     <- MPIDR Aff0..Aff3, U, MT fields as one packed value
  ArchProfile     <- PFR0 EL0/EL1/EL2/EL3 nibbles packed
  GicVersion      <- PFR0.GIC (0 = no CPU-side GIC interface)
  PaRange         <- MMFR0.PARange encodings -> bits
  Granule4k      <- MMFR0.TGran4: 0 or 1 supported (1 includes LPA2)
  Granule16k     <- MMFR0.TGran16: 1 or 2 supported (2 includes LPA2)
  Granule64k     <- MMFR0.TGran64: 0 supported
                   other encodings map conservatively to Absent
  VirtualHostExtensions <- MMFR1.VH == 1 (VHE, not Stage-2 presence)
  CounterFrequency<- CNTFRQ[31:0] hertz; 0 maps to Absent
  El2VirtualTimer <- derived from VirtualHostExtensions (no separate read)
  El2PhysicalTimer<- constant Present (architectural whenever EL2 is)
Validation: W03-DV01 fact-set review; decode spot-checks against the
  architecture reference recorded in the implementation record.
```

```text
Name and stability: extract_affinity() -> FactValue (composite; calls
  read_mpidr + decode) and extract_execution_level() -> FactValue
  (composite; calls read_current_el); internal; stable within P1.
Purpose and caller: give the two "identity" facts a single call site so the
  phase body reads uniformly. Caller: the phase body.
Inputs / outputs / rules: as §2; the composites are the only functions that
  combine a read and a decode.
Validation: W03-DV01.
```

## 3. `FactId`, `FactValue`, `FactRecord`

```text
Name and stability: FactId (Copy + PartialEq enum over the §2 fact list,
  with const ALL: [FactId; N] in report order and fn label(self) ->
  &'static str); FactValue (newtype over u64 with typed accessor helpers
  per fact kind, or an Absent marker); FactRecord { id, classification,
  observation }; all internal; stable within P1.
Purpose and caller: the report's vocabulary. Callers: extraction outputs,
  the report, the query API, rendering.
Inputs / outputs: plain data.
Preconditions / postconditions: FactValue never carries a decoded value for
  an Absent observation; label strings are the render tokens of
  [03-code-contracts-classification-and-report.md](03-code-contracts-classification-and-report.md)
  §5 and are fixed before first verdict-bearing evidence (W10 precedent).
State and ownership change: none (value types).
Concurrency/allocation context: no allocation; Copy-friendly.
Errors and failure guarantee: cannot fail.
Security/authorization checks: none.
Logic: plain definitions; ALL is the authoritative iteration order.
Validation: W03-DV01 review that ALL matches the §2 fact table of
  [01-architecture-and-state.md](01-architecture-and-state.md).
```

The fact vocabulary is P1-internal boot-scope structure — not ABI, not a
P2 interface. P2 designs its own discovery types; this set exists so P1
mechanisms never branch on platform names.
