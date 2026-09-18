# P8-W12 Implementation Workflow and Review Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W12 detailed design](README.md).

## 1. Preconditions and failure boundary

Before any step, the implementer verifies it has loaded the documents named in
the parent README and inspects the actual state of the prerequisites: the P2/P4
stage implementation and verification material referenced by the P2-W10
(`../../../p2/plans/p2-w10-p3-p4-handoff-contract.md`) and P4-W09
(`../../../p4/plans/p4-w09-closeout-p5-handoff.md`) handoffs, the approved
machine map facts routed by P8-W02, the W09/W10 Linux records, the W15 fixture
manifest, and the W16 harness design. P8 consumes only evidenced predecessor
facts ([plans index](../../plans/README.md)).

Stop and obtain direction instead of improvising when any of the following
occurs:

- a [01 §1](01-memory-matrix-and-boundaries.md) contract row is not evidenced
  or differs from the assumed fact — record a `P4DependencyIssue` /
  `P2DependencyIssue` / `P8IntegrationGap` and block the affected rows; do not
  simulate the missing capability;
- the approved map facts needed for class capacities or probe expectations are
  absent — block capacity recording and L2 descriptor validation; do not
  invent addresses or sizes;
- a validation need appears to require Stage-2 changes, a new ownership
  mechanism, runtime map/unmap, or a new diagnostic field — stop; that is an
  `Architecture Change Request` (Stage-2/ownership) or a W13 gap (diagnostic
  fields), never a W12 edit;
- the W15 fixture lacks the [02 §5](02-probe-and-workload-contracts.md)
  programs, or W16's harness realizes the 02 contracts with different
  semantics — record an integration gap against the owning package and block
  affected rows; this design owns the semantics;
- an observation fits no expected outcome and no documented P4-W06
  classification — record it as an explicit block and carry it to closure
  review; do not force a classification;
- a Linux fault produces an EL2-level event — that is a
  `HypervisorInvariantViolation` ([01 §6](01-memory-matrix-and-boundaries.md));
  record as hypervisor-level failure per P0-W14; never absorb it as a VM-facing
  outcome.

## 2. Ordered implementation steps

### Step 1 — verify prerequisites and record the contract baseline

Target: implementation record (`../p8-w12-linux-memory-model-record.md`,
created in this step).

Work: for each row of [01 §1](01-memory-matrix-and-boundaries.md), locate the
evidenced fact and record its pointer, or mark the row blocked with the reason.
Record the approved map facts (RAM window, reserved regions, MMIO window,
permission distinctions if any) that the matrix will instantiate.

Suggested observation: read the P2/P4 verification records and the approved
machine contract record; no runtime action.

**Acceptance:** every row has an evidence pointer or an explicit blocked entry;
map facts are named with their authority.
**Failure/blocker:** missing map facts block L2 rows; missing P4/P2 evidence
blocks the whole matrix (W12-DV04/DV05 are blocked, not skipped).

### Step 2 — record RAM-class capacities and derive probe expectations

Target: implementation record.

Work: instantiate the [01 §4](01-memory-matrix-and-boundaries.md) classes:
record each capacity with its map bounds and justification. For each L2 probe
kind, derive the concrete expectation (expected classification and contained
outcome) from the approved map facts and record the probe addresses.

**Acceptance:** three class capacities recorded within the approved window with
justifications; every N1–N5 expectation is derivable from recorded facts.
**Failure/blocker:** an expectation that cannot be derived makes its
descriptor blocked (per 02 §2); a capacity outside the window is invalid.

### Step 3 — review the matrix and descriptors

Target: `MemoryScenarioDescriptor` instances covering M12-1…M12-10.

Work: build descriptors per [02 §2](02-probe-and-workload-contracts.md); verify
layer assignments ([01 §2](01-memory-matrix-and-boundaries.md)), class
coverage ([01 §4](01-memory-matrix-and-boundaries.md)), and probe expectations
([01 §5](01-memory-matrix-and-boundaries.md)). Confirm fixture consistency with
the W15 manifest and harness consistency with W16's design.

**Acceptance:** W12-DV02/DV03 reviews pass; descriptors reference only recorded
facts and declared markers.
**Failure/blocker:** invalid descriptor fails review; fix the descriptor, not
the rules.

### Step 4 — execute the matrix through the W16 harness

Target: verification record
(`../../verification/p8-w12-linux-memory-model-verification.md`).

Work: run the matrix in dependency order — L1 boot and workload rows per class,
then L2 boundary probes per class. Record commands, environment, transcripts
(console including Linux-visible fault output), Stage-2 observations, and
console-health evidence for N4. Record what was not run and why.

**Acceptance:** every executed descriptor has complete evidence; L2 probes
correlate their `MEM-PROBE start` marker with the observed Stage-2 event.
**Failure/blocker:** a run failure is evidence — record failed/blocked; do not
adjust probes or expectations to force outcomes.

### Step 5 — evaluate isolation verdicts and diagnostic sufficiency

Target: verdicts in the verification record.

