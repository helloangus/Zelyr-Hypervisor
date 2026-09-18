# P3-W04 Per-CPU Runtime — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The independent execution and CPU-local state foundations for
every online physical CPU required by
[P3-W04](../../plans/p3-w04-per-cpu-runtime.md).  
**Owner/change context:** P3-W04 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P3-W04. It defines the per-CPU
data area and its slots, the independent per-CPU runtime stack, the
CPU-local access mechanism that replaces any global "current CPU" notion,
and the installation sequence that gives each online CPU a private,
identity-correct local environment. It deliberately does **not** define the
*contents* of the notification-reception slot
([P3-W07](../p3-w07-cross-cpu-notification/README.md)), the TLB-request
reception slot ([P3-W08](../p3-w08-tlb-shootdown-transport/README.md)), the
telemetry counter catalog ([P3-W11](../p3-w11-smp-observability/README.md)),
or any scheduler/current-vCPU field (P4/P7 — only reserved, opaque
capacity). It consumes the lifecycle registry
([P3-W03](../p3-w03-physical-cpu-lifecycle/README.md)) at install time and
hands its readiness signal to the boot rendezvous
([P3-W05](../p3-w05-smp-boot-synchronization/README.md)).

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step:

| Supporting file | Load it for |
|---|---|
| [01-scope-and-foundations.md](01-scope-and-foundations.md) | goal-to-baseline ledger, scope classification, resolved decisions |
| [02-architecture-and-state.md](02-architecture-and-state.md) | per-CPU area model, slot ownership, CPU-local access model, concurrency rules |
| [03-code-contracts-percpu-area.md](03-code-contracts-percpu-area.md) | area header, slot contracts, allocation and installation contracts |
| [04-code-contracts-cpu-local-access.md](04-code-contracts-cpu-local-access.md) | CPU-local handle, current() accessor, validation and iterator contracts |
| [05-implementation-workflow.md](05-implementation-workflow.md) | ordered implementation steps |
| [06-validation-and-handoff.md](06-validation-and-handoff.md) | validation matrix, failure model, handoff checklist |

Before editing it must also follow the Coding Guidelines preflight. This
document is a proposed design; it contains no implementation or validation
claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P3 task book → P3-W04 plan → this
design → Coding Guidelines. Binding constraints:

- The ADR object model gives every pCPU per-CPU state and local resources
  (ADR-015 context: "all pCPUs own per-CPU state"); ADR-014 permits the
  dynamic allocation such state requires, through the P2 allocator
  contracts ([P2-W04](../../../p2/plans/p2-w04-physical-page-allocation.md),
  [P2-W05](../../../p2/plans/p2-w05-dynamic-small-allocation.md)) — assumed
  contracts with failure boundaries in
  [01](01-scope-and-foundations.md) §1.2.
- The P1 exception baseline
  ([P1-W05](../../../p1/plans/p1-w05-el2-exception-entry-baseline.md)) owns
  vector and entry mechanics; W04 provides per-CPU *storage* that
  [P3-W09](../p3-w09-cpu-local-exception-interrupt/README.md) later fills
  with per-CPU exception/interrupt diagnostic state. W04 must not redefine
  entry behavior.
- The task book and the plan prohibit global arrays with implicit sharing
  and any implicit global current-CPU/context contract; the plan requires
  reserved capacity for later scheduler/current-vCPU needs *without
  defining them*.
- ADR-048: the telemetry basis is a designed interface; W04 provides the
  per-CPU counter storage, the catalog is W11's.

Classification:

- **Required** for W04 closure: the per-CPU data area (header + typed
  slots + reserved regions), one independent runtime stack per CPU, the
  CPU-local access mechanism, the install/registration sequence integrated
  with W03's gate and W05's readiness signal, the isolation diagnostics,
  and P3-V04 acceptance evidence.
- **Reserved** with recorded triggers: stack guard pages and overflow
  detection (trigger: an approved host-address-space change with the
  P1-W08 owner); a general per-CPU allocation API for arbitrary types
  (trigger: a consumer design that needs it); CPU hotplug-time area
  teardown (trigger: approved hotplug design); replacement of the W02
  provisional stacks on the boot CPU's own timeline (W04 defines the
  runtime stack and the transfer point; the boot CPU may keep its
  boot stack for P3 — recorded decision 6).
