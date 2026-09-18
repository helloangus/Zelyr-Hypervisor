# P0-W09 QEMU Automation Entry Baseline — Detailed Implementation Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Scope:** The single documented QEMU virt runner entry contract —
responsibility boundary, parameter-carrying model, serial capture, timeout,
exit-status taxonomy, and evidence requirements — required by
[P0-W09](../../plans/p0-w09-qemu-automation-entry-baseline.md).  
**Owner/change context:** P0-W09 implementation handoff.  
**Supersedes:** None.

## Purpose and use

This is the implementation-level design for P0-W09. It converts the bounded
work-package plan into one normative runner-entry contract document plus
minimal discovery wiring. In P0 the entry is an **interface-only
placeholder**: the contract fixes the one invocation surface future test
packages implement against, and nothing is executed. It deliberately does
**not** deliver a runner program or script, does **not** run QEMU, does
**not** require an EL2 log, a guest boot, or any passing run (plan
out-of-scope), and does **not** define CI wiring or check classification
(P0-W20; its design is being prepared in parallel) or quality gates
([P0-W07](../p0-w07-development-quality-gates/README.md)). The first concrete
runner implementation belongs to the P1 package that first executes QEMU (per
the P1 plan index, [P1-W10 QEMU boot
regression](../../../p1/plans/p1-w10-qemu-boot-regression.md), whose plan
names the W09/W01 canonical path and P0 runner boundary as its basis).

An implementing agent starts with this file and the mandatory
[Coding Guidelines](../../../../development/coding-guidelines.md). It then
loads only the linked supporting file needed for its assigned step. Before
editing it must also follow the Coding Guidelines preflight, including the
repository `AGENTS.md`, documentation index, [ADR
baseline](../../../../adr/adr-000-architecture-baseline-v0.1.md), P0 task
book, and P0-W09 plan. This document is a proposed design; it contains no
implementation or validation claim.

## Authority, constraints, and scope classification

The governing order is ADR baseline → P0 task book → P0-W09 plan → this
design → Coding Guidelines. In particular:

- The task-book outcome for W09 is: **one reusable QEMU virt runner entry
  point** that can later carry test parameters and evidence, validated by
  P0-V13 — "exactly one documented runner interface is available for future
  P1 smoke evidence". P0-V13 is a documentary review; it requires the
  interface, not an execution.
- ADR-003 makes QEMU virt the reference platform for deterministic testing;
  the contract assumes a QEMU system emulator supporting the virt machine and
  must not name a host board.
- The plan requires: the runner's single responsibility and usage boundary
  (no drifting QEMU command lines), the reserved parameter-carrying surface
  (boot smoke, SMP, memory, GIC, SMMU, guest image, regression), unified
  handling of serial capture, timeout, exit status, and failure evidence, and
  verification that the entry is inspectable and marked as a P0
  runner/placeholder rather than an EL2 test.
- W03 (phase-A package, expected complete; not a listed contract
  prerequisite) supplies the AArch64 build surface whose artifact a future
  runner invocation would boot; the contract references it without defining
  it. W02 supplies the pinned toolchain constraint the contract records for
  future implementers.

Classification: the runner-entry contract — single-entry rule, responsibility
boundary, invocation grammar, parameter-class model, runtime behavior
(serial, timeout, termination), exit-status taxonomy, evidence content set,
versioning, and the P0 placeholder marking — is **Required** for W09 closure.
Concrete QEMU flag recipes, the runner program's language and location, actual
parameter values, timeout defaults per test class, guest images, and any EL2
or guest expectation are **Reserved** to the implementing designs that first
need them (P1 and later). CI wiring, gate classification, hypervisor code,
and any P0 execution are **Out of Scope**.

| Plan requirement | Detailed-design location | Acceptance |
|---|---|---|
| Runner's single responsibility and usage boundary | [Runner entry contract](01-runner-entry-contract.md) §2–§3 | W09-DV01 |
| Reserved parameter-carrying model | [Runner entry contract](01-runner-entry-contract.md) §4 | W09-DV03 walkthrough |
| Unified serial capture, timeout, exit status, evidence requirements | [Runner entry contract](01-runner-entry-contract.md) §5–§7 | W09-DV01/DV03 |
| Entry verifiable by inspection; P0 placeholder marking | [Runner entry contract](01-runner-entry-contract.md) §8; [workflow](02-implementation-and-validation.md) steps 3–4 | P0-V13 (W09-DV02–DV04) |
| Single-source: no competing automation entries | [Runner entry contract](01-runner-entry-contract.md) §2; [workflow](02-implementation-and-validation.md) step 5 | P0-V13 (W09-DV05) |
| Document discoverability and coherence | [workflow](02-implementation-and-validation.md) step 4 | P0-V09 (W09-DV06) |
| Downstream handoff to W19/W20 and P1 | [workflow](02-implementation-and-validation.md) handoff checklist | W09 closure (W09-DV07) |

