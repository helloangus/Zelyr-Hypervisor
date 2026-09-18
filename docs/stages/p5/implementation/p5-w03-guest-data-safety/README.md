# P5-W03 Guest-Data Safety Boundary — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The reusable Guest-data boundary required by
[P5-W03](../../plans/p5-w03-guest-data-safety.md): Guest IPA/range legality,
Stage-2 presence and access-rights checks, checked length arithmetic, and
controlled treatment of zero/maximum length, cross-page and partial mapping,
unmapped targets, read-only writes, invalid memory types, and forbidden
boundaries.  
**Owner/change context:** P5-W03 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P5-W03. It converts the bounded
work-package plan into a typed validation-and-access boundary that every
future HVC buffer consumer composes with. It deliberately does **not** select
copy APIs or page-walk algorithms beyond the stated contract shape, define
Host pointer representations, design shared-memory IPC, fix final maximum
sizes as an ABI promise, redesign P4 Stage-2, or define a machine-memory ABI.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the supporting file its assigned step needs:

| Assigned step | Load |
|---|---|
| Understand scope classes, the goal-to-baseline ledger, assumed contracts, or a decision's rationale | [01-scope-and-foundations.md](01-scope-and-foundations.md) |
| Understand modules, ownership, the access lifecycle, or the concurrency boundary | [02-architecture-and-state.md](02-architecture-and-state.md) |
| Implement Guest range types and the validator | [03-code-contracts-guest-range.md](03-code-contracts-guest-range.md) |
| Implement the page-bounded accessor and fault domain | [04-code-contracts-guest-accessor.md](04-code-contracts-guest-accessor.md) |
| Execute the ordered workflow | [05-implementation-workflow.md](05-implementation-workflow.md) |
| Judge acceptance, review security behavior, or hand off | [06-validation-and-handoff.md](06-validation-and-handoff.md) |

Before editing, the agent must also have read the repository `AGENTS.md`,
[documentation index](../../../../README.md),
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P5 task book](../../task-book-v0.1.md), the
[P5-W03 plan](../../plans/p5-w03-guest-data-safety.md), the W01 entry
boundary ([entry README](../p5-w01-entry-contract-reconciliation/README.md)
and [ledger](../p5-w01-entry-contract-reconciliation/01-reconciliation-ledger.md)
§3), and the sibling boundaries it composes with:
[error boundary](../p5-w02-hypercall-abi-error-boundary/README.md) (W02) and
[dispatch](../p5-w06-dispatch-permission-containment/README.md) (W06, its
primary consumer). This document is a proposed design; it contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P5 task book → P5-W03 plan → this
design → Coding Guidelines. Binding constraints:

- ADR-007 and ADR §19: a Guest-supplied address, length, or index never
  reaches a memory access unvalidated; Guest-caused faults are recoverable
  VM-facing outcomes, never Host panics; no Host pointer crosses to a Guest.
- ADR-018: Stage-2 address spaces and ownership are first-class; this
  boundary consumes the P4 Stage-2 capability and adds nothing to it.
- Task book §8: Guest-data mapping/partial-access behavior is a
  **Specification Investigation** — architectural claims cite the AArch64
  Stage-2 basis; QEMU behavior is never a Core contract.
- Plan out-of-scope boundaries honored: no copy/mapping API spelling beyond
  contract shape, no final maximum-size value frozen as ABI, no P4 Stage-2
  redesign, no shared-memory IPC.

Classification:

- **Required** — the Guest range vocabulary with checked arithmetic; the
  validate-then-access two-phase contract; per-page Stage-2 validation
  covering presence, direction-consistent permission, and memory type; the
  zero-length and bounded-length rules with the per-call limit mechanism; the
  fault-cause domain mapping to the W02 `GUEST_MEMORY_FAULT` class; the
  forbidden-boundary invariant; the host-testable fake-space seam.
- **Reserved** — the concrete per-call limit values (fixed at implementation
  with recorded rationale, reviewable but not an ABI promise at major 0);
  cross-CPU concurrent-Stage-2-mutation safety (requires the P3 TLB-shootdown
  integration, not proven by P4); multi-granule (huge-page) walk support;
  copy performance policies; SharedRegion-style sharing semantics.
