# P8-W20 Closure Artifact Contract

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W20 detailed design](README.md).

## 1. Logical artifact groups

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Factual machine specification | published under `docs/stages/p8/implementation/` per the W02-governed location, only after approval and evidence | approved machine-contract facts (W02 route); implementation records | the factual `rusthv-arm-virt-v1` contract; it does not select values (W02 route) or define Virtio |
| Factual Linux boot specification | `docs/stages/p8/implementation/` (same publication rule) | approved boot-contract facts (W03 route); fixture and boot evidence | factual boot inputs/lifecycle; it does not approve loaders or addresses |
| Factual compatibility policy | `docs/stages/p8/implementation/` (same publication rule) | approved compatibility route (W14) and drift-test evidence | the v1 compatibility and escalation policy; it does not perform drift tests |
| P8 validation report and evidence index | `docs/stages/p8/verification/` | per-package verification records W01–W19 | indexed evidence with statuses; it re-runs and re-judges nothing |
| Limitations and unresolved-decision register | `docs/stages/p8/verification/` (referenced from the validation report) | owning packages' recorded limits and escalations | the consolidated limit/conflict list; it resolves nothing |
| Unsafe/dependency delta record | `docs/stages/p8/verification/` (referenced from the validation report) | per-package unsafe inventories and dependency changes under P0 governance | the stage-level delta; it audits on top of, not instead of, package reviews |
| P9 consumer statement | `docs/stages/p8/verification/` (referenced from the validation report) | evidenced facts only ([02](02-closure-workflow-and-handoff.md) §4) | the bounded handoff; it contains no Virtio content |
| Closure review record | `../../verification/p8-w20-documentation-closure-handoff-verification.md` (created when review runs) | the gate matrix evaluation | gate statuses and the closure decision *inputs*; it does not itself declare P8 closed |

The artifact named in the second column is the sole authoritative home for
its content; other documents link, never duplicate. The stage implementation
index remains owned by the stage index itself — W20 adds its design rows
through the coordinator's process, not by editing plans or the task book.

## 2. Publication rule

A factual document from §1 is published only when **both** hold:

1. its governing decision exists (machine values approved through the W02
   route; boot-contract facts approved through W03; compatibility rules
   approved through W14); and
2. the implementation and verification records it states actually exist and
   are linked.

Until both hold, the artifact group's row in the gate matrix is
`missing` or `blocked`, and the document must not exist in draft form that
could be mistaken for factual content. Drafts live only inside the approved
design documents (like this one), which carry the status header that
distinguishes them.

## 3. Document-kind discipline

Every closure artifact states, in its status header, exactly one kind, and
the kinds must never merge:

```text
plan            bounded work statement (docs/stages/p8/plans/) — intent, not state
detailed design proposed/approved design (docs/stages/p8/implementation/<slug>/) —
                what may be built; never evidence
implementation  factual records (implementation/<slug>-record.md, factual
                specifications) — what was built and decided, with evidence links
verification    evidence and review outputs (verification/) — what ran and passed,
                failed, blocked, or was not run
```

A closure artifact that mixes kinds (a "report" that restates plan goals as
achievements, or a specification citing planned evidence) fails review. This
is the mechanical enforcement of P8-V26's "planned evidence is not treated as
evidence".

## 4. Gate matrix

The evidence index's core is this matrix, evaluated once per closure review.
Gate statuses: `evidenced` (linked real verification material exists),
`missing` (no record), `blocked` (recorded blocker or open `ADR Required` /
`Architecture Change Request`), `failed` (evidence of failure exists),
`superseded` (superseding evidence linked). Groups follow the task book §5
requirement-to-validation mapping; every P8-V01–V26 ID appears exactly once.

