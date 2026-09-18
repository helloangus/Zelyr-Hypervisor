# P1-W01 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P1-W01 detailed design](README.md).

## 1. Preconditions and failure boundary

W01's contract content can be written immediately; its executable boundary
lands inside the W02-delivered entry module, so W01's implementation is
complete when the contract is materialized, the boundary contracts are handed
to W02, and the review/evidence obligations are recorded. Before changing any
file, the implementer verifies the mandatory reading (parent README), inspects
the current tree (`git ls-files`; confirm which P0 contracts and P1 designs
actually exist), and collects the P0 target/runner contracts at whatever level
they exist.

Stop and obtain direction instead of guessing when any of the following
occurs:

- a required P0 contract (target/build artifact, runner entry, build
  metadata) is absent or contradicts the canonical-path properties of
  [01-boot-contract.md](01-boot-contract.md) §2 — record the upstream defect
  and the exact dependency; do not redesign P0 inside P1;
- the reference platform's direct-kernel mechanism demonstrably cannot
  deliver a §2 property (for example, no EL2 entry at any machine setting) —
  that falsifies the canonical path itself: record an `ADR Required` issue
  (boot-method change), do not quietly weaken a check;
- closure appears to require DTB parsing, memory discovery, a console, a
  second canonical recipe, or a bootloader framework — those are Out of
  Scope (parent README); stop;
- the W09 design rejects the §5 routing refinement — record the coordination
  issue per
  [02-entry-validation-contracts.md](02-entry-validation-contracts.md) §5;
  do not edit either matrix silently.

## 2. Ordered implementation steps

### Step 1 — inspect the P0 target, image and QEMU entry contracts

Target: implementation record (`../p1-w01-reference-boot-contract-record.md`,
created in this step).

Work: for P0-W03 (target/build/artifact), P0-W09 (runner entry), and P0-W16
(build metadata), extract what exists at plan or design level: image form,
entry placement rules, runner invocation shape, capture/timeout/exit
semantics. Record each as an assumed contract with its failure boundary, and
list the contract fields of
[01-boot-contract.md](01-boot-contract.md) §7 that remain
implementation-selected.

Suggested observation: read-only `git ls-files` and reading the P0 plans and
any accepted designs; no repository change.

**Acceptance:** the record names each P0 dependency, its observed state, and
the fields deferred to implementation time.  
**Failure/blocker:** a missing or contradicting P0 contract is a recorded
upstream defect (§1); W01 proceeds only with contract content that does not
depend on it.

### Step 2 — record the supported entry state, inputs, ranges and assumptions

Target: implementation record (contract content, per
[01-boot-contract.md](01-boot-contract.md) §1 artifact table).

Work: materialize the contract content: canonical path (§2), entry-state
table (§3), boot parameters and DTB/memory treatment (§4), including the
explicit assumption register (E2, E3, E5 as assumptions; E1/E7 as checks).
State every unchecked assumption as an assumption — none may be silently
strengthened into a check.

Suggested observation: none (authoring step).

**Acceptance:** the record contains the full §2–§4 content; the
assumption/check split matches §3's table exactly.  
**Failure/blocker:** an assumption that turns out to be checkable
pre-transfer is moved into Tier A by a design change, not left as a
misclassified check.

### Step 3 — define the normal-continuation and fail-fast boundary

Target: implementation record (boundary content) and the boundary contracts
handed to W02.

Work: materialize §5–§6 (rejection classes, R1–R5, transfer guarantee) and
confirm
[02-entry-validation-contracts.md](02-entry-validation-contracts.md) §1–§4
are implementable inside the entry module W02 delivers: no stack, no
allocation, fixed literals, bounded stop. Walk the check order and the
reporter's failure modes.

**Acceptance:** both rejection classes are implementable pre-transfer with
R1–R5 intact; the transfer guarantee's recording point is stated in W02
terms.  
**Failure/blocker:** a boundary semantic that cannot be implemented without
runtime establishment is a design error here — fix the design, not the
boundary.

### Step 4 — reconcile the contract with ADR layering and reserved scope

Target: implementation record (§8 content).

Work: write the layering reconciliation (reference-platform constant
placement, single-source rule, no platform-name branches), enumerate the
Reserved items touched, and re-read the task book's out-of-scope list against
every W01 artifact produced so far.

**Acceptance:** no W01 artifact contains a board-name branch, a Guest
mechanism, discovery, or a second canonical path; the Reserved list is
explicit.  
**Failure/blocker:** a scope violation is removed or the conflicting item is
labelled `ADR Required`; neither is absorbed.

### Step 5 — define acceptance evidence for normal and unsupported reference boots

Target: implementation record (§9 evidence split) and verification record
(`../../verification/p1-w01-reference-boot-contract-verification.md`, created
when evidence exists).

