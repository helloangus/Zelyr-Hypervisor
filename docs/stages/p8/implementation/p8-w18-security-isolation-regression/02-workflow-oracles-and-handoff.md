# P8-W18 Workflow, Oracle Review, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W18 detailed design](README.md).

## 1. Preconditions and failure boundary

Before any execution, the implementer verifies it has loaded the documents
named in the parent README and inspects the tracked tree. The following are
**assumed contracts**; if any delivers differently, the affected row stops as
a recorded blocked prerequisite — never repaired locally, never weakened:

- **W16 envelope** (`p8-w16-automated-linux-regression`): harness contract,
  timeout/contamination oracles, failure classification. W18 rows run under
  it; W18 does not build a second execution path.
- **W13 diagnostic taxonomy** (`p8-w13-guest-fault-diagnostics`): the fault
  ledger and minimum diagnostic context that oracles ORA-PANIC/ORA-CLASS and
  the containment expectations depend on. *Failure boundary:* no taxonomy ⇒
  matrix review is blocked at W18-DV02; rows are blocked, not reclassified.
- **W05 behavior classification** and **W06 PSCI boundary**: expected-result
  content for ISO-02..ISO-05. *Failure boundary:* missing classification ⇒
  those rows cannot declare controlled results and stay blocked.
- **W15 fixture and P4–P7 VG triggers** (`p8-w15-reproducible-linux-fixture`,
  `p4-w05-validation-guest`, `p5-w07-validation-guest-isolation-suite`,
  `p6-w11-validation-guest-interrupt-suite`, `p7-w10-validation-guest-suite`):
  declared fault/storm/crash triggers. *Failure boundary:* a vector without
  a declared trigger stays blocked; improvising a trigger is prohibited.
- **P5 security facts** (`p5-w10-closeout-p6-handoff` contract): the
  authority model underlying §4.1's no-residual-authority check. *Failure
  boundary:* without it, check 4 degrades to record-only and the limitation
  is stated in the verification record.
- **P8-W01 reconciliation** (`p8-w01-entry-contract-reconciliation`): the
  evidence-backed statement of which P0–P7 security facts actually exist.
  W18 rows may only cite facts W01 reconciled as evidenced.

Stop and obtain direction when a required decision is missing, when an
observed failure indicates a boundary contradiction with ADR §19 or the P5
authority model, or when containment and the declared expected result
diverge.

## 2. Ordered implementation workflow

### Step 1 — reconcile security inputs

Target: implementation record (`../p8-w18-security-isolation-regression-record.md`,
created in this step).

Work: for each §1 contract, record its evidence status (evidenced / planned /
absent) and mark every matrix row executable or blocked, with the blocker
named. Cross-check against W01's reconciliation rather than re-auditing
upstream evidence.

**Acceptance:** per-row executability statement with owner attribution.  
**Failure/blocker:** a contradiction between contracts (e.g., W13 and W05
classifying the same vector differently) is recorded as a conflict and
escalated per §5; W18 does not pick a winner.

### Step 2 — execute the matrix

Target: verification record and `../../verification/assets/p8-w18/`.

Work: run executable rows through the W16 envelope in a declared order
(clean baseline rows before abuse rows; recovery check after each
state-leaking row). Record repetition and storm-duration parameters per row
before execution. Retain full transcripts.

**Acceptance:** every executed row has a run record with verdict, oracle
evaluation per §5 of [01](01-isolation-scenario-matrix.md), and transcript
reference; blocked rows remain listed.  
**Failure/blocker:** a harness-infra failure (not a containment outcome) is
recorded as not-run for that row; it is never scored as pass or fail.

### Step 3 — classify outcomes

Target: verification record.

Work: classify every verdict per the W16 §6 classes (reproducible evidence /
blocked prerequisite / regression), extended by W18's escalation rule (§4).
For failed rows, attach the transcript pair and the oracle that fired.

**Acceptance:** no unclassified verdict remains.  
**Failure/blocker:** an unclassifiable failure is itself a recorded block
pending owner-package diagnosis.

### Step 4 — containment-limit statement

Work: from executed evidence only, draft the factual containment-limit
statement: which vectors are contained, in which environment, within which
declared bounds, with which explicitly unproven boundaries (multi-VM, DMA,
hardware, unlimited storms). Unexecuted or blocked rows appear as absent
coverage, never as assumed containment.

**Acceptance:** the statement names only observed rows; every P8-V24 scope
limit is stated.  
**Failure/blocker:** absence of evidence for a vector is stated as absence;
no generalization from QEMU to hardware is permitted.

### Step 5 — closure review

Work: run the validation matrix (§4); confirm the handoff checklist (§6);
verify escalation items are visible to their owning packages. Completion is
claimed only in the verification record, with evidence, for what was
actually run.

