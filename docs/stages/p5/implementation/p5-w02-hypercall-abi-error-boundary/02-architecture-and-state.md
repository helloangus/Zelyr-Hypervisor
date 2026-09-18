# P5-W02 Architecture, State, and Concurrency

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W02 detailed design](README.md).

## 1. Logical modules

These are logical modules with layer placement, not a file tree; no crate or
path layout is fixed by this design (Coding Guidelines: layering without
invented structure).

| Module | Layer | Responsibility | Owned state / artifacts | Inputs | Outputs | Non-responsibility | Failure boundary |
|---|---|---|---|---|---|---|---|
| M1 Trap adapter | Arch (AArch64) | Recognize an HVC-class synchronous exception originating in Guest EL1; capture the ABI-visible register values and ESR immediate into an immutable frame; hand it to Core | `HypercallFrame` value type | Exception path state from the P1/P4 boundary (AC-02.1, AC-02.2) | One captured frame per trap | Decoding, semantics, dispatch policy, other exception classes | Frame capture failure or non-EL1 origin is not a hypercall: it returns to the P1/P4 classification path (Decision 11) |
| M2 Envelope decoder | Core | Validate the envelope: immediate, call-number width/reserved halves, version word presence, reserved-register zero policy | None (pure) | `HypercallFrame` | `DecodedCall` or `HypercallStatus` rejection | Call semantics, authorization, Guest-data handling | Rejection is a Guest-facing status; a decoder that cannot decide is an invariant, not a rejection |
| M3 Call registry | Core | Static read-only map of call number → descriptor {mnemonic, feature bit, handler entry} | The table itself (compile-time) | Call number | Descriptor or absent | Any mutable registration; handler behavior | Absent number → `UNKNOWN_CALL`; present but feature-gated → `UNSUPPORTED_FEATURE` |
| M4 Version/compatibility service | Core | Version word construction/parsing; stateless compatibility check | Constants (version word, feature bits) | Host version, requested version | Ok or `VERSION_MISMATCH` | Negotiation, stored sessions | A malformed version word is a `MALFORMED_REQUEST`, not a mismatch (it never parses as a version) |
| M5 Error domain | Core | `HypercallStatus` taxonomy; single internal-error → status mapping; invariant type with no status conversion | The taxonomy types | Boundary outcomes from all modules | Guest-visible status class | Policy for what an operation *does* after denial (W06); fatal handling itself (P0-W14) | Any attempt to convert an invariant failure into a status is a compile-time/type error |
| M6 Result composer | Core/Arch boundary | Write status and result values into the frame for Guest resumption; zero all other ABI-defined registers | None | Frame (mutable result view), outcome | Updated frame ready for `ERET` resumption | Guest execution; context switching | Composition happens exactly once per trap; double composition or composition after an invariant escalation is an invariant violation |

Layering rule: M1 is the only module that touches architectural trap state;
M2–M5 are architecture-independent and host-testable; M6 writes through an
Arch-provided result view. No module may branch on a board, SoC, or QEMU
identity (ADR §19; Coding Guidelines).

## 2. Core objects and ownership

| Object | Kind | Owner | Lifetime | Invariants |
|---|---|---|---|---|
| `HypercallFrame` | Immutable capture value | M1 creates; passed by shared reference | One trap; dead after Guest resumption | Register values are exactly the Guest's values at trap; the immediate is the ESR-delivered immediate; origin level recorded |
| `DecodedCall` | Value | M2 creates | Until dispatch completes | `call_number` is a registry-known number; `requested_version` parsed or the call is discovery; reserved registers were zero (else the frame was rejected before construction) |
| `CallDescriptor` | Static entry | M3 | Program lifetime | One entry per defined call number; feature bit names a defined bit; mnemonic is diagnostic-only (never Guest-visible) |
| `HypercallStatus` | Closed enum | M5 | Per outcome | Bijective with the numeric code table; `OK` unreachable from any rejection path |
| `DiscoveryResult` | Value | M4 creates for call 0 | Per discovery call | Version word equals the constant; undefined feature bits are zero |
| `GuestBoundaryError` | Enum of Guest-caused failure kinds | M5 | Per outcome | Every kind maps to exactly one `HypercallStatus` |
| `InvariantViolation` | Distinct type | Discovering module | Until fatal path | No conversion to `HypercallStatus` exists; not Guest-visible |

