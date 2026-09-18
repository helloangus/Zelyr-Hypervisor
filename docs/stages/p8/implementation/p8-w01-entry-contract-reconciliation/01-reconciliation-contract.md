# P8-W01 Reconciliation Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P8-W01 detailed design](README.md).

## 1. Logical artifact groups and ownership

W01 is review and documentation work, so its logical modules are authoritative
artifact groups, not Rust modules.

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Entry reconciliation record | `docs/stages/p8/implementation/p8-w01-entry-contract-reconciliation-record.md` (created when work starts) | this design, ADR, task book, plan index, P0–P7 handoff plans and any existing records | the sole per-input disposition list, constraint register, conflict register, and consumer map; it does not restate upstream contracts, resolve conflicts, or fix any machine value |
| Review evidence | `docs/stages/p8/verification/p8-w01-entry-contract-reconciliation-verification.md` (created when evidence exists) | actual review walkthrough | run/not-run evidence per the validation matrix; not part of the design |
| Stage status entry | one truthful status row in `docs/stages/p8/implementation/README.md`, per that index's own conventions | design status | discoverability; added without claiming completion; coordinated with parallel W06–W20 work so rows do not conflict |

No other artifact is created or modified. Upstream stage documents, the task
book, plans, and the ADR are read-only inputs to W01; a needed change there is
a routed conflict, never a local edit.

## 2. Input inventory

Each row names the reconciliation unit: the P8-facing handoff or evidence set
that the task book §2 table turns into a P8 input. Evidence paths are stated as
the locations where evidence *would* exist; their current emptiness is a
finding to record, not a justification to skip the row.

| Stage | P8-facing reconciliation unit | Handoff plan (authority for the claimed input) | Evidence locations to inspect | Task-book §2 boundary restated for the record |
|---|---|---|---|---|
| P0 | repeatable build/test/QEMU route, unsafe/dependency governance, diagnostics, trace and documentation rules | P0-W01 record `../../../p0/implementation/p0-w01-repository-baseline-record.md`; P0-W02 design `../../../p0/implementation/p0-w02-rust-toolchain-baseline/README.md`; P0 plans W03–W22 | `docs/stages/p0/verification/` (P0-W01 record present; W02+ absent) | reproducible fixtures and evidence; no toolchain redesign |
| P1 | stable AArch64 Non-secure EL2 entry, exception capture, Host-MMU and early diagnostic boundary | [P1-W12 documentation and P2 handoff](../../../p1/plans/p1-w12-p1-documentation-handoff.md) | `docs/stages/p1/implementation/`, `docs/stages/p1/verification/` | Guest exits and failures return to existing EL2 control |
| P2 | PlatformInfo, protected/reserved memory, allocation/ownership and host-device facts | [P2-W10 P3/P4 handoff contract](../../../p2/plans/p2-w10-p3-p4-handoff-contract.md) | `docs/stages/p2/implementation/`, `docs/stages/p2/verification/` | bounded Guest backing and no host-fact leakage; no discovery redesign |
| P3 | pCPU identities, SMP synchronization, notification and TLB transport | [P3-W14 P4 SMP handoff](../../../p3/plans/p3-w14-p4-smp-handoff.md) | `docs/stages/p3/implementation/`, `docs/stages/p3/verification/` | Host-SMP substrate only; no pCPU or TLB redesign |
| P4 | Guest EL1 entry/exit, Stage-2 address space, bounded Guest memory and Validation Guest facts | [P4-W09 factual closeout and P5 handoff](../../../p4/plans/p4-w09-closeout-p5-handoff.md) | `docs/stages/p4/implementation/`, `docs/stages/p4/verification/` | Linux compatibility gap closure; no Stage-2 or VM foundation redesign |
| P5 | checked Guest input, capability/handle, controlled Guest-failure and diagnostic boundary | [P5-W10 factual closeout and P6 handoff](../../../p5/plans/p5-w10-closeout-p6-handoff.md) | `docs/stages/p5/implementation/`, `docs/stages/p5/verification/` | no management ABI, role shortcut, or authority policy design |
| P6 | timer, GICv3, vIRQ, maintenance, SGI and interrupt telemetry facts | [P6-W13 telemetry, regression, and handoff](../../../p6/plans/p6-w13-telemetry-regression-handoff.md) | `docs/stages/p6/implementation/`, `docs/stages/p6/verification/` | Linux-facing compatibility validation; no foundational GIC/timer/LR redesign |
| P7 | scheduler-controlled execution, lifecycle, preemption, blocking/wakeup, placement, SMP and accounting facts | [P7-W14 documentation and P8 handoff](../../../p7/plans/p7-w14-documentation-p8-handoff.md) | `docs/stages/p7/implementation/`, `docs/stages/p7/verification/` | prove Linux integration; no scheduler algorithm or policy redesign |

The record may add sub-rows (for example, per-package facts inside a stage)
but may not drop a row or merge stages, because the task book assigns the
boundaries per stage.

## 3. Reconciliation classes

### 3.1 Class definitions

