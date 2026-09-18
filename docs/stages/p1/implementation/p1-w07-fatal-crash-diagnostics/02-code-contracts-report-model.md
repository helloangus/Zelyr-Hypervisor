# P1-W07 Report Model Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W07 detailed design](README.md).

Pseudocode is an outline, not runnable production code. No allocation; the
bounded formatter discipline is
[W02's formatting contract](../p1-w02-minimal-rust-el2-runtime/04-code-contracts-panic-identity.md)
§4 (fixed stack buffer, truncation never panic). All names are internal
boot-scope items owned by this design; the marker prefixes and field
vocabulary are P1-internal boot diagnostics consumed by W10/W11 matching
rules, not an ABI.

## 1. Required-context field set (work seq 1)

The P1-V11 required context, mapped to sources and kinds. `R` = required
in every report of the kind; `N/A` = not available to the kind (labeled,
never fabricated — parent README decision 6).

| Field | Source | Kind P (panic) | Kind E (exception) | Kind F (phase failure) |
|---|---|---|---|---|
| Marker class | link-time prefix (§4) | panic-class | fatal-class | fatal-class |
| Build/version identity | W02 `BuildIdentity` accessor (or recorded unavailable literal) | R | R | R |
| CPU / exception level | `CurrentEL` read at report time (audited boundary, [fatal path](03-code-contracts-fatal-path.md) §7) | R | R | R |
| Startup phase | W09 tracker position → `LifecyclePosition` (`Unknown(raw)` renders raw) | R | R | R |
| PC / return state | kind P: handler-entry SP + LR capture (approximate call-site location, labeled as handler-entry-captured); kind E: `frame.pc` (ELR_EL2) + `frame.spsr` (SPSR_EL2); kind F: N/A | R (entry pair) | R | N/A |
| Syndrome | kind E: `frame.esr` (ESR_EL2) + W05 `SyndromeClass` label (+ IL bit if set); kind P/F: N/A | N/A | R | N/A |
| Fault address | kind E: `frame.far` when `far_valid`, else N/A label (HPFAR rendered when valid — architecturally never in P1, recorded); kind P: N/A; kind F: N/A | N/A | R (as available) | N/A |
| Panic message | kind P: `PanicInfo` message slice (truncated); kind E/F: N/A | R (if present; else labeled) | N/A | N/A |
| Register context | kind E: `frame.x[0..31]` + `frame.sp`; kind P: the handler-entry SP/LR pair only (general registers are not captured on the panic path — labeled); kind F: N/A (labeled) | R (entry pair) | R | N/A (labeled) |
| Disposition / reason | kind E: W05 `ExceptionDisposition` + origin/category labels; kind F: the `FailureReason` payload | disposition line rendered for E; kind F carries reason | R | R |

Kind F carries `InitPhase` and the reason carrier (W09's `FailureReason`:
the failing phase and a short static reason; richer payloads belong to the
supplying contracts, W09 §7).

## 2. Field availability classes

Each field is one of:

- `Required` — present in every report of the kind; its absence is a
  defect of the report, not a runtime contingency.
- `AsAvailable` — present when the source is architecturally valid for
  the event (fault address under `far_valid`; panic message when
  `PanicInfo` carries one); rendered with the N/A label otherwise.
- `UnavailableByKind` — the kind's input cannot carry it; always the N/A
  label. This is a truthful property of the kind, recorded in the
  implementation record and in W12's contract content — never a silent
  omission.

## 3. Degradation rules (partial-init and unsupported cases — work seq 4)

- Identity unresolved → the recorded unavailable literal (W02 rule); the
  report continues.
- Tracker at `PreBoot`/undecodable → `Unknown(raw)` rendered; continues.
- Panic message absent → labeled "no message"; continues.
- Truncation at any field or line → truncation applied, report continues;
  the formatter never panics (W02 §4).
- W03 rejection flowing through the panic route (unsupported capability,
  pre-fatal-path) → kind P report with the message carrying W03's
  rejection vocabulary and phase attribution `capabilities` — the
  matrix's explicit-requirement reason, not an unrelated panic (W09
  `capabilities` row).
- No degradation path constructs a fabricated value or aborts the report.

## 4. Output ordering (fixed)

One report is a fixed line sequence; ordering is part of the contract
(reviewable, and stable for W10/W11 evidence comparison):

```text
line 1  <marker-class prefix> <kind-tag>            [class identification]
line 2  build=<identity>                             [common context]
line 3  cpu=EL<x> el_ok=<bool>                       [common context]
line 4  ph=<LifecyclePosition rendering>             [common context]
line 5+ kind-specific core, in this order:
        P: msg=<message> loc=<file:line> sp0=<16-hex> lr0=<16-hex>
        E: org=<origin> cat=<category> disp=<disposition>
           esr=<16-hex> cls=<syndrome label> il=<bool>
           pc=<16-hex> spsr=<16-hex>
           far=<16-hex | na> hpfar=<16-hex | na>
           x00..x30=<16-hex each> sp=<16-hex>
        F: phase=<label> reason=<reason>
last    <end-marker literal>                          [terminal line]
        -> bounded stop
```

Rules: every line is rendered through the bounded formatter; fixed-width
hex (16 digits, no prefixes); the end marker literal is recorded with the
prefixes and lets W10/W11 check "nothing after the terminal marker"
(W11's acceptance conjunction).

## 5. Marker classes and sizing arithmetic (work seq 2)

```text
Name and stability: PANIC_CLASS_PREFIX, FATAL_CLASS_PREFIX, END_MARKER
  (link-time literals); internal; stable within P1.
Purpose and caller: the machine-matchable identification of report kinds.
  Callers: the renderer (emission); W05's pre-arm summary (fatal class);
  W10's forbidden-class matching; W11's scenario expectations.
Inputs / outputs: none; constants.
Preconditions / postconditions: distinct from each other, from W06's
  marker/channel prefixes, from W03's `cap` lines, and from QEMU's own
  output (uniqueness reviewed whenever any token changes — W10 R6
  posture). Recorded in the implementation record before the first
  verdict-bearing run; never adjusted afterwards.
State and ownership change: none.
Concurrency/allocation context: none.
Errors and failure guarantee: cannot fail.
Security/authorization checks: none.
Sizing arithmetic (recorded with the implementation): the report buffer
  is a fixed stack array sized to the longest contracted line plus slack;
  the register block (32 × 18 characters + labels) is the sizing
  driver — the arithmetic and the resulting capacity are recorded in the
  implementation record and re-reviewed when any field grows (Reserved
  trigger, parent README).
Validation: W07-DV02 (ordering/prefix review); W10-DV02 consumes the
  classes.
```
