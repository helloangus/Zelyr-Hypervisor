# P1-W09 Interface Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W09 detailed design](README.md).

All names below are internal boot-scope items owned by this design (parent
README decision 2). They are not ABI, not public API of any future crate
boundary, and may be renamed only by a recorded design change. Pseudocode is
an outline, not runnable production code. No allocation exists anywhere in
this design: P1 has no heap (W02 scope), so every type is static or
stack-local and every collection is a fixed-size array.

## 1. `InitPhase` — phase identifier

```text
Name and stability: InitPhase, internal enum; Copy + PartialEq; stable within P1.
Purpose and caller: identifies one lifecycle phase for the tracker, markers,
  failure routing, and downstream consumers (W07 phase field, W10 evidence).
Inputs / outputs: variants Entry, Runtime, Capabilities, El2Baseline,
  Exceptions, Console, FatalPath, Stage1, Stable (terminal; not sequenced).
  Provides: const ALL: [InitPhase; 8] (production order, excludes Stable);
  fn index(self) -> u8 in 1..=8; fn label(self) -> &'static str returning the
  §1 tokens of [01-init-state-machine.md](01-init-state-machine.md).
Preconditions / postconditions: none at type level; ordering functions are
  total over ALL.
State and ownership change: none (pure value type).
Concurrency/allocation context: no allocation; no synchronization needed.
Errors and failure guarantee: cannot fail.
Security checks: none (not a trust boundary).
Logic: derived impls only; ALL lists variants in the decision-1 order.
Validation: W09-DV01 review that ALL matches the state-machine table.
```

## 2. `LifecyclePosition` — decoded lifecycle snapshot

```text
Name and stability: LifecyclePosition, internal enum; Copy + PartialEq.
Purpose and caller: human-legible snapshot of lifecycle position; produced by
  the tracker for diagnostics and by any exception-context reader.
Inputs / outputs: variants PreBoot, Entered(InitPhase), Completed(InitPhase),
  Stable, Unknown(u16). Entered/Completed never carry Stable; Stable is its
  own variant.
Preconditions / postconditions: decoding is total; an encoded value outside
  the legal set decodes to Unknown and never panics — the fatal path must not
  be able to fail while reporting.
State and ownership change: none.
Concurrency/allocation context: no allocation; plain data.
Errors and failure guarantee: Unknown is the failure representation; it
  preserves the raw value for evidence.
Security checks: none.
Logic: decode from the tracker's usize encoding (§3).
Validation: W09-DV02 reviews that every legal encoding decodes and that
  Illegal encodings are impossible to produce via the tracker API.
```

## 3. `BootPhaseTracker` — sole owner of lifecycle position

```text
Name and stability: BootPhaseTracker, internal type; exposed as one static
  instance TRACKER: BootPhaseTracker = BootPhaseTracker::NEW.
Purpose and caller: records and reports the single authoritative lifecycle
  position. Callers: phase_enter/phase_complete (writer), W07 fatal reporting
  and any exception-context diagnostic (reader), console materialization.
Inputs / outputs: interior state is exactly one core::sync::atomic::AtomicUsize
  encoding: PreBoot = 0; Entered(P[i]) = 2i-1; Completed(P[i]) = 2i for
  i in 1..=8; Stable = 17.
  API:
    const fn NEW() -> Self
    fn position(&self) -> LifecyclePosition            // wait-free load
    fn advance_enter(&self, p: InitPhase) -> Result<(), SequenceError>
    fn advance_complete(&self, p: InitPhase) -> Result<(), SequenceError>
    fn replay_events(&self) -> ...   // derived, see materialize contract §5
Preconditions / postconditions: advance_enter(P[i]) requires the encoded value
  to equal Completed(P[i-1]) (with Completed(P[0]) meaning the value left by
  the W02-delegated entry/runtime records); advance_complete(P[i]) requires
  Entered(P[i]). On success the value increases by exactly one; on any
  mismatch it is unchanged and SequenceError is returned. The value never
  decreases (monotone).
State and ownership change: the tracker is the only authoritative owner of
  lifecycle position; no other component stores phase state (parent README
  decision 5).
Concurrency/allocation context: single writer (boot CPU, straight-line boot
  context), single advance per call implemented as one compare_exchange
  against the exact required predecessor value, so misuse cannot corrupt the
  encoding. Readers may run in exception context on the same CPU; Relaxed
  ordering is sufficient because there is no second core in P1 and the
  exception context observes the same hart; a cross-CPU reader would require
  the P3 synchronization design and is out of scope. No allocation, no
  blocking, no lock.
Errors and failure guarantee: SequenceError::NotAppropriate { current } on
  precondition mismatch; state unchanged; caller routes via §4 of the parent
  state machine (misuse routing). The tracker never panics.
Security checks: none; the tracker records, it does not authorize.
Logic:
  advance_enter(p):
    required = encoding_of(Completed(prev(p)))   // prev over ALL, per decision 6
    if TRACKER.compare_exchange(required, encoding_of(Entered(p))) is Err(c)
      -> Err(NotAppropriate { current: decode(c) })
    record_marker(p, Enter); Ok(())
  advance_complete(p): symmetric against Entered(p); then record_marker(p, Complete).
Validation: W09-DV01/DV02; host-side unit tests of the encoding and CAS
  behavior belong to the host-test baseline where the P0 design permits them.
```