- **Out of Scope** — dispatch integration and check ordering
  ([P5-W06](../p5-w06-dispatch-permission-containment/README.md)); handle and
  capability semantics (W04/W05); Guest scenario markers
  ([P5-W07](../p5-w07-validation-guest-isolation-suite/README.md)); fuzz
  generators ([P5-W08](../p5-w08-host-fuzz-stress-smp-baseline/README.md));
  P4 Stage-2 internals; any machine-memory ABI.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Guest IPA validity, Stage-2 presence/access checks (P5-T02/T03) | [03 §4](03-code-contracts-guest-range.md) | P5-V03 (W03-DV02) |
| Checked length arithmetic; zero and maximum length (P5-T03) | [03 §2–§3](03-code-contracts-guest-range.md) | P5-V03 (W03-DV01, DV02) |
| Cross-page/partial mapping; unmapped; read-only write; invalid memory type | [03 §4](03-code-contracts-guest-range.md), [04 §2](04-code-contracts-guest-accessor.md) | P5-V03 (W03-DV02) |
| Forbidden-boundary treatment; no Host exposure | [02 §5](02-architecture-and-state.md), [04 §4](04-code-contracts-guest-accessor.md) | P5-V03/V09/V10 (W03-DV03, DV04) |
| Guest data as untrusted input; QEMU ≠ Core rule | [01 §3](01-scope-and-foundations.md) decisions 1, 5, 9 | review W03-DV05 |
| Request validation integrated for all future HVC buffer consumers | [README handoff](#downstream-handoff), [06 §4](06-validation-and-handoff.md) | W03 closure (W03-DV06) |
| Host-testable range/parsing properties for W08 | [03 §5](03-code-contracts-guest-range.md), [04 §5](04-code-contracts-guest-accessor.md) | P5-V13 basis (W03-DV02) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18): no Rust sources exist; the P4 Stage-2,
Guest-memory, and fault packages and the P2 ownership/allocation packages are
plans without implementation or verification records; P0-W15's address
newtype vocabulary is a plan. Every input below is an **assumed contract**
under the W01 model, cited by plan path, with an explicit failure boundary.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| A Guest-supplied address+range is never an unchecked Host pointer (P5-V03) | No Guest-data code exists; nothing constrains future call handlers | Typed range vocabulary + validator + accessor as the single composition point for buffer calls | Only a boundary every consumer must route through makes "never" structural rather than disciplinary | W03 (this design); consumers W06+ | W03-DV01–DV03 |
| Stage-2 presence/permission/type checks (P5-T03) | P4-W02 capability is planned, not implemented | Assumed contract AC-03.1 (translate/query with permission and type) with `Blocked Prerequisite` boundary | The validator can only be as strong as the Stage-2 query it consumes | P4 owners; W03 consumes | W03-DV02 blocked until P4 evidence; design review meanwhile |
| Forbidden-boundary treatment (P5-V03) | P2-W03/P4-W02 protection rules are plan text | Belt-and-braces invariant: accessor accepts only Stage-2-translated pages; a protected-range translation is an `InvariantViolation` (AC-03.2) | Stage-2 exclusion is the primary control; the accessor must fail closed if it ever lies | P2/P4 owners; W03 guard | W03-DV03 |
| Controlled outcomes for all required negative cases (P5-V03) | No case vocabulary exists | Fault-cause domain mapped to the W02 `GUEST_MEMORY_FAULT` class with internal causes | W02 fixed the Guest-visible class set; W03 owns the causes inside it | W02 boundary; W03 causes | W03-DV02 |
| Zero/max-length and overflow handled (P5-T03) | Nothing exists | Checked-construction rule, vacuous-zero rule, per-call limit mechanism | Each is a distinct controlled outcome required by P5-V03; overflow arithmetic must be impossible to bypass | W03 | W03-DV01/DV02 |
| Host-testable range properties (P5-V13 basis) | P0 host gates planned only | Fake Stage-2 space seam + property harness requirements | Range logic must be exercisable without a VM or QEMU | W03 seam; P0 gates; W08 fuzz | W03-DV02; W08 evidence |
| QEMU behavior never becomes a Core contract | Nothing observed | Specification-investigation notes citing the AArch64 Stage-2 basis for each permission/type claim ([01 §3](01-scope-and-foundations.md) decision 9) | Task book §8 makes this a required handling, not a preference | W03 documentation duty | W03-DV05 |

