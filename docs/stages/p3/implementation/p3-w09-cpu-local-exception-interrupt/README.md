# P3-W09 CPU-Local Exception and Interrupt Foundations — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** Making the P1 exception and interrupt diagnostic foundation
correct for every online physical CPU — per-CPU exception-local state,
CPU identity in exceptional paths, non-overlapping exception state,
CPU-attributed fatal diagnostics, and safe simultaneous logging — required
by [P3-W09](../../plans/p3-w09-cpu-local-exception-interrupt.md).  
**Owner/change context:** P3-W09 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P3-W09. It turns the P1
exception baseline — which is correct for one CPU by construction — into a
foundation that is correct for every online CPU at once: exception-local
state lives in each CPU's own
[P3-W04](../p3-w04-per-cpu-runtime/README.md) area slot; identity in an
exceptional path is resolved through a three-rank attribution rule rooted
in W04's `current()` guarantee and
[P3-W02](../p3-w02-secondary-cpu-bring-up/README.md)'s entry-stub
attribution; fatal diagnostics carry that attribution; and the only
cross-CPU shared resource on an exceptional path — the diagnostic console —
is serialized under the [P3-W06](../p3-w06-concurrency-synchronization/README.md)
Diagnostics rules with a bounded fatal-path strategy. It deliberately does
**not** design the full host IRQ subsystem, GICv3 virtualization, guest
interrupt handling, or the implementation-level vector/context layout: the
vector mechanics and context-capture register lists remain the
[P1-W05](../../../p1/plans/p1-w05-el2-exception-entry-baseline.md)
contract, the fatal-path field set remains
[P1-W07](../../../p1/plans/p1-w07-fatal-crash-diagnostics.md)'s, and
interrupt enablement beyond the P1 posture stays out of P3.

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

| Supporting file | Load it for |
|---|---|
| [01-scope-and-foundations.md](01-scope-and-foundations.md) | goal-to-baseline ledger, assumed-contract failure boundaries, scope classification |
| [02-architecture-and-state.md](02-architecture-and-state.md) | attribution ranks, per-CPU state model, ownership, failure model |
| [03-code-contracts-exception-local-state.md](03-code-contracts-exception-local-state.md) | exception-local slot contents, nesting, entry/exit contracts |
| [04-code-contracts-attribution-and-logging.md](04-code-contracts-attribution-and-logging.md) | attribution resolution, fatal integration, concurrent logging rules |
| [05-implementation-workflow.md](05-implementation-workflow.md) | ordered implementation steps |
| [06-validation-and-handoff.md](06-validation-and-handoff.md) | validation matrix, failure model, handoff checklist |

Before editing it must also follow the Coding Guidelines preflight. This
document is a proposed design; it contains no implementation or validation
claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P3 task book → P3-W09 plan → this
design → Coding Guidelines. Binding constraints:

- The task book requires "local runtime/exception/interrupt state ...
  for every online CPU" (T04 context) and makes T09 "exception and
  interrupt diagnostic foundations are correct per CPU" with P3-V09
  requiring "correct CPU identity and local diagnostic context on
  supported boot and secondary CPU exceptional paths".
- The plan's out-of-scope list removes full host IRQ subsystem design,
  GICv3 virtualization, guest interrupt handling, and
  implementation-level vector/context layout. The layout reservation is
  read precisely: the *contents bookkeeping* this design defines (nesting
  depth, attribution, in-exception flags) is W09's; the saved-register
  layout and vector-table mechanics stay P1-W05's contract.
- [P1-W05](../../../p1/plans/p1-w05-el2-exception-entry-baseline.md)'s
  handoff sentence — "IRQ dispatch and GIC behavior remain explicitly
  unimplemented" — is the P1 posture W09 preserves; the P3 interrupt
  posture is the P1 baseline's, and no new interrupt source is enabled by
  this package.
- [P1-W07](../../../p1/plans/p1-w07-fatal-crash-diagnostics.md) owns the
  fatal-path field set and its bounded, non-recursive property; W09 owns
  its per-CPU correctness: which CPU is reporting, from where in its
  lifecycle, and how its output avoids deadlocking on shared resources.
