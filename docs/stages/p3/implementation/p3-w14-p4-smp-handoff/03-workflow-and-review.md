# P3-W14 Workflow, Review, and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W14 detailed design](README.md).

## 1. Preconditions and failure boundary

Before any work, the implementer verifies it has loaded the parent README
and [02-p4-consumer-map.md](02-p4-consumer-map.md), and inspects which
P3 implementation and verification records actually exist
(`git ls-files` under `docs/stages/p3/`). Stop and record instead of
improvising when:

- a reliance section's evidence does not exist — the section is drafted
  as a limitation and its item enters the unresolved register; upgrading
  a status to make the handoff look complete is forbidden;
- consolidating surfaces a conflict between two P3 packages, or between
  P3 and P2/P4 — label it `ADR Required` or `Architecture Change
  Request` in the register with the owners; never resolve it inside the
  contract;
- drafting appears to require defining a P4 mechanism or a new P3
  interface — scope violation; the item becomes a limitation;
- the P4 planning set changes its consumption list — re-issue the
  contract per its lifecycle; do not chase unpublished intent.

## 2. Ordered workflow

### Step 1 — build the evidence inventory

Target: the inventory (part of the implementation record).

Work: for each W01–W13 package, record design path, implementation-record
path, verification-record path, and honest status (evidenced / planned /
blocked / absent), citing real files only.

**Acceptance:** a complete inventory covering all fourteen packages;
every path resolves.  
**Failure/blocker:** an unreadable or missing path is recorded as such —
the inventory's honesty is the contract's honesty.

### Step 2 — draft the contract artifact

Target:
`docs/stages/p3/implementation/p3-w14-p4-smp-handoff-contract.md`
(created in this step, only after step 1).

Work: write R1–R9 per the fixed structure, each statement in P3's own
terms with its evidence status; write limits/non-guarantees, the
unresolved register (seeded from the inventory and the plans' recorded
open items — P2-ACR-01 included by reference), the independence list,
and the consumer map.

**Acceptance:** no statement without a cited source and status; every
unevidenced guarantee rendered as a limitation; register complete.  
**Failure/blocker:** a section that cannot be written honestly blocks its
guarantee claim — the limitation form is always available.

### Step 3 — reading-order integration

Target: the contract's placement in the documented flow.

Work: verify the contract is reachable from the P4 reading order as that
order directs (P4 plan index: "upstream P0–P3 handoff records"), and from
W15's stage navigation obligations; check every relative link resolves.

**Acceptance:** a P4 planner following only the P4 documents reaches the
contract; all links resolve from a fresh checkout.  
**Failure/blocker:** index/stage files are owned by their own packages
and the coordinator — integration gaps are recorded, not patched by
editing those files here.

### Step 4 — boundary and ADR review

Target: the drafted contract.

Work: review every statement against host/guest separation (nothing
readable as a vCPU/VM/guest contract), the ADR object model, and the
reserved splits; verify no upstream term is redefined and no conflict is
silently resolved.

**Acceptance:** W14-DV04 evidence recorded; zero boundary violations.  
**Failure/blocker:** a violation is fixed in the contract; an unavoidable
one is labeled and registered.

### Step 5 — P4-consumer sufficiency review

Target: the contract versus the consumer map.

Work: walk each P4 package row of the consumer map; for each consumption
need, record sufficient / insufficient against the contract's sections
and bounds; an insufficiency becomes either a registered limitation (if
P3 genuinely does not provide it and P4 will design it) or a blocking
issue (if P3 was supposed to provide it per its plan). The review is a
desk review by the P3 planner role; it does not require P4 participation
and claims none.

**Acceptance:** W14-DV05 evidence: every consumer row has a result;
P3-V14's condition — a P4 planner can locate and rely on the stated
foundations without redesigning P3 — is met for every `sufficient` item,
with `insufficient` items explicitly registered.  
**Failure/blocker:** an unresolvable insufficiency is a blocking issue
with an owner; it does not silently shrink P3's plan outcomes.

### Step 6 — records and closure

Target: implementation record and verification record.

Work: record decisions (statuses, register entries, map adjustments) in
`../p3-w14-p4-smp-handoff-record.md`; record review evidence in
`../../verification/p3-w14-p4-smp-handoff-verification.md`; run the
validation matrix and handoff checklist.

**Acceptance:** records complete; no completion claim beyond the review
evidence.  
**Failure/blocker:** honest statuses only — `planned`, `blocked`,
`not-run` are legal closure states for this package when the P3
implementation they describe does not exist yet.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W14-DV01 → P3-V14 | contract structure review | inspect the artifact against [02](02-p4-consumer-map.md) §2 | R1–R9 present with source, evidence, bound; limits/register/independence/consumer sections complete | the contract is explicit and bounded; not that P3 works |
| W14-DV02 → P3-V14 | evidence-link audit | resolve every cited path; compare statuses with the inventory | every statement's evidence status truthful; no unevidenced guarantee; register complete incl. P2-ACR-01 by reference | consolidation honesty; not the underlying evidence itself |
| W14-DV03 → P3-V14 | P4 reading-order integration review | follow the P4 index/task-book reading order as a P4 planner | the contract is reached and its map addresses each P4 package | locatability; not P4 acceptance |
| W14-DV04 → W14 closure | host/guest and ADR boundary review | step 4 checklist over every statement | no vCPU/VM/guest-readable statement; no redefined upstream term; no silently resolved conflict | boundary integrity |
| W14-DV05 → P3-V14 | P4-consumer sufficiency review | step 5 per consumer row | every row resulted; sufficient items satisfy P3-V14's condition; insufficient items registered with owners | sufficiency for P4 planning; not P4 implementation or P3 completion |
| W14-DV06 → W14 closure | independence-list completeness review | check §4 items against both task books' reserved lists | every reserved mechanism is listed with its authority | the handoff is bounded on both sides |

Record each validation as **passed**, **failed**, **blocked**, or
**not run** with input, timestamp, and reason. Until P3 packages produce
implementation and verification records, W14 can honestly complete only
the structure/availability portions (DV01 over a draft is impossible —
the artifact requires the inventory; this is recorded as the deferral,
not as completion).

## 4. Error, security, and observability model

- **Failure mode of a handoff is silent overclaim.** This design's
  controls are the evidence-status rule, the audit (DV02), and the
  register; a contract defect is any statement whose cited evidence does
  not carry it.
- **Security posture.** The contract exposes no new surface; it
  summarizes host-internal contracts. It must restate no secret, key, or
  credential material (none exists in P3) and no guest-reachable
  behavior.
- **Observability.** The contract is itself the observability artifact
  for stage transition: its register is the single list a closure
  reviewer (W15) and the P4 entry reviewer (P4-W01) both consume.

## 5. Handoff checklist

Before handing W14 to a reviewer, provide:

- the contract artifact path and version, plus the changed-file list;
- the evidence inventory with statuses and the register's entries
  (including every `ADR Required` / `Architecture Change Request` label
  and its owner);
- the consumer map's review results per P4 package (DV05);
- confirmation that no P4 mechanism, new P3 interface, upstream edit, or
  completion claim was introduced;
- open items for W15 (navigation/traceability hooks) and the standing
  note that P4-W01 owns the P4-side reconciliation.

## 6. Future record paths

Implementation record: `../p3-w14-p4-smp-handoff-record.md` (created only
when implementation begins). Verification record:
`../../verification/p3-w14-p4-smp-handoff-verification.md` (created only
when evidence exists). The contract artifact itself is created only in
workflow step 2 — never by this design.