Work: apply `evaluate_memory_isolation` per probe and the L1 completion checks
per category; apply `diagnostic_sufficient` to every observed fault. A
sufficiency failure fails the row's "actionable diagnostics" clause and is
handed to [W13](../p8-w13-guest-fault-diagnostics/README.md) as a diagnostic
gap.

**Acceptance:** executed matrix satisfies P8-V17's substance — classes honor
boundaries, Host isolation holds, diagnostics are actionable per event.
**Failure/blocker:** violations route per §1; no expectation is weakened.

### Step 6 — ownership, invariant, and closure review

Work: perform the [01 §6](01-memory-matrix-and-boundaries.md) ownership and
Guest-untrusted review across all runs (single ownership, donation bounds,
protected-range exclusion, no EL2-level event from a Linux fault, no host
leakage). Then run the validation matrix (§3), confirm the handoff checklist
(§5), and verify the package against the plan's acceptance wording and the
task-book P8-V17 row. Completion is claimed only in the verification record,
with evidence, and only for what was actually run.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W12-DV01 → prerequisite review | upstream contract review | inspect P2/P4/P8 evidence against 01 §1 rows | every row evidenced or explicitly blocked | the baseline is real; not that the matrix runs |
| W12-DV02 → P8-V17 | category/descriptor review | review descriptors against 01 §2–§3 and 02 §2 | all categories covered; layers correct; no runtime mapping declared | the matrix is declared and checkable; not that Linux behaves |
| W12-DV03 → P8-V17 | RAM-class and fixture review | review recorded capacities vs map bounds; fixture vs 02 §5 | classes within window and justified; programs/markers match | classes are bounded without selecting machine values; not that capacities are optimal or ABI facts |
| W12-DV04 → P8-V17 | memory-size matrix execution | run class rows via W16 | each class boots and completes its declared L1 set without unclassified Stage-2 events | Linux memory behavior operates over Stage-2 at declared sizes; not overcommit, ballooning, snapshot, or performance |
| W12-DV05 → P8-V17 | boundary/isolation execution | run N1–N6 with verdicts and sufficiency checks | expected faults observed and classified; contained; context sufficient; console unaffected (N4); ownership clean (N6) | map boundaries, Host isolation, and actionable diagnostics hold under Linux; not security certification or IOMMU/DMA isolation |
| W12-DV06 → P8-V17 | ownership/invariant review | step-6 review over all runs | all 01 §6 invariants hold; zero Guest-fault-to-EL2-event occurrences | the ownership/untrusted constraints held; not Stage-2 internals |
| W12-DV07 → closure | consumer consumability review | read outputs as W16 (matrix rows), W18 (isolation foundation), W13 (sufficiency + subclasses), W15 (workload needs) | each consumer can act without inventing requirements | handoff readiness; not that consumers are done |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. Declaring the matrix
without W12-DV04/DV05 execution does not satisfy P8-V17. Nothing here
contributes to P8-V12–V16 (owned by W09–W11) or P8-V18–V26.

## 4. Error, security, and observability model

**Errors and failure guarantee.** W12 adds no runtime error path. Failures are
evidential: a blocked prerequisite, an invalid descriptor, a probe whose
outcome differs from expectation, or an insufficient diagnostic is recorded as
failed/blocked with evidence refs. The guarantee to preserve: a Linux-caused
fault never escalates beyond the VM context; the only hypervisor-level outcome
is a genuine `HypervisorInvariantViolation` handled per P0-W14
(`../../../p0/plans/p0-w14-panic-failure-classification.md`).

**Security.** Linux is untrusted (ADR-007). Probe addresses and workloads are
validation parameters derived from approved facts; observed Guest behavior is
recorded, never trusted for control flow. The boundary matrix is functional
isolation evidence: it demonstrates exclusion of Guest access from Host-owned
and reserved ranges and MMIO-window containment, but it is not a security
certification and does not cover DMA/IOMMU, interrupt storms, or adversarial
pattern coverage — those are [W18](../p8-w18-security-isolation-regression/README.md)'s
lane. Host physical addresses must not appear in Guest-visible diagnostics
(host-leak rule, 01 §6).

**Observability.** The evidence surface is the verification record: per-run
console transcripts (including Linux's own fault output), Stage-2 observations
with W13-sufficient context, probe verdicts, ownership-query results, and the
explicit not-run list. Per-event sufficiency checks make "actionable
diagnostics" reviewable rather than asserted.

## 5. Handoff checklist

Before handing W12 to a reviewer, provide:

- the exact changed-file list (expected: the implementation record; verification
  entries; no source or memory-mechanism changes);
- the contract baseline with pointers or explicit blocked entries (W12-DV01);
- recorded RAM-class capacities with map bounds and justifications, and derived
  probe expectations (step 2);
- W12-DV04/DV05 outcomes with transcripts, observations, verdicts, and
  sufficiency results, including explicit not-run entries;
- the ownership/invariant review findings (W12-DV06);
- confirmation that no Stage-2, page-table, allocator, ownership, VMID,
  Host-allocator, machine-map, or DTB change was made or requested from W12;
- open items: diagnostic gaps for W13, `P4DependencyIssue`/`P2DependencyIssue`
  records for the P4-W09/P2-W10 owners, fixture gaps for W15 — without
  resolving them here.