- [P3-W04](../p3-w04-per-cpu-runtime/README.md) guarantees `current()`
  is valid from the first post-install instruction and reserves the
  `ExceptionLocalSlot` for this design — the attribution foundation W09
  builds on.
- [P3-W06](../p3-w06-concurrency-synchronization/README.md) CR-4/CR-5 and
  the Diagnostics ladder class are binding: handler-context code does not
  allocate, does not take non-Diagnostics locks, and the fatal path uses
  bounded try-lock with a best-effort fallback.

Classification:

- **Required** for W09 closure: the exception-local slot contents and
  entry/exit bookkeeping, the three-rank attribution rule, the
  non-overlap invariant (no cross-CPU scratch state) as a checkable rule,
  the CPU-attributed fatal integration, the concurrent-logging rules,
  and P3-V09 acceptance evidence (boot + secondary exceptional paths).
- **Reserved** with recorded triggers: enabling any host interrupt
  source (trigger: an approved host-IRQ design — P3 keeps the P1
  posture); per-CPU IRQ-stack switching (trigger: same — at P3 exception
  entry uses the P1-baseline stack discipline); timer/fault-grade
  interrupt handling (trigger: P6); nested-exception capture depth beyond
  the bounded diagnostic nesting defined here (trigger: an approved
  design that needs deeper nesting).
- **Out of Scope:** GIC initialization, dispatch, or virtualization
  (P6); guest interrupts and vIRQ injection (P6/P8); scheduler preemption
  and deferred work (P7); the vector table's mechanics and saved-register
  layout (P1-W05 contract); crash storage and remote logging (P1-W07
  out of scope); telemetry catalog (W11); board-specific exception
  handling (ADR-043/ADR-052).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Legal per-CPU exception entry/local context | [slot/entry contracts](03-code-contracts-exception-local-state.md) §2, §4 | P3-V09 (W09-DV01, DV02) |