- **Out of Scope:** the notification primitive and its slot contents
  (W07), TLB transport semantics (W08), exception/interrupt enablement
  and vector content (P1/P3-W09), telemetry catalog and rates (W11),
  boot-phase ordering (W05), lifecycle transitions (W03 — W04 calls the
  gate only), scheduler and current-vCPU semantics (P7/P4+), guest state
  (P4+), global array shortcuts with implicit sharing (prohibited).

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Independent stack per online CPU | [area/stack contracts](03-code-contracts-percpu-area.md) §2, §4 | P3-V04 (W04-DV03) |
| Logical identity locally available | [area header](03-code-contracts-percpu-area.md) §3, [access contract](04-code-contracts-cpu-local-access.md) §2 | P3-V04 (W04-DV02) |
| Runtime/exception/interrupt-local state foundations | [slot contracts](03-code-contracts-percpu-area.md) §5.3 | P3-V04 (W04-DV02); per-CPU correctness is [P3-W09](../p3-w09-cpu-local-exception-interrupt/README.md) |
| Notification and TLB reception state (reserved, owned downstream) | [slot contracts](03-code-contracts-percpu-area.md) §5.1–§5.2 | W04 closure review (W04-DV01) |
| Telemetry basis | [slot contracts](03-code-contracts-percpu-area.md) §5.4 | W04 closure review (W04-DV01) |
| Reserved capacity for later scheduler/current-vCPU needs, undefined | [slot contracts](03-code-contracts-percpu-area.md) §5.5 | W04 closure review (W04-DV01) |
| Explicit boundary from global shared state; no implicit global current-CPU | [access contract](04-code-contracts-cpu-local-access.md) §1, §4; [architecture](02-architecture-and-state.md) §5 | P3-V04 (W04-DV04) |
| Isolation and CPU-identity evidence | [workflow](05-implementation-workflow.md) steps 5–6; matrix in [validation](06-validation-and-handoff.md) | P3-V04 (W04-DV03); matrix execution is [P3-W13](../p3-w13-qemu-smp-regression/README.md) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p3-implementation-designs`
at `4e631ee`): P0 documentation scaffold only — no workspace, no sources,
no runtime state. W01–W03 and W05–W15 designs are being prepared in
parallel on this branch; this design references them by path and P3-Wxx ID
and treats W02/W03 as upstream contracts, W05/W07/W08/W09/W11 as downstream
consumers.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Every online CPU has independent execution foundations | Nothing per-CPU exists; W02 provides only provisional entry scaffolding | One runtime stack per CPU plus the per-CPU data area, allocated before secondary release, ownership transferred from W02's provisional environment on success | "Independent execution" cannot mean sharing the boot stack or a global structure | W04 (this design); W02 transfer contract; P2-W04 pages | W04-DV03 (distinct-stack evidence) |
| Logical identity is locally available and correct | Identity exists only in the frozen topology (W01) | Identity fields in the area header installed from `TopologyInputs` after the W03 gate confirms the CPU | A CPU must be able to answer "who am I" without a global lookup that presumes which CPU is asking | W04; W01 types; W03 gate | W04-DV02 |
| Runtime/exception/interrupt-local state foundations | None | Typed reserved slots in the area with documented downstream owners | Slots without owners become dumping grounds; owners without slots invent global state — the prohibition the plan enforces | W04 slot shapes; W07/W08/W09/W11 contents | W04-DV01 ownership review |
| No implicit global current-CPU/context remains | No accessor exists at all | The CPU-local access mechanism (register-based) and the prohibition rule stated as a review check | Any `static current_cpu` would be wrong under SMP from the first secondary | W04 (mechanism is stage-local freedom, decision 3) | W04-DV04 review + tests |
| P4-reserved storage recorded without asserting VM/vCPU | Nothing reserved | An opaque reserved region with a size rationale and a not-a-vCPU statement | Reserving capacity is required by the plan; defining vCPU fields is P4's design | W04 (capacity only); P4 defines contents via W14 | W04-DV01 |
| Isolation and CPU-identity acceptance evidence | No QEMU harness (W13 is a plan) | Isolation diagnostics (per-CPU identity + distinct stack/area addresses in boot output) exercisable per declared count | P3-V04 requires observable distinctness, not just correct code | W04 output; W13 matrix | W04-DV03 |

No ledger row requires fixing crate names or downstream slot semantics, so
no new decision blocker is outstanding here.

## Resolved design decisions and their authority

1. **One page-aligned `PerCpuArea` per CPU, boot-CPU-allocated.** All areas
   (and stacks) for all candidates are allocated from the P2 page
   allocator during global initialization, before any secondary is
   released. Rationale: keeps every allocation single-threaded (the P2
   allocator's SMP-safety is exactly what [P3-W10](../p3-w10-smp-safety-audit/README.md)
   will audit later — W04 must not depend on it), and guarantees an area
   exists the moment a CPU passes the W03 gate.
2. **CPU-local access via `TPIDR_EL2`.** The per-CPU area's address is
   held in `TPIDR_EL2`, an EL2 software register the architecture defines
   for exactly this habitable purpose; `current()` is a read of that
   register. Rationale: atomic by construction per CPU, no global lookup,
   no bare `static mut`, and it removes the "which CPU am I" paradox —
   any lookup-by-MPIDR scheme must read a register anyway and then search
   a global table, which is both slower and a soft global-current-CPU
   assumption. Prerequisite-compatibility check: if the approved P1
   design reserves `TPIDR_EL2` for another purpose, that is an
   Architecture Change Request to resolve with the P1 owner — W04 does
   not silently pick a different register.
3. **Slot ownership is designed, not incidental.** Every typed slot names
   its downstream owner (W07 notification reception, W08 TLB reception,
   W09 exception/interrupt local state, W11 counter block); W04 fixes
   offsets, alignment, and size; owners define contents and protocols.
   The reserved region for later scheduler/current-vCPU needs is opaque,
   explicitly not typed, and not initialized by W04 beyond a fill pattern.
4. **No `Drop`, no free.** Areas and runtime stacks live for the whole
   boot; P3 has no hotplug and no teardown. This makes the `'static`-like
   lifetime of `current()` sound without runtime bookkeeping. Reclamation
   is Reserved for a hotplug design.
