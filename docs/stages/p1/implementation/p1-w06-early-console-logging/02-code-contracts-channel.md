# P1-W06 Channel Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W06 detailed design](README.md).

Pseudocode is an outline, not runnable production code. No allocation; the
bounded formatter discipline is
[W02's formatting contract](../p1-w02-minimal-rust-el2-runtime/04-code-contracts-panic-identity.md)
§4 (fixed stack buffer; truncation, never panic; `core::fmt` against an
in-memory sink only if allocation-free). All names are internal boot-scope
items owned by this design.

## 1. Channel phase body — mechanism entry

```text
Name and stability: fn bring_up_early_console(); internal; stable within
  P1. (Naming rule: W09 owns the `console_step` adapter name; the
  supplying mechanism does not reuse it — W03 §6 precedent.)
Purpose and caller: the `console` phase's entire mechanism — init the
  transport, emit the start line, set the availability signal. Caller:
  run_init_sequence via the W09 `console_step` adapter (W09 §8: "Channel
  write; availability signal; marker format").
Inputs / outputs: none; on normal return the channel is available.
Preconditions / postconditions: W09 phase prerequisites (exceptions
  complete; vectors live). Postcondition: transport initialized; start
  line emitted; CHANNEL_AVAILABLE true.
State and ownership change: UART hardware state (init); the availability
  flag; nothing else.
Concurrency/allocation context: boot context; DAIF masked; no allocation.
Errors and failure guarantee: an init failure constructs ConsoleError and
  routes via the W09 matrix `console` row — best-effort panic-route
  attempt, then bounded stop; no silent continuation, no retry, no
  degraded "partial channel" state (W09 H2).
Security/authorization checks: none; bring-up output is not an
  authorization point.
Logic:
  console_init()?                                # §2; Err -> route_console_failure
  emit_line(channel_start_line())                # §3 start line w/ identity
  set_channel_available()                        # §4 signal
Validation: W06-DV02; W09-DV02 reads the same point (replay trigger).
```

## 2. Transport — init and polled write

```text
Name and stability: fn console_init() -> Result<(), ConsoleError>;
  fn console_write_bytes(bytes: &[u8]); fn console_write_line(s: &str);
  internal; stable within P1.
Purpose and caller: the only console MMIO access in P1 outside W01/W02's
  recorded paths. Callers: the phase body; marker/content transport;
  W07's renderer (via console_write_line); the mapping-requirement
  interface (constant only).
Inputs / outputs: byte slices / strings; transmitted bytes.
Preconditions / postconditions: console_write_* require channel
  availability (post-phase); callers' contracts inherit the precondition.
  On return from write, all bytes were handed to the transmitter or the
  transmitter is stuck (see below — no error path, no drop).
State and ownership change: UART data/flag registers; flag register at
  init.
Concurrency/allocation context: boot or guarded exception context; no
  allocation; no lock (single-consumer discipline per the architecture
  file §6); volatile MMIO access only.
Errors and failure guarantee: init returns ConsoleError on an unexpected
  register state (the read-back of the init sequence mismatches — the W04
  read-back posture); writes have no error path: the transmit poll loops
  until the flag clears. A permanently stuck transmitter therefore loops
  in the poll — the recorded limitation; W10's wall-clock bound is the
  bounding mechanism (parent README decision 6). No software timeout, no
  byte dropping (decision 6).
Security/authorization checks: writes only caller-supplied bounded
  content; no formatting of untrusted data (P1 has no untrusted input
  path; the rule is stated for future reuse).
Init logic (minimal; exact sequence per the reference platform, recorded):
  program the enable bits required for polled transmit
  read back the enable state; mismatch -> Err(ConsoleError)
Write logic:
  for byte in bytes: poll flag until writable; store byte (volatile)
Line logic: write_bytes(s.as_bytes()); write_bytes(CRLF)
Validation: W06-DV02/DV03; the volatile-access and barrier discipline of
  the Coding Guidelines' MMIO section applies (Device memory ordering is
  architectural; no additional barrier is required for polled TX on the
  reference platform — recorded implementation note).
```

The audited `unsafe` boundary is exactly the MMIO accessor pair over the
two register kinds (data, flag) plus the init writes — one primitive per
direction over a closed register set, with the region constant defining
the only valid base (P0 unsafe inventory entry; same shape as W04 §3).

## 3. Channel start line and identity association

```text
Name and stability: fn channel_start_line() -> BoundedLine; internal;
  stable within P1.
Purpose and caller: satisfy P0-W12's identity association for the human
  log at the moment the channel becomes available. Caller: the phase
  body, once.
Inputs / outputs: none; one bounded line.
Preconditions / postconditions: transport initialized; identity resolves
  or degrades to W02's recorded unavailable literal (never fabricated —
  the W02 BuildIdentity rule).
State and ownership change: none.
Concurrency/allocation context: boot context; bounded buffer; no
  allocation.
Errors and failure guarantee: cannot fail; truncation only.
Security/authorization checks: identity is diagnostic content.
Logic:
  format "<CHANNEL-prefix> ready ident=<identity>" into the bounded buffer
Validation: W06-DV02; P0-W12 semantic conformance noted in W06-DV01.
```

## 4. Availability signal

```text
Name and stability: fn channel_available() -> bool; internal; stable
  within P1.
Purpose and caller: the availability signal W09's design consumes — the
  materialization trigger for deferred markers (W09 §5) and the transport
  preference input for W07's renderer. Callers: W09 `console_step`
  wiring; W07 renderer; review evidence.
Inputs / outputs: none; the flag.
Preconditions / postconditions: false until the phase body sets it;
  monotone within a boot; never fabricated (a false read before the
  `console` phase is the honest state, and consumers' preconditions are
  phase-gated, not flag-gated — the flag is evidence, not a lock).
State and ownership change: set exactly once by the phase body inside the
  audited once-write cell (the shared pattern family; SAFETY: single boot
  CPU, DAIF masked, one boot path).
Concurrency/allocation context: no allocation; readers from boot or
  exception context.
Errors and failure guarantee: cannot fail.
Security/authorization checks: none.
Logic: set/read inside the once-write cell.
Validation: W06-DV02; W09-DV02 (the replay trigger reads this signal).
```

## 5. Marker line format and content transport

```text
Name and stability: fn format_marker(phase_label: &str, event: MarkerEvent)
  -> BoundedLine; fn transport_line(line: &str); internal; stable within
  P1. MarkerEvent (Enter | Complete) mirrors W09's event kinds.
Purpose and caller: the W06-owned framing of W09-owned vocabulary (parent
  README decision 5). Callers: W09's `record_marker` emission path
  (markers, including the replay's); W03's emit callback implementation
  (content lines); W07's renderer (report lines).
Inputs / outputs: phase label and event kind; or a complete producer line;
  output through §2's write path.
Preconditions / postconditions: channel available; line length bounded by
  the channel's maximum (constant; sizing recorded with the formatter
  arithmetic) — an over-long producer line truncates with a recorded
  truncation marker, never panics (W09 M4: W06 does not interpret content,
  but it does bound transport).
State and ownership change: none beyond the UART.
Concurrency/allocation context: boot or guarded exception context; bounded
  stack buffer; no allocation.
Errors and failure guarantee: cannot fail; truncation only.
Security/authorization checks: labels are static vocabulary; content lines
  arrive from contracted producers (W03/W07) whose own contracts bound
  their content.
Line format (fixed):
  "ZELYR P1 PHASE <phase-label> <enter|complete>\r\n"
  — the prefix literal is the marker class W10 matches as boot-start
    evidence; exact text recorded before the first verdict-bearing run.
Validation: W06-DV02 (format review against W09's vocabulary); W10
  consumes the class (W06-DV06).
```

## 6. Mapping-requirement interface (for W08)

```text
Name and stability: CONSOLE_REGION (static: base = the reference-console
  constant, size = the implemented register-window size) and
  CONSOLE_REQUIRED_ATTRIBUTES (Device memory, execute-never, read/write)
  as documented constants; internal; stable within P1.
Purpose and caller: the named input of W08's mapping-class table for the
  early-console MMIO class (its plan work seq 1 inventory). Caller: W08's
  design/implementation reads the contract; reviews cite it.
Inputs / outputs: none; declarative constants.
Preconditions / postconditions: the constants state the requirement; they
  do not perform mapping and carry no runtime behavior.
State and ownership change: none.
Concurrency/allocation context: none.
Errors and failure guarantee: cannot fail.
Security/authorization checks: none.
Logic: none — a recorded requirement.
Validation: W06-DV03/DV04 (requirement stated; class confirmed in W08's
  table).
```
