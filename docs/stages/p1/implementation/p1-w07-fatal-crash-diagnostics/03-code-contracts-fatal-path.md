# P1-W07 Fatal Path Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W07 detailed design](README.md).

Pseudocode is an outline, not runnable production code. All names are
internal boot-scope items owned by this design. The panic entry assumes
W02's handler registration per the ownership transfer
([architecture](01-architecture-and-state.md) §4).

## 1. Panic entry — the superseding `p1_panic` body

```text
Name and stability: p1_panic(info: &core::panic::PanicInfo) -> !; the
  binary's single #[panic_handler]; ownership: W07 (transferred from W02
  via the recorded extension seam — parent README decision 2); stable
  within P1.
Purpose and caller: the panic route W09's routing matrix relies on from
  phase `runtime` onward; renders the full kind P report. Callers: the
  Rust runtime on panic; W11's NC4 trigger (validation selection).
Inputs / outputs: PanicInfo (location and message as available) plus the
  handler-entry state pair (SP, LR captured at handler entry — the
  approximate call-site location the report labels as
  handler-entry-captured); output is one kind P report through the
  transport seam, then the terminal stop.
Preconditions / postconditions: callable from any phase after W02
  establishment stage 3 (static data ready; W02's own contract) — with
  the degradation rules of §3 of the report model covering everything
  else. Never returns.
State and ownership change: sets the shared guard; captures the entry
  pair into stack locals; writes the selected transport only.
Concurrency/allocation context: no allocation; no locking; the guard is
  the entire synchronization story (single CPU, masks set — the W02
  guard discipline, transferred).
Errors and failure guarantee: bounded by construction: fixed capacity,
  fixed line count, no re-entry (guard-true → silent bounded stop), no
  unwinding, no fallback work. The stop is reached whether or not output
  succeeds (W02 R4 posture).
Security/authorization checks: the report reflects invariants, machine
  facts, and static identity only — it never echoes memory contents.
Report (order and fields per report model §4, kind P rows).
Logic:
  (sp0, lr0) = fatal_entry_state_read()          # audited boundary, §7
  if guard was already set: bounded_stop
  set guard
  render kind P (common context; msg/loc + entry pair core)
  emit via transport seam (§5)
  emit end marker; bounded_stop
Validation: W07-DV02 (contract review); NC4 execution belongs to W11;
  non-recursion is review-checked here (W07-DV04).
```

## 2. Exception entry — `report_fatal_exception`

```text
Name and stability: fn report_fatal_exception(frame: &ExceptionFrame,
  cls: SyndromeClass, disp: ExceptionDisposition) -> !; internal;
  diverging; stable within P1.
Purpose and caller: the post-arm full report for classified exceptions —
  the seam [W05's routing
  contracts](../p1-w05-el2-exception-entry-baseline/04-code-contracts-classification-routing.md)
  §4 consume once this design arms. Caller: W05's `route_classified`
  (post-arm branch) only.
Inputs / outputs: W05's completed frame, syndrome class, and
  disposition; output is one kind E report, then the terminal stop.
Preconditions / postconditions: called only by W05's router after
  `fatal_path_ready()`; the caller holds the W05 entry-path guard. Never
  returns. Postcondition: report emitted or transport contained; stop.
State and ownership change: none of its own (the W05 guard is already
  held; this entry does not re-guard — one guard per event path, §6).
Concurrency/allocation context: exception context; masks set; no
  allocation; bounded buffers.
Errors and failure guarantee: bounded by construction; a fault during
  this report is a second fatal entry — reached paths are guarded, so it
  terminates silently (P1-V12).
Security/authorization checks: frame contents are machine facts; no
  dereference of fault-controlled addresses; no enrichment.
Report (kind E rows; every required field present — the frame carries
  them all).
Validation: W07-DV02/DV03; NC3 (post-arm variant), NC5, NC6 executions
  belong to W11.
```

## 3. Phase-failure entry — `report_fatal_phase`

```text
Name and stability: fn report_fatal_phase(phase: InitPhase,
  reason: FailureReason) -> !; internal; diverging; stable within P1.
Purpose and caller: the full report for W09's `fail_phase` Stage1/Stable
  arms (W09 §7) — the non-frame failure class, including W08's
  transition/step failures (its plan work seq 4 obligation). Caller:
  W09's `fail_phase` only.
Inputs / outputs: failing phase + W09's `FailureReason`; output is one
  kind F report, then the terminal stop.
Preconditions / postconditions: called only where the W09 matrix routes
  here (post-arming by construction — Stage1/Stable follow the
  `fatal-path` phase). Never returns.
State and ownership change: sets the shared guard (the caller came from
  boot context, not the guarded exception path).
Concurrency/allocation context: boot context; no allocation.
Errors and failure guarantee: bounded by construction; guard-true second
  entry stops silently.
Security/authorization checks: reason payloads are static strings from
  the supplying contracts; nothing untrusted reaches the report.
Report (kind F rows: phase + reason core; unavailable fields labeled).
Validation: W07-DV02; W08/W09 closure reviews read the same seam;
  scenario execution (transition failures) belongs to W08/W11 evidence.
```

## 4. Terminal behavior

