# P5-W05 Capability, Rights, Bootstrap, and Revocation — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The P5 authorization foundation required by
[P5-W05](../../plans/p5-w05-capability-rights-bootstrap-revocation.md):
caller-associated capability records with operation-level rights, explicit
bootstrap grants, the permission check and its denial classes, the
no-identity-shortcut rule, and basic grant/use/revoke/reject-old-authority
semantics.  
**Owner/change context:** P5-W05 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P5-W05. It converts the bounded
work-package plan into a typed authority mechanism that composes with the
[W02 error boundary](../p5-w02-hypercall-abi-error-boundary/README.md) and
the [W04 identity boundary](../p5-w04-handle-lifecycle-type-safety/README.md).
It deliberately does **not** design a delegation tree, attenuation,
parent-child revocation, authentication, RBAC, a policy broker, a Control
Domain, or complete management operations, and does not freeze rights or
capability encodings as public contracts.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the supporting file its assigned step needs:

| Assigned step | Load |
|---|---|
| Scope classes, goal-to-baseline ledger, assumed contracts, decision rationale | [01-scope-and-foundations.md](01-scope-and-foundations.md) |
| Modules, ownership, the authority-record lifecycle, concurrency model | [02-architecture-and-state.md](02-architecture-and-state.md) |
| Implement the rights vocabulary and the grant/bootstrap path | [03-code-contracts-rights-and-grant.md](03-code-contracts-rights-and-grant.md) |
| Implement the authorization check, revocation, and denial classes | [04-code-contracts-check-revoke.md](04-code-contracts-check-revoke.md) |
| Execute the ordered workflow | [05-implementation-workflow.md](05-implementation-workflow.md) |
| Judge acceptance, review security behavior, or hand off | [06-validation-and-handoff.md](06-validation-and-handoff.md) |

Before editing, the agent must also have read the repository `AGENTS.md`,
[documentation index](../../../../README.md),
[ADR baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md),
[P5 task book](../../task-book-v0.1.md), the
[P5-W05 plan](../../plans/p5-w05-capability-rights-bootstrap-revocation.md),
the W01 entry boundary
([entry README](../p5-w01-entry-contract-reconciliation/README.md) and
[ledger](../p5-w01-entry-contract-reconciliation/01-reconciliation-ledger.md)
§3), and the two sibling boundaries cited above plus
[P5-W06](../p5-w06-dispatch-permission-containment/README.md), the primary
consumer of the check. This document is a proposed design; it contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P5 task book → P5-W05 plan → this
design → Coding Guidelines. Binding constraints:

- ADR-013: Capability v0 is object handle + rights + generation, with
  delegation/attenuation/revocation-tree evolution reserved. This design
  implements exactly the basic grant/check/revoke loop.
- ADR-051 and ADR §19: no `vm_id == 0` privilege, no first-VM privilege, no
  role bypass; capability-check failure must never degrade to an
  identity-based pass-through.
- ADR-038: initial capability originates from a Hypervisor-created bootstrap
  context; in P5 the grant source is the Hypervisor/test bootstrap (task
  book P5-V07), not a Control Domain.
- ADR §12: authentication, RBAC, and secrets stay out of EL2 — this
  mechanism authorizes, it never authenticates.
- Task book §8: rights representation, capability encoding, and revocation
  mechanics are **Implementation Choice**, selected in approved detailed
  design — which this document is.

Classification:

- **Required** — the authority-record model (caller association, target
  binding to slot+generation+class, rights set, Active/Revoked state); the
  rights vocabulary with named classes; the bootstrap-only grant path with
  an internal grant handle; the authorization check with its two denial
  classes and mandatory required-rights parameter; one-way revocation with
  tombstones; structural no-identity-shortcut guards; target-destroy
  interplay (authority dies with the object generation); the host-testable
  seam.
- **Reserved** — delegation, attenuation, derived capabilities, revocation
  trees (ADR-013 evolution; task book §1 Reserved); guest-initiated grants
  (P10+ management work); per-vCPU authority refinement; quotas; a
  guest-visible rights-encoding (none exists in v0 — rights never cross the
  boundary as Guest input or output).
