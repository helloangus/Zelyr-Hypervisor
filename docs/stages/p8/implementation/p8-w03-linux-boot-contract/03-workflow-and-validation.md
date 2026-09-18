# P8-W03 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W03 detailed design](README.md).

## 1. Preconditions and failure boundary

Before editing, the implementer verifies it has loaded the documents named in
the parent README and confirms the observable state: no Linux boot contract
or machine specification exists; `docs/machine-types/` contains only the
stub README (plus the W02 governance document if W02 has landed). It
confirms the state of the assumed contracts it must cite: the W01
reconciliation record and the W02 governance document (or, if absent, the
interim rule of the W02 workflow step 1 applies), and the planned-only P4
boot/entry contracts.

Stop and obtain direction instead of guessing when any of the following
occurs:

- writing a fact appears to require naming an artifact address, a maximum
  size, a PSCI function list, a register convention not fixed by the pinned
  external protocol, or a Linux version — record the route (W02 §6) in the
  fact row; do not select the value;
- the pinned Linux boot protocol cannot be resolved (no documentation
  revision can be identified for the fixture family) — record the blocker;
  do not paraphrase the protocol from memory or from QEMU behavior;
- a W02 category, the ADR, or this design's fact catalog conflict — raise
  the conflict per the task book §8 handling; do not reword the catalog to
  fit;
- the contract text starts to specify loader code, fixture content, or a
  test harness — that is W09/W15/W16 scope; stop and restate as a
  requirement.

## 2. Ordered implementation steps

### Step 1 — verify the assumed inputs

Target: working context; contract §1 (Identity and binding).

Work: confirm the W02 governance document (or interim) categories and the
W01 constraint register are available; confirm which P4/P5 lifecycle
contracts are cited and their planned-only status. Record the citation
status in the implementation record
(`../p8-w03-linux-boot-contract-record.md`, created in this step).

**Acceptance:** every assumed contract the catalog cites is identified with
its planned-only or evidenced status.  
**Failure/blocker:** a cited input that contradicts the catalog is a routed
conflict, not a local rewording.

### Step 2 — draft the boot-contract document

Target: `docs/machine-types/linux-boot-contract-v0.1.md`.

Work: write the document with the status header and the eleven sections
fixed in [the fact catalog](01-boot-contract-facts.md) §1, instantiating
fact groups B1–B8, the external-standard bindings (§2 there), and the
testability/compatibility sections per
[02](02-fixture-and-shutdown-evidence.md) §3–§4. Every fact row carries its
class and authority basis; every routed value carries its route instead of a
value.

**Acceptance:** all sections present; every B1–B8 fact stated; every fact
classifiable and testable per [the catalog](01-boot-contract-facts.md) §9;
no value, loader design, fixture content, or Linux configuration selected.  
**Failure/blocker:** a fact that cannot be written without a value is
written as its route; if even that fails, raise the gap as an open item for
the W02 route table.

### Step 3 — isolation and shutdown consistency review

Target: the draft document.

Work: review the draft against [02](02-fixture-and-shutdown-evidence.md)
§2 and §4: backing provenance, no-host-fact disclosure, bounded acceptance,
lifecycle closure, and the five shutdown-evidence requirements are stated
and consistent with B7/B8. Cross-check the DTB facts against the
[P8-W04](../p8-w04-guest-dtb-contract/README.md) plan scope (content is
referenced, never restated).

**Acceptance:** every §2/§4 requirement has a corresponding contract
statement; no DTB content fact leaked into the boot contract.  
**Failure/blocker:** a missing or contradictory statement is fixed in the
contract; a discrepancy that is really a design conflict is raised per §1.

### Step 4 — wire discovery

Target: `docs/README.md` routing table;
`docs/stages/p8/implementation/README.md` status row (per that index's
conventions, coordinated with parallel W06–W20 work).

Work: add one routing row pointing Linux boot-input/boot-contract work to
the contract; add the W03 design row with truthful status. Change nothing
else.

**Acceptance:** one-link reachability from `docs/README.md`; truthful status;
all relative links resolve from a fresh checkout.  
**Failure/blocker:** broken or duplicating links fail review.

### Step 5 — Guest-EL1 and host-leakage review

Target: the draft document; verification record.

Work: review every Guest-visible statement against the constraints: Guest
EL1 with no virtual EL2 (ADR-022) is preserved; no host physical address,
IRQ, board/SoC identity, or host-firmware fact appears as a contract
statement (W02 §7 rule; P8-V03 discipline applied to this contract); every
external-standard binding names a resolvable pinned revision. Record the
review in the verification record.

