# P6-W05 Code Contracts — Time Values and Monotonic Clock

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Parent:** [P6-W05 design entry](README.md). Contracts follow the
implementation-design checklist §3 template. All signatures are pseudocode:
they fix names, types, and semantics for design review; the implementation
record maps them onto the crate/module layout chosen at implementation time
subject to ADR layering (no system-register access outside the Arch-domain
module).

## 1. Types

### 1.1 `TimerCounter`

```text
Name and stability: TimerCounter(newtype over u64) — internal to W05
  consumers (W06, W13) may read but not construct it. Stable within P6.
Purpose and caller: an observed value of the EL2 physical counter
  (CNTPCT_EL0). Produced by clock_read()/read_at_service(); consumed by
  deadline construction, monotonicity checks, and late-fire measurement.
Inputs / outputs: none (value type).
Preconditions / postconditions: values are monotonically non-decreasing
  across successive observations on the platform; no ordering guarantee is
  implied between different counters if intake ever allowed multiple
  domains (it does not in P6).
State and ownership change: none (value semantics).
Concurrency/allocation context: construction performs one ordered counter
  read (see 1.4); no allocation; callable in IRQ context.
Errors and failure guarantee: construction cannot fail; mis-ordering is a
  property of the observer, detected by checks (1.5), not by the type.
Security/authorization checks: none (Host-internal trusted source).
Logic: single-field newtype; Display/Debug formatting allowed; arithmetic
  on counters is only via the functions below — never raw u64 math at call
  sites (Coding Guidelines).
Validation: host-side unit tests for type behavior; QEMU ordering checks
  (W05-DV03).
```

### 1.2 `TimerDelta`

```text
Name and stability: TimerDelta(newtype over u64) — internal to W05 and its
  consumers; stable within P6.
Purpose and caller: a duration in counter ticks. Constructed by conversion
  from milliseconds/microseconds via TimerFrequency, or directly from a tick
  count for architecture-known intervals.
Inputs / outputs: none (value type).
Preconditions / postconditions: a valid delta is representable in u64 ticks
  and, when converted from time units, was produced through the checked
  conversion (1.3).
State and ownership change: none.
Concurrency/allocation context: value semantics; no allocation; IRQ-safe.
Errors and failure guarantee: construction from time units is fallible
  (1.3); direct tick construction is infallible.
Security/authorization checks: none in P6 (no guest-controlled deltas reach
  W05; W06 must apply its own validation before converting guest-derived
  quantities).
Logic: newtype; saturating comparisons only via provided functions.
Validation: host-side unit tests including boundary deltas (0, u64::MAX).
```

### 1.3 `TimerFrequency` and checked conversion

```text
Name and stability: TimerFrequency(newtype over u32 ticks/second) with
  associated functions:
    fn from_intake(raw_hz: u32) -> Result<Self, IntakeError>
    fn delta_from_micros(&self, micros: u64) -> Result<TimerDelta, ConversionError>
    fn delta_from_millis(&self, millis: u64) -> Result<TimerDelta, ConversionError>
  — internal to W05; stable within P6.
Purpose and caller: carries the intake-validated counter frequency; the sole
  authority for time-unit-to-tick conversion (arm callers, W06 conversion,
  diagnostics thresholds).
Inputs / outputs: raw Hz value from intake; micros/millis to ticks.
Preconditions / postconditions: from_intake rejects zero and values outside
  a declared plausible range; post: the stored frequency is the one used for
  all conversions (no ambient constant).
Concurrency/allocation context: value semantics; integer mul/div only; no
  allocation; IRQ-safe; overflow-checked.
Errors and failure guarantee: ConversionError::Overflow when micros * freq
  exceeds u64 (checked_mul), ConversionError::TooLarge when the resulting
  tick count exceeds the declared maximum armable horizon (design constant,
  see 2.2); inputs unchanged on error.
Security/authorization checks: from_intake is the trust boundary for the
  platform-supplied frequency (untrusted until validated).
Logic:
    delta_from_micros(m):
        ticks = m.checked_mul(freq_hz)? / 1_000_000   (order chosen so the
        division follows the multiply; intermediate overflow returns
        Overflow before any truncation)
        if ticks > MAX_ARM_HORIZON_TICKS { return TooLarge }
        return TimerDelta(ticks)
Validation: host-side property tests (round-trip against a reference table;
  overflow boundaries; zero-frequency rejection).
```

## 2. Deadline arithmetic

### 2.1 `TimerDeadline` and `deadline_after`