- **Out of Scope** — dispatch ordering and the minimal operation's right
  requirement ([P5-W06](../p5-w06-dispatch-permission-containment/README.md)
  declares them using this vocabulary); handle and lifecycle mechanics
  ([P5-W04](../p5-w04-handle-lifecycle-type-safety/README.md)); Guest
  scenario markers and two-context setup
  ([P5-W07](../p5-w07-validation-guest-isolation-suite/README.md)); fuzz and
  stress ([P5-W08](../p5-w08-host-fuzz-stress-smp-baseline/README.md));
  telemetry transport ([P5-W09](../p5-w09-telemetry-safe-logging-regression/README.md));
  authentication, policy persistence, Control Domain, IPC (all excluded by
  task book §1).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Object reference + rights + caller association (P5-T07) | [02 §2](02-architecture-and-state.md), [03 §3](03-code-contracts-rights-and-grant.md) | P5-V06 (W05-DV01) |
| Observe/control/modify/destroy/delegate classes (P5-T08) | [03 §1–§2](03-code-contracts-rights-and-grant.md) | P5-V06 (W05-DV02) |
| Explicit bootstrap grants (P5-T09) | [03 §4](03-code-contracts-rights-and-grant.md) | P5-V07 (W05-DV03) |
| Valid / no-right / no-capability / wrong-caller cases (P5-T10) | [04 §2](04-code-contracts-check-revoke.md) | P5-V06 (W05-DV02, DV04) |
| No VM-0 / first-VM / role bypass (P5-T11) | [02 §5](02-architecture-and-state.md), [04 §4](04-code-contracts-check-revoke.md) | P5-V07 (W05-DV03, DV05) |
| Grant / use / revoke / reject-old loop (P5-T12) | [04 §3](04-code-contracts-check-revoke.md) | P5-V08 (W05-DV04) |
| Authority ≠ handle/role/VM-ID/first-VM in the W06 sequence | [README handoff](#downstream-handoff), [06 §4](06-validation-and-handoff.md) | W05 closure (W05-DV06) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18): no Rust sources exist; the W02 and W04
sibling boundaries are designs in progress, not implementations; P4 caller
context and P3 synchronization are planned only. Inputs below are **assumed
contracts** under the W01 model, cited by path, with explicit failure
boundaries.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| An explicitly granted caller succeeds only within its rights (P5-V06) | No authority mechanism exists | Record model + rights vocabulary + check (this design) | "Only within its rights" must be a property of the check, not of handler discipline | W05 (this design) | W05-DV01, DV02 |
| Identity shortcuts never authorize (P5-V07) | Nothing exists to shortcut | Structural guards: mandatory required-rights parameter, private record construction, no identity-valued logic | Guards that live in review discipline degrade; guards that live in types do not | W05 | W05-DV03, DV05 |
| Revoked authority cannot be reused (P5-V08) | Nothing exists | One-way tombstone state + generation binding to the target | Revocation must survive target reuse and caller repetition | W05 | W05-DV04 |
| Authority keys to caller identity from the execution context (P5-T07) | P4 context facts planned only | Assumed contract AC-05.3 (caller identity) with failure boundary | Association without a trustworthy caller identity is decoration | P4/P3 owners; W05 consumes | W05-DV01 blocked until P4 evidence for guest paths; host-seam evidence independent |
| Denials are distinguishable controlled outcomes (P5-V06) | W02 classes 7/8 defined in sibling design | Cause→class conversion composing with W02 | The Guest-visible vocabulary is W02's; W05 owns the causes | W02 sibling; W05 causes | W05-DV02 |
| Two-context isolation is demonstrable (P5-V12, via W07) | No second-context mechanism exists | Subject-keyed records: another VM holding the same handle value has no record | Isolation must fall out of the data model, not from extra checks | W05 model; W07 demonstrates | W05-DV05 basis; W07 evidence |

No ledger row requires an architecture decision beyond the task book's
Implementation-Choice classification; each selection is recorded with
rationale in [01-scope-and-foundations.md](01-scope-and-foundations.md)
§decisions.

## Resolved design decisions and their authority

Full statements and rationale in
[01-scope-and-foundations.md](01-scope-and-foundations.md) §decisions.
Summary:

1. Authority is an explicit record {subject, target slot+generation+class,
   rights, state}; the caller presents only the object handle (implicit
   lookup by caller association).
2. Caller identity is the VM identity of the execution context; all vCPUs of
   one VM share its authority in P5.
3. Rights are a fixed u32 bitset of five named classes (observe, control,
   modify, destroy, delegate); `delegate` is defined but unimplemented; the
   required-right set is a mandatory parameter supplied by each call's
   design.
4. Grants originate only from the Hypervisor/test bootstrap through an
   internal API returning an internal, non-Guest-visible grant handle.
5. Revocation is one-way to a tombstone; never-granted and revoked are
   internally distinct, uniformly `NO_AUTHORITY` Guest-visible.
6. Records bind to the target's slot+generation: target destruction or
   recreation neuters every record; a destroyed-event hook reclaims
   tombstones.
7. No-identity-shortcut is structural: no default rights, no
   identity-valued logic, no construction path outside `grant`.
8. The capability table has its own lock; W04 and W05 locks are never held
   simultaneously (W06 sequencing obligation).
9. Record capacity is fixed; exhaustion is `RESOURCE_EXHAUSTED`, never a
   denial that blames the caller.

## Work breakdown and loading order

1. Read [01](01-scope-and-foundations.md) for the ledger, assumed contracts
   AC-05.1–AC-05.5, and decisions 1–9.
2. Read [02](02-architecture-and-state.md) for modules, the record state
   machine, and the concurrency/lock-order rules.
3. Implement the rights vocabulary and grant path per
   [03](03-code-contracts-rights-and-grant.md), then the check, revocation,
   and denial classes per
   [04](04-code-contracts-check-revoke.md), in the order of
   [05](05-implementation-workflow.md).
4. Record decisions and deviations in
   `../p5-w05-capability-rights-bootstrap-revocation-record.md` when
   implementation starts; record commands and results in
   `../../verification/p5-w05-capability-rights-bootstrap-revocation-verification.md`
   when evidence exists. Neither this design nor any record claims W05 or
   P5 complete.

## Explicitly excluded interfaces

No delegation, attenuation, derived-capability, or revocation-tree
mechanism; no guest-initiated grant or revoke call in the v0 call set; no
authentication, credential, RBAC, or policy-persistence surface; no
Control Domain interface; no per-vCPU authority refinement; no rights bits
or capability encodings exposed to Guests (rights never cross the boundary
as Guest input or output in v0); no dispatch ordering (W06); no handle
lifecycle mechanics (W04); no machine ABI. Any implementation that lets a
handle value, VM identifier, first-VM status, or role influence the check
outcome other than through an explicit record violates the design and is a
review stop.

## Downstream handoff

Per the [P5 plan index](../../plans/README.md) consumer map:

- **W06**
  ([dispatch, permission, containment](../p5-w06-dispatch-permission-containment/README.md))
  receives the required authorization ordering input: `authorize` with a
  mandatory required-rights parameter, to be sequenced after W04 identity
  validation and before execution; the lock-order rule (never hold W04 and
  W05 locks together); and the two denial classes with their internal
  causes. W06 owns the actual ordering and the minimal operation's declared
  right requirement.
- **W07**
  ([validation guest suite](../p5-w07-validation-guest-isolation-suite/README.md))
  receives the observable authority semantics: valid use, `NO_AUTHORITY`
  uniformity for never-granted/revoked/wrong-caller, `INSUFFICIENT_RIGHTS`
  for right shortfalls, and the bootstrap setup hooks (two contexts, grants,
  revocation) its scenarios drive.
- **W08**
  ([host fuzz/stress/SMP baseline](../p5-w08-host-fuzz-stress-smp-baseline/README.md))
  receives the grant/check/revoke stress seam and the invariants fuzzing
  must preserve (no unauthorized success, tombstone persistence, record
  invariants).
- **W09/W10** receive the authority-event vocabulary (granted / revoked /
  denied causes) for redaction and regression; P6 receives only the evidenced
  extensible authority facts through W10 — interrupt and virtual-IRQ objects
  will enter the same record model through their own designs.
