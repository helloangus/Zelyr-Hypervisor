# P6-W01 Validation and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P6-W01 detailed design](README.md).

## 1. Scope of validation

W01 is hardware-free. Its validation is review- and fixture-based. QEMU rows
appear only as forward references: the runtime decision artifact and probe
evidence come into existence through
[P6-W02](../p6-w02-physical-gic-bring-up/README.md); a QEMU observation is
never proof of a W01 capability claim about real hardware.

## 2. Validation matrix

Each validation is recorded as **passed**, **failed**, **blocked**, or
**not run** with command, input, environment, timestamp, and reason, in
`../../verification/p6-w01-gic-capability-discovery-verification.md`.

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W01-DV01 → P6-V01 | Plan work-sequence 1: upstream inspection | prerequisite review | check each §1 assumed contract of [01](01-scope-and-foundations.md) against the actual upstream records existing at implementation time | every consumed contract is located and its evidence status recorded; gaps are listed as investigations, not silently assumed | the entry basis is real; not that upstream facts are correct |
| W01-DV02 → P6-V01 | Reconcilable capability model | model review + unit compile gates | review types of [03](03-code-contracts-capability-model.md) §1–§2 against the input register | one type per register row; aggregate invariants expressible; no naked IDs | the model covers the plan's required inputs; not runtime correctness |
| W01-DV03 → P6-V01 | Reconciliation rules | fixture tests (host-side) | run the fixture classes of [workflow](04-implementation-workflow.md) step 3 | each fixture yields the exact verdict set and grade; purity check (repeat call equal) | the rules implement the taxonomy deterministically; not that a real platform is usable |
| W01-DV04 → P6-V01 | Acceptance/rejection outcomes | verdict-matrix review | review [01](01-scope-and-foundations.md) §4 + escalation owners against the task book's rejection wording | every defect class has a distinct verdict, action, and owner; nothing resolves by silent default | the rejection path is explicit; not that rejection has fired on hardware |
| W01-DV05 → P6-V01 | Layering/policy separation | architecture review | search the module for platform names, register access, `unsafe`; check fact-vs-policy split of [02](02-architecture-and-state.md) §5 | zero hits; facts, findings, decision separated | capability-driven layering; not W02's implementation layering |
| W01-DV06 → P6-V01 | Record discoverability and truthfulness | record review | review the record artifact per [workflow](04-implementation-workflow.md) step 5 | states implemented facts, open investigations, and the unclaimed P6-V01 path; all links resolve | factual traceability; not completion |
| W01-DV07 → P6-V01 | Downstream consumability | consumer review | read the decision contract as W02 (can I confirm identity?), W08 (is my gate defined?), W13 (is my evidence chain started?) | each consumer can act without re-deriving rules | handoff readiness; not that downstream work is done |

Explicitly deferred to later packages and **not run** in W01: distributor /
redistributor / virtualization probes (W02), `PhysicalOnly` and `Rejected`
boot reactions (W02 + initialization owner), telemetry correlation
(W13), real-hardware capability confirmation (P15+).

## 3. Handoff checklist

Before handing W01 to a reviewer, provide:

- the exact changed-file list;
- the pinned architecture and GIC specification revisions with provenance;
- W01-DV01…DV07 evidence paths and run status, including explicit not-run
  entries;
- the open-investigation list (platform / specification / upstream-contract
  items) with owners;
- confirmation that no hardware access, `unsafe`, board constant, crate
  boundary change, or new dependency was introduced;
- confirmation that the expected-identity artifact and probe contracts are
  the only W02-facing surface, and that no consumer receives a frozen API or
  machine-specific IRQ layout;
- the graded-decision vocabulary (`ReadyForP6` / `PhysicalOnly` / `Rejected`)
  as the sole P6-wide capability outcome vocabulary.

## 4. Error, security, and observability model

- *Error model:* defects are findings with fixed escalation owners; no
  panic path; boot-gating reactions belong to consumers.
- *Security model:* platform description data re-validated before use;
  fail-closed on contradiction; no Guest input exists in W01.
- *Observability model:* three trace-event kinds with fixed payloads plus
  the decision record; the W01 record is the first link of the P6 evidence
  chain that [W13](../p6-w13-telemetry-regression-handoff/README.md)
  finalizes (P6-V27).

## 5. Future record and verification locations

Implementation facts: `../p6-w01-gic-capability-discovery-record.md`
(created when work starts; not created by this design). Verification
evidence: `../../verification/p6-w01-gic-capability-discovery-verification.md`
(created only by real verification work). Neither file exists yet and neither
may claim completion.