## 3. Escalation of unresolved failure classes

A failure whose diagnosis indicates any of the following is a **security or
architecture block**, not a fixable regression:

1. a Hypervisor-invariant violation (ADR §19: unvalidated Guest address used;
   writable page shared across distrust boundaries; capability-check failure
   degraded to implicit allow);
2. Guest-caused fault escalating beyond the VM context;
3. an unclassifiable fault class persisting after owner-package diagnosis;
4. a divergence between containment and the approved W05/W06 classification
   of the same operation.

Handling: record the block in the verification record with the transcript,
notify the owning package, and label it `Architecture Change Request` or
`ADR Required` when it contradicts an ADR/task-book constraint. P8 closure
([W20](../p8-w20-documentation-closure-handoff/README.md)) treats an open
security block of this kind as an unclosed exit criterion. Silently patching
the mechanism, reclassifying the fault, or narrowing the scenario to make a
block disappear is prohibited.

## 4. Validation matrix

All rows are planned evidence; none asserts a run occurred. Record each as
**passed**, **failed**, **blocked**, or **not run** with command, input,
environment, timestamp, and reason.

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W18-DV01 → prerequisites | contract review | inspect §1 assumed contracts and per-row executability | every contract named with owner and failure boundary; executability statement complete | security-input coherence; not that inputs delivered |
| W18-DV02 → P8-V24 | matrix completeness review | review [01](01-isolation-scenario-matrix.md) §3 against task-book P8-V24 vectors and plan scope | every plan-named vector (illegal IPA/MMIO/sysreg/PSCI/topology, crash, loop, storm) has a row with property, trigger, expected result, containment expectation, and oracles | declared abuse coverage; not that containment holds |
| W18-DV03 → P8-V24 | boundary-observation review | review [01](01-isolation-scenario-matrix.md) §4 | Host-protection oracle set complete; other-VM proof boundary stated with Reserved items explicit | honest scope boundary; not multi-VM containment |
| W18-DV04 → P8-V24 | isolation regression run | execute executable rows per Step 2 | every executed row: no firing oracle; Guest-scoped outcomes match the declared classification; recovery checks pass | containment of the declared vectors in the declared QEMU environment within declared bounds; not hardware behavior, not multi-VM, not DMA, not unlimited-duration guarantees |
| W18-DV05 → W18 closure | escalation review | audit verdict classification and §3 blocks | every verdict classified; open security blocks visible to owning packages; no silent narrowing | review discipline; not P8 closure |
| W18-DV06 → handoff | consumability review | read the containment-limit statement as W20 and as P9 | consumers can act without overstating or under-reporting the boundary | handoff readiness; not that downstream work is done |

Only W18-DV04 with real runs (plus DV05's audit) can satisfy P8-V24. All
containment evidence is QEMU/TCG-scoped and bound to the declared
parameters; a QEMU containment pass never proves hardware containment.

## 5. Error, security, and observability model

- **Failure model:** the six named oracles (ORA-PANIC, ORA-HANG, ORA-CLASS,
  ORA-LEAK, ORA-CROSS, ORA-ESCALATE) are the complete failure vocabulary;
  every run record maps its verdict to exactly one oracle or to a pass.
  Oracle evolution is a reviewed design change, never a run-time choice.
- **Security position:** W18 observes the Guest-untrusted boundary; it
  creates none. Its own artifacts must not become an attack surface: stored
  transcripts may contain hostile Guest output, so any tooling that later
  consumes them parses them as untrusted input (consistent with the Coding
  Guidelines' untrusted-input rule); transcripts are evidence, never
  executable content.
- **Observability:** full diagnostic transcripts are the proof surface.
  Summaries may index them but cannot replace them, because diagnostic
  *content* (context completeness, absence of Host disclosure) is part of
  the oracle.

## 6. Handoff checklist

Before handing W18 to a reviewer, provide:

- the exact artifact list (matrix design files, verification record,
  transcripts) and per-row executability status;
- W18-DV01–DV06 evidence paths and run status, including explicit blocked
  and not-run rows with owning packages named;
- the containment-limit statement and the list of explicitly unproven
  boundaries;
- the escalation register: any §3 block, its label (`Architecture Change
  Request` / `ADR Required`), and its owning package;
- confirmation that no trigger, mechanism, policy, or classification was
  invented or altered under W18 authority;
- confirmation that no result or containment claim appears in any design
  artifact;
- open items: approved machine values (task book §8 `ADR Required` via
  W02), storm-workload fixture dependency (W15), and the P5 authority-model
  dependency for the §4.1 residual-authority check — without resolving any
  of them here.
