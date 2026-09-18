# P5-W02 Code Contracts — ABI Surface

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W02 detailed design](README.md). Contracts follow the
implementation-design checklist §3 template. All names are **internal Rust
API**: ABI stability attaches to the register conventions, call numbers, and
status codes (the guest-visible surface), not to these identifiers, which may
be renamed during implementation without an ABI event. Pseudocode is logic,
not runnable production code.

## 1. ABI constants (the guest-visible surface)

```text
Name and stability: ABI envelope constants — guest-visible, fixed by this
  design under task book §8; change after implementation = ABI compatibility
  event routed per 06-validation-and-handoff.md §3.
Purpose and caller: every module of 02-architecture-and-state.md; the guest
  assembler helper in W07's asset; the factual ABI document (later).
Constants:
  HVC_IMMEDIATE        = 0                    // the only valid entry immediate
  CALL_DISCOVERY       = 0                    // X0 value for discovery
  CALL_MINIMAL         = 1                    // allocated by this ABI; the call's
                                              // semantics are W06's design
  ABI_VERSION_WORD     = 0x0000_0001          // major 0, minor 1 (Decision 4)
  FEATURE_BASE_SET     = 1 << 0               // bit 0: base v0 calls present
  STATUS_*             = per 04-code-contracts-error-boundary.md §2
Layout rules:
  - Call numbers and status codes: u32 semantics in X0/X registers.
  - A logically 32-bit field occupies bits[31:0]; bits[63:32] reserved zero.
  - Register roles per Decision 3/8: entry X0 call, X1 version (0 for
    discovery), X2–X4 arguments, X5–X7 reserved-zero; return X0 status,
    X1–X3 results (OK only), X4–X7 zero.
Validation: constant review against this file; guest-side and host-side
  constants must be generated from one source or reviewed in lockstep.
```

## 2. `HypercallFrame` (type)

```text
Name and stability: HypercallFrame (Arch module M1); internal.
Purpose and caller: the immutable input of the whole boundary; created by the
  trap adapter, read by decode/dispatch, written-through by the composer via
  an Arch-provided result view.
Inputs / outputs: captures, at minimum — x0..x7 (ABI-defined set), the ESR
  ISS immediate, the origin exception level, and the current Guest execution
  context reference (caller identity, W01 vocabulary). Additional saved
  context stays owned by the P4 entry/exit mechanism.
Preconditions / postconditions: exists only for an HVC-class exception whose
  origin is Guest EL1 (Decision 11); immutable after construction; dead after
  Guest resumption.
State and ownership: per-trap value; no shared state; the underlying register
  storage remains owned by the P4 context mechanism — the frame aliases it
  immutably until composition.
Concurrency/allocation: constructed in synchronous-exception context;
  allocation-free; per-pCPU.
Errors and failure guarantee: construction cannot fail; a trap that cannot be
  framed as an EL1-origin HVC never becomes a HypercallFrame (returns to the
  P1/P4 path).
Security/authorization checks: every field is untrusted data by definition;
  the frame carries no sanitized values.
Logic: capture-once struct; no accessor mutates storage; the composer uses a
  separate mutable result view (see §7) rather than interior mutability.
Validation: Arch-level review that capture matches the P1/P4 saved-context
  layout; QEMU trap test (W07) exercises real frames.
```

## 3. `decode_envelope` (function, M2)

