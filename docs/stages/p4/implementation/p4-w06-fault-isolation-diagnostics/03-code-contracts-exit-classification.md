# P4-W06 Code Contracts — Exit Classification and Diagnosis

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W06 detailed design](README.md).  
**Companion:** module map, fault-domain model, and IS matrix in
[02-architecture-and-state.md](02-architecture-and-state.md).

All names are P4-internal and unstable-by-declaration; P4-W09 records them as
implemented facts only. Pseudocode is an algorithm outline, not runnable
production code; the Coding Guidelines govern the final Rust shape. The W04
frame and W02 `QueryResult` shapes are consumed as fixed seams (assumed
contracts M2/M4) and are not redefined here.

## 1. `EsrView` decode (module `exit-decode`)

- **Name and stability:** `fn decode_esr(esr: u64) -> EsrView` with
  `EsrView { ec: ExceptionClass, il: InstructionLength, iss: IssView }` and
  `IssView` carrying, for abort classes only, the extracted sub-fields
  (`fault_status: FaultStatusFamily`, `access: Option<FaultAccess>`,
  `is_stage2: bool`). Internal.
- **Purpose and caller:** turn the raw syndrome into typed detail; called by
  `diagnose` ([§3](#3-diagnose-and-exitdiagnostic-module-fault-diag)) only.
- **Inputs/outputs:** raw `esr` from the frame → decoded view; total function.
- **Preconditions/postconditions:** none beyond totality: unknown EC values
  and reserved status encodings are represented, never rejected — the view
  records `ExceptionClass::Unknown(raw)`-style variants rather than failing.
- **State/ownership change:** none.
- **Concurrency/allocation:** pure, constant-time, no allocation; callable in
  the exit-handler context.
- **Errors:** none (totality is the error strategy; unknown values are data).
- **Security/authorization checks:** consumes Guest-influenced bits as data
  only; no field influences control flow beyond enum selection; no field is
  used as an address.
- **Logic:**

```text
decode_esr(esr):
    ec  = bits(esr, 31..26)          // exception class
    il  = bit(esr, 25)
    iss = bits(esr, 24..0)
    match ec:
      stage2_data_abort_family:
        fs  = decode_fault_status(iss)     // per pinned revision; unknown -> Other(raw)
        access = from_wnr_and_abt_bits(iss) // Read | Write; Execute for insn abort
        return EsrView { ec: Abort, il, iss: IssView { fs, access, is_stage2: true } }
      other known classes: return typed variant
      _: return EsrView { ec: Unknown(raw = ec), .. }
```

- **Validation:** table-driven unit tests over synthesized ESR words covering
  every class W04's routing table can produce plus a sweep of unknown/reserved
  encodings (host-side); encoding review against the pinned architecture
  revision; QEMU-observed divergences recorded as Specification Investigation
  items, never absorbed into the tables silently.

## 2. Faulting-IPA reconstruction (module `exit-decode`)

- **Name and stability:** `fn reconstruct_fault_ipa(frame: &GuestExitFrame)
  -> IpaReconstruction` with
  `IpaReconstruction — Available(GuestPhysAddr) | Unavailable`. Internal.
- **Purpose and caller:** produce the faulting Guest IPA for Stage-2 aborts
  from the captured FAR/HPFAR pair; called by `diagnose`.
- **Inputs/outputs:** frame fields (raw `far`, `hpfar`) → reconstruction
  result.
- **Preconditions:** the frame was captured by the W04 stub (unconditional
  capture guarantees FAR/HPFAR hold the architecturally defined values for
  the abort that occurred).
- **Postconditions:** for a Stage-2 data/instruction abort whose status code
  defines an IPA, `Available(ipa)` where `ipa` is composed exactly per the
  pinned revision (FAR low bits + HPFAR high bits, width-limited to the
  space's IPA span); for any other class, `Unavailable`.
- **State/ownership change:** none.
- **Concurrency/allocation:** pure, no allocation.
- **Errors:** none; `Unavailable` is the defined not-computable outcome and is
  never replaced by a guessed value (D4).
- **Security/authorization checks:** the composed IPA is data for comparison
  with the boot-info probe/window addresses and the W02 ledger; it is never
  dereferenced as a Host address; composition uses checked shifts/masks that
  cannot overflow the target type.
- **Logic:**

```text
reconstruct_fault_ipa(frame):
    view = decode_esr(frame.esr)
    if view.iss.is_stage2 and view.iss.fs is Translation|Permission:
        low  = mask(frame.far, IPA_LOW_BITS)        // checked widths
        high = mask(frame.hpfar, IPA_HIGH_BITS)
        return Available(GuestPhysAddr::from_parts(high, low))  // width-limited
    return Unavailable
```

- **Validation:** unit tests with synthesized FAR/HPFAR pairs including
  maximum-width addresses, zero, and misaligned low-bit cases; cross-check
  against the pinned revision's composition rule; on-target agreement via
  IS-01/IS-02 (faulting IPA must equal the marked probe address).

## 3. `diagnose` and `ExitDiagnostic` (module `fault-diag`)

### 3.1 `ExitDiagnostic`

- **Name and stability:** `ExitDiagnostic { domain: FaultDomain, class:
  ExitClass, raw_esr: u64, view: EsrView, access: Option<FaultAccess>,
  fault_status: Option<FaultStatusFamily>, ipa: IpaReconstruction, guest_pc:
  u64, guest_pstate_summary: PstateSummary, vcpu: VcpuId, mapping:
  Option<QueryResult>, mapping_agreement: MappingAgreement, verdict:
  DiagnosisVerdict, stop_cause: StopCause }` with
  `MappingAgreement — Consistent | Mismatch | NotApplicable` and
  `DiagnosisVerdict — Expected | Unexpected | Unclassified`. Internal.
- **Purpose and caller:** the complete per-exit diagnostic record; produced by
  `diagnose`, consumed by the matcher, the report, and the W07 run record.
- **Contract notes:** `mapping` is present only when `ipa` is `Available` and
  the caller supplied the space; `verdict = Unclassified` exactly when the
  W04 class is `UnknownSync` and no refinement applies — raw data is retained
  regardless. The type is immutable after production.

### 3.2 `diagnose`

- **Name and stability:** `fn diagnose(frame: &GuestExitFrame, space:
  Option<&GuestAddressSpace>, vcpu: VcpuId, stop_cause: StopCause) ->
  ExitDiagnostic`. Internal.
- **Purpose and caller:** the single diagnosis entry; called by the W04 exit
  handler after the stop decision (containment-first ordering, D7).
- **Inputs/outputs:** captured frame; optional space for the mapping
  cross-check; identity and the already-chosen stop cause → the record.
- **Preconditions:** the stub captured the frame completely (M2); called in
  the exit-handler context after `Stopped` was recorded; host interrupts
  masked per the run-segment policy; no concurrent diagnosis (single exit
  path).
- **Postconditions:** the record's fields are a pure function of the inputs;
  when `space` is `Some` and `ipa` is `Available`, `mapping` holds the W02
  `query` snapshot and `mapping_agreement` is `Consistent`/`Mismatch` per
  §3.3; when no comparison applies it is `NotApplicable`.
- **State/ownership change:** none (pure value production; retention is the
  stop path's duty).
- **Concurrency/allocation:** no allocation; one space-lock read at most;
  constant time.
- **Errors:** none — totality again; every failure to compute a field is
  represented in the field (`Unavailable`, `NotApplicable`, `Unclassified`).
- **Security/authorization checks:** all Guest-influenced inputs are treated
  as untrusted data: syndrome bits select enum variants only; the IPA is
  compared, never dereferenced; `guest_pc` is recorded, never used as a call
  target or Host address. The Guest cannot influence the *interpretation*
  layer, only the data in it.
- **Logic:**

```text
diagnose(frame, space, vcpu, stop_cause):
    view  = decode_esr(frame.esr)
    ipa   = reconstruct_fault_ipa(frame)
    (mapping, agreement) =
        match (space, ipa):
          (Some(s), Available(a)) where view.iss.is_stage2:
              q = s.query(a)
              agreement = classify_agreement(view, q)   // §3.3
              (Some(q), agreement)
          _ -> (None, NotApplicable)
    class = w04_class_of(frame)                // mirrors W04's minimal decode,
                                               // cross-checked, never overridden
    verdict = match class:
      UnknownSync -> Unclassified              // fail-closed, raw retained
      _           -> Expected                  // expectedness is finalized by
                                               // the matcher against IS rows (§04)
    return ExitDiagnostic { .. }
```

- **Validation:** host-side unit tests over synthesized frames for every W04
  class: field completeness, totality on unknown syndromes, agreement
  classification; on-target cross-checks via the IS rows (DV03–DV08).

### 3.3 Agreement classification

- **Name and stability:** `fn classify_agreement(view: &EsrView, q:
  &QueryResult) -> MappingAgreement`. Internal, private to `fault-diag`.
- **Purpose:** the stale-translation detector behind P4-V07/V08's "does not
  rely on stale translations."
- **Logic:**

```text
classify_agreement(view, q):
    match (view.iss.fs, q):
      (Translation, Unmapped)            -> Consistent
      (Permission, Mapped { flags, .. }) -> if flags explain the fault access
                                            (write on RO / execute on XN)
                                            then Consistent else Mismatch
      (Translation, Mapped { .. })       -> Mismatch   // ledger says mapped,
                                                       // hardware faults: stale
                                                       // or inconsistent state
      (Permission, Unmapped)             -> Mismatch   // hardware saw a mapping
                                                       // the ledger does not know
      _                                  -> NotApplicable
```

- **Failure guarantee:** `Mismatch` never panics and never changes the stop
  outcome; it marks the record so the run record flags the episode failed and
  an invariant-review item is raised (D5). The Guest access itself remains
  contained exactly as any other fault.
- **Validation:** exhaustive pair tests (status family × query result ×
  access); the on-target Mismatch case is exercised by an intentional
  fault-injection review test (host-side synthesis) and must never occur in
  passing runs (DV08).

## 4. Cross-check with W04's minimal decode

- `diagnose` recomputes the W04 class from the frame through the same decode
  tables and asserts equality with the class recorded in the stop cause. A
  disagreement is impossible under the contract (same frame, same tables) and
  is treated as a host invariant violation (fatal escalation per M5) if ever
  observed — it would mean the two layers diverged, i.e., a broken build
  contract, not a Guest event.
- This is a consistency duty, not a second action policy: W06 never selects a
  different action, and W04 never consults `ExitDiagnostic`.

- **Validation:** review row plus a host-side assertion test feeding identical
  frames to both decoders; the dual-table risk is bounded by keeping W04's
  decode the caller and W06's decode the refinement of the same `exit-decode`
  functions (single implementation, two consumption heights).

## 5. Safety and inventory notes

- No `unsafe` is designed for this file (D10): all inputs are captured values;
  all composition uses checked arithmetic; no memory beyond the frame and the
  returned snapshot is read.
- If implementation discovers an `unsafe` need (for example, an architecture
  revision requiring a raw-register re-read), that is a design deviation: the
  workflow stops, the design gains a contract here first, and the inventory
  entry with SAFETY justification is added before merge.
