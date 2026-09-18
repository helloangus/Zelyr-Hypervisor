# P0-W10 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P0-W10 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the documents
named in the parent README and inspects the current tracked tree. Useful
read-only discovery: `git ls-files` (confirm no policy or inventory exists and
no Rust source exists, hence zero unsafe) and a search of tracked documents
for existing unsafe statements.

Prerequisite-surface expectations at implementation time:

- **W05/W06** (declared prerequisites, sibling designs in preparation): the
  documentation taxonomy and the ADR escalation path this policy references.
  Assumption and boundary: the policy references their *subjects* (document
  conventions, ADR change path), not their content, so authorship proceeds;
  if their delivered contracts contradict a rule fixed here (for example the
  escalation route), the conflict is raised per the authority order and this
  policy yields — the ADR path is the higher authority, not this document.
- **W11/W14** (related designs in preparation): layering constraints and
  failure-class taxonomy are referenced by subject with explicit placeholder
  wording in the policy. When those designs deliver, their taxonomies
  supersede the placeholders at the next audit trigger — recorded, not
  silent.
- **No code exists**: if any Rust source or `unsafe` has appeared in the tree
  through an out-of-band change by W10's implementation time, stop and record
  a process violation — the governance must land before the first unsafe.

Stop and obtain direction instead of guessing when: defining a rule appears
to require fixing a concrete API, register interface, or ASM implementation
(plan out-of-scope); a maintainer asks to pre-authorize a future unsafe
segment (that authorization belongs to the approving design of the change
that introduces it); or a rule would contradict ADR-006 (ADR path, not local
rewording).

## 2. Ordered implementation steps

### Step 1 — record prerequisite-surface findings

Target: implementation record
(`../p0-w10-unsafe-rust-governance-record.md`, created in this step).

Work: inspect the tree; record that no unsafe exists (and how that was
verified), the availability of the W05/W06/W11/W14 subjects, and any tracked
unsafe statement found outside the Coding Guidelines.

**Acceptance:** the record states the zero-unsafe observation with its
verification method and each reference's availability.  
**Failure/blocker:** pre-existing unsafe code or an authorizing statement
outside this governance is a process conflict — stop and record.

### Step 2 — author the unsafe policy document

Target: `docs/security/unsafe-rust-policy.md`.

Work: write the document with the status header required by `docs/README.md`
(status, scope, version `v0.1`, owner/change context, supersedes: none) and
the normative content fixed by the [policy contract](01-unsafe-policy-contract.md)
§2–§8: safe-Rust-first principle, justification and SAFETY-comment template,
review rules, boundary categories with placement constraints, forbidden
patterns, escalation thresholds, and the future-gate predicate.

**Acceptance:** every required rule present; no concrete API, wrapper,
register interface, or ASM decision is made; the document contradicts no
ADR, task-book, or Coding-Guidelines rule (it operationalizes them).  
**Failure/blocker:** contradictions with governing documents are raised per
§1, not reworded away.

### Step 3 — create the empty inventory

Target: `docs/security/unsafe-inventory.md`.

Work: create the register per the [inventory contract](02-inventory-contract.md)
§2: status header, schema pointer, zero entries. Do not add speculative
entries.

**Acceptance:** the inventory exists, points to the policy schema, and
contains no entries.  
**Failure/blocker:** any pressure to pre-register hypothetical unsafe is a
scope boundary — the register holds real, merged unsafe only.

### Step 4 — first-unsafe walkthrough (documentary validation)

Target: verification record
(`../../verification/p0-w10-unsafe-rust-governance-verification.md`).

Work: on paper, play a P1-style first unsafe change — a hypothetical EL2
register read in an arch-layer module (explicitly hypothetical: no API,
path, or type is committed anywhere) — through the full institution:
approving design names the segment → category assignment and placement
check → necessity statement → SAFETY-comment draft per the template →
inventory entry with all fifteen schema fields → second-reviewer rule →
same-change rule → future-gate predicate check. For each institution step,
confirm it resolves without an undocumented decision and produces a
traceable artifact. Record the trace and every gap; fix gaps in the policy
or schema, then re-trace. Also trace one negative path: a change that tries
to introduce unsafe its design did not name, and confirm the institution
stops it.

**Acceptance:** both traces resolve end to end with no undocumented decision;
every gap found was fixed and re-traced.  
**Failure/blocker:** a gap that would require deciding a concrete
implementation is fixed by strengthening the *process wording*, never by
authorizing an implementation.

### Step 5 — wire discovery

Targets: `docs/README.md`, `docs/stages/p0/implementation/README.md` (and,
optionally, a pointer line in `docs/security/README.md` that does not
restate policy).