```text
Name and stability: decode_envelope(frame) -> Result<DecodedCall,
  HypercallStatus>; internal; total.
Purpose and caller: sole envelope authority; called once per trap by the
  dispatch flow (W06); also called directly by host tests and fuzz harnesses
  (W08 seam).
Inputs / outputs: in — &HypercallFrame. out — DecodedCall { call_number: u32,
  requested_version: Option<AbiVersionWord>, args: [u64; 3] } or a
  HypercallStatus rejection.
Preconditions / postconditions: frame satisfies the type contract (§2).
  Post: on Ok, call_number has only registry-meaningful width used (no
  junk in bits[63:32]), reserved registers x5–x7 are zero, the immediate was
  HVC_IMMEDIATE, and — for call 0 — x1 is zero; for other calls,
  requested_version parses. On Err, exactly one of:
  MALFORMED_REQUEST (any envelope violation), or nothing else — decode
  returns only that status class.
State and ownership: none; pure.
Concurrency/allocation: thread-safe by purity; allocation-free; no blocking.
Errors and failure guarantee: total — every input yields Ok or Err; no panic
  on any bit pattern; no partial output on Err.
Security/authorization checks: all inputs untrusted; reserved-half and
  reserved-register verification is the forgery-narrowing step; no identity
  is consulted.
Logic:
  function decode_envelope(frame):
    if frame.immediate != HVC_IMMEDIATE:  return Err(MALFORMED_REQUEST)
    if frame.x0 > u32::MAX:               return Err(MALFORMED_REQUEST)   // bits[63:32] nonzero
    if frame.x5 != 0 or frame.x6 != 0 or frame.x7 != 0:
                                         return Err(MALFORMED_REQUEST)
    let call = frame.x0 as u32
    if call == CALL_DISCOVERY:
      if frame.x1 != 0:                  return Err(MALFORMED_REQUEST)
      return Ok(DecodedCall { call, requested_version: None,
                              args: [0, 0, 0] })   // discovery takes no args
    if frame.x1 > u32::MAX:              return Err(MALFORMED_REQUEST)
    let ver = parse_version_word(frame.x1 as u32)?   // 0 is invalid -> malformed
    return Ok(DecodedCall { call, requested_version: Some(ver),
                            args: [frame.x2, frame.x3, frame.x4] })
Validation: host unit tests per envelope case (bad immediate, junk halves,
  nonzero reserved registers x5–x7, zero version, max-value fields, all
  three argument slots carrying arbitrary bits); property test: totality
  and "Err is always MALFORMED_REQUEST" (W08).
```

Note: the argument slots carried in `DecodedCall` (X2–X4) are opaque u64
values; their semantics belong to the call's own contract (W06 minimal
call) or to the W03/W04 boundaries. Decode grants them no interpretation.

## 4. `AbiVersionWord` (type + functions, M4)

```text
Name and stability: AbiVersionWord(u16 major, u16 minor) with
  parse_version_word / to_version_word / check_compatibility; internal.
Purpose and caller: version discovery (call 0) and the per-call stateless
  compatibility check (S5).
Inputs / outputs: parse — u32 -> Option<AbiVersionWord> (zero word invalid,
  per Decision 4); to_version_word — (major, minor) -> u32;
  check_compatibility(host, requested) -> Result<(), HypercallStatus>.
Preconditions / postconditions: check_compatibility requires a parsed
  requested word; Ok iff host.major == requested.major and
  requested.minor <= host.minor.
State and ownership: none; constants only.
Concurrency/allocation: pure; allocation-free.
Errors and failure guarantee: parse is total; check returns
  VERSION_MISMATCH as the only Err.
Security/authorization checks: untrusted input; an unparsable word never
  becomes a version (malformed, not mismatched — Decision 5 note in §3 of
  decode).
Logic:
  function parse_version_word(w):
    if w == 0: return None
    return Some(AbiVersionWord(major = w >> 16, minor = w & 0xFFFF))
  function check_compatibility(host, requested):
    if host.major != requested.major: return Err(VERSION_MISMATCH)
    if requested.minor > host.minor:  return Err(VERSION_MISMATCH)
    return Ok(())
Validation: unit tests for equality, lower-minor, higher-minor,
  cross-major, zero-word; boundary minors 0 and 0xFFFF.
```

## 5. `registry_lookup` (function, M3)

