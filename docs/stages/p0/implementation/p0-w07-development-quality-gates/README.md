# P0-W07 Development Quality Gates — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The required/informational/future check classification, per-gate
minimum standards, development and integration minimum verification sets, and
CI failure-handling semantics required by
[P0-W07](../../plans/p0-w07-development-quality-gates.md).  
**Owner/change context:** P0-W07 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P0-W07. It converts the bounded
work-package plan into one normative quality-gates contract document plus
minimal discovery wiring. The gates are defined here as **policy and
classification**, not as committed scripts or CI configuration: the contract
owning this design fixes what each gate checks, what passing means, whether it
blocks merge, how failures are handled, and how evidence is attributed. It
deliberately does **not** implement GitHub workflows or required checks
(P0-W20, CI baseline; its design is being prepared in parallel per the plan
index), create the Cargo workspace, build
targets, or warning enforcement mechanics
([P0-W03](../p0-w03-aarch64-build-target-baseline/README.md)), define the
host-test baseline ([P0-W08](../p0-w08-host-side-testing-baseline/README.md)),
define the QEMU runner entry
([P0-W09](../p0-w09-qemu-automation-entry-baseline/README.md)), or implement
any hypervisor integration test or EL2 smoke check (plan out-of-scope).

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then loads
only the linked supporting file needed for its assigned step. Before editing it
must also follow the Coding Guidelines preflight, including the repository
`AGENTS.md`, documentation index, [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), P0 task book,
and P0-W07 plan. This document is a proposed design; it contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P0 task book → P0-W07 plan → this design
→ Coding Guidelines. In particular:

- The task-book outcome for W07 is: format, lint, host-test, target-build,
  documentation, warning, and CI-failure **rules are defined**, validated by
  P0-V06–V08. Defining the rules is W07 scope; executing them in GitHub CI and
  enforcing `main` protection is W20.
- The plan requires three deliverables: check categories with blocking
  semantics, minimum standards per check area, and development/release minimum
  sets with failure-handling principles, all consumable by CI with clear
  evidence attribution.
- [P0-W02](../p0-w02-rust-toolchain-baseline/README.md) hands W07 a pinned
  toolchain with `rustfmt` and `clippy` components and delegates "the exact
  gate command spelling and failure semantics" to W07; this design accepts
  that delegation.
- The [integration
  workflow](../../../../development/integration-workflow.md) is normative
  policy: a missing, cancelled, skipped, or failed required check is not
  passing evidence. The failure semantics defined here must restate and
  operationalize that rule for gates, not weaken it.
- W03 and W08 own the build and test entries that two gates bind to. This
  design defines the binding and the blocking semantics; it does not define
  the bound entries. The plan index does not list W03 as a W07 prerequisite;
  the task-book phase order (phase A before phase B) makes W03's delivery an
  implementation-time assumption with an explicit failure boundary (workflow
  §1).