Each input row receives exactly one disposition:

| Class | Decision rule | Consequence for P8 designs | Consequence for P8 implementation |
|---|---|---|---|
| **Evidenced** | A verification or implementation record exists at the inspected location and its claims cover the required input; the record's own limits are copied into the row | may be relied on as stated, within the recorded limits | may be relied on as stated, within the recorded limits |
| **Planned-only** | An approved plan names the input, but no evidence record exists | may be assumed only as an explicit assumed contract, with the failure boundary stated in the design (if the predecessor delivers differently, the P8 design step stops and re-routes) | must not be relied on; the dependent implementation step is blocked until the class changes |
| **Blocked** | The input is absent or its absence is recorded upstream (no plan, or a recorded blocker) | the dependent P8 design must state the gap and its owner | blocked |
| **Conflict** | The input contradicts the ADR, task book, another input, or this stage's constraints | the affected design step stops; no local resolution | blocked |

### 3.2 Class-change rule

A disposition changes only when the underlying evidence changes, and the
record is updated in the same change as the P8 work that starts relying on the
new class. Re-running W01's review is the mechanism; silently upgrading a row
inside a later package is a review failure.

### 3.3 Conflict routing

Conflict rows additionally name the owning route:

- **Machine-model inputs** — concrete IPA values, GIC/PCI windows, virtio slot
  count, freeze authority, CPU feature baseline, PSCI subset, timer/console
  details — route to the [P8-W02](../p8-w02-machine-contract-governance/README.md)
  decision route, keeping the task book §8 labels
  (`ADR Required / Specification Investigation` where so classified).
- **Contradictory architecture, security, or upstream-scope inputs** route to
  an `Architecture Change Request` / `ADR Required` record per the task book
  §8 handling and repository `AGENTS.md` issue rules. W01 registers the
  conflict and stops; it never repairs upstream scope.
- **Missing inputs that a downstream P8 package cannot proceed without** are
  recorded as Blocked with the owning stage named; they are not recreated
  inside P8.

## 4. Constraint register

The record carries one register of cross-cutting constraints, each entry with
a normative citation. The required entries (the plan's work-sequence item 4)
are:

| Constraint | Normative basis | Bound it places on W02–W20 |
|---|---|---|
| Guest input is untrusted | ADR-007, ADR §19; task book §2 (P5 row) | every Linux-provided address, length, register state, DTB, and bootargs path is validated; a Linux-caused fault is a recoverable VM-facing error, never a hypervisor panic (Linux is a guest, not the host) |
| Capability/handle authority | ADR-013, ADR-051; task book §2 (P5 row) | no P8 mechanism substitutes role, VM ID, or "Linux is special" for a capability check |
| Platform independence | ADR-043, ADR-044, ADR-052; task book §1 | no QEMU or RK3566 fact enters Guest-visible contracts or Core code paths; QEMU `virt` is a reference test environment only |
| Telemetry as designed interface | ADR-048; task book §2 (P0, P6 rows) | fault, exit, and diagnostic paths specify structured telemetry, not debug prints |
| Versioned machine ABI | ADR-024, ADR-040; task book §2 Required | every Guest-visible contract carries an identity and version and explicit compatibility rules |
| Layering | ADR-041; AGENTS.md scope rules | P8 work consumes P3 pCPU and P4 vCPU contracts; it does not redesign EL2, Stage-2, GIC, timer, or scheduler foundations |

The register may add entries only with a citation; an uncited constraint is a
review failure.

## 5. Record artifact contract

The record is a normative implementation-stage document with the status header
required by `docs/README.md` (status, scope, version `v0.1`, owner/change
context, supersedes: none) and exactly the following sections:

1. **Scope and authority** — one paragraph citing the task book, plan, and
   this design; states that the record is a review, not runtime evidence.
2. **Per-stage reconciliation table** — the §2 inventory with, per row: the
   required input (task-book wording), inspected evidence location, observed
   state, disposition class (§3.1), limits (copied from evidence where
   present), P8 consumers (plan-index rows), and conflict route if
   **Conflict**.
3. **Constraint register** — per §4.
4. **Conflict and block register** — every **Conflict** and **Blocked** row
   restated with its owner and the decision route; open items must be
   answerable ("who decides, where") or the row is incomplete.
5. **Design-time versus implementation-time reliance summary** — which inputs
   P8 designs may assume (with failure boundaries) and which are closed for
   implementation until re-reviewed.
6. **Evidence index** — links to the verification record and to every upstream
   record cited; all links must resolve.

Required content is stated per section; additional informative detail is
allowed but must not contradict a required statement. The record never
asserts that an upstream stage completed, never freezes a machine value, and
never resolves a **Conflict** row.

## 6. Non-responsibilities

The record does not: define the machine contract or its categories (W02);
state Linux boot facts (W03); enumerate DTB facts (W04); classify Linux CPU
behavior (W05); implement or validate any mechanism; or claim any P0–P7 stage
complete. A finding that belongs to another package's scope is registered and
routed, not expanded in place.
