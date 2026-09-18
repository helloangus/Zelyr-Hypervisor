# P8-W13 Diagnostic Context Contracts

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W13 detailed design](README.md).

These are normative interface obligations, not implementations. No crate,
module, file layout, or target is selected here; realization belongs to the
implementing designs of the owning packages (EL2 diagnostic path per the P4-W06
module contract; W16 for regression assertions). Names are fixed by this design
for the P8 stage; renaming requires a revision. None of these is an EL2 or
Guest ABI surface, and none enters the
[P8-W14](../p8-w14-machine-abi-compatibility/README.md) compatibility matrix.

## 1. Contract status and stability

| Contract | Kind | Realized by | Stability |
|---|---|---|---|
| `LinuxGuestFaultClass` | enum | classification step of the EL2 diagnostic path | Stage-local, fixed by this design |
| `FaultDiagnosticContext` | record | context assembly on the evidenced P4-W06 fields | Stage-local; field set fixed here |
| `classify_guest_fault` | function (semantic) | EL2 diagnostic path | Stage-local; rules fixed in 01 §4 |
| `diagnostic_context_sufficient` | predicate | shared with W12 sufficiency checks; asserted by W16 | Stage-local |
| Trace-window requirement | semantic constraint | trace path per P0-W12/P7-W09/P6-W13 fields | Stage-local; bound recorded per implementation |

## 2. `LinuxGuestFaultClass`

```text
Name and stability: LinuxGuestFaultClass — closed enum; stage-local; labels
  only (the encoding/transport of labels is P0-W12/P7-W09 governance, not this
  contract).
Purpose and caller: name one failure occurrence per the 01 §3 taxonomy so
  diagnostics, containment, and regression assertions share one vocabulary.
  Callers: EL2 diagnostic path; W11/W12 routing; W16/W18 assertions.
Inputs / outputs: not applicable (enum). Variants: GuestKernelPanic,
  GuestSynchronousException{subclass per W05 classification},
  Stage2Fault{translation | permission | other-per-P4-W06}, InvalidMmioAccess,
  PsciViolation, VgicViolation, TimerContractViolation, VcpuStateAnomaly,
  HypervisorInvariantViolation, PlatformFailure, Unclassified.
Preconditions / postconditions: a value is assigned only by
  classify_guest_fault (02 §4); containment follows 01 §5 by family.
State and ownership change: none.
Concurrency/allocation context: assignment happens on the exit/diagnostic path
  in the owning design's context; no allocation or blocking is authorized here
  beyond what the P4-W06-evidenced diagnostic path already performs.
Errors and failure guarantee: the enum cannot express "multiple classes"; an
  event with mixed evidence takes the first matching rule (01 §4) and carries
  the remainder in its context.
Security/authorization checks: variant selection must not be influenced by
  Guest-writable memory contents (01 §2).
Validation: W13-DV02 taxonomy review; exercised per 01 §7 (W13-DV04).
```

## 3. `FaultDiagnosticContext` and the trace-window requirement

```text
Name and stability: FaultDiagnosticContext — record assembled by the EL2
  diagnostic path from evidenced fields; stage-local.
Purpose and caller: the minimum actionable context for one fault event
  (P8-V18). Callers: diagnostic path; W12 sufficiency; W16/W18 assertions; W20
  closeout citation.
Inputs / outputs: not applicable (record). Required fields:
  identity        build/version identity per P0-W12 alignment rules
  vm_id, vcpu_id  the affected context
  class           LinuxGuestFaultClass (02 §2)
  pc, pstate      Guest PC and PSTATE/SPSR at the exit (as captured)
  syndrome        ESR-class and ISS fields per the P4-W06-evidenced capture
  fault_address   faulting IPA (and VA/FAR as captured by the boundary)
  exit_reason     the P4-W06-evidenced categorization
  mapping_query   mapped/unmapped/protected result where applicable
  vcpu_state      scheduler lifecycle state per P7-W02
  trace_window    bounded recent-event window (see below)
  console_ref     pointer to the captured console transcript region
Preconditions / postconditions: every field populated from evidenced capture
  paths only; "not captured by the boundary" is an explicit absence recorded as
  a diagnostic gap, never a fabricated value.
State and ownership change: the record is immutable once assembled.
Concurrency/allocation context: assembly occurs in the owning design's
  declared context; this contract adds no new context.
Errors and failure guarantee: incomplete assembly is detectable via 02 §5 and
  fails the actionable-diagnostics clause; it never blocks containment itself.
Security/authorization checks: no Host physical address may appear in any
  portion of this record that becomes Guest-visible (01 §6/§W12 host-leak
  rule); Guest-produced text appears only as recorded console bytes.
Trace-window requirement: the window carries the last declared number of
  evidenced scheduler/interrupt/timer/exit events preceding the fault
  (P7-W09/P6-W13 fields), bounded so the record stays "minimum actionable"
  (01, decision 4). The bound is recorded in the implementation record;
  unbounded capture is out of scope (crash-dump boundary).
Validation: W13-DV03 context review; per-event sufficiency (W13-DV04).
```

