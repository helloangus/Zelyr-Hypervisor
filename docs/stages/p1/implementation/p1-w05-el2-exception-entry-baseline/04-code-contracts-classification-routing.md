# P1-W05 Classification and Routing Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W05 detailed design](README.md).

Pseudocode is an outline, not runnable production code. No allocation; all
fixed-size data; every string in the vocabulary is a `&'static str`. All
names are internal boot-scope items owned by this design.

## 1. Origin and category vocabulary

```text
Name and stability: OriginClass (Copy + PartialEq enum: CurrentElSpx |
  CurrentElSp0 | LowerElA64 | LowerElA32); ExceptionCategory (Copy +
  PartialEq enum: Synchronous | Irq | Fiq | SError); both internal; stable
  within P1.
Purpose and caller: the origin/context axis of P1-V08. Callers: the stubs
  (stamping); classification; report rendering.
Inputs / outputs: plain data; fn label(self) -> &'static str for each.
Preconditions / postconditions: the legitimate-in-P1 predicate is fixed
  design authority (architecture file §2): only CurrentElSpx is legitimate;
  no runtime recomputation.
State and ownership change: none.
Concurrency/allocation context: none needed.
Errors and failure guarantee: cannot fail.
Security/authorization checks: none.
Validation: W05-DV01/DV03 vocabulary review.
```

## 2. Syndrome interpretation boundary — the EC-class table

The boundary is exactly this table plus the two validity flags; nothing
below class level (no ISS-field decoding) exists in P1. Exact EC encodings
are recorded in the implementation record against the recorded architecture
revision (the W03 decode-reference discipline). The table is closed:
values outside every named class fall into `Unknown`.

| SyndromeClass | EC group (recorded at implementation) | far_valid | hpfar_valid | Label content |
|---|---|---|---|---|
| `Unknown` | anything unmatched | no | no | "unknown" |
| `TrappedInstruction` | WFx, A64 system-register trap, LDP/STP trap group | no | no | which trap group |
| `IllegalState` | illegal AArch64 execution state (PSTATE.IL) | no | no | "illegal state" |
| `InstructionAbort` | current-EL instruction abort | yes | no | abort class |
| `DataAbort` | current-EL data abort; DFSC group (translation / alignment / permission / other) recorded as the label | yes | no | abort class + DFSC group |
| `SpAlignment` | SP alignment fault | yes | no | "sp alignment" |
| `PcAlignment` | PC alignment fault | yes | no | "pc alignment" |
| `Debug` | BRK / software step / watchpoint group at current EL (lower-EL debug reaches EL2 only via W04's C5 traps — recorded) | per sub-class | no | which debug event |
| `SError` | SError interrupt / physical error abort | no | no | "serror" |
| `GuestClassOnly` | abort/trap ECs only reachable from lower ELs (reachable in P1 only via invalid origins) | no | no | "lower-el-only" |

```text
Name and stability: SyndromeClass (Copy + PartialEq enum per the table);
  fn syndrome_class(esr: u64) -> SyndromeClass with fn label(self) ->
  &'static str, fn far_valid(self) -> bool, fn hpfar_valid(self) -> bool;
  internal pure functions; stable within P1.
Purpose and caller: the bounded syndrome interpretation P1 commits to.
  Callers: capture (validity flags gate the FAR/HPFAR reads); routing and
  W07's report (labels).
Inputs / outputs: raw ESR_EL2 value; the classified class.
Preconditions / postconditions: pure; total over all 2^64 ESR values
  (closed table); deterministic.
State and ownership change: none.
Concurrency/allocation context: no allocation.
Errors and failure guarantee: cannot fail; unmatched ECs classify Unknown.
Security/authorization checks: none; syndrome bits are machine facts.
Logic:
  ec = (esr >> 26) & 0x3F
  match ec against the table; DataAbort refines its label by the recorded
  DFSC group nibble; everything else class-level only.
Validation: W05-DV03 table review; NC3's expected "syndrome identifying
  the exception class" reads these labels (W11).
```

## 3. Dispositions — the recoverable-versus-fatal boundary (work seq 2)

```text
Name and stability: ExceptionDisposition (Copy + PartialEq enum:
  FatalSyndrome | UnexpectedEvent | Recoverable); internal; stable within
  P1.
Purpose and caller: the defined outcome per event class (plan goal). The
  vocabulary exists stage-wide; P1's policy constructs only the first two
  (parent README decision 3). Callers: classification; routing.
Inputs / outputs: plain data.
Preconditions / postconditions: classification maps every (origin,
  category) pair to exactly one constructed disposition:
    any origin invalid for P1            -> UnexpectedEvent (InvalidOrigin
                                            context in the report)
    Synchronous during P1 execution      -> FatalSyndrome
    SError                               -> FatalSyndrome
    IRQ / FIQ arrival                    -> UnexpectedEvent
  `Recoverable` is Reserved: no P1 code constructs it; a later stage that
  can legitimately recover an event class (P6 IRQ delivery first) claims it
  through its own design, together with the return machinery it requires.
State and ownership change: none.
Concurrency/allocation context: none.
Errors and failure guarantee: classification cannot fail (closed inputs:
  four categories × four origins).
Security/authorization checks: none; disposition is not an authorization
  decision (nothing below EL2 exists to authorize against).
Logic:
  if origin != CurrentElSpx: UnexpectedEvent
  else match category: Synchronous | SError -> FatalSyndrome
                       Irq | Fiq            -> UnexpectedEvent
Validation: W05-DV03/DV05; NC6's "classification present with origin and
  context" reads this mapping (W11).
```

Both constructed dispositions route to the same terminal outcome in P1
(routing §4); they differ in the diagnostic class rendered — which is what
W11's NC3 (FatalSyndrome: syndrome and location) and NC6 (UnexpectedEvent:
origin/context) respectively assert.