```text
Name and stability: TimerDeadline(newtype over u64) — the compare-value
  domain value; plus
    fn deadline_after(base: TimerCounter, delta: TimerDelta)
        -> Result<TimerDeadline, DeadlineOverflowError>
  — internal to W05; the deadline-timer module is the primary caller; W06
  converts virtual-counter deadlines through its own explicit domain
  conversion (never reuses this function on unconverted values).
Purpose and caller: compute the absolute counter value at which the timer
  must fire, with checked arithmetic (D2).
Inputs / outputs: observed counter + delta -> absolute deadline.
Preconditions: base was observed with the ordering discipline (1.4);
  postconditions: on success, deadline is representable and unconsumed by
  any prior arm.
State and ownership change: none.
Concurrency/allocation context: pure checked arithmetic; IRQ-safe; no
  allocation.
Errors and failure guarantee: DeadlineOverflowError on base + delta
  overflow (u64); inputs unchanged.
Security/authorization checks: none (Host-internal values only).
Logic: base.0.checked_add(delta.0).map(TimerDeadline) with the error mapped.
Validation: host-side unit tests for wrap boundary.
```

### 2.2 Armable-horizon constant

```text
Name and stability: MAX_ARM_HORIZON_TICKS — declared design constant
  (stage-local design freedom, owned by this design; rationale below).
Purpose: bounds a single arm to a horizon far beyond any P6 use (declared
  value chosen at implementation from the intake frequency so the horizon is
  at least one hour of counter time), giving cancel/rearm and monotonicity
  checks a sane envelope and rejecting absurd deltas before they reach
  hardware.
Rationale for ownership: the task book leaves timer programming policy to
  detailed design; a horizon bound is a mechanism-safety bound, not a
  scheduling policy. P7 may propose a different bound through its own design.
```

## 3. Monotonic clock read

### 3.1 `clock_read()`

```text
Name and stability: fn clock_read() -> TimerCounter — internal to W05 and
  its consumers (W06 deferred-expiry evaluation; W13 latency sampling may
  layer on it). Stable within P6.
Purpose and caller: the sole sanctioned observation of the EL2 monotonic
  counter.
Inputs / outputs: none -> current counter value.
Preconditions: intake passed (frequency validated) on the executing pCPU;
  postconditions: the returned value was read under the ordering discipline
  below and is comparable with other TimerCounter values.
State and ownership change: none.
Concurrency/allocation context: single ordered read; callable from IRQ
  context and VM-exit context; no allocation; bounded O(1).
Errors and failure guarantee: infallible; intake failure is a bring-up
  error, not a read error.
Security/authorization checks: none (trusted Host source).
Logic (pseudocode; the exact instruction spelling is fixed against the
locked architecture revision at implementation, per task book §8):
    // Prevent the counter read from being hoisted above prior
    // system operations whose effects the caller is ordering against.
    instruction_barrier()            // ISB
    value = read_sysreg(CNTPCT_EL0)  // volatile system-register read
    return TimerCounter(value)
Barrier rationale: the architecture permits speculative counter reads; the
ISB orders the read with respect to preceding context (e.g. after disabling
a timer, after Guest exit bookkeeping) so decisions use post-event values.
Validation: QEMU monotonicity checks (W05-DV03, W05-DV08); the read appears
in the unsafe inventory with its SAFETY justification.
```

### 3.2 `monotonic_pair_check()`

```text
Name and stability: fn monotonic_pair_check(before: TimerCounter,
  after: TimerCounter) -> Result<(), MonotonicityViolation>
Purpose and caller: decision helper asserting observed non-regression; used
  by the entry/exit monotonicity scenario and any W05-internal ordering
  assertion.
Preconditions: both values were produced by clock_read() on the same pCPU.
Errors: MonotonicityViolation carries both values for diagnostics; it is a
  counted diagnostic (never a panic in released profiles — a Guest entry/
  exit window must not turn a host observation into a fatal event).
Logic: if after.0 < before.0 { Err } else { Ok } (equality allowed:
  the counter may not advance between two close reads).
Validation: host-side unit tests; QEMU scenario W05-DV08.
```

## 4. Explicitly excluded

No wall-clock epoch, no offset/scaling conversion, no `Instant`-style
cross-domain API, no sleep/delay loop, no interface exposing raw u64 counter
values (all observations are typed), and no conversion between the physical
counter domain and the Guest virtual-counter domain (that conversion is
W06-owned with an explicit `CNTVOFF` value; see
`../p6-w06-guest-generic-timer/README.md`).
