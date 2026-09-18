# P8-W05 Linux CPU Virtualization Compatibility — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The Guest CPU behavior classification and controlled-failure
boundary for Linux required by
[P8-W05](../../plans/p8-w05-linux-cpu-virtualization.md).  
**Owner/change context:** P8-W05 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P8-W05. P8-W05 fixes *how Linux
CPU behavior is classified and how unsupported operations fail* — the
five-class posture (Direct / Emulate / Reject / Hidden / Unsupported) the
plan names, the per-area classification inventory, the exit-path decision
point that applies it, and the controlled VM-facing rejection and diagnostic
boundary. It deliberately does **not** implement system-register emulation,
define trap tables, choose CPU-feature values or register encodings, decide
an unsupported-operation policy beyond controlled diagnostics, or redesign
any P4–P7 mechanism it consumes. Domain behavior is owned where the task
book assigns it: PSCI by [P8-W06](../p8-w06-psci-virtualization/README.md),
the virtual GIC by [P8-W07](../p8-w07-linux-vgicv3/README.md), timer by
[P8-W08](../p8-w08-linux-timer-integration/README.md), console by
[P8-W09](../p8-w09-virtual-console-single-cpu-linux/README.md), and
fault-diagnosability breadth by [P8-W13](../p8-w13-guest-fault-diagnostics/README.md).

This is a code-bearing design: P8-V07 and P8-V08 are runtime evidence, so
the design specifies the minimal code contracts that make classification
operational — the classification data model, the decision point on the
established P4 vCPU exit path, the handler registry, and the controlled
rejection path. Consistent with the task book (P8 defines no crate/module
tree), these contracts are logical: each names its responsibility,
signature-shape, ownership, and failure boundary; the home crate/module is
assigned at implementation time inside the ADR crate boundaries, and the
Coding Guidelines govern the code that realizes them.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md) — for this
package the guest-input, MMIO/register, error-classification, and
unsafe-boundary rules are load-bearing, and any change crossing the trap or
security boundary makes the detailed-reference consultation mandatory. The
agent then loads only the linked supporting file needed for its assigned
step. Before editing it must also follow the repository `AGENTS.md`,
documentation index, ADR baseline, P8 task book, and the P8-W05 plan. This
document is the proposed detailed design; it contains no implementation or
validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P8 task book → P8-W05 plan → this
design, with the established P4–P7 contracts as assumed inputs. In
particular:

- ADR-022 fixes Guest EL1 with no virtual EL2; ADR-007/ADR §19 make Guest
  input untrusted and Guest-caused faults VM-scoped, never a global panic;
  ADR §13 distinguishes `GuestFault` from `InvariantViolation` — the
  classification boundary reuses that distinction and adds nothing new.
- The task book §2 (P4–P7 rows) fixes the consumption boundary: this design
  consumes the P4 vCPU entry/exit and Stage-2 facts, the P5 checked-input
  and failure-classification facts, the P6 timer/interrupt facts, and the P7
  run-state/block-wakeup facts. It redesigns none of them; where a consumed
  contract is contradicted, the affected step stops and the conflict is
  recorded (`Architecture Change Request`).
- The task book §8 routes CPU-feature values and per-register detail through
  `Specification Investigation` into the W02 machine categories: this design
  fixes classification classes and posture; it proposes per-area
  classifications as reviewable rows whose concrete values (feature
  baseline, ID-register presentation, per-register traps) remain routed.
- The PSCI/SMC conduit and all interrupt injection stay inside the P5
  hypercall and P6 interrupt contracts: classification routes PSCI attempts
  to [W06](../p8-w06-psci-virtualization/README.md) and GIC/timer attempts
  to [W07](../p8-w07-linux-vgicv3/README.md)/[W08](../p8-w08-linux-timer-integration/README.md)
  — it never gives this package its own conduit or injection path.

