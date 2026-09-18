# P2-W10 Evidence Map, Gate Checklist, and Workflow

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W10 detailed design](README.md).  
This file fixes the record's evidence-map and gate sections and W10's own
authoring workflow.

## 1. Evidence map (record section)

One row per validation ID. The status field is filled only from the
referenced record's actual content; statuses are exactly: `passed`,
`failed`, `blocked`, `not run`, `record absent` (the referenced file does
not exist yet). The map ships with every row `record absent` and is
updated only as records appear.

| ID | Evidence sought (task book §7) | Record path (when created) | Gate condition (task book §7/§8) |
|---|---|---|---|
| P2-V01 | Boot-input boundary tests | `docs/stages/p2/verification/p2-w01-boot-platform-description-intake-verification.md` | Absence, location, length, access-range, overlap each yield an explicit outcome |
| P2-V02 | DTB structural/encoding tests | `.../p2-w01-boot-platform-description-intake-verification.md` | Malformed structures rejected with bounded diagnostics; valid encodings interpreted in bounds |
| P2-V03 | Discovery-result tests | `.../p2-w02-platform-discovery-normalization-verification.md` | Required CPU/RAM/reservation/GIC/timer/PSCI/console facts collected when declared |
| P2-V04 | Normalization/portability review | `.../p2-w02-platform-discovery-normalization-verification.md` | Consumers use normalized facts; states distinguishable; no board-name branch in Core |
| P2-V05 | Boot-map tests | `.../p2-w03-boot-memory-map-ownership-verification.md` | Map sorted/normalized/checked; holes and conflicts non-ambiguous |
| P2-V06 | Page-allocation tests | `.../p2-w04-physical-page-allocation-verification.md` | Allocate/free/alignment/multi-region/OOM/debug/accounting explicit; no protected page returned |
| P2-V07 | Small-allocation stress evidence | `.../p2-w05-dynamic-small-allocation-verification.md` | Repeated alloc/free, exhaustion, recovery preserve invariants |
| P2-V08 | Inspection review | `.../p2-w06-platform-memory-inspection-verification.md` | Summary, map dump, statistics derive from active normalized state |
| P2-V09 | Offline-checker fixture evidence | `.../p2-w07-offline-dtb-compatibility-verification.md` | Both fixtures get objective readiness reports; no runtime board-support claim |
| P2-V10 | Negative/property regression | `.../p2-w08-host-robustness-regression-verification.md` | Every P2-J group reproducible per matrix acceptance |
| P2-V11 | QEMU integration evidence | `.../p2-w09-qemu-integration-regression-verification.md` | Required configurations and repeated boots pass; accounting in documented domain |
| P2-V12 | P3/P4 handoff review | `.../p2-w10-p3-p4-handoff-contract-verification.md` | Consumers can find supported inputs, limitations, evidence locations without inferring P2's implementation design |
| P2-V13 | Stage governance review | `.../p2-w10-p3-p4-handoff-contract-verification.md` | One plan per package; conditions objective; links resolve; dependencies acyclic; P2-ACR-01 visible |

Paths are pointers, created only by their owning packages when evidence
exists; the W10 record never creates or pre-fills them.

## 2. Stage-gate checklist (record section)

P2 may be marked complete only when every P2-V01–V13 row shows `passed`
and the six task-book §8 conditions hold, each cross-checked:

1. QEMU `virt` yields required facts and a normalized representation that
   consumers do not obtain by reparsing DTB (V03/V04/V11; D-01).
2. Every protected range is excluded from allocation for every valid
   allocation/free sequence — the hard gate (V05/V06/V10/V11; D-03/D-04).
3. Page and small-object allocation have explicit normal, exhaustion, and
   release behavior with observable accounting (V06/V07; D-04/D-05).
4. Platform summary, map dump, and allocator statistics reflect the active
   normalized state (V08; D-06).
5. Host-side negative regression and QEMU integration evidence exist, and
   the offline fixture check demonstrates semantic portability without
   claiming board runtime support (V09/V10/V11).
6. P3 and P4 can locate the handoff contract and evidence, including known
   limitations and P2-ACR-01 (V12/V13).

Completion-review questions (task book §9) are reproduced in the record as
the reviewer's instrument, including: trustworthiness distinction,
allocation-path protection, untrusted-input handling, absence of P3/P4 or
board-specific mechanisms, and P2-ACR-01 status. Any negative answer
prevents completion.