```text
Name and stability: registry_lookup(call_number: u32) -> Option<&'static
  CallDescriptor>; internal; static table.
Purpose and caller: S4 routing; host tests.
Inputs / outputs: in — decoded call number; out — descriptor { mnemonic,
  feature_bit, handler_entry } or None.
Preconditions / postconditions: None ⇒ number is not defined by this ABI.
State and ownership: compile-time table; never mutated.
Concurrency/allocation: lock-free read of static data; allocation-free.
Errors and failure guarantee: infallible lookup; absence is a normal value,
  mapped to UNKNOWN_CALL by the caller (W06 flow), not inside the registry.
Security/authorization checks: none here (routing is not authorization).
Logic: match over the table; v0 entries: CALL_DISCOVERY → discovery
  descriptor (feature FEATURE_BASE_SET); CALL_MINIMAL → descriptor whose
  handler_entry is implemented by W06's integrated operation — the table
  entry is added by W06's reviewed implementation, never at runtime
  (Decision 9 of 01-scope-and-foundations.md).
Validation: table review (one entry per defined number, mnemonics unique,
  feature bits defined); unit test for known/unknown/gated lookups.
```

## 6. `execute_discovery` (function, M4/M5)

```text
Name and stability: execute_discovery() -> DiscoveryResult; internal.
Purpose and caller: the handler of call 0; invoked from the dispatch flow
  after decode and routing.
Inputs / outputs: none in; out — DiscoveryResult { version:
  AbiVersionWord, features: u64 }.
Preconditions / postconditions: decode has verified x1 == 0 for call 0.
  Post: version equals ABI_VERSION_WORD; features has FEATURE_BASE_SET set
  and all other bits zero (v0).
State and ownership: none.
Concurrency/allocation: pure constant read; allocation-free.
Errors and failure guarantee: infallible; discovery cannot be denied (it
  predates authority and version knowledge — Decision 10).
Security/authorization checks: returns no Host address, no build path, no
  internal detail; only the version word and feature bitmap.
Logic:
  function execute_discovery():
    return DiscoveryResult { version: ABI_VERSION_WORD,
                             features: FEATURE_BASE_SET }
Validation: unit test equality with constants; guest QEMU discovery
  scenario (W07) asserts the same values from EL1.
```

## 7. `compose_result` (function, M6)

```text
Name and stability: compose_result(result_view, outcome); internal.
Purpose and caller: the single writer of Guest-visible result registers;
  called exactly once per request at S7 by the dispatch flow (W06).
Inputs / outputs: in — a mutable Arch-provided result view over the trapped
  Guest's register file; outcome — one of Ok(OkResult{values: [u64;3]}),
  Err(HypercallStatus rejection), or the function is not called at all after
  an invariant escalation (Decision 7).
Preconditions / postconditions: pre — outcome is well-formed; the Guest is
  stopped in the P4 exit state. Post — x0 = status code; if OK, x1–x3 =
  values and x4–x7 = 0; if rejection, x1–x7 = 0. No other register is
  modified by this function.
State and ownership: mutates the Guest register file inside the P4-owned
  context; ownership of the context stays with P4.
Concurrency/allocation: runs once, in trap context, no allocation; no lock —
  the register file is exclusively accessible to the trapping pCPU while the
  Guest is stopped.
Errors and failure guarantee: infallible by contract; a second composition
  attempt or composition with an invariant failure is an InvariantViolation
  (not a status).
Security/authorization checks (the INV-P5-01 enforcement point): the value
  source is the typed outcome only; no function path exists from a Host
  address, object pointer, or internal table location into values[]; values
  are produced by the called operation's contract (W06), which must return
  Guest-meaningful quantities.
Logic:
  function compose_result(view, outcome):
    match outcome:
      Ok(r):        view.x0 = STATUS_OK
                    view.x1 = r.values[0]; view.x2 = r.values[1]
                    view.x3 = r.values[2]
      Err(status):  view.x0 = status.to_guest_code()   // single mapping point
                    view.x1..x3 = 0
    view.x4 = 0; view.x5 = 0; view.x6 = 0; view.x7 = 0
Validation: unit tests per outcome shape; review that no other code path
  writes the ABI-defined registers after capture (grep/review gate); QEMU
  guest scenario asserts zeros in undefined results (W07 marker).
```