## 4. `phase_enter` / `phase_complete` — transition functions

```text
Name and stability: module functions phase_enter(InitPhase) and
  phase_complete(InitPhase); internal; stable within P1.
Purpose and caller: the only way boot code declares a transition. Callers:
  the W02-owned entry/runtime records, run_init_sequence, and the phase
  adapters (§6).
Inputs / outputs: phase; no return (errors route, they do not propagate —
  see below).
Preconditions / postconditions: called on the boot CPU in boot context, at
  most once per (phase, event), in the §2 order. After phase_enter returns,
  position is Entered(phase); after phase_complete returns, Completed(phase).
State and ownership change: tracker advance plus marker recording.
Concurrency/allocation context: boot context; no allocation; no blocking.
Errors and failure guarantee: a SequenceError is an invariant violation:
  route through the misuse routing rule of
  [01-init-state-machine.md](01-init-state-machine.md) §4 and never return
  normally. A mechanism failure inside the phase body is not a
  phase_enter/complete error; the body routes via fail_phase (§7).
Security checks: none beyond ordering integrity.
Logic:
  phase_enter(p):  if TRACKER.advance_enter(p) is Err -> misuse route (§4)
  phase_complete(p): if TRACKER.advance_complete(p) is Err -> misuse route
Live-emission rule: record_marker(p, e) emits through the W06 channel iff
  (p, e) is at or after (Console, Complete) — pure positional rule, no
  availability flag (parent README decision 3; see §5).
Validation: W09-DV02 (every transition recorded; live rule matches §3 of the
  state machine).
```

## 5. `materialize_replay` — one-shot deferred-marker emission

```text
Name and stability: fn materialize_replay(); internal.
Purpose and caller: emits the replay line covering all recorded events from
  PreBoot through Entered(Console). Caller: exactly the console phase step,
  immediately after W06's channel-availability signal.
Inputs / outputs: none; output via the W06 channel in W06's format.
Preconditions / postconditions: position must equal Entered(Console) and the
  channel-availability signal must have been received. The replay content is
  a pure function of position (the total order makes history derivable:
  Entered(P[i]) implies Completed(P[1..i]) and Entered(P[1..i-1])), so no
  history buffer exists. Must be called exactly once per boot; a second call
  is misuse and routes per §4.
State and ownership change: none beyond output.
Concurrency/allocation context: boot context; fixed-size formatting; no
  allocation (bounded line length per W06 contract).
Errors and failure guarantee: channel failure during replay routes via the
  `console` row of §4; the tracker state is not mutated by output failure.
Security checks: none.
Logic:
  for event in derive_history(TRACKER.position()): emit_marker(event)
Validation: W09-DV02 marker-coverage review; W10 regression observes the
  replay in every passing boot (execution evidence later, per W10).
```

## 6. Phase adapters and sequencer

Each production phase from `capabilities` onward has one adapter, named
`<phase>_step()` (`capabilities_step`, `el2_baseline_step`, `exceptions_step`,
`console_step`, `fatal_path_step`, `stage1_step`). Shared contract shape:

```text
Name and stability: as above; internal; stable within P1.
Purpose and caller: invokes the owning package's mechanism exactly once
  between the phase's enter/complete pair. Caller: run_init_sequence only.
Inputs / outputs: none; mechanism arguments, if any, are fixed by the owning
  design and listed in the implementation record when wired.
Preconditions / postconditions: phase prerequisite set per
  [01-init-state-machine.md](01-init-state-machine.md) §1; on return the
  owning contract's exit condition holds. The adapter adds nothing else.
State and ownership change: whatever the owning package's contract states.
Concurrency/allocation context: as the owning contract states; the adapter
  adds no synchronization, allocation, or blocking.
Errors and failure guarantee: a mechanism failure routes through fail_phase
  (§7) with this phase; the adapter never invents a fallback, retry, or
  partial success (parent README decision 7: fail-closed wiring).
Security checks: whatever the owning contract requires; the adapter adds none.
Logic: body is the single owning-package call plus nothing else. Until the
  owning design fixes its mechanism entry, the adapter does not compile.
Validation: W09-DV03 hidden-dependency review reads the adapter bodies.
```