| Gate group | Validation IDs | Evidence location (owner) | Closure requirement inherited from task book §7 |
|---|---|---|---|
| Entry reconciliation | P8-V01 | W01 verification record | every P0–P7 input linked or explicitly blocked |
| Machine governance | P8-V02, P8-V03 | W02 verification record | complete decision route; no Host fact in the contract boundary; concrete values still routed per task book §8 |
| Boot contract | P8-V04 | W03 verification record | boot facts testable without frozen implementation choices |
| DTB contract | P8-V05, P8-V06 | W04 verification record | Guest DTB consistent with approved facts; no host leakage |
| CPU compatibility | P8-V07, P8-V08 | W05 verification record | declared paths operate under classification; controlled rejection works |
| PSCI lifecycle | P8-V09 | W06 verification record | standard PSCI path for secondary/off/system-off |
| vGIC behavior | P8-V10 | W07 verification record | boot/secondary IRQs, timer IRQ, SGI, SPI, masking under declared stress |
| Timer integration | P8-V11 | W08 verification record | counter, timer IRQ, monotonicity, preemption, WFI wakeup semantics |
| Console and single-CPU boot | P8-V12, P8-V13 | W09 verification record | console containment; one-vCPU Linux to interactive userspace (`start_kernel` insufficient) |
| SMP boot and stability | P8-V14, P8-V15 | W10 verification record | 2/4-vCPU enumeration, PSCI secondary start, per-CPU paths, stability workloads |
| Scheduler integration | P8-V16 | W11 verification record | 1:1 and declared M:N progress with preserved semantics; no fairness claim |
| Memory model | P8-V17 | W12 verification record | RAM classes, reserved/MMIO boundaries, Host isolation, actionable Stage-2 diagnostics |
| Fault diagnostics | P8-V18 | W13 verification record | listed fault classes with VM/vCPU context and recent trace; no full crash-dump claim |
| ABI compatibility route | P8-V19 | W14 verification record | drift detection on the same approved configuration |
| Reproducible fixture | P8-V20 | W15 verification record | versioned or reproducibly generated fixture definition |
| Automated regression | P8-V21, P8-V22 | [W16 verification record](../../verification/p8-w16-automated-linux-regression-verification.md) | determinate matrix and repeated-boot conditions |
| Performance baseline | P8-V23 | [W17 verification record](../../verification/p8-w17-linux-performance-baseline-verification.md) | baseline record with environment and limits; no KPI claim |
| Security and isolation | P8-V24 | [W18 verification record](../../verification/p8-w18-security-isolation-regression-verification.md) | declared containment evidence per scenario; Guest panic ≠ Hypervisor panic |
| Dual-track regression | P8-V25 | [W19 verification record](../../verification/p8-w19-validation-guest-dual-track-verification.md) | Validation Guest mechanism suite retained alongside Linux |
| Closure and handoff | P8-V26 | W20 verification record (this package) | the review itself, evaluated last |

Rules: a gate is `evidenced` only with a working relative link to real
verification content; links to plans or designs never qualify. All gates
`evidenced` (or `superseded` with linked replacement) is the *condition
under which* P8 closure can be proposed; the closure decision itself belongs
to the project owners via the integration workflow, not to W20.

## 5. Registers and delta records

### 5.1 Limitations and unresolved-decision register

One row per recorded limitation or unresolved decision, each traceable:

```text
id            P8-LIMIT-nn / P8-OPEN-nn
statement     what is limited or undecided, stated factually
source        the owning package record or review that recorded it
class         limitation | Specification Investigation | ADR Required |
              Architecture Change Request
effect        which gate(s) it caps (e.g., "P8-V24 scoped to single-VM,
              QEMU-only") or blocks
resolution    owning route (later stage, ADR process), or "none assigned"
```

Known register seeds (from the plans and task book; the register verifies and
extends, never shrinks, them): the concrete v1 IPA map / GIC/PCI windows /
virtio slot count item (`ADR Required`, task book §8 row 1, W02 route);
QEMU-vs-hardware observation limits (task book §8 Platform Investigation);
P8-V24's single-VM, QEMU-scoped containment boundary (W18); the non-KPI
baseline boundary (W17); any W19 lost-coverage blocks.

### 5.2 Unsafe/dependency delta record

One row per stage-level delta, assembled from package records under the P0
unsafe and dependency governance:

```text
area          unsafe inventory delta | new dependency | dependency removal |
              public API/ABI surface change | toolchain/policy change
packages      which work packages introduced it
record        link to the owning package's implementation record
review        link to the review that accepted it
```

The record summarizes; it does not re-approve. A delta with no linked
approving review is a closure-review finding (`missing`), not a formatting
problem to fix silently.