5. **Identity installation is gated, exactly-once.** Installation writes
   the area header, the CPU-local register, and the lookup table entry
   only for a CPU whose W03 eligibility is `EligibleForLocalInstall`
   (every CPU installs while its lifecycle state is `Initializing`,
   boot CPU included — it enters Initializing via its own boot-variant
   transition first), executed by that CPU for itself, coordinated so it
   happens exactly once (the registry's transitions provide the
   exactly-once property; W04 never re-installs).
6. **The boot CPU keeps its P1 boot stack during P3.** Replacing the boot
   stack mid-boot buys nothing at this stage and risks the P1 entry
   contract; the boot CPU still gets a full `PerCpuArea` and appears in
   every isolation invariant. Rationale: minimal change to the P1
   baseline; revisit trigger recorded (a later design may unify boot and
   runtime stacks).
7. **Runtime stack sizing is a recorded constant, not a guess.** One
   stage-local constant bounds the per-CPU runtime stack; the value and
   rationale are fixed in the implementation record with a revisit
   trigger (first real workload or the P7 scheduler design). Guard bands
   are Reserved (decision in scope classification), so sizing discipline
   is the only overflow defense at P3 — stated plainly as a limitation.

## Work breakdown and loading order

1. Read [01-scope-and-foundations.md](01-scope-and-foundations.md) for the
   ledger, scope split, and decisions.
2. Read [02-architecture-and-state.md](02-architecture-and-state.md) for the
   area model, slot ownership, access model, and concurrency rules.
3. Implement per [05-implementation-workflow.md](05-implementation-workflow.md):
   area and allocation with
   [03](03-code-contracts-percpu-area.md) (steps 1–3), access mechanism
   with [04](04-code-contracts-cpu-local-access.md) (steps 4–5).
4. Record implementation decisions in
   `../p3-w04-per-cpu-runtime-record.md` and evidence in
   `../../verification/p3-w04-per-cpu-runtime-verification.md` only when
   the work is performed. Validation conditions and the handoff checklist
   are in [06-validation-and-handoff.md](06-validation-and-handoff.md).

## Explicitly excluded interfaces

No lifecycle transition (W03's operations only), no rendezvous protocol
(W05's), no lock type or atomic-policy framework (W06), no notification
send/receive semantics (W07), no TLB request format (W08), no exception
vector or trap-policy change (P1/P3-W09), no telemetry event catalog
(W11), and no scheduler or vCPU field is designed or authorized by W04.
In particular the reserved region must not gain a type, an accessor, or a
"current vCPU" reader at P3 — any such addition is a scope violation and a
P4-stage intrusion to stop at review. A generic `PerCpu<T>` allocation
framework is likewise excluded; if a P3 consumer needs one, that is a
design change for this package, not a local utility.

## Downstream handoff

- **W05** receives the install-completion signal point: the per-CPU
  readiness signal (W05's gate API) is called as the last step of W04's
  install sequence, giving the rendezvous its "local initialization
  complete" definition.
- **W06** receives the boundary statement that W04 areas are
  per-CPU-private and that any cross-CPU access W06/W07 designs must pass
  through explicit, documented shared surfaces (the lookup table for
  diagnostics, or W07's own mailbox protocol).
- **W07** receives the notification-reception slot (offset, size,
  alignment, cache-line placement) as reserved storage; W07 defines its
  contents, protocol, and ordering.
- **W08** receives the TLB-request reception slot under the same terms.
- **W09** receives the exception/interrupt local-state slot and the
  guarantee that `current()` works from the first instruction after
  installation, which is the attribution foundation.
- **W10/W11/W12/W13** receive the isolation invariants and the per-CPU
  counter block base as audit, telemetry, stress, and regression surfaces.
- **W14/P4** receive the recorded reserved capacity and the explicit
  statement that its contents are undefined at P3; P4 designs current-vCPU
  storage against this reservation without redefining the area layout
  (layout changes are W04 design changes).
