# P1-W02 Panic Route and Build Identity Contracts

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P1-W02 detailed design](README.md).

Pseudocode is an outline, not runnable production code. No allocation; no
formatting beyond the bounded writer of §3.

## 1. Early panic route (`#[panic_handler]`)

```text
Name and stability: p1_panic(info: &core::panic::PanicInfo) -> !; the
  binary's single #[panic_handler]; internal; stable within P1.
Purpose and caller: the panic route W09's routing matrix relies on from
  phase `runtime` onward, and the classification sink for P0-W14's
  invariant-failure semantics. Caller: the Rust runtime on panic; nothing
  calls it directly except W11's NC4 trigger (validation selection).
Inputs / outputs: PanicInfo (location and message as available); output is
  one bounded report through the early diagnostic writer, then a terminal
  stop.
Preconditions / postconditions: callable from any phase after stage 3
  (static data ready); stages 1–3 faults are outside P1's owned surface
  (W09 unowned-window limitation) and cannot be caused by W02 code. On
  return: never — the route is terminal by construction.
State and ownership change: flips the single-entry guard; writes the UART
  only.
Concurrency/allocation context: no allocation (the handler cannot afford a
  failed allocation path); no locking; single-entry guard is the entire
  synchronization story (single CPU, DAIF masked — a plain static flag
  inside the audited boundary; the route never re-enters itself because
  guard-true diverges immediately).
Errors and failure guarantee: bounded by construction: fixed maximum report
  length, fixed output path, no re-entry, no unwinding, no fallback work.
  If the report cannot be emitted (transmitter stuck), the terminal stop is
  still reached (R4 posture inherited from W01's boundary semantics).
Security/authorization checks: the report reflects only invariants and
  static identity — it never echoes memory contents or attacker-shaped data
  (P1 has no guest; the rule is stated for future re-use).
Report content (minimal set, in order):
  1. the panic-route marker prefix (fixed literal; the token text is
     recorded in the implementation record before W10/W11 consume it,
     per the W10 token precedent)
  2. build identity line (§2), if resolvable
  3. the panic message slice, if present, truncated to the bounded writer
     capacity
  4. the panic location (file/line), if present, in fixed form
Logic:
  if guard was already set: bounded stop
  set guard
  emit report (§3 writer)
  bounded stop (branch-to-self)
Validation: W02-DV04 review; NC4 execution belongs to W11; non-recursion is
  review-checked here and evidence-checked by W11's NC4 class.
```

Extension seam (Reserved, recorded trigger): W07's accepted design supersedes
the report body — it owns crash-report fields, ordering, and the fatal-path
route. The handover replaces the report-rendering body only; the handler
registration, the single-entry guard, and the bounded-stop discipline
transfer through W07's design, not by local edit. Until that design lands,
this minimal body is the P1 panic route.

## 2. `BuildIdentity`

```text
Name and stability: BuildIdentity; internal struct; static data; stable
  within P1.
Purpose and caller: answer "what am I running?" for the panic route and
  every later diagnostic, per P1-V03's identity requirement. Callers: the
  panic route; W06/W07 rendering; evidence tooling.
Inputs / outputs: fields sourced from the P0-W16 metadata contract (assumed
  contract): project version, revision/profile/target, dirty indicator,
  timestamp policy as P0-W16 fixes them. Provides one accessor returning a
  &'static BuildIdentity.
Preconditions / postconditions: resolvable at link time; the readiness
  assertion of [03-code-contracts-rust-runtime.md](03-code-contracts-rust-runtime.md)
  §3 requires the accessor to resolve before Runtime.complete.
State and ownership change: none (immutable static).
Concurrency/allocation context: no allocation; plain data.
Errors and failure guarantee: if the P0-W16 embedding mechanism is absent,
  identity degrades to a recorded "unavailable" literal with the blocker
  recorded — identity is never fabricated (parent README decision 6 of the
  ledger).
Security/authorization checks: none; identity is diagnostic content.
Logic: static definition fed by the P0-W16 mechanism (build-script or
  environment embedding, whichever its accepted design fixes); the exact
  mechanism is recorded, not invented here.
Validation: W02-DV04; P0-W16 consumability is re-checked in W02-DV07.
```

## 3. Early diagnostic writer

```text
Name and stability: early_write_bytes(bytes: &[u8]); internal; stable within
  P1 (superseded for markers by W06's channel; retained as the panic route's
  channel-independent output).
Purpose and caller: the only post-transfer raw output path in P1 outside
  W06's channel. Caller: the panic route (and, after W07's design lands, its
  report body through the extension seam).
Inputs / outputs: byte slice; output through the raw reference-UART polling
  write.
Preconditions / postconditions: stage 2 complete (stack exists; DAIF
  masked); on return all bytes were written or the transmitter is stuck
  (bounded by the poll's architectural outcome — stuck transmitters end in
  the caller's terminal stop, not in a return path).
State and ownership change: UART data register only (single-consumer rule,
  [01-architecture-and-state.md](01-architecture-and-state.md) §3).
Concurrency/allocation context: no allocation; polling loops only; callable
  from boot or exception context.
Errors and failure guarantee: no error path; output is best-effort at the
  hardware level and the caller's bounded stop does not depend on it.
Security/authorization checks: writes only the bytes given by internal
  callers; no formatting of untrusted data.
Logic:
  for byte in bytes: poll UART flag until writable; store byte
Layering: the UART constant is the shared boot-entry constant per
  [W01's single-source rule](../p1-w01-reference-boot-contract/02-entry-validation-contracts.md)
  §6 — one definition, reference-platform-documented, consumed here by
  contract. No abstraction, no init, no probing; W06's channel is a
  different mechanism and must not consume this writer's constant.
Validation: W02-DV04; observed indirectly by every later executed scenario
  (all diagnostic evidence flows through some consumer of this path or
  W06's).
```

## 4. Bounded formatting

Panic-report rendering uses a fixed-capacity stack buffer and a minimal
integer/hex formatter; `core::fmt` with an in-memory `Write` adapter is
acceptable if and only if it stays allocation-free, which is verified by
review (no `alloc`, no `format!` against heap types — the buffer is the only
sink). Formatting capacity is part of the recorded sizing arithmetic (parent
README decision 8); exceeding capacity truncates, never panics.

## 5. Relationship to other output mechanisms

| Mechanism | Owner | Window | Relationship to §3 |
|---|---|---|---|
| Rejection reporter | W01 (implemented in the entry module) | pre-transfer only | separate path; shares the UART constant only |
| Early diagnostic writer | W02 (this design) | post-transfer, all phases | the panic route's channel-independent route |
| Early console / marker channel | W06 | from its availability signal | supersedes §3 for markers and phase output; §3 remains the panic path's fallback route per W07's boundary |
| Fatal report | W07 | from its readiness point | owns the report fields; consumes §3 or W06's channel per its design |

No other output path is authorized in W02. A review finding any additional
print path is a scope failure (W02-DV05).