Classification: the gate register, the six baseline required gates, the
failure-handling principles, the development and integration minimum sets, and
evidence-attribution rules (all in the supporting files) are **Required** for
W07 closure. Future-class membership (unsafe-audit consistency, rustdoc
generation, QEMU-class checks, fuzz/property, hardware) is **Reserved** with a
promotion rule, not implemented. CI workflows, required-check configuration,
branch protection, crate manifests, target definitions, gate-runner scripts,
and all hypervisor code are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Required / informational / future categories with blocking semantics | [Gate register](01-gate-register.md) §2–§3 | W07-DV01 |
| Minimum standards: formatting, lint, warning | [Gate standards](02-gate-minimum-standards.md) §2–§4 | P0-V06/V07 definitions (W07-DV02) |
| Minimum standards: host test, target build, documentation checks | [Gate standards](02-gate-minimum-standards.md) §5–§7 | W07-DV02 (bindings to W08/W03) |
| Development and integration minimum verification sets | [Gate standards](02-gate-minimum-standards.md) §8 | W07-DV03 |
| Check-failure handling principles | [Gate standards](02-gate-minimum-standards.md) §9 | W07-DV03 |
| Consumability by CI and evidence attribution | [Gate register](01-gate-register.md) §4, [workflow](03-implementation-and-validation.md) step 5 | W07-DV04/DV05; P0-V08 evidence arrives with W20 |
| Document discoverability and coherence | [workflow](03-implementation-and-validation.md) step 4 | P0-V09 (W07-DV06) |
| Downstream handoff to W20 and later work | [workflow](03-implementation-and-validation.md) handoff checklist | W07 closure (W07-DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p0-implementation-designs` at
`4e631ee`): no tracked document defines any quality gate, lint policy, warning
policy, or CI failure rule; `.github/workflows/` contains only a `.gitkeep`
marker; no Cargo workspace, manifest, or Rust source exists, so no gate has
anything to execute today. `docs/development/` contains the agent guides and
the integration workflow only. The W01 repository baseline is completed with
recorded verification; the W02 toolchain design is proposed (its pin manifest
and contract document do not exist yet). W03, W05, and W08 sibling designs are
being prepared per the plan index; until accepted, their contracts do not
exist. Each ledger row below states the missing foundation the plan outcome
necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Check categories and merge-blocking semantics are defined | No classification of any check exists in tracked files | Gate register with required/informational/future classes, blocking defaults, and a promotion rule | Without a register, W20 has nothing to classify and every future check invents its own status | W07 (this design) | W07-DV01 register review |
| Minimum standards for formatting, lint, and warnings | No lint/warning policy document; W02 design guarantees components only | Per-gate standards fixing purpose, invocation semantics, passing condition, and failure semantics | "Defined rules" requires per-gate meaning, not just tool presence | W07; mechanics inside W03's build baseline | W07-DV02 standards review |
| Minimum standards for host test, target build, documentation checks | Bound entries do not exist (W03/W08 proposed or in preparation) | Binding-based standards: each gate names the owning entry contract and the blocking semantics | W07 owns classification and blocking; entry ownership stays with W03/W08 | W07 bindings; W03/W08 entries | W07-DV02; W07-DV07 consumability |
| Development and release minimum verification sets | Absent | Two named sets mapped onto the register with required membership | Contributors and integrators need to know which gates run where | W07 | W07-DV03 |
| Check-failure handling principles | Absent; integration workflow states the PR-level rule only | Gate-level failure principles aligned with the integration workflow | The plan requires the failure-handling 处理原则 (principles), not merely classification | W07 | W07-DV03 |
| Rules consumable by CI with clear evidence attribution | No CI exists (`.gitkeep` only) | Register fields W20 can map 1:1 to checks; per-gate evidence label and attribution rule | CI wiring is only mechanical if the register is complete and stable | W07 register; W20 wiring | W07-DV04/DV05; P0-V08 completes with W20 |

No row requires inventing a crate, target triple, script, or CI mechanism, so
no decision blocker is outstanding for the design itself. Two conditional
dependencies are recorded as open items, not resolved here: W03 must deliver a
stable target-build entry and W08 a stable host-test entry, or the respective
gate bindings cannot be completed (workflow §1 states the failure boundary).

## Resolved design decisions and their authority

1. **Policy home:** `docs/development/quality-gates.md` is the sole normative
   home of gate classification, standards, minimum sets, and failure
   semantics, following the W02 precedent of a development-governance contract
   document inside the plan's scope (质量门禁规则). [P0-W05](../p0-w05-documentation-baseline/README.md)
   may later re-home it; semantic ownership stays with the document.
2. **Gate identity:** stable gate IDs (`QG-FMT`, `QG-LINT`, `QG-WARN`,
   `QG-TEST-HOST`, `QG-BUILD-TARGET`, `QG-DOCS`) are the machine-consumable
   classification keys W20 maps to GitHub checks. IDs are documentary data,
   not script or program names.
3. **Baseline required set:** exactly the six gates above, all required and
   merge-blocking. They cover the plan's named areas (format, lint, host test,
   AArch64 build, documentation, warning policy) with no speculative
   additions.
4. **Informational and future classes:** the informational class is defined
   and may be empty at P0; the future class is non-blocking by default and
   enters through the register's promotion rule, which requires an owning
   design and a recorded rationale. Expected future members are named only by
   owner (unsafe audit → [W10](../p0-w10-unsafe-rust-governance/README.md);
   QEMU-class checks → [W09](../p0-w09-qemu-automation-entry-baseline/README.md)
   and P1; fuzz/property and hardware → later stages per ADR-049).
5. **Command-spelling ownership:** W07 fixes invocation semantics for
   QG-FMT/QG-LINT/QG-WARN (the W02 delegation). For QG-TEST-HOST and
   QG-BUILD-TARGET, W07 fixes the binding and blocking semantics and the bound
   entry spelling is recorded from the W08/W03 delivered contracts at
   implementation time. For QG-DOCS, W07 fixes the check semantics; the
   mechanical realization inside CI is W20's, and any tooling dependency it
   introduces follows dependency governance ([W18](../p0-w18-dependency-governance/README.md))
   once available.
6. **Failure semantics:** a required gate that fails, or does not report a
   passing result (missing, cancelled, skipped), blocks merge; reruns are
   bounded and recorded; a flaky required gate is a defect, not tolerated
   variance. This operationalizes, and never weakens, the integration
   workflow's rule.
7. **Mutation thresholds:** changing a gate's classification, scope, or
   passing condition is a design-level change recorded against this contract;
   adjusting literal invocation spelling to track a delivered W03/W08 entry is
   a recorded minor change; making a hypervisor-TCB-affecting check optional is
   an architecture-level decision requiring the [ADR
   path](../../../../adr/README.md).

## Work breakdown and loading order

1. Read the [gate register](01-gate-register.md) for the artifact groups, the
   register table, the classification and promotion rules, and the
   machine-consumability fields.
2. Read the [gate standards](02-gate-minimum-standards.md) for each gate's
   minimum standard, invocation semantics, and the development/integration
   minimum sets and failure-handling principles.
3. Apply the changes in the order stated in the [implementation
   workflow](03-implementation-and-validation.md): verify prerequisite
   surfaces, author the gates contract document, run local gate dry-runs where
   the tree allows, wire discovery, run the consumability review, close.
4. Store actual commands, output, environment, and result in
   `../../verification/p0-w07-development-quality-gates-verification.md`, and
   record decisions taken and changed artifacts in
   `../p0-w07-development-quality-gates-record.md` only when implementation
   begins. Neither this design nor a written record may claim W07 complete.

## Design-level state and lifecycle

W07 adds no runtime state, code path, or executable program. The authoritative
state is one tracked contract document plus its discovery links. Its
documentary lifecycle:

```text
no gate rules
  -> quality-gates contract committed (register + standards + sets + failure rules)
  -> discovery links (docs/README routing row, stage index) committed
  -> local dry-runs of executable gates evidenced where prerequisite surfaces exist
  -> W20 maps the register onto GitHub required checks (P0-V08)
  -> later changes mutate only through the contract's thresholds
     (W08/W03 entries finalize bindings; W10 promotes the unsafe-audit gate;
      W09/P1 promote QEMU-class checks; W19 documents the contributor path)
```

The contract document is the owner of every gate statement. A GitHub check
whose semantics contradict the register is a review failure, not a local
choice; W20 must raise the conflict, not redefine the gate.

## Explicitly excluded interfaces

No script, program, CI workflow file, crate, Rust type, function, trait,
public API, ABI, or persistent data layout is designed or authorized by W07.
The only machine-facing surface is the register's documentary fields (gate ID,
class, blocking flag, entry binding, evidence label), consumed as data by W20;
see [the gate register](01-gate-register.md) §5. Adding a gate-runner script
or workflow here is a scope conflict against W20/W03 ownership and must be
stopped at review.

## Downstream handoff

- **W20** receives the register as the classification source of truth: it maps
  each required gate to a GitHub check, sets required status, implements
  `main` protection, and owns evidence that enforcement is real. W20 must not
  redefine gate semantics; conflicts go back through the contract's
  thresholds.
- **W08** receives the requirement that QG-TEST-HOST binds to exactly one
  documented host-test execution entry; W08's design must publish that entry
  in a form the register can bind (stable spelling, truthful exit status).
- **W03** receives the requirement that QG-BUILD-TARGET binds to its delivered
  target-build entry and that warning-enforcement mechanics land in its build
  configuration under QG-WARN's policy.
- **W19** receives the development and integration minimum sets as the
  contributor-facing summary of which gates run where on the clone-to-PR path.
- **W09 and P1** receive the future-class promotion rule as the only path by
  which QEMU-based checks become register members; W09's runner entry is not a
  gate and gate status is not granted by W09.
- **W10** receives the promotion path for an unsafe-inventory-consistency
  gate; the inventory policy the future gate would check is owned by W10.
- **W05** may re-home the contract document under its documentation taxonomy;
  **W18** owns intake for any gate tooling dependency introduced later.