## 4. Routing — `route_classified`

```text
Name and stability: fn route_classified(frame: &ExceptionFrame) -> !;
  internal; diverging; stable within P1.
Purpose and caller: carry a classified event to its defined outcome — the
  W09 matrix `exceptions`-row route (pre-arm) or the W07 fatal path
  (post-arm). Caller: the capture entry, once per guarded exception.
Inputs / outputs: the completed frame; no return.
Preconditions / postconditions: guard held; frame complete. Never returns.
State and ownership change: none of its own; the guard stays set (a second
  entry anywhere downstream terminates silently — R1).
Concurrency/allocation context: exception context; no allocation; masks
  set throughout.
Errors and failure guarantee: terminal by construction; output transport
  failure is contained by the transport's own contract (the early writer's
  architectural poll semantics; W06's channel contract post-arm); the
  bounded stop is reached regardless (R4 posture inherited from W02).
Security/authorization checks: rendered content is machine facts and
  static labels only; no fault-controlled memory is dereferenced for
  enrichment.
Logic:
  cls = syndrome_class(frame.esr)
  disp = classify(frame.origin, frame.category)      # §3; pure, total
  if fatal_path_ready():                             # W07 declaration
    report_fatal_exception(frame, cls, disp)         # W07 seam; diverges
  emit_pre_arm_summary(frame, cls, disp)             # §5; via W02's writer
  bounded_stop()
```

The two-tier route is the parent-README decision 6 reconciliation: W09's
matrix routes `exceptions`-phase events "via the panic route" because the
fatal path is established only at phase 7 (W09 H4); the pre-arm summary is
that panic-route-era diagnostic, carrying syndrome and location "as
available" — which is exactly what the matrix's observable column names.
Once W07's `fatal-path` phase completes, the same event produces the full
P1-V11 report through the W07 seam.

## 5. Pre-arm summary — token classes (work seq 5)

```text
Name and stability: emit_pre_arm_summary(frame, cls, disp); internal;
  stable within P1.
Purpose and caller: the pre-arm observable NC3 asserts. Caller:
  route_classified only.
Inputs / outputs: frame + classification; one bounded line set through
  early_write_bytes (W02's channel-independent writer).
Preconditions / postconditions: guard held; on return (or non-return —
  the writer's stuck-transmitter semantics apply) the stop follows.
State and ownership change: UART only (single-consumer rule: post-transfer
  the early writer owns it until W06's channel supersedes markers; the
  summary is a panic-route-era diagnostic and inherits that path — see the
  relationship table in W02's
  [panic/identity contracts](../p1-w02-minimal-rust-el2-runtime/04-code-contracts-panic-identity.md)
  §5).
Concurrency/allocation context: exception context; bounded stack buffer +
  the bounded formatter discipline (W02 §4); truncation, never panic.
Errors and failure guarantee: bounded by construction (fixed capacity);
  output is best-effort at the hardware level.
Security/authorization checks: fixed literals, hex of machine registers,
  static labels; no echo of other memory.
Line format (fixed, single line):
  <fatal-class-prefix> exc cat=<category label> org=<origin label>
    esr=<16-hex> pc=<16-hex> far=<16-hex or "->" when invalid>
    ph=<tracker phase label>
  The fatal-class prefix is owned by W07's report model (one vocabulary
  across both eras; W10's forbidden-class matching covers pre-arm and
  post-arm uniformly). Exact prefix text is recorded in the implementation
  record before the first verdict-bearing run (the W10 token precedent)
  and never adjusted afterwards.
Validation: W05-DV06 review; NC3 (pre-arm variant) and NC6 observe the
  line (W11 execution, deferred).
```

Phase attribution uses the tracker's read API (W09 §3 `LifecyclePosition`),
which never panics and never yields an impossible snapshot (W09 H6) — the
summary's `ph=` field is safe to produce from exception context.

## 6. Install-failure route — `VectorError`

```text
Name and stability: VectorError { control: VectorControl, expected: u64,
  observed: u64 } with fn fail_vector(e: VectorError) -> !; internal;
  stable within P1. VectorControl enumerates exactly { Vbar, SpSel, Daif,
  HcrRouting, HcrTraps, CptrFp } (the §3 assertion set plus the install
  target).
Purpose and caller: the only failure class of the install mechanism (the
  `exceptions` phase's route per the W09 matrix). Callers: the phase
  body's assertions and read-back.
Inputs / outputs: control identity and the compared values; output is the
  panic-route report carrying the control's static name (through W02's
  bounded formatter).
Preconditions / postconditions: called only inside the phase body; never
  returns (panic route; phase-attributed `exceptions` by tracker position).
State and ownership change: none of its own.
Concurrency/allocation context: boot context; no allocation.
Errors and failure guarantee: terminal; no retry (W04 decision 6 posture;
  retrying an unresponsive machine state would mask divergence).
Security/authorization checks: none; masked register values, not secrets.
Logic: render via the bounded formatter; invoke the panic route.
Validation: W05-DV04; the vocabulary travels to W11's expectations for
  install-time failures.
```
