# P5-W02 Hypercall ABI and Error Boundary — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The versioned P5 HVC service boundary required by
[P5-W02](../../plans/p5-w02-hypercall-abi-error-boundary.md): the minimum
calling envelope, discovery and version-compatibility semantics, request and
result conventions, reserved-field treatment, and the structured error
boundary that keeps compatible, unsupported, malformed, Guest-caused, and
invariant-failure outcomes distinguishable.  
**Owner/change context:** P5-W02 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P5-W02. It converts the bounded
work-package plan into an implementable boundary with named logical modules,
typed contracts, and pseudocode. It deliberately does **not** design the
integrated dispatch sequence, object lookup, authority checks, or Guest-data
copying (P5-W03–W06 scope); it does not publish the factual ABI document; and
it does not freeze a management ABI, machine ABI, or any public compatibility
promise.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the supporting file its assigned step needs:

| Assigned step | Load |
|---|---|
| Understand what is Required/Reserved/Out of Scope, the goal-to-baseline ledger, or a design decision's rationale | [01-scope-and-foundations.md](01-scope-and-foundations.md) |
| Understand module responsibilities, ownership, the request lifecycle, or the concurrency model | [02-architecture-and-state.md](02-architecture-and-state.md) |
| Implement the frame capture, envelope decode, version/discovery, or result composition | [03-code-contracts-abi-surface.md](03-code-contracts-abi-surface.md) |
| Implement the status taxonomy, error classification, or invariant separation | [04-code-contracts-error-boundary.md](04-code-contracts-error-boundary.md) |
| Execute the ordered workflow | [05-implementation-workflow.md](05-implementation-workflow.md) |
| Judge acceptance, review security behavior, or hand off | [06-validation-and-handoff.md](06-validation-and-handoff.md) |

Before editing, the agent must also have read the repository `AGENTS.md`,
[documentation index](../../../../README.md),
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P5 task book](../../task-book-v0.1.md), the
[P5-W02 plan](../../plans/p5-w02-hypercall-abi-error-boundary.md), and the
W01 entry boundary
([entry review](../p5-w01-entry-contract-reconciliation/README.md) and its
[ledger](../p5-w01-entry-contract-reconciliation/01-reconciliation-ledger.md)
§3 assumed-contract rules). This document is a proposed design; it contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P5 task book → P5-W02 plan → this
design → Coding Guidelines. Binding constraints:

- ADR-007: every Guest-supplied register value is untrusted input.
- ADR-013 and ADR-051: no identity-derived authority anywhere in the error or
  routing path; authority is W05's mechanism, and this boundary must not
  pre-empt or bypass it.
- ADR-036/ADR-040: the P5 HVC service boundary is **not** the native
  management ABI and must not claim its status; ADR-040's independent
  versioning rule applies to any version identifier this design defines.
- ADR-056 is a **pending** decision about the native management ABI encoding;
  this design's envelope choices are scoped to the Guest service boundary v0
  and do not settle ADR-056 (see
  [01](01-scope-and-foundations.md) §decisions, decision 2).
- Task book §8: HVC register/call-number/error-number encoding, discovery
  representation, reserved-field policy, and the ABI compatibility commitment
  are **Implementation Choice subject to ABI review**, to be selected only in
  an approved detailed design — which this document is. The task book and
  plans freeze none of these values.
- Task book §1: malformed requests remain Guest-facing controlled outcomes;
  Guest-caused failure is distinct from Hypervisor invariant failure
  (P5-T13/T14); the boundary must neither expose Host information nor become
  an identity-based authority bypass.

Classification:

- **Required** — the calling envelope (immediate, register conventions,
  widths), ABI version word and compatibility rule, discovery call, call
  registry mechanism, status taxonomy with its guest-visible numeric mapping,
  reserved-field policies, the invariant-failure separation rule, and the
  host-testable decoder seam. Concrete selected values are stage-local design
  freedom owned by this design under the task book §8 classification, each
  recorded with rationale in [01](01-scope-and-foundations.md).
- **Reserved** — additional call numbers and feature bits; convergence of
  this envelope with a future native management ABI encoding (ADR-056); a
  public or external compatibility commitment; memory-mediated ABI arguments
  beyond the W03 boundary; per-call timeout or re-entrancy policy.
- **Out of Scope** — the integrated dispatch order and containment design
  ([P5-W06](../p5-w06-dispatch-permission-containment/README.md)); Guest-data
  validation ([P5-W03](../p5-w03-guest-data-safety/README.md)); handle and
  capability semantics ([P5-W04](../p5-w04-handle-lifecycle-type-safety/README.md),
  [P5-W05](../p5-w05-capability-rights-bootstrap-revocation/README.md));
  telemetry transport and redaction
  ([P5-W09](../p5-w09-telemetry-safe-logging-regression/README.md)); the
  factual ABI document publication
  ([P5-W10](../p5-w10-closeout-p6-handoff/README.md)); Guest-visible
  scenario markers ([P5-W07](../p5-w07-validation-guest-isolation-suite/README.md)).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Discovery, version and compatibility semantics (P5-T01) | [03 §4–§5](03-code-contracts-abi-surface.md) | P5-V02 (W02-DV02) |