## Current-state findings and goal-to-baseline ledger

Observed tracked state (2026-09-18, branch `docs/p0-implementation-designs` at
`4e631ee`): no QEMU document, runner interface, or automation script exists
anywhere; `scripts/` contains only a `.gitkeep` marker; `.github/workflows/`
contains only a `.gitkeep` marker; `docs/testing/README.md` is a three-line
informative placeholder. There is no workspace and no AArch64 build surface
yet (W02's toolchain design and W03's build-target design are proposed or in
preparation). W01 is completed with recorded verification. Each ledger row
below states the missing foundation the plan outcome necessarily requires.

| Plan outcome / acceptance wording | Current observable state | Required foundation deliverable | Why it follows from the outcome | Authority / owner | Evidence needed |
|---|---|---|---|---|---|
| Exactly one reusable QEMU virt runner entry | No runner interface documented anywhere | Runner-entry contract fixing the single invocation surface and the single-entry rule | "Reusable entry" means one authoritative interface all future callers share; without it, QEMU commands drift across documents and CI | W09 (this design) | W09-DV01/DV05 review |
| Entry can later carry test parameters | Absent | Named parameter-class model (boot smoke, SMP, memory, GIC, SMMU, guest image, regression) with a versioned extension rule | Parameter surface must be planned as a contract, or every future test package redefines the CLI | W09 contract; values from future designs | W09-DV03 walkthrough |
| Unified serial capture, timeout, exit status, evidence | Absent | Contract sections fixing capture completeness, mandatory timeout mechanics, the exit-status taxonomy, and the evidence content set | Failure evidence and verdicts must be comparable across all future QEMU tests | W09 contract; per-test defaults from future designs | W09-DV01/DV03 |
| Entry inspectable and marked P0 placeholder | Nothing to inspect; no EL2/guest expectation may be claimed | Documentary validation (grammar walkthrough) plus an explicit placeholder-status section | P0-V13 is a review; marking prevents the entry being mistaken for a passing EL2 test | W09 | W09-DV02–DV04 |
| Consumable by W19/W20/P1 | No workflow or CI exists | Contract fields W20 can classify as non-P0/future and W19 can document as the placeholder boundary | Consumers must bind to a stable interface, not to an implementation | W09 contract; W19/W20 wiring | W09-DV07 |

No row requires inventing a script, target, dependency, or QEMU flag recipe,
so no decision blocker is outstanding for the design itself. The contract's
references to W02/W03 surfaces are implementation-time assumptions with the
failure boundary stated in the workflow §1.

## Resolved design decisions and their authority

1. **Contract home:** `docs/testing/qemu-runner-entry.md` is the sole
   normative home of the runner interface, matching the directory's declared
   role (test strategy and environments). [P0-W05](../p0-w05-documentation-baseline/README.md)
   may re-home it; semantic ownership stays with the document.
2. **Interface-only placeholder in P0:** no runner program is committed and
   nothing is invoked. The plan's own verification wording ("invoked **or
   inspected**", "runner/placeholder rather than EL2 test") and P0-V13's
   review character make documentary delivery the honest P0 closure; the
   first concrete implementation is P1-W10's, whose plan already names the
   P0 runner boundary as its basis.
3. **Single-entry rule scope:** the rule governs *automated* QEMU execution —
   any repeatable invocation whose output is used as evidence goes through
   the entry. Human-investigation QEMU commands remain permissible and are
   not automation entries; documents must not promote them into scripts or
   CI. This keeps P1's boot-recipe documentation (per the P1 task book)
   distinct from automation.
4. **Exit-status taxonomy is the stable machine-facing surface:** six fixed
   outcome classes separating runner health from target-under-test outcome
   ([contract](01-runner-entry-contract.md) §6), versioned with the entry
   contract so future consumers can branch on it safely.