`console_step` additionally performs, after its mechanism reports the channel
available: `materialize_replay()` then `phase_complete(Console)`. The
availability signal itself is the W06 contract's; W09 does not probe hardware
to guess availability.

```text
Name and stability: fn run_init_sequence(); internal.
Purpose and caller: the ordered composition of phases Capabilities..Stage1;
  called once by the W02 runtime at the seam fixed by W02's design, after
  W02 records runtime completion (parent README decision 6).
Inputs / outputs: none; on normal return the lifecycle position is
  Completed(Stage1) and the caller (W02-owned glue) records Stable.
Preconditions / postconditions: position == Completed(Runtime) on entry;
  phases execute in ALL order; W09 requires the postcondition that after
  return every production phase is Completed.
State and ownership change: lifecycle position only; mechanisms own their own
  state.
Concurrency/allocation context: boot context; no allocation; straight-line.
Errors and failure guarantee: any failure routes terminally per §4; the
  function never returns mid-sequence to "try the next phase".
Security checks: none beyond the routing matrix.
Logic:
  phase_enter(Capabilities);   capabilities_step();   phase_complete(Capabilities);
  ... identical pairs for the remaining phases in ALL order ...
  phase_enter(Stage1); stage1_step(); phase_complete(Stage1);
Validation: W09-DV01 (order matches the state machine); W09-DV04 (routes).
```

## 7. `fail_phase` — terminal route dispatcher

```text
Name and stability: fn fail_phase(p: InitPhase, reason: FailureReason) -> !;
  internal; diverging.
Purpose and caller: single dispatcher from any phase failure to the §4 route.
  Callers: phase adapters and phase bodies; the misuse route of §4.
Inputs / outputs: phase and a reason carrier owned by this design (at minimum:
  which phase failed and a short static reason string; richer payloads are
  owned by the supplying contracts).
Preconditions / postconditions: not called from inside a fatal or panic path
  (recursion containment is owned by W07's boundary); the route for p per the
  §4 matrix is established. Never returns.
State and ownership change: none of its own; delegates to the route owner.
Concurrency/allocation context: boot or exception context per route; no
  allocation.
Errors and failure guarantee: terminal by construction; retains phase and
  reason for the route's report (W07 phase field or the early panic route).
Security checks: none; fault handling is not an authorization point.
Logic:
  match p: Runtime..Exceptions -> panic route (W02/P0 contract)
           Console             -> best-effort panic route (§4 console row)
           FatalPath           -> W07 non-recursive readiness boundary
           Stage1 | Stable     -> W07 fatal path
  (entry has no dispatcher: rejection belongs to the W01 boundary before
   transfer.)
Validation: W09-DV04 routing review including the H4 precedence rule.
```

## 8. Integration seams

Consumed from assumed contracts (each names its failure boundary: a missing or
contradicting seam is a recorded blocker per
[03-implementation-and-review.md](03-implementation-and-review.md) §1, never a
local workaround):

| Seam | Supplied by | Used for | W09 owns |
|---|---|---|---|
| Validated-entry transfer guarantee; rejection boundary | W01 | delegating the `entry` records; `entry` route | the record point, not the validation |
| Rust entry establishment; panic route | W02 (P0 panic baseline) | `runtime` records; invocation seam for `run_init_sequence`; early failure output | the call ordering requirement only |
| Capability mechanism + required/optional classification | W03 | `capabilities_step` body; fail-fast reason | sequencing only |
| EL2 baseline mechanism | W04 | `el2_baseline_step` body | sequencing only |
| Vector installation; unexpected-event classification | W05 | `exceptions_step` body; post-`stable` fault classification | the unowned-window limitation record |
| Channel write; availability signal; marker format | W06 | live emission and replay output | label tokens and emission points |
| Fatal report entry; startup-phase field consumer | W07 | `fatal_path_step`; crash phase attribution | the tracker position read |
| Stage-1 transition mechanism; post-MMU continuity | W08 | `stage1_step` body | ordering relative to phases 6–7 |

Produced for consumers: phase label tokens and event vocabulary
([P1-W10](../p1-w10-qemu-boot-regression/README.md) verdict tokens;
[P1-W11](../p1-w11-negative-fault-validation/README.md) scenario attribution);
tracker position as the W07 startup-phase field; the declared stable
environment for [P1-W12](../p1-w12-p1-documentation-handoff/README.md) and P2.

No seam authorizes W09 to modify a supplying package's mechanism, contract, or
naming; conflicts are raised, not absorbed.
