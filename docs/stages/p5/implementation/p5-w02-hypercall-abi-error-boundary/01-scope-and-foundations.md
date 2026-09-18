# P5-W02 Scope, Foundations, and Design Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W02 detailed design](README.md).

## 1. Scope classification in detail

### 1.1 Required

| Item | Statement |
|---|---|
| Trap-to-boundary seam | An Arch-layer capture of an HVC-class synchronous exception taken from Guest EL1, exposing the ABI-visible register values as immutable input |
| Calling envelope | Fixed immediate; call number, version word, and argument/result register assignment; register widths; caller-saved/preserved split |
| Version semantics | ABI version word layout, discovery call, stateless per-call compatibility rule, version-mismatch outcome |
| Discovery result | Host version word, feature bitmap, reserved-zero result fields |
| Call registry | Static read-only mapping of call number to descriptor (name, feature bit, handler entry) |
| Status taxonomy | Closed set of guest-visible outcome classes, one numeric code each, single mapping point |
| Reserved-field policy | Verify-zero on reserved argument registers; write-zero on reserved result registers; nonzero reserved arguments are malformed |
| Error classification | Rules mapping internal boundary outcomes to status classes; malformed vs invalid-argument boundary |
| Invariant separation | Structural rule that Hypervisor invariant failures never surface as guest-visible status codes and never resume the Guest |
| Composition contract | Result writing into the frame with reserved zeroing; no Host address or internal state ever composed |
| Host-testable seam | Decode, compatibility, discovery, and classification callable from host tests with synthetic frames |

### 1.2 Reserved (must not be blocked by, or implemented ahead of, an approved design)

| Item | Trigger for activating |
|---|---|
| New call numbers beyond discovery and the W06 minimal call | The owning future package's approved design allocates from the call-number space and adds a feature bit if non-base |
| Feature-gated calls | First call whose availability differs between builds |
| Memory-mediated ABI arguments (descriptor-block calls) | First call that needs one; must then compose with the W03 boundary and revisit endianness/padding rules explicitly |
| Convergence with the future native management ABI encoding (ADR-056) | The ADR-056 decision process; P5 must not pre-empt it |
| Public/external compatibility commitment | A future superseding ADR; P5's major-0 versioning explicitly excludes it |
| Per-call timeout, re-entrancy, or nested-hypercall policy | First mechanism that can block or nest inside dispatch (none exists in P5) |

### 1.3 Out of Scope

Integrated dispatch ordering and fault containment (P5-W06); Guest-data
range/copy semantics (P5-W03); object handles and lifecycle (P5-W04);
capability rights, grants, revocation (P5-W05); Guest scenario markers and
two-context tests (P5-W07); fuzz generators and stress (P5-W08); telemetry
transport, redaction, regression integration (P5-W09); factual ABI/security
document publication and closeout (P5-W10); scheduler, timer, IRQ, vGIC,
Linux, machine ABI (P6+); and any completion or evidence claim.

## 2. Prerequisite assumed contracts

In the form fixed by the W01 ledger §3
([../p5-w01-entry-contract-reconciliation/01-reconciliation-ledger.md](../p5-w01-entry-contract-reconciliation/01-reconciliation-ledger.md)):

```text
AC-02.1  An HVC-class synchronous exception taken at Guest EL1 is recognized
         and classified by the EL2 exception path, with the Guest register
         frame capturable and the EL2 state recoverable afterwards.
Source:  ../../../p1/plans/p1-w05-el2-exception-entry-baseline.md;
         ../../../p4/plans/p4-w04-vcpu-entry-exit.md;
         ../../../p4/plans/p4-w06-fault-isolation-diagnostics.md
Failure: Blocked Prerequisite — without the classified path and frame
         capture, W02 implementation cannot integrate; design and host-side
         decoder work may proceed, QEMU/guest evidence may not. If P4's
         implemented path terminates unknown synchronous exceptions without
         an extension seam, that is a Contract Conflict against P4-W06.

AC-02.2  A current Guest execution context (caller) exists at trap time and
         is recoverable from the P4-established VM/vCPU state; guest GPRs
         are preserved across exit/re-entry outside the ABI-defined set.
Source:  ../../../p4/plans/p4-w04-vcpu-entry-exit.md
Failure: Blocked Prerequisite — dispatch cannot identify a caller context.
         If preservation of X8–X30 cannot be provided by the P4 context
         save/restore, the preserved-register list must be renegotiated as a
         design revision of this document, never by silent narrower
         preservation.

AC-02.3  A panic/failure classification distinguishes fatal Hypervisor
         invariant failures from Guest-caused faults and provides the fatal
         diagnostics path.
Source:  ../../../p0/plans/p0-w14-panic-failure-classification.md;
         ../../../p1/plans/p1-w07-fatal-crash-diagnostics.md
Failure: Blocked Prerequisite for the invariant-failure path; W02 still
         defines the structural separation (types that cannot convert), so
         the classification can be attached when it exists.

AC-02.4  Host-side unit/integration test gates execute Rust host tests.
Source:  ../../../p0/plans/p0-w07-development-quality-gates.md;
         ../../../p0/plans/p0-w08-host-side-testing-baseline.md
Failure: Blocked Prerequisite for DV03 evidence; design review still applies.

AC-02.5  Logging levels and a trace-event namespace exist for event
         vocabulary.
Source:  ../../../p0/plans/p0-w12-logging-diagnostic-baseline.md;
         ../../../p0/plans/p0-w13-trace-event-namespace-baseline.md
Failure: Documentation Gap — W02 defines the event vocabulary it needs and
         attaches transports when W09 delivers them.

AC-02.6  A governed ABI document location exists and is empty of P5 content.
Source:  ../../../../abi/README.md
Failure: Contract Conflict if factual ABI content appears anywhere outside
         the governed route.
```