5. **Parameter model:** named parameter classes reserved per the plan (boot
   smoke, SMP, memory, GIC, SMMU, guest image, regression); the contract
   fixes the carrying mechanism and extension/versioning rule, never a
   default value or a concrete setting — those belong to the first design
   that defines a test needing them.
6. **Evidence content set vs naming:** the contract fixes what every run must
   record (invocation metadata, complete serial capture, timeout and exit
   records); file/artifact naming and identity follow the artifact-naming
   baseline delivered by [P0-W17](../p0-w17-artifact-naming-baseline/README.md)
   — an assumption with a failure boundary, since W17 is not a W09
   prerequisite ([workflow](02-implementation-and-validation.md) §1).
7. **QEMU identity:** every run records the emulator version and machine
   model; a repository-wide QEMU version pin is **Reserved** — pinning policy
   is a reproducibility decision for [W19](../p0-w19-reproducible-development-workflow/README.md)
   (declared prerequisites) or a later design, not silently taken here.

## Work breakdown and loading order

1. Read the [runner entry contract](01-runner-entry-contract.md) for the
   artifact groups, single-entry rule, responsibility boundary, invocation
   grammar, parameter model, runtime behavior, exit taxonomy, evidence
   requirements, and placeholder marking.
2. Apply the changes in the order stated in the [implementation
   workflow](02-implementation-and-validation.md): verify prerequisite
   surfaces, author the contract, run the grammar walkthrough, wire
   discovery, run the single-source and consumability reviews, close.
3. Store actual review commands, output, environment, and result in
   `../../verification/p0-w09-qemu-automation-entry-baseline-verification.md`,
   and record decisions taken and changed artifacts in
   `../p0-w09-qemu-automation-entry-baseline-record.md` only when
   implementation begins. Neither this design nor a written record may claim
   W09 complete, and no artifact may claim a QEMU run.

## Design-level state and lifecycle

W09 adds no runtime state and no executable. The authoritative state is one
tracked contract document plus its discovery links. Documentary lifecycle:

```text
no QEMU automation interface
  -> runner-entry contract committed (grammar, behavior, exit taxonomy, evidence, placeholder mark)
  -> discovery links (docs/README routing row, stage index) committed
  -> grammar walkthrough evidenced; single-source review clean
  -> W19 documents the placeholder boundary; W20 classifies the entry as future/non-P0
  -> first implementing design (expected P1-W10) realizes the runner against the contract
     and may extend parameters only through the contract's versioning rule
  -> later semantic changes mutate the contract through its thresholds
```

The contract document is the owner of every runner-interface statement. A
future runner that deviates from the recorded grammar, taxonomy, or evidence
set is a contract violation to be raised, not a local variant.

## Explicitly excluded interfaces

No script, program, binary, CI workflow, crate, Rust API, target triple, or
dependency is designed or authorized by W09. The runner CLI grammar, exit
taxonomy, and evidence content set are documentary contracts — outlined in
the contract file as usage sketches and tables, never as committed runnable
code. Concrete QEMU command lines, machine options, and images are owned by
the implementing design (expected P1-W10) subject to this contract's
requirements. Committing a runner script under W09 is a scope conflict and
must be stopped at review.

## Downstream handoff

- **W19** receives the entry as the "QEMU entry" step of the clone-to-build
  path, including the explicit P0 placeholder boundary its plan requires it
  to document, and the requirement to document how a contributor obtains a
  QEMU emulator (declared prerequisites; W09 records versions per run, it
  does not define procurement).
- **W20** receives the entry as a named future check: non-required in P0,
  classified so that its absence is never reported as verified, with the
  promotion path through W07's future-class rule once a runner exists.
- **P1 (expected P1-W10)** receives the invocation grammar, exit taxonomy,
  serial-capture and timeout mechanics, and evidence content set as the
  contract its reference invocation, verdict logic, and 100-cycle evidence
  set must implement; extensions go through the entry contract's versioning
  rule, and semantic deviations are conflicts to raise, not fork.
- **W17** receives the evidence content set as a consumer of its naming
  baseline; until W17 delivers, the placeholder naming rule in the contract
  §7 applies and is superseded by W17's.
- **W07** receives the confirmation that the runner entry is not a gate;
  QEMU-class checks reach the gate register only through promotion.
- **W05** may re-home the contract document under its documentation taxonomy.