Classification: the classification model and per-area inventory
([01](01-classification-model.md)), the code contracts
([02](02-code-contracts-classification.md)), the controlled-failure and
diagnostics boundary ([03](03-controlled-failure-and-diagnostics.md)), the
implementation workflow ([04](04-implementation-workflow.md)), and the
validation/handoff material ([05](05-validation-and-handoff.md)) are
**Required**. Per-register trap tables, feature-value presentation sets, and
performance instrumentation beyond the counters named in
[03](03-controlled-failure-and-diagnostics.md) are **Reserved**. System-register
emulation bodies, CPU-feature value selection, register encodings, an
unsupported-operation policy beyond controlled diagnostics, scheduler policy,
PSCI/vGIC/timer/console mechanisms, and any new security model are **Out of
Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Inspect W01/W03 and P4–P7 execution and exception facts | [workflow](04-implementation-workflow.md) step 1 | P8-V07 (W05-DV01) |
| Inventory Linux-required behavior by classification rather than mechanism | [classification model](01-classification-model.md) §2–§4 | P8-V07 (W05-DV02) |
| Define evidence for normal Linux paths | [validation and handoff](05-validation-and-handoff.md) §2 (V07 rows) | P8-V07 (W05-DV05) |
| Define evidence for unsupported operations | [controlled failure](03-controlled-failure-and-diagnostics.md) §2–§4, [validation](05-validation-and-handoff.md) §2 (V08 rows) | P8-V08 (W05-DV04) |
| Relate classified failures to VM-facing containment and diagnostics | [controlled failure](03-controlled-failure-and-diagnostics.md) §2–§3, [code contracts](02-code-contracts-classification.md) §4 | P8-V08 (W05-DV04) |
| Review against Guest EL1, host independence, and no-global-panic rules | [workflow](04-implementation-workflow.md) step 6 | P8-V07/V08 (W05-DV06) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p8-implementation-designs`
at `4e631ee`): the repository contains no Rust source at all
(`hypervisor/src/.gitkeep`, `crates/.gitkeep`); P0–P7 are plans and task
books without implementation or verification records. There is no exit path,
no exception vector, no allocator, no telemetry, and no scheduler to consume:
every contract this design consumes (P3 pCPU identity, P4 exit path and
Stage-2, P5 error classes, P6 timer/interrupt injection, P7 run states) is a
planned-only assumed contract. W05 therefore defines its code contracts
against those assumed interfaces and states, per contract, the failure
boundary that applies if a predecessor delivers a different shape.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| A compatibility classification for Linux Guest CPU behavior exists | No classification, exit path, or trap code exists anywhere | The five-class model with assignment rules and invariants ([01](01-classification-model.md) §2) | A "classification" is only real as a fixed, reviewable model the exit path can apply | W05 (this design) | W05-DV02 model review |
| Linux-required behavior is inventoried | Nothing inventories Linux's CPU expectations | Per-area classification inventory with authority basis and routed values ([01](01-classification-model.md) §4) | P8-V07 names the areas (sysreg, MMU/TLB/cache/barrier/WFI/WFE/features); each needs a declared class | W05 proposes; W02 C2 review owns values | W05-DV02/DV03 |
| Unsupported operations fail in a controlled, VM-scoped way | Nothing defines the failure boundary | Controlled-rejection boundary + diagnostic record + containment rules ([03](03-controlled-failure-and-diagnostics.md) §2–§3) | P8-V08 requires "explicit VM diagnostic/controlled outcome, not silent corruption or global panic" | W05; failure classes per P5 assumed contract | W05-DV04 negative review |
| Classification is operational on the exit path | No exit path exists (P4 planned-only) | Decision-point and handler-registry contracts shaped to the P4 exit facts ([02](02-code-contracts-classification.md) §3–§4) | V07/V08 are runs; the classification must be reachable from the real exit path, not a parallel demo path | W05 contracts; P4 exit facts assumed | W05-DV03 contract review |
| Normal-path evidence defined | No Linux fixture or regression exists | V07 scenario rows bound to the fixture/regression packages ([05](05-validation-and-handoff.md) §2) | Normal-path evidence must name scenarios without inventing W15/W16 scope | W05 scenarios; W15/W16 own fixtures/runners | W05-DV05 |
| No global panic; no host dependence; Guest EL1 | Constraints exist only in ADR/task book | Guardrail checks in the workflow ([04](04-implementation-workflow.md) step 6) and invariants in [01](01-classification-model.md) §5 | The plan's review item 5 requires these checks explicitly | W05 | W05-DV06 |

No row requires selecting a CPU-feature value or register encoding, so no
decision blocker arises from this design. The standing routed items (feature
baseline, ID-register presentation values, per-register trap detail) remain
`Specification Investigation` per the task book; the dominant limitation —
no implemented predecessor exists — is a recorded evidence boundary, not a
blocker to designing the contracts.

## Resolved design decisions and their authority

1. **Class set and semantics.** Exactly the plan's five classes —
   Direct, Emulate, Reject, Hidden, Unsupported — with the semantics and
   assignment rules of [01](01-classification-model.md) §2. Rationale: the
   plan authorizes the vocabulary; inventing more classes would alter P8
   scope, fewer would lose the distinction between concealment posture
   (Hidden) and known-gap admission (Unsupported).
2. **Classification as data, applied at one decision point.** Behavior
   classes live in a read-only-after-init classification registry; the
   vCPU exit path consults it once per trapped operation and acts per the
   class ([02](02-code-contracts-classification.md) §3). Rationale: keeps
   mechanism out of the classification (plan: "by classification rather
   than mechanism"), makes the fail-closed default enforceable in one place,
   and lets W06–W09 attach handlers without touching the decision point.
3. **Fail-closed default.** Any trapped operation that does not resolve to a
   registry entry — including a registered class whose handler is missing or
   fails to produce an outcome — is treated as Reject with full diagnostics
   ([02](02-code-contracts-classification.md) §4; [03](03-controlled-failure-and-diagnostics.md)
   §2). Rationale: ADR-007 (Guest untrusted); silent fallthrough is the
   specific failure P8-V08 forbids.
4. **Rejection posture.** Reject produces an architecturally legal Guest
   fault (the Guest observes a fault at its own exception level), a
   structured diagnostic, contained containment per the P5 failure classes,
   and telemetry — default containment stops the offending vCPU; escalation
   to a VM-level faulted state only when the P5-established criteria say the
   VM state is unrecoverable ([03](03-controlled-failure-and-diagnostics.md)
   §3). The exact per-class stop/fault mapping is proposed here and must be
   reconciled with the P5 closeout contract before implementation relies on
   it. Rationale: plan scope is "controlled diagnostics" — this is the
   minimal policy that satisfies P8-V08 without designing a new failure
   policy.
5. **Discovery-shaped features (Hidden) precede usage.** Feature
   presentation is controlled at discovery (ID-register reads present the
   approved baseline); a Guest that attempts a hidden feature anyway falls
   through to the Reject path. Rationale: gives Linux a coherent feature
   view without pretending unsupported features work; keeps the
   concealment mechanism (ID presentation) inside the classification
   posture while values stay routed.
6. **Domain routing, not domain redesign.** Timer, GIC/system-register, and
   PSCI attempts resolve to their owning packages' contracts through named
   handler routes; this design defines the route interface and the
   missing-handler behavior only. Rationale: task book work map; P5/P6
   contract protection.
7. **WFE posture.** Proposed: WFE executes directly (architectural
   semantics, no trap) while WFI is classified Emulate and integrates with
   the P7 block/wakeup contract; escalation of WFE to Emulate is Reserved
   with the trigger "P7 M:N overcommit evidence shows shared-pCPU starvation
   attributable to Guest WFE". Rationale: WFI blocking is required for
   scheduler correctness (P7-W06); trapping WFE adds exit cost without a
   correctness need at the P8 posture; the trigger keeps the door open
   without pre-deciding scheduler policy (explicitly out of scope).
8. **Logical contracts, deferred homes.** No crate, module, or file path is
   fixed; each contract in [02](02-code-contracts-classification.md) names
   its layer-respecting home requirements (arch-side for syndrome decode,
   core-side for classification policy — per ADR §13 layering) and the home
   assignment happens at implementation within the established workspace.
   Rationale: the task book forbids P8 from defining crate/module trees;
   P0–P7 crate structures do not exist yet.

## Work breakdown and loading order

1. Read [the classification model](01-classification-model.md): class
   semantics, assignment rules, per-area inventory, invariants.
2. Read [the code contracts](02-code-contracts-classification.md): the
   classification registry, the exit-path decision point, the handler
   route interface, and the rejection function.
3. Read [the controlled-failure boundary](03-controlled-failure-and-diagnostics.md):
   diagnostic record, containment rules, telemetry, and the
   Guest-fault-versus-invariant line.
4. Apply [the implementation workflow](04-implementation-workflow.md), then
   close with [the validation and handoff file](05-validation-and-handoff.md).
5. Record implementation decisions in
   `../p8-w05-linux-cpu-virtualization-record.md` when work starts, and
   evidence in
   `../../verification/p8-w05-linux-cpu-virtualization-verification.md` when
   validation is exercised. Neither this design nor any record may claim W05
   complete; P8-V07/V08 claims live only in verification material and only
   for what actually ran.

## Explicitly excluded interfaces

W05 authorizes no public ABI, wire format, or Guest-visible value: no
CPU-feature value, ID-register presentation set, register encoding, trap
table layout, PSCI function, vGIC or timer register model, or console
model. The only Guest-visible surface it changes is the *class* of a
failure (controlled and diagnosable), which is posture, not value. The
design also authorizes no new security model, no scheduler policy, no new
conduit, and no redesign of the P4 exit path, P5 error classes, P6
interrupt/timer mechanisms, or P7 run states. Contracts in
[02](02-code-contracts-classification.md) are internal design constructs;
none of them freezes a public API signature — home crates, exact types, and
signatures are fixed by the implementation following the Coding Guidelines
and the then-established module contracts. Emulation bodies for individual
system registers are out of scope and belong to the consuming domain
packages or their integration work.

## Downstream handoff

Per the [P8 plan index](../../plans/README.md), W05 feeds W06, W09–W10, W13,
and W18 (and, through the W02 categories, W07/W08):

- **W06** receives the classification classes for PSCI attempts (Emulate,
  routed handler) and the requirement that its PSCI contract register
  against the handler route of
  [02](02-code-contracts-classification.md) §4; the conduit stays inside
  the P5 hypercall contract.
- **W07 / W08** receive the same handler-route requirement for GIC
  system-register interface and timer attempts, plus the invariants their
  handlers must preserve (fail-closed, bounded exit-path work).
- **W09 / W10** receive the classified CPU posture their boot and SMP paths
  rely on: EL1-only presentation, WFI scheduling integration, MMIO console
  traps handled as classified MMIO (route to the console device, not to
  this package), and the Reject boundary their negative tests will hit.
- **W13** receives the diagnostic record shape and the
  Guest-fault-versus-invariant boundary as the fault-diagnostics baseline;
  W13 owns diagnosability breadth beyond this boundary.
- **W18** receives the classification inventory and rejection path as the
  isolation regression's target surface (illegal sysreg, unsupported
  feature, and topology attempts must land in the Reject path with
  contained outcomes).

No consumer may implement a mechanism by reclassifying a behavior outside
[01](01-classification-model.md)'s change rules, and none may treat the
per-area inventory as frozen values before the W02 C2 review resolves them.