## 4. `classify_guest_fault`

```text
Name and stability: classify_guest_fault — total, deterministic semantic
  function fixed by this design; realized in the EL2 diagnostic path.
Purpose and caller: assign exactly one LinuxGuestFaultClass to a fault event.
  Callers: the diagnostic path; indirectly W11/W12 routing and W16/W18.
Inputs / outputs:
  inputs  — the P4-W06-evidenced exit facts (categorization, syndrome,
            captured addresses, mapping query), the W05 classification of the
            underlying operation where applicable, and the console-capture
            reference
  output  — LinuxGuestFaultClass; never panics, never fails open
Preconditions / postconditions: inputs complete per the evidenced boundary;
  postconditions: total (Unclassified fallback), deterministic for identical
  inputs, first-match-wins per the ordered rules in 01 §4.
State and ownership change: none; pure over its inputs.
Concurrency/allocation context: the owning design's exit-path context; the
  function performs no allocation, blocking, or Guest-memory access.
Errors and failure guarantee: no error path; malformed or missing input yields
  Unclassified with the gap recorded (01 §3.3), never a default Guest-facing
  class.
Security/authorization checks: decision inputs are EL2-captured facts only;
  Guest-writable contents (including console text) may corroborate but never
  determine the class; a Guest cannot select its own classification.
Logic (outline, not production code):
  if EL2-side fault or EL2-state corruption        -> F13-9
  if environment-level failure                     -> F13-10
  if console shows Linux-initiated panic, no fault -> F13-1
  if Stage-2 fault                                 -> F13-3 (subtype from query)
  if trapped EL1 sync exception                    -> F13-2 (W05 subclass)
  if target outside declared MMIO windows          -> F13-4
  if PSCI call outside declared subset/params      -> F13-5
  if vGIC contract violation                       -> F13-6
  if timer contract violation                      -> F13-7
  if lifecycle-inconsistent event                  -> F13-8
  otherwise                                        -> F13-0 (block)
Validation: W13-DV02 rule review; W13-DV04 exercised per induced class;
  forbidden-outcome assertions active on every run (01 §6).
```

## 5. `diagnostic_context_sufficient`

```text
Name and stability: diagnostic_context_sufficient — predicate fixed by this
  design; shared with the W12 memory sufficiency check.
Purpose and caller: make "actionable diagnostics" checkable per event rather
  than asserted. Callers: EL2 diagnostic path (self-check); W12 evaluation;
  W16/W18 regression assertions.
Inputs / outputs: a FaultDiagnosticContext plus the event's class → boolean.
  True when: every field required for the class is present and non-fabricated;
  the trace window respects the recorded bound; the console reference resolves;
  no Host physical address appears in any Guest-visible portion.
Preconditions / postconditions: total; a false result is a recorded diagnostic
  gap that fails the affected run's actionable-diagnostics clause.
State and ownership change: none.
Concurrency/allocation context: same as assembly (02 §3).
Errors and failure guarantee: no error path; absence of a required field is
  false, not an exception.
Security/authorization checks: host-leakage check is part of the predicate.
Validation: W13-DV03/W13-DV04; reused by W12-DV05.
```

## 6. Regression observable contract (for W16/W18)

For each induced event, the regression assertion set is:

```text
expected_class      the class from the 01 §7 coverage map
contained           the class's 01 §5 contained outcome observed
context_sufficient  diagnostic_context_sufficient == true
console_observed    the class's console observable present (01 §6)
forbidden_absent    no EL2 panic/hang, no cross-VM effect, no host-address leak
```

All five must hold for the run to count toward P8-V18 coverage of that class.
The assertion semantics are fixed here; their mechanical realization belongs to
[W16](../p8-w16-automated-linux-regression/README.md). W18 additionally treats
any forbidden-outcome occurrence as a security-regression failure, not merely a
diagnostic gap.