| Inputs/results and register conventions without freezing a public promise (P5-T01, ABI part of T22) | [03 §2–§3](03-code-contracts-abi-surface.md) | P5-V02 (W02-DV01); ABI-artifact route review W02-DV06 → P5-V15 |
| Reserved-space treatment (P5-T01) | [03 §3.4](03-code-contracts-abi-surface.md) | P5-V02/V10 (W02-DV02, DV05) |
| Unknown/unsupported/malformed behavior (P5-T01) | [04 §3](04-code-contracts-error-boundary.md) | P5-V02/V06 (W02-DV03) |
| Structured error categories, distinguishable outcomes (P5-T13) | [04 §2–§3](04-code-contracts-error-boundary.md) | P5-V06 (W02-DV03, DV04) |
| Guest-facing failure vs Hypervisor invariant separation (P5-T14) | [04 §4](04-code-contracts-error-boundary.md) | P5-V09/V10 (with W06; boundary-level review W02-DV04) |
| No Host information exposure; not a management ABI; no identity authority bypass | [04 §5](04-code-contracts-error-boundary.md), README exclusions | P5-V15 review (W02-DV05) |
| Factual ABI artifact and compatibility-analysis route | [06 §3](06-validation-and-handoff.md) handoff; [01](01-scope-and-foundations.md) §routing | P5-V15 (with W09/W10; W02-DV06) |
| Dispatch contract handed to W05–W07; documentation route to W09–W10 | [README §Downstream handoff](#downstream-handoff) | W02 closure review (W02-DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18): no Rust sources exist anywhere in the
repository; the P5-W02 plan is unimplemented; P1/P4 trap, entry/exit, and
fault-classification packages are planned with no implementation or
verification records; P0 host-test, QEMU-runner, diagnostics, and
panic-classification packages are planned likewise; `docs/abi/` contains only
its governance stub. The only tracked P0 implementation evidence is the
P0-W01 repository-baseline verification record, and the P0-W02 toolchain
design is proposed. Every upstream input below is therefore an **assumed
contract** under the W01 model
([ledger §3](../p5-w01-entry-contract-reconciliation/01-reconciliation-ledger.md)),
cited by plan path, with an explicit failure boundary.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| A versioned HVC service boundary whose outcomes remain distinguishable (P5-V02) | No hypercall mechanism, ABI value, or decoder exists in any tracked file | Envelope + version + status-taxonomy design (this design), then implementation in the Core/Arch boundary | Distinguishability must be a property of the designed types and tables, not of call-site discipline | W02 (this design) | W02-DV01–DV03 |
| A decoder that Guest scenarios and fuzz paths can exercise | No code exists | Decoder specified as a total, allocation-free, host-callable function of the captured frame | P5-V13 fuzz readiness is only real if parsing is a pure seam, not entangled with trap state | W02; consumed by [P5-W08](../p5-w08-host-fuzz-stress-smp-baseline/README.md) | W02-DV03; W08 fuzz evidence |
| Guest-caused vs invariant failure are distinguishable at the boundary (P5-T14) | P0-W14 panic-classification package is planned, not implemented | Status taxonomy with an invariant class structurally outside the guest-visible code space; fatal path delegated to the P0-W14 mechanism | Masking invariants as Guest errors is exactly the failure P5-T14 forbids | W02 taxonomy; P0-W14 fatal path | W02-DV04; P0-W14 record at entry |
| Discovery answers version queries (P5-T01; EC-P5 ABI version query) | Nothing exists | Discovery call with fixed result layout and compatibility rule | A version query that is not itself a defined call cannot satisfy the exit criterion | W02 | W02-DV02 |
| The boundary exposes no Host information | Nothing exists to expose, but the result-composition surface would be the leak | Result-composition contract that writes only ABI-defined values and zeroes all other defined registers | Inv-P5-01 (no Host pointer via hypercall) is preserved or lost at composition time | W02 composition contract; reviewed per P5-V15 | W02-DV05 |
| Upstream trap path can deliver an HVC-class exception with a capturable register frame | P1-W05/P4-W04/P4-W06 are plans only | Assumed contract FD-1 with `Blocked Prerequisite` boundary | Frame capture is a consumer of the exception path, not a replacement for it | P1/P4 owners; W02 consumes | Evidence-level reconciliation at entry (W01 DV02) |
| Host-side unit/property tests can run | P0-W07/W08 planned only | Assumed contract FD-5 with `Blocked Prerequisite` boundary at implementation entry | Decoder evidence requires a host test path | P0 owners; W02 consumes | W02-DV03 marked blocked until P0 evidence exists |

No ledger row requires selecting an architecture beyond the task book's
Implementation-Choice classification; the concrete value selections this
design makes are each recorded with rationale in
[01-scope-and-foundations.md](01-scope-and-foundations.md) §decisions.

## Resolved design decisions and their authority

Full statements, rationales, and authority bases are in
[01-scope-and-foundations.md](01-scope-and-foundations.md) §decisions.
Summary:

1. Bespoke minimal register envelope, independent of SMCCC (task book §8
   Implementation Choice; ADR-056 untouched).
2. Single fixed HVC immediate; other immediates are Guest-facing malformed
   input.
3. 64-bit register envelope; 32-bit fields in the low half; no
   memory-mediated argument format in this boundary (W03 owns buffer calls).
4. Version word `(major=0, minor=1)` with major 0 declared experimental-only;
   stateless per-call compatibility check.
5. Reserved argument registers verified zero; reserved result registers
   written zero.
6. Closed guest-visible status table with a single mapping point.
7. Hypervisor invariant failures are structurally outside the guest-visible
   status space and take the P0-W14 fatal-classification path.
8. Only X0–X7 are ABI-defined; remaining registers are preserved via the
   P4-established Guest context save/restore.
9. Call registry is a static, read-only structure; no runtime registration.
10. Discovery is exempt from the per-call version check; a feature bitmap is
    defined with bit 0 as the v0 base set.
11. Dispatch is defined only for HVC originating in Guest EL1 of a current
    execution context; other origins follow the P1 unhandled-path
    classification.

## Work breakdown and loading order

1. Read [01](01-scope-and-foundations.md) for the ledger, the Required /
   Reserved / Out-of-Scope detail, and decisions 1–11 with their authority.
2. Read [02](02-architecture-and-state.md) for the logical modules, the
   per-request lifecycle, and the concurrency/layering model.
3. Implement the ABI surface per [03](03-code-contracts-abi-surface.md)
   (frame capture, decode, version, discovery, composition) and the error
   boundary per [04](04-code-contracts-error-boundary.md) (status taxonomy,
   classification, invariant separation), in the order given by
   [05](05-implementation-workflow.md).
4. Record decisions taken, deviations, and changed artifacts in
   `../p5-w02-hypercall-abi-error-boundary-record.md` when implementation
   starts; record actual commands and results in
   `../../verification/p5-w02-hypercall-abi-error-boundary-verification.md`
   when evidence exists. Neither this design nor any record may claim W02 or
   P5 complete; the factual ABI document is published only under the
   [06](06-validation-and-handoff.md) §3 route.

## Explicitly excluded interfaces

No management ABI, IPC protocol, Control Domain interface, or wire memory
format is designed or authorized; this boundary must never be presented as
the native management ABI (ADR-036/ADR-040) and its version identifiers must
not be conflated with `schema_version` or `machine_version` (ADR-040). No
handle, capability, rights, or Guest-data type is defined here (W03–W05). No
module file layout, crate name, or cargo structure is fixed by this design;
it defines logical modules and type/function contracts only. No runtime
plugin/registration interface for call handlers exists in v0. No telemetry
transport, log format, or redaction policy is chosen here (W09) — only the
event vocabulary W02 emits. No public compatibility commitment, external
documentation, or ABI freeze is made; the in-repo factual ABI document is a
later, evidence-gated deliverable under `docs/abi/`. Any implementation that
exposes Host virtual/physical addresses, object-table internals, or
validation internals beyond the defined status classes through this boundary
violates the design and must be stopped at review.

## Downstream handoff

Per the [P5 plan index](../../plans/README.md) consumer map:

- **W05**
  ([capability, rights, bootstrap, revocation](../p5-w05-capability-rights-bootstrap-revocation/README.md))
  receives the error/authority routing boundary: the status classes for
  absent authority, insufficient rights, bad state, and resource exhaustion,
  and the rule that authority checks never degrade to identity. W05's denial
  outcomes must be expressible as these classes without redefining them.
- **W06**
  ([dispatch, permission, containment](../p5-w06-dispatch-permission-containment/README.md))
  receives the integration contract: the captured-frame value, the decode →
  route → compatibility stages with their rejection semantics, the
  validation-before-effect rule, the result-composition contract, and the
  invariant-failure escalation path. W06 owns the ordering of W03/W04/W05
  checks between compatibility and execution.
- **W07**
  ([validation guest suite](../p5-w07-validation-guest-isolation-suite/README.md))
  receives the Guest-visible outcome vocabulary: the status code values,
  discovery result layout, and reserved-result-zero guarantees that Guest
  scenario markers may assert.
- **W08**
  ([host fuzz/stress/SMP baseline](../p5-w08-host-fuzz-stress-smp-baseline/README.md))
  receives the total, host-callable decode/classification seam and the
  invariants fuzzing must preserve (totality, no panic, bijective status
  mapping).
- **W09/W10**
  ([telemetry/regression](../p5-w09-telemetry-safe-logging-regression/README.md),
  [closeout](../p5-w10-closeout-p6-handoff/README.md)) receive the
  result-category event vocabulary and the ABI-artifact publication route
  (`docs/abi/`, compatibility analysis, evidence preconditions) required for
  P5-V15/V16/V17. P6 receives only the evidenced boundary through W10; P5
  hands off no management ABI and no public promise.