Work: record the evidence definitions of
[01-boot-contract.md](01-boot-contract.md) §9: which proofs W01 owns as
reviews, which are executed by W10 (normal boot) and W11 (NC1), and the
explicit `not run — deferred` entries. Perform every review that is executable
now (all of §3's matrix except the two deferred executions).

**Acceptance:** the verification record distinguishes passed reviews from
deferred executions and gives the exact deferral owners.  
**Failure/blocker:** a failed review is recorded as failed with diagnosis;
completion is not claimed around it.

### Step 6 — hand off the stable entry contract

Target: implementation record (handoff section); downstream handoff in the
parent README.

Work: confirm the handoff checklist (§5) is answerable; confirm W02, W03,
W09, W10, W11, W12 each have their named input; record any deviation from
this design.

**Acceptance:** every consumer listed in the parent README's downstream
handoff can name what it receives.  
**Failure/blocker:** an unanswerable handoff item is a closure failure, not
a downstream problem.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W01-DV01 → W01 closure | P0 prerequisite inspection | read P0-W03/W09/W16 plans and any accepted designs; `git ls-files` | every consumed contract named with observed state and failure boundary; implementation-selected fields listed | prerequisite awareness; not that P0 delivers on time |
| W01-DV02 → P1-V01 | Contract content review | read the materialized contract against [01-boot-contract.md](01-boot-contract.md) §2–§4, §6–§7 | canonical path, entry-state table, parameters/DTB/memory, transfer guarantee and recipe all present; assumptions vs checks correctly split | the contract is reviewable and complete; not that a boot satisfies it |
| W01-DV03 → P1-V02 | Boundary semantics review | walk §5 classes and R1–R5 against [02-entry-validation-contracts.md](02-entry-validation-contracts.md) §1–§3 | classes exhaustive over the tier; R1–R5 hold by construction; single-source rule intact | the boundary is explicit and non-continuing by design; not that it fires on real firmware |
| W01-DV04 → P1-V01 | Layering and reserved-scope review | search W01 artifacts for platform constants, board names, scope items; read §8 | one reference-platform constant, single-source, documented; Reserved/Out-of-Scope items untouched | ADR-041/043 compatibility of the boot path; not general platform-portability compliance (P0-W11 owns the rules) |
| W01-DV05 → P1-V02 | Transfer and routing review | read §4–§5 against the W09 design's §8 and decision 6 | guarantee wording matches W09's seam; refinement row is H4-consistent; conflict handling stated | seam compatibility as designed; not W09's acceptance of the refinement (recorded issue if rejected) |
| W01-DV06 → P1-V01/P1-V02 | Executed evidence (deferred) | W10 regression (normal boots) and W11 NC1 (unsupported environment) on the integrated path, per contract §9 | canonical boots reach validated EL2; NC1 shows the rejection line and no continuation, reproducibly | the boundary fires and holds on the reference platform; nothing about other environments or real hardware |
| W01-DV07 → W01 closure | Consumability review | read the contract as W02 (can I implement the entry against it?), W03 (is my EL precondition declared?), W10 (can I boot and match it?), W11 (is NC1 fully specified?), W12 (is the content assemblable?) | each consumer can act without inventing W01 policy | handoff readiness; not downstream completion |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. Writing the contract
without W01-DV02/DV03 does not satisfy P1-V01/P1-V02; the two executed proofs
of W01-DV06 belong to W10/W11 verification records, and until they exist
P1-V01/P1-V02 are unproven — no W01 artifact may report otherwise. No
validation here proves P1-V03 through P1-V21.

## 4. Error, security, and observability model

**Errors.** W01's only runtime failure classes are the two rejection reasons;
both are terminal by construction (R3), with no retry, fallback, or degraded
mode anywhere on the path. Contract/documentation defects fail their reviews
and are recorded as such in the verification record.

**Security.** The entry validation is the hypervisor's first trust decision:
it refuses to execute privileged runtime code outside the declared privilege
(ADR-007 posture applied to the boot path; the firmware is treated as an
untrusted input source whose delivered state is checked). The Non-secure
residual assumption is recorded, not hidden, and W03's required-fact check is
its named compensating control. The rejection reporter emits fixed literals
only and performs no initialization — it cannot become an attack surface
beyond the (already-bounded) stop.

**Observability.** The boundary's observable is exactly the fixed rejection
line and the bounded stop; the normal path's observables begin at W02's
runtime establishment and the W09 markers. W01 adds no counters, prints, or
telemetry. Evidence lives in the verification record; the implementation
record carries the contract content and decisions.

## 5. Handoff checklist

Before handing W01 to a reviewer, provide:

- the exact changed-file list;
- the materialized contract content (record), including the
  assumption/check register and every implementation-selected field with its
  selection rationale;
- W01-DV01..DV07 evidence paths and run status, including the explicit
  `not run — deferred to W10/W11` entries;
- the boundary contracts as handed to W02, and confirmation that the entry
  module implements them verbatim (or the recorded deviation);
- the §5 refinement's status with W09 (accepted, or the recorded coordination
  issue);
- confirmation that no DTB parsing, memory discovery, console abstraction,
  second canonical recipe, bootloader framework, or platform-name branch was
  introduced, and that no new `unsafe` beyond W02's entry assembly belongs to
  a W01 contract (the tier is check-and-branch only);
- open items: implementation-selected recipe fields pending P0 contracts;
  NC2-adjacent coordination with W03/W11 if any required class proves
  non-variable on the reference platform — recorded, not resolved here.