```text
Name and stability: fatal_bounded_stop() -> !; internal; stable within
  P1 (the W01/W02/W05 stop discipline, shared by all three kinds).
Purpose and caller: the defined terminal outcome of every kind. Caller:
  each entry after the end marker; the guard-true paths.
Inputs / outputs: none; never returns.
Preconditions / postconditions: masks remain set; no further output.
State and ownership change: none.
Concurrency/allocation context: any fatal-path context; no allocation.
Errors and failure guarantee: cannot fail; the harness (W10) owns the
  wall-clock interpretation of the halt.
Security/authorization checks: none.
Logic: branch-to-self.
Validation: W07-DV04; observed by every executed fault scenario (W11).
```

## 5. Transport seam (work seq 3)

```text
Name and stability: fn emit_report_line(line: &str); internal; stable
  within P1.
Purpose and caller: the single output chokepoint implementing the
  transport preference. Caller: the renderer for every report line.
Inputs / outputs: one bounded line; transmitted bytes.
Preconditions / postconditions: preference evaluated once per report
  (not per line) and held for the whole report: W06's channel when
  channel_available(), else W02's early_write_bytes. Never switches
  mid-report (architecture file §6).
State and ownership change: the selected transport's registers only.
Concurrency/allocation context: boot or exception context; no
  allocation; callable with masks set (W06 §1 guarantee; W02 writer
  contract).
Errors and failure guarantee: no error path; stall semantics are the
  selected transport's; the stop does not depend on output success.
Security/authorization checks: lines are renderer-produced bounded
  content.
Logic:
  if channel_available(): console_write_line(line)
  else: early_write_bytes(line.as_bytes()); early_write_bytes(CRLF)
Validation: W07-DV03; NC4/NC5 observations (W11, deferred) exercise both
  arms across the transition variants.
```

## 6. Guard and recursion containment (work seq 4)

```text
Name and stability: FATAL_PATH_GUARD (one static flag inside the audited
  boundary) with fn fatal_guard_acquire() -> bool; internal; stable
  within P1.
Purpose and caller: the single-entry discipline for the whole fatal path,
  transferred from W02's panic-entry guard (parent README decision 2).
  Callers: the panic entry and the phase-failure entry (acquire);
  W05's router holds its own entry-path guard before calling the
  exception entry, so the exception entry checks-and-stops rather than
  re-acquires.
Inputs / outputs: none; acquire returns false if already held.
Preconditions / postconditions: on acquire-true, the caller must reach a
  terminal stop; the guard is never released (every path through it is
  terminal — release is meaningless in P1 and is not implemented).
State and ownership change: the guard flag only.
Concurrency/allocation context: plain static flag; single CPU, masks set;
  the same justification family as W02's guard (audited boundary; P0
  unsafe inventory).
Errors and failure guarantee: guard-true → silent bounded stop (no
  output, no recursion).
Security/authorization checks: none.
Layering with W05's guard: W05's guard is set first (exception entry)
  and stays set; a fault while W07's renderer runs finds the fatal guard
  rules already effective — either guard being held yields the silent
  stop. The two flags exist because the two paths arm at different
  phases (W05's from `exceptions`, the fatal path's from its first
  entry); merging them is a recorded design change, not a local edit.
Validation: W07-DV04 walk; W11's terminal-outcome checks observe the
  property.
```

## 7. Arming contract (the `fatal-path` phase body)

```text
Name and stability: fn arm_fatal_path(); internal; stable within P1.
  (Naming rule: W09 owns `fatal_path_step`; the supplying mechanism does
  not reuse it — W03 §6 precedent.)
Purpose and caller: perform the readiness assertions and set
  FATAL_PATH_READY. Caller: run_init_sequence via the W09
  `fatal_path_step` adapter (W09 §8: "Fatal report entry; startup-phase
  field consumer").
Inputs / outputs: none; on normal return the fatal path is ready.
Preconditions / postconditions: W09 phase prerequisites (console
  complete). Postcondition: FATAL_PATH_READY true.
State and ownership change: the readiness once-flag only.
Concurrency/allocation context: boot context; no allocation.
Errors and failure guarantee: an assertion failure routes via W07's own
  non-recursive readiness boundary (architecture §6): one minimal marker
  line through the early writer (best effort), then the bounded stop.
  The flag stays false — W05's router keeps using the pre-arm summary,
  and W09's misuse routing keeps using the panic route. No fabricated
  readiness.
Security/authorization checks: none.
Asserted set: identity resolves or recorded degradation is in force;
  guard clear; a transport reachable (channel_available() or the linked
  writer constant resolves); renderer linked (by construction).
Logic:
  for assertion in the set: if !ok -> readiness_boundary_route()
  set FATAL_PATH_READY (audited once-write cell)
Validation: W07-DV03/DV04; W05-DV07 consumability reads the readiness
  declaration from its side.
```

The audited `unsafe` boundary of W07 is exactly: the guard and once-flag
statics (per the shared pattern family), the one `CurrentEL` read used by
the common context block, and `fatal_entry_state_read()` — a read-only
asm primitive returning the current SP and LR at panic-handler entry
(closed-register, no writes). No other register access exists in the
package; the approximation semantics (LR as an approximate call-site
address, invalidated by tail calls and inlining) are recorded in the
implementation record and carried into W12's contract content.