No ledger row requires an architecture decision beyond the task book's
classifications; the limit *values* are deliberately mechanism-now /
value-at-implementation (decision 6 in [01](01-scope-and-foundations.md)).

## Resolved design decisions and their authority

Full statements and rationale in
[01-scope-and-foundations.md](01-scope-and-foundations.md) §decisions.
Summary:

1. Guest buffers are IPA ranges (base + length) in typed newtypes; Host
   pointer types are unreachable from Guest input.
2. Validate-then-access, two phases, all-or-nothing Guest-visible effect.
3. Zero length is a valid vacuous request that accesses nothing.
4. Per-page validation and per-page translated segments; no physical
   contiguity assumption; range may end mid-page.
5. Permission and memory-type rules fixed (direction-consistent; normal
   cacheable only for Guest data buffers).
6. Bounded length via a named per-call limit mechanism; value fixed at
   implementation with recorded rationale; loop trip count bounded by the
   limit.
7. Translations captured at validation and reused at access; single-threaded
   validity per operation; concurrent Stage-2 mutation is outside the P5
   proven scope and is a Reserved integration.
8. Forbidden-boundary belt-and-braces: any translation to a protected range
   is an `InvariantViolation` (W02 escalation model), never a Guest fault.
9. Every architectural permission/type claim carries its AArch64 basis
   (Specification Investigation duty); QEMU observations are test
   expectations only.

## Work breakdown and loading order

1. Read [01](01-scope-and-foundations.md) for the ledger, assumed contracts
   AC-03.1–AC-03.6, and decisions 1–9.
2. Read [02](02-architecture-and-state.md) for modules, the access lifecycle,
   and the concurrency boundary.
3. Implement the range vocabulary and validator per
   [03](03-code-contracts-guest-range.md), then the accessor and fault domain
   per [04](04-code-contracts-guest-accessor.md), in the order of
   [05](05-implementation-workflow.md).
4. Record decisions, deviations, and changed files in
   `../p5-w03-guest-data-safety-record.md` when implementation starts; record
   commands and results in
   `../../verification/p5-w03-guest-data-safety-verification.md` when evidence
   exists. Neither this design nor any record claims W03 or P5 complete.

## Explicitly excluded interfaces

No dispatch ordering or check sequencing (W06); no handle/capability types
(W04/W05); no hypercall envelope values (W02); no Guest scenario markers
(W07); no fuzz generator (W08); no telemetry transport (W09). No Stage-2
mutation API is added or altered — mapping/unmap/protect remain P4's
contracts, and this boundary consumes queries only. No shared-memory,
IPC, grant-table, or balloon mechanism is designed. No final maximum size is
declared an ABI promise, and no QEMU-specific layout, address, or behavior
enters Core. No file layout or crate name is fixed by this design. Any
implementation that converts a Guest-supplied value into a Host pointer
outside the accessor's translated segments violates the design and is a
review stop.

## Downstream handoff

Per the [P5 plan index](../../plans/README.md) consumer map:

- **W06**
  ([dispatch, permission, containment](../p5-w06-dispatch-permission-containment/README.md))
  receives the validated parameter boundary: the two-phase contract
  (validate → access), the fault class mapping, and the rule that every
  buffer-taking call validates fully before any effect. W06 sequences this
  check with W04/W05 checks; it does not reimplement them.
- **W07**
  ([validation guest suite](../p5-w07-validation-guest-isolation-suite/README.md))
  receives the expected negative-scenario vocabulary: the cause list
  (unmapped, permission, type, range-invalid, length-exceeded) and the
  Guest-visible `GUEST_MEMORY_FAULT` uniformity, for determinate markers.
- **W08**
  ([host fuzz/stress/SMP baseline](../p5-w08-host-fuzz-stress-smp-baseline/README.md))
  receives the fake-space seam and the range/validator invariants fuzzing
  must preserve (totality, checked arithmetic, bounded walks, no panic).
- **W09/W10** receive the fault-cause event vocabulary for redaction and
  regression; W10 records the limit values and their rationale as
  implementation facts. P6+ receives the boundary only through evidenced P5
  closeout; interrupt-object and device calls compose with it in their own
  designs.