## 3. Resolved design decisions and their authority

Each decision below is stage-local design freedom owned by this design under
the [P5 task book](../../task-book-v0.1.md) §8 classification ("select only
in approved detailed design"). None alters an accepted ADR decision. Changes
to any of these after implementation are ABI compatibility events and follow
the review route in [06](06-validation-and-handoff.md) §3.

### Decision 1 — Bespoke minimal register envelope, independent of SMCCC

The v0 boundary is a fixed-immediate HVC with a Zelyr-defined register
assignment. It does not adopt the SMCCC function-identifier format, does not
claim SMCCC conformance, and does not share its identifier space.

Rationale: the boundary must be parseable by a `no_std` core with no
specification dependency beyond the AArch64 architecture; SMCCC conformance
would import an external compatibility commitment (a public-promise class
the task book withholds from P5) and would overlap the firmware/SMC
proxing space the Hypervisor handles separately. A future convergence with
SMCCC or with the native management ABI (ADR-056) remains Reserved.
Authority: task book §8; ADR-008 (no EL3 firmware ownership); ADR-056 left
pending.

### Decision 2 — Single fixed immediate; other immediates are malformed

The ABI v0 entry is `HVC #0`. The immediate arrives in the ESR_EL2 ISS field
of the HVC-class exception and is validated by the decoder. Any other
immediate yields `MALFORMED_REQUEST` (a Guest-facing outcome) with a
telemetry note.

Rationale: a single entry point eliminates hidden multiplexing, makes the
fuzz surface one decoder, and gives a deterministic answer to "what happens
on a wrong immediate" (P5-V10 requires malformed calls to stay Guest-facing).
The immediate is checked, never used as a covert selector.
Authority: task book P5-T13/T14; P5-V10.

### Decision 3 — 64-bit envelope; 32-bit fields in the low half; no memory-mediated arguments

All ABI values travel in X registers. A logically 32-bit field occupies bits
[31:0]; bits [63:32] are reserved and must be zero. The v0 envelope defines
no pointer-, descriptor-, or buffer-typed argument; Guest buffers appear only
through per-call contracts that use the W03 Guest-data boundary.

Rationale: endianness and padding questions do not arise for register values,
removing a class of ambiguity from v0; deferring memory-mediated formats
keeps this design free of wire-format commitments the task book reserves.
Authority: task book §8 (encoding as Implementation Choice); plan
out-of-scope ("wire representation").

### Decision 4 — Version word layout and experimental major

`ABI_VERSION_WORD: u32 = (major << 16) | minor`, with
`major = 0, minor = 1` for this boundary (value `0x0000_0001`). Major `0`
declares the boundary experimental and internal-only; no external
compatibility is promised at major 0. Version word `0` is never valid.

Rationale: the task book requires the exit-criteria ABI artifact to
distinguish an experimental internal compatibility commitment from any later
public contract; encoding that distinction in the major field makes it
machine-checkable instead of documentary. Independent version identifiers
(`schema_version`, `machine_version`, management ABI) are untouched, per
ADR-040.
Authority: task book §7; ADR-040; ADR-036.

### Decision 5 — Stateless per-call compatibility check

Every call except discovery carries the caller's target version word in X1.
The check `host.major == requested.major && requested.minor <= host.minor`
runs after decode and before any further processing; failure yields
`VERSION_MISMATCH`. No compatibility session, handshake, or stored per-VM
negotiation state exists.

Rationale: stateless checking removes hidden cross-call state (no session to
invalidate, no SMP-shared negotiation table), keeps the compatibility
property per-call testable, and satisfies "version mismatch" as a
distinguishable outcome (P5-V02) without inventing a negotiation protocol.
Authority: P5-V02; task book §1 (version-compatibility); design-owned
mechanism.

### Decision 6 — Closed status table with a single mapping point

The guest-visible outcome space is the closed set in
[04 §2](04-code-contracts-error-boundary.md): `OK` plus exactly ten rejection
classes. Numeric values are fixed by this design; the only conversion from
internal error types to numeric codes is one total function.

Rationale: distinguishable outcomes (P5-V06) require a closed, reviewed
vocabulary; a single mapping point prevents call-site drift and gives fuzzing
and telemetry one surface to check for bijectivity.
Authority: P5-V06; task book P5-T13; design-owned numbering.

### Decision 7 — Invariant failures are outside the guest-visible status space

A Hypervisor invariant violation discovered during hypercall processing is
represented by a type that has no conversion into the guest-visible status
space. It escalates to the fatal-classification path (AC-02.3) and does not
resume the Guest with a defined result. No code path maps an invariant
failure to `OK` or to any rejection code.

Rationale: P5-T14 and the ADR §12/§19 separation (Guest-caused faults are
VM-facing; hypervisor invariants are not) make masking an invariant as a
Guest error a correctness violation, not a style choice; representing the
classes by distinct types makes the masking unrepresentable rather than
merely forbidden.
Authority: ADR-000 §12, §19; task book P5-T14; P0-W14.

### Decision 8 — ABI-defined register set X0–X7; others preserved

On entry the Guest may rely on X0–X7 carrying ABI-defined values on return
(status/result or explicit zeros). X8–X30 and SP are preserved by the
existing Guest context save/restore (AC-02.2), so the Guest keeps everything
it held across the call.

Rationale: a small defined set keeps the envelope minimal and the fuzz
surface bounded; full preservation of the remainder is already the P4
entry/exit contract, so redefining a caller-saved subset would add churn
without a P5 need. If AC-02.2 fails to preserve, the boundary revision route
is a recorded design change, not silent narrowing.
Authority: P4-W04 assumed contract; design-owned envelope.

### Decision 9 — Static read-only call registry

Call dispatch consults a compile-time-known table: call number → descriptor
{mnemonic, feature bit, handler entry}. No runtime registration, no mutable
dispatch state, no dynamic loading.

Rationale: a static table is trivially SMP-safe (no locks in the W02-owned
stages), auditable, and total; runtime registration would add an ownership
and lifetime problem P5 has no consumer for. Extension is by reviewed table
growth (Reserved item 1.2).
Authority: task book Reserved (object-table/locking mechanics as
Implementation Choice); design-owned mechanism.

### Decision 10 — Discovery exempt from the version check; feature bitmap defined

Call `0` (discovery) requires X1 zero and returns the host version word and a
64-bit feature bitmap. Bit 0 is "base v0 call set present"; all other bits
are reserved zero. Calls present in the registry but gated by an absent
feature bit return `UNSUPPORTED_FEATURE`; numbers absent from the registry
return `UNKNOWN_CALL`.

Rationale: discovery must be callable before any version is known (it is how
the version is learned); the bitmap gives forward compatibility a defined
mechanism without reserving call-number policy. The
unknown-vs-unsupported distinction is required by the plan's work sequence
item 3.
Authority: P5-W02 plan work sequence item 3; P5-V02.

### Decision 11 — Guest-EL1-only dispatch scope

The boundary is defined only for HVC executed in Guest EL1 within a current
execution context. HVC from other exception levels or without a Guest context
is not a hypercall; it follows the P1/P4 unhandled-path classification.

Rationale: P4 establishes only the Guest-EL1 execution fact; defining EL2- or
firmware-origin behavior would design beyond evidence. ADR-008 places SMC/EL3
matters outside this boundary.
Authority: FD-1 assumed contracts; ADR-008; task book §1.

## 4. Artifact routing (W02 portion)

- Proposed values (register conventions, call numbers, status codes) live in
  this design and its implementation record; they are proposals until
  implemented and evidenced.
- The factual HVC ABI document — the versioned statement of the implemented
  boundary with its compatibility analysis — is published under `docs/abi/`
  only after implementation, per
  [W01 routing](../p5-w01-entry-contract-reconciliation/01-reconciliation-ledger.md)
  §4. Producer: W02 content, W10 publication. Preconditions and the review
  route are in [06](06-validation-and-handoff.md) §3.
- No P5 design text, code comment, or record is the ABI contract; consumers
  cite the `docs/abi/` artifact once it exists.

## 5. Open questions and labels

| Item | Classification | Handling |
|---|---|---|
| Whether the W06 minimal call should carry arguments in X2–X4 or use only registers defined so far | Design freedom of W06 within this envelope | W06's design must fit Decision 3 and allocate no new call number beyond `1` without the 1.2 Reserved route |
| Behavior when the Guest re-enters a hypercall concurrently from two vCPUs of one VM | Concurrency of the *called operation*, not of this boundary; owned by W04/W05/W06 designs under the P3-W06 discipline | This boundary is stateless per call (Decision 5); no additional constraint imposed here |
| Timing side channels across status classes | Reserved | Not addressable in P5; noted for the security deliverable's limitations section (W10) |
| Any need to weaken the invariant separation or identity rules | `ADR Required` | Stop; ADR process; never a local choice |