**Acceptance:** no host fact in contract position; EL1 facts cite the ADR;
bindings are resolvable.  
**Failure/blocker:** a failing statement is rewritten as a category/route or
removed.

### Step 6 — closure review

Work: run the validation matrix, confirm the handoff checklist, and verify
the contract against the plan's acceptance wording ("required boot facts are
testable and no implementation choice or address is silently frozen").
Completion is claimed only in the verification record, with evidence, and
only for what was actually run.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W03-DV01 → P8-V04 | input and binding review | inspect the contract's identity/binding and external-standard sections against W01/W02 and the P4 lifecycle plans | machine binding, family-contract rules, and pinned-standard requirements present; assumed contracts cited with status | the contract rests on stated authority; not that any predecessor is implemented |
| W03-DV02 → P8-V04 | fact completeness review | map fact groups B1–B8 to the plan scope (boot-vCPU state, entry, DTB transfer, regions/lifetime, EL/MMU, secondary state, shutdown, fixture) | every scope element has at least one fact row; no scope element silently absent | the contract covers the required boot facts; not that Linux boots |
| W03-DV03 → P8-V04 | fact quality review | per fact row, check class, authority basis, and testability form ([catalog](01-boot-contract-facts.md) §9) | every row has class + authority; every Guest-visible row is testable without an implementation choice | the facts are reviewable; not that the tests exist |
| W03-DV04 → P8-V04 | isolation consistency review | check contract statements against [02](02-fixture-and-shutdown-evidence.md) §2 | all four isolation requirements stated; backing, disclosure, bounded acceptance, lifecycle closure each traceable | the boot boundary preserves Guest isolation requirements; not any runtime isolation |
| W03-DV05 → P8-V04 | fixture/shutdown evidence review | check the testability section against [02](02-fixture-and-shutdown-evidence.md) §3–§4 | all fixture categories required with owners; all five shutdown-evidence elements present; marker discipline stated | P8-V20/P8-V13 dependencies are wired; not that a fixture exists (W15) |
| W03-DV06 → P8-V04 | EL1/no-host-leakage review | scan contract text for EL2 exposure, host addresses, host IRQs, board/SoC/firmware facts | none present; EL1 facts cite ADR-022; QEMU appears only in test-environment context | the no-host-leakage property of this contract; not of the machine spec (W02/W14 own that) |
| W03-DV07 → P8-V04 | frozen-choice scan | search the contract for addresses, sizes, PSCI functions, register conventions beyond the cited protocol, Linux versions/configs | none present except routed-value placeholders with their routes | no implementation choice is silently frozen; nothing else |
| W03-DV08 → W03 closure | claim hygiene review | search all W03 artifacts for freeze/completion/loader/fixture language | none present | scope discipline; nothing else |

Record each validation as **passed**, **failed**, **blocked**, or **not
run** with method, input, date, and reason. P8-V04 is satisfied only when
DV01–DV08 are recorded. No validation here boots Linux, proves P8-V05 or any
other P8 validation, or substitutes for the W02 freeze gate.

## 4. Error, security, and observability model

W03 adds no runtime path. Its error model is contractual: the contract
defines the boot-input rejection class (B7-4) that future loader
implementations must produce, and the shutdown failure path
([02](02-fixture-and-shutdown-evidence.md) §4.5) — both VM-facing, structured,
and never hypervisor-fatal; Linux is a guest, and boot-input mistakes are
configuration errors, not attacks on hypervisor invariants (though the
contract requires treating them as untrusted input regardless of intent).
Security properties: untrusted-input validation before Guest entry; no
host-fact disclosure through artifacts; isolation-preserving artifact
lifetime. Observability: the contract's testability section fixes the
evidence expectations (markers, terminal state, resource visibility) that
W16's matrix will automate; W03's own evidence lives in the verification
record with planned/run/blocked/failed kept distinct.

## 5. Handoff checklist

Before handing W03 to a reviewer, provide:

- the exact changed-file list (expected: the contract, routing row, status
  row, implementation record, verification record);
- DV01–DV08 evidence paths and run status, including explicit not-run entries;
- the pinned Linux boot-protocol documentation revision recorded in the
  contract (or the recorded blocker if it could not be resolved);
- confirmation that no address, size bound, PSCI function, register
  convention, Linux version/config, loader design, fixture artifact, or DTB
  content fact was fixed in any W03 artifact;
- the open items handed onward: every routed value the contract defers
  (placement, bounds, PSCI subset), for W02's route table and the consuming
  packages; and
- the assumed-contract citation status (W01/W02/P4/P5) for W04, W09–W10,
  W15–W16 consumers.