## 3. Record-authoring workflow (W10 implementation steps)

### Step 1 — collect and verify sources

Target: working notes only (not committed).

Work: read all nine W01–W09 designs and any existing records; extract
deliverable summaries, limitation statements, and evidence paths.
**Acceptance:** every catalog row and limitation traces to a cited design
section; nothing is paraphrased into new semantics.
**Failure/blocker:** a design gap or contradiction stops authoring of the
affected row and is recorded as an open issue — not smoothed over.

### Step 2 — author the record

Target: `docs/stages/p2/implementation/p2-w10-p3-p4-handoff-contract-record.md`
(created in this step; this is W10's implementation record and published
contract).

Work: write it per
[01-consumer-contract.md](01-consumer-contract.md) §1–§6 and this file's
§1–§2, with all evidence statuses `record absent` unless a real record
exists at the referenced path at authoring time.

**Acceptance:** structure complete; every link resolves from a fresh
checkout; statuses truthful; non-authorization clauses present in both
handoff statements; P2-ACR-01 quoted and marked **ADR Required**;
strongest status wording is "in progress".
**Failure/blocker:** a missing upstream design section is a recorded gap
in the record, never invented content.

### Step 3 — governance cross-check (P2-V13 preparation)

Work: verify one-plan-per-package mapping (task book §4), validation-ID
coverage (§6/§7), link resolution, dependency-map acyclicity
(plans/README), and P2-ACR-01 visibility; record the check in the
verification file.

**Acceptance:** the P2-V13 row's gate condition is demonstrably checkable
from the record alone.
**Failure/blocker:** a broken link or mapping gap is fixed in the record
(or raised against the owning plan) — never waved through.

### Step 4 — review and handoff

Work: run the review matrix in §4, then hand P3/P4 planners the record
path and the completion reviewer the gate checklist. Evidence goes to
`../../verification/p2-w10-p3-p4-handoff-contract-verification.md`.
Nothing here claims W10 or P2 complete.

## 4. Package validation matrix

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W10-DV01 → P2-V12 | Contract fidelity | Review record vs [01](01-consumer-contract.md) and the nine designs | Row-by-row comparison | Summaries match cited designs; no new semantics; consumers match plans' prerequisite lists | The record is a faithful map; not that deliverables exist |
| W10-DV02 → P2-V12 | Consumer usability | Role-play review as p3-w01/p4-w02 planners | "Can I find my inputs, their states, and the limits from this record alone?" | Each role can proceed without reading all P2 designs | Handoff usefulness; not P3/P4 design work |
| W10-DV03 → P2-V12 | Limitations + ACR visibility | Checklist review | Mandatory five + residuals + P2-ACR-01 quoted with **ADR Required** | All present and findable in one section | Honesty of the handoff; not ACR resolution |
| W10-DV04 → P2-V13 | Governance mapping | Cross-check per Step 3 | Mechanical link/mapping check | All checks pass; acyclicity confirmed | Map integrity; not stage completion |
| W10-DV05 → P2-V12/V13 | Status truthfulness | Audit statuses vs referenced records | Open each referenced record | Every non-`record absent` status matches the record's content | The map's evidentiary value; not evidence existence |
| W10-DV06 → W10 closure | No-completion-claim review | Full-text review | Scan for DONE/complete/verified language | None outside conditional gate wording | The record's compliance with plan out-of-scope |

Evidence statuses for these reviews: passed / failed / blocked / not run,
with reviewer, date, and method in the verification record.

## 5. Handoff checklist

Before handing W10 to review, provide:

- the record file (and its change note) plus the verification record;
- W10-DV01–DV06 outcomes;
- confirmation that no other file was modified, no evidence was created or
  pre-filled, all statuses start truthful, and no completion claim exists;
- named-consumer readiness: P3 planners (p3-w01/p3-w02/p3-w04/p3-w06/
  p3-w11/p3-w13/p3-w14), P4 planners (p4-w01/p4-w02/p4-w03/p4-w04/p4-w05/
  p4-w08), P2 completion reviewer;
- open items recorded, not resolved: P2-ACR-01 (**ADR Required**),
  Reserved versioned cross-stage contract, downstream acknowledgement
  flow.