Ownership rule: W02 introduces no long-lived mutable state. All W02-owned
values are per-trap and immutable after creation; the only mutable transition
is M6's single composition write. Persistent state introduced by called
operations (objects, capabilities) belongs to W04/W05 and is not touched in
the W02-owned stages.

## 3. Request lifecycle

One hypercall request moves through exactly this pipeline; the W02-owned
stages are marked. Stages between compatibility and composition belong to
[P5-W06](../p5-w06-dispatch-permission-containment/README.md), which orders
the W03/W04/W05 checks.

```text
Guest EL1 executes HVC
  |
  v
[S1 Trapped]      M1: exception recognized as HVC-class, origin Guest EL1
  |                  other classes/origins -> P1/P4 path (not a hypercall)
  v
[S2 Captured]     M1: immutable HypercallFrame (registers + immediate + origin)
  |
  v
[S3 Decoded]      M2: immediate==0? widths/reserved halves ok? reserved
  |                  registers zero? version word well-formed where required?
  |                  no -> MALFORMED_REQUEST (compose rejection, S7)
  v
[S4 Routed]       M3: registry lookup
  |                  absent -> UNKNOWN_CALL; gated -> UNSUPPORTED_FEATURE
  |                  (both compose rejection at S7)
  v
[S5 Compatible]   M4: discovery skips; else stateless version check
  |                  fail -> VERSION_MISMATCH (compose rejection, S7)
  v
[S6 Validated/Executed]   W06: caller/object/type/rights/data checks, then
  |                          the operation, per the sibling contracts;
  |                          outcomes are OK, a rejection class, or an
  |                          InvariantViolation
  |                          InvariantViolation -> fatal path, STOP (no S7)
  v
[S7 Composed]     M6: write status (+ result values iff OK); zero every
  |                  other ABI-defined register
  v
[S8 Resumed]      Guest execution resumes via the P4 re-entry path with the
                   composed frame; the request is complete
```

Rules:

- Validation before effect: no stage before S6 performs a Guest-visible side
  effect; a rejection anywhere composes a status and nothing else.
- Exactly one outcome per request: `OK`, one rejection status, or an
  invariant escalation. No path reaches S7 twice, and no path reaches S8
  after an invariant escalation.
- Rejections carry no result values: X1–X3 are zeroed on rejection
  (composition contract, [03 §7](03-code-contracts-abi-surface.md)).

## 4. Concurrency and allocation model

- **No shared mutable state in W02-owned stages.** M2–M5 are pure functions
  over immutable inputs; M3 is compile-time constant; M1/M6 operate on the
  per-pCPU trap frame. Two vCPUs (of the same or different VMs) can be inside
  S1–S7 concurrently without locks in W02 code.
- **Allocation:** decode, routing, compatibility, discovery, and composition
  are allocation-free; the error domain types are fixed-size. Called
  operations in S6 own their own allocation discipline (W03–W06 contracts),
  which must respect the no-blocking rule for VM-exit paths (Coding
  Guidelines: bounded work in exception/exit context).
- **Re-entrancy:** no W02 stage calls Guest memory, schedulers, or blocking
  primitives; the boundary never waits.
- **Ordering:** the Guest observes the composed result only after S7
  completes; there is no partial-result visibility because composition is a
  single register-write step within the already-stopped Guest context.
- **Interrupt context:** the boundary runs in the synchronous-exception
  context established by P1/P4; it adds no IRQ masking policy of its own and
  inherits the P3-W06 rules for any S6 operation that takes locks (their
  responsibility, not this boundary's).

## 5. Security model at the boundary

- Every value in the frame is untrusted Guest input, including fields this
  design defines as "reserved"; trust is established only by the checks of
  S3–S6, never by convention.
- The boundary discloses nothing: status codes are the closed taxonomy;
  result registers carry only the call's defined values or zeros; mnemonics
  and internal diagnostics never enter registers. The INV-P5-01 rule (no Host
  pointer via hypercall) is enforced structurally by the composition
  contract — there is no value path from a Host address to a composed
  register in W02 code.
- Failure containment: Guest-caused failures end at S7 as a status; Hypervisor
  invariant failures never reach S7. The boundary cannot be used to convert
  one into the other (Decision 7).
- No authority semantics live here: `UNKNOWN_CALL`, `MALFORMED_REQUEST`, and
  `VERSION_MISMATCH` are decided without consulting identity; identity-based
  behavior is absent by construction, preserving the ADR-013/ADR-051 rule
  that authority checks (W05) are the only authorization mechanism.