| CPU identity in exceptional paths | [attribution ranks](04-code-contracts-attribution-and-logging.md) §2 | P3-V09 (W09-DV03) |
| Non-overlapping exception state | [non-overlap invariant](03-code-contracts-exception-local-state.md) §6 | P3-V09 (W09-DV04) |
| CPU-attributed fatal diagnostics | [fatal integration](04-code-contracts-attribution-and-logging.md) §3 | P3-V09 (W09-DV05) |
| Safe simultaneous diagnostic/logging behavior | [logging rules](04-code-contracts-attribution-and-logging.md) §4 | P3-V09 (W09-DV06) |
| Integration with audit, telemetry, stress, regression consumers | [integration map](02-architecture-and-state.md) §7; [handoff](06-validation-and-handoff.md) §3 | W09 closure review (W09-DV07) |
| Boot/secondary exceptional-path acceptance evidence | [workflow](05-implementation-workflow.md) steps 5–6; matrix in [validation](06-validation-and-handoff.md) | P3-V09 (W09-DV02, DV03); matrix execution is [P3-W13](../p3-w13-qemu-smp-regression/README.md) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p3-implementation-designs`):
P0 documentation scaffold only — no workspace, no sources, no vector
table, no exception code. The P1 exception/fatal plans exist as planning
documents without implementation or verification evidence; their inputs
remain conditions for implementation per the task book. Sibling P3
designs W01–W08 exist as proposed designs on this branch; W10–W15 are
being prepared in parallel. W09 consumes W02/W03/W04 and the P1 contracts
as upstream, and serves W10–W15 and P4 downstream, referenced by path and
P3-Wxx ID.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Exception-local context is legal per CPU | No exception state exists anywhere; P1 baseline is single-CPU by construction | The `ExceptionLocalSlot` contents (nesting, flags, fatal record) with entry/exit bookkeeping rules | Two CPUs sharing one exception scratch is undefined at best; per-CPU state is the only legal shape | W09 contents; W04 slot placement | W09-DV01 |
| CPU identity is correct in exceptional paths | Identity exists only via W04's `current()` (post-install) and W02's stub (pre-install) | The three-rank attribution rule resolving identity from `current()`, MPIDR fallback, or an explicit unknown | An exception on a half-initialized CPU must still say which CPU it was on, or diagnostics mislead | W09 rule; W04/W02 guarantees | W09-DV03 |
| Exception state does not overlap across CPUs | Nothing enforces it | The non-overlap invariant as a checkable rule (all exception-writable state is in the owning CPU's area) + review/audit hook | A single global save/scratch region is the classic SMP bring-up corruption; P3-V09 forbids it structurally | W09 invariant; W10 audit item | W09-DV04 |
| Fatal diagnostics are CPU-attributed | P1-W07's field set is plan-level; no SMP story exists | The fatal integration: per-CPU record first (lock-free), console under CR-5 rules, attribution line mandatory | A fatal report without a CPU is undiagnosable on an 8-CPU boot | W09 integration; P1-W07 fields | W09-DV05 |
| Simultaneous logging is safe | P1 console is single-CPU by construction | The Diagnostics-lock rules for normal logging and the bounded fatal-path strategy | Simultaneous unlocked console writes on 8 CPUs interleave lines into noise | W09 rules; W06 Diagnostics class | W09-DV06 |
| Boot and secondary exceptional paths are exercised | No QEMU harness (W13 is a plan) | Intentional synchronous-fault evidence per CPU role (boot + secondary), per the P1-W05 acceptance pattern | P3-V09 requires both roles demonstrated, not just the boot CPU | W09 scenarios; W13 matrix | W09-DV02/DV03 |

No ledger row requires this design to fix vector layouts, enable
interrupts, or select crate names; the P1-contract conflict boundary is
recorded in [01 §1.2](01-scope-and-foundations.md). No new decision
blocker is outstanding here.

## Resolved design decisions and their authority

1. **All exception bookkeeping is per-CPU, in the W04 area's
   `ExceptionLocalSlot`.** Nesting depth, in-exception flag, current
   vector class, and the fatal record live in the owning CPU's area; no
   global exception state of any kind remains. Rationale: the plan's
   "non-overlapping exception state" is structural only as
   CPU-privateness; W04 already guarantees the slot and the validity of
   `current()` from the first post-install instruction.
2. **Three-rank attribution.** Rank 1: `current()` header validation
   (post-install CPUs — full attribution: logical id, hardware id, boot
   flag, lifecycle state via W03). Rank 2: MPIDR read (the W02
   entry-stub mechanism) for CPUs that fault before or during install —
   degraded attribution, no per-CPU state writes. Rank 3: an explicit
   `unknown` attribution when even MPIDR cannot be trusted. Every fatal
   or diagnostic output names its rank. Rationale: P3-V09's "correct CPU
   identity ... on supported boot and secondary CPU exceptional paths"
   includes the paths where per-CPU state is not yet trustworthy;
   pretending otherwise would produce confident wrong answers.
3. **The P1 vector contract is consumed, not amended.** W09 requires of
   the P1-W05 baseline only what its own contract already provides (an
   exception path that can emit context) and relocates its *state* to
   per-CPU storage; the vector-table mechanics, saved-register layout,
   and origin classification stay P1's. If the delivered P1 contract
   freezes a global save region or context buffer, that is an
   Architecture Change Request to resolve with the P1 owner — W09 does
   not silently fork the entry path. Rationale: authority order; the
   plan's out-of-scope line on vector/context layout.
4. **Bounded diagnostic nesting; recursion is a terminal state.** Each
   CPU's nesting depth is counted in its own slot; a fault while
   handling a fault is allowed one bounded diagnostic level (the P1-W05
   non-recursive requirement), and exceeding the bound takes the
   terminal fatal path with the interrupted context's attribution and
   no further capture. Rationale: P1-W07's "does not itself become an
   unobservable recursive crash", made per-CPU.
5. **The interrupt posture is the P1 baseline's; W09 enables nothing.**
   Exceptional-path rules are defined for all exception classes (they
   can occur regardless of the DAIF posture), but no host IRQ source is
   enabled and no IRQ dispatch is designed. The W06 irq-save flavor
   rules (CR-2/CR-3) are declared binding so the future enablement
   design inherits them without rework. Rationale: P1-W05's handoff
   sentence and the plan's out-of-scope line; the task book's
   interrupt-sensitive requirement is satisfied by rules + posture, not
   by new delivery mechanisms.
6. **Normal logging takes the Diagnostics lock; the fatal path is
   bounded-try-lock with best-effort fallback.** Normal diagnostic
   emission acquires the Diagnostics-class lock (W06) so multi-CPU lines
   stay whole; the P1-W07 fatal path attempts the same lock with a
   bounded spin and, on failure, emits best-effort with an explicit
   interleave marker — a corrupted-but-attributed report beats a dead
   one. Rationale: CR-5 (fatal path must not deadlock on a possibly-
   never-released lock) and P3-V09's "safe simultaneous diagnostic/
   logging behavior" which must hold *during* faults, not only in their
   absence.
7. **Attribution is mandatory metadata, not decoration.** Every
   exceptional-path diagnostic line carries the CPU's logical id,
   hardware id (as available per rank), boot/secondary role, and
   lifecycle state — the fields W11's observability contract needs.
   Rationale: P3-V11 requires CPU-attributed observability; W09 is where
   the exceptional-path half of that guarantee is made true.

## Work breakdown and loading order

1. Read [01-scope-and-foundations.md](01-scope-and-foundations.md) for
   the ledger, assumed-contract boundaries, and scope split.
2. Read [02-architecture-and-state.md](02-architecture-and-state.md) for
   the attribution ranks, state model, ownership, and failure model.
3. Implement per [05-implementation-workflow.md](05-implementation-workflow.md):
   the exception-local state with
   [03](03-code-contracts-exception-local-state.md) (steps 1–3), the
   attribution and logging integration with
   [04](04-code-contracts-attribution-and-logging.md) (steps 4–6).
4. Record implementation decisions in
   `../p3-w09-cpu-local-exception-interrupt-record.md` and evidence in
   `../../verification/p3-w09-cpu-local-exception-interrupt-verification.md`
   only when the work is performed. Validation conditions and the handoff
   checklist are in [06-validation-and-handoff.md](06-validation-and-handoff.md).

## Explicitly excluded interfaces

No vector-table layout, saved-register list, or origin-classification
scheme (P1-W05 contract); no GIC register, interrupt route, or dispatch
design (P6); no guest interrupt or vIRQ surface (P6/P8); no IRQ stack
switching or deferred-work machinery (Reserved/future); no telemetry
catalog (W11); no crash storage (P1-W07 out of scope). W09 does not
modify W03's registry semantics (it reads state for attribution only),
does not change W02's entry path (it consumes its attribution
guarantee), and does not alter W04's area layout (it fills the reserved
slot per its own contents contract). Enabling any interrupt source, or a
"convenient" global exception buffer, is a scope violation to stop at
review.

## Downstream handoff

- **W10** receives the non-overlap invariant and the attribution-rank
  rules as audit criteria for P0–P2 diagnostic/console state and for the
  P1 baseline's exception surfaces.
- **W11** receives the attribution field set and the exceptional-path
  emission points as the observability contract's SMP half.
- **W12** receives the concurrent-fault and simultaneous-logging
  scenarios as stress stimuli; W13 receives the boot/secondary
  exceptional-path scenarios as regression rows.
- **W15** receives the diagnostic-foundation limits for stage
  documentation.
- **P4** (through [P3-W14](../p3-w14-p4-smp-handoff/README.md)) receives
  the per-CPU diagnostic foundation: correct CPU attribution on any
  online CPU, a bounded fatal path, and safe logging — P4 designs its
  guest-fault diagnostics against this without reworking P3.
- **Future host-IRQ design owner** receives the declared posture, the
  CR-2/CR-3 bindings, and the recorded Reserved triggers as its starting
  contract.