Work: add one routing-table row to `docs/README.md` pointing unsafe-writing
and unsafe-review work at the policy, and add the W10 design row to the
stage implementation index with a truthful status. Change nothing else.

**Acceptance:** one-link reachability from `docs/README.md`; all new
relative links resolve from a fresh checkout.  
**Failure/blocker:** broken or duplicating links fail review.

### Step 6 — closure review

Work: run the validation matrix below, confirm the handoff checklist, and
verify the package against its task-book requirement (P0-W10), prerequisite
compatibility, document links, and downstream handoff wording. Completion is
claimed only in the verification record, with evidence, and only for what
was actually reviewed. No unsafe was introduced and none was authorized.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W10-DV01 → P0-V11 | Policy completeness review | inspect the policy against [policy contract](01-unsafe-policy-contract.md) §2–§8 and ADR-006 + Coding Guidelines | principle, justification template, review rules, six categories with placement constraints, forbidden patterns, and thresholds all present and unambiguous | the governance rules exist; not that any unsafe has passed through them |
| W10-DV02 → P0-V11 | Inventory existence and location review | inspect `docs/security/unsafe-inventory.md` against [inventory contract](02-inventory-contract.md) §2 | register exists at the declared location, points to the schema, holds zero entries | a usable inventory location exists; not future compliance |
| W10-DV03 → P0-V11 | Schema auditability review | inspect the fifteen schema fields against [inventory contract](02-inventory-contract.md) §3–§4 | every field defined with content, filler, and lifecycle; same-change and audit rules explicit | entries will be auditable; not that any entry exists |
| W10-DV04 → P0-V11 | First-unsafe walkthrough | step 4 positive and negative traces | both traces resolve without undocumented decisions; gaps fixed and re-traced | the institution is usable for a first unsafe change without re-deciding basics; not that a real change has run |
| W10-DV05 → P0-V11 | Zero-unsafe confirmation | step 1 verification, repeated at closure | tree still contains no unsafe and no pre-authorization statement | governance landed before first unsafe; not future enforcement (gate is W07/W20 future class) |
| W10-DV06 → P0-V09 | Discovery and link review | resolve routing row, policy/inventory links, and index row from a fresh checkout | one-link reachability; truthful status; all links resolve | documentation navigation; not W05's taxonomy |
| W10-DV07 → W10 closure | Consumability review | read the institution as a P1 arch designer (can I draft my unsafe record?), W07 (is the gate predicate checkable?), W18 (is the first/third-party split clear?), W11/W14 (are the hooks unambiguous?) | each perspective can act without inventing policy | handoff readiness; not that consumers are done |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with method, input, timestamp, and reason. P0-V11 is satisfiable by this
documentary evidence set — that is its definition ("policy and inventory
location exist and are usable for a first unsafe change"). No W10 validation
proves any future unsafe is sound, and none may be reported as doing so.

## 4. Error, security, and observability model

W10 adds no runtime error model, synchronization, guest input handling, or
`unsafe` code — it is the security governance *for* future unsafe. Its design
points:

- **Security posture:** unsafe is treated as privileged capability requiring
  explicit authorization (approved design), a stated safety argument
  (SAFETY template), a named accountable owner, and an auditable register —
  the same authorize-then-exercise shape the project applies to capabilities
  at runtime. Convenience expansion is the threat; the anti-expansion rules
  and escalation thresholds are its controls.
- **Failure handling:** the institution's failure mode is a blocked or
  failed review (missing entry, unnamed segment, forbidden pattern, failed
  audit), never a silent acceptance. The negative walkthrough proves the
  stop path exists.
- **Observability:** the inventory is the audit trail — bidirectionally
  consistent with the code (policy §8), history-retaining across
  supersession/removal, and re-audited on every trigger that could invalidate
  a safety argument. The verification record's walkthrough traces are the
  accepted proof surface for W10 itself.

## 5. Handoff checklist

Before handing W10 to a reviewer, provide:

- the exact changed-file list (policy, empty inventory, discovery wiring);
- the zero-unsafe confirmation and its verification method;
- DV01–DV07 evidence paths and status, including both walkthrough traces;
- the open hooks recorded for sibling designs (W11 placement constraints,
  W14 failure-class supersession, W05 re-homing) and their trigger rules;
- confirmation that no code, API, wrapper, register interface, ASM, crate,
  gate, or CI artifact was created and no unsafe was authorized; and
- open items for W07 (future gate promotion), W18 (third-party split), and
  P1+ low-level designs (record expectations) — without resolving their
  contracts here.
