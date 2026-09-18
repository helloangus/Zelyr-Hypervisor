# P0-W13 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P0-W13 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the documents
named in the parent README and inspects the current tracked tree. Useful
read-only discovery: `git ls-files` (confirm no trace-event-namespace document
exists yet) and a search of tracked documents for existing event-naming
statements (ADR-048, §12, and any sibling boundary pointers). No telemetry
implementation exists, so no event name exists to migrate or grandfather.

Stop and obtain direction instead of guessing when any of the following
occurs:

- [W12](../p0-w12-logging-diagnostic-baseline/README.md) or
  [W05](../p0-w05-documentation-baseline/README.md) has delivered a contract
  conflicting with the boundary or header statements of this design — raise
  the conflict; do not re-decide the sibling's subject here;
- implementing appears to require declaring an event, defining a payload or
  encoding, or adding any code — that is future telemetry design scope, and
  reaching for it is a scope violation;
- a domain definition cannot be written without designing an event for it —
  the registry defines subjects, not events; a subject that needs an event
  to be explainable is a gap in this design to raise, not a registry entry
  to add; or
- a maintainer requests dropping a plan-named domain or ad-hoc string naming
  — that is the ADR path per [the namespace contract](01-namespace-contract.md)
  §8, never a local edit.

## 2. Ordered implementation steps

### Step 1 — verify prerequisite and sibling surfaces

Target: implementation record (created in this step).

Work: record whether W05 and W12 have delivered documents, which convention
source this implementation follows (delivered rules if present; otherwise
the existing `docs/README.md` header mandate), and any conflict surface —
in particular whether W12's channel-boundary language has landed, since the
boundary section of the namespace document references it by subject.

**Acceptance:** the record names the convention source and every
blocked-by-prerequisite surface.  
**Failure/blocker:** an unresolved governing conflict stops the work per §1.

### Step 2 — author the namespace document

Target: `docs/development/trace-event-namespace.md`.

Work: write the document with the status header and all content required by
[the namespace contract](01-namespace-contract.md): the fourteen-domain
registry with subject definitions and boundary notes, the grammar, the
declaration rule with the empty canonical-event registry, the compatibility
and deprecation rules, the channel-boundary section, and the thresholds. No
event may be declared; no payload, encoding, transport, or code may appear.

**Acceptance:** every required section is present; the registry has zero
entries; the document names no event, payload, transport, or code identifier;
it contradicts no ADR, task-book, or Coding-Guidelines rule.  
**Failure/blocker:** a contradiction is raised per §1, not absorbed by
rewording.

### Step 3 — wire discovery

Targets: `docs/README.md`, `docs/stages/p0/implementation/README.md`.

Work: add one routing row to `docs/README.md` pointing trace-event naming/
classification work at the namespace document; add the W13 design row to the
stage implementation index with status "Proposed design; implementation not
claimed" (updated truthfully as work proceeds). Change nothing else in these
files.

**Acceptance:** a contributor starting from `docs/README.md` reaches the
namespace document in one link; links resolve from a fresh checkout.  
**Failure/blocker:** a broken or duplicating link fails review.

### Step 4 — attribution drill

Target: verification record.

Work: execute the drill of W13-DV04 against clearly hypothetical future P1
event categories, labeled as drill material only. Required minimum set: an
EL2 boot-stage milestone (attribution exercise between `boot` and the target
subsystem domain); a Stage-2 permission fault (between `stage2` and
`memory`); a vCPU switch with its scheduling reason (between `vcpu` and
`scheduler`); a virtio queue-kick fact (between `virtio` and `device`); a
capability revocation performed from the management plane (between
`capability` and `management`). For each: apply the grammar to form a
candidate name, apply the domain subject definition and boundary note, record
the attribution and the rule that decided it, and confirm no registry entry
and no payload was created.

**Acceptance:** every drill item resolves to exactly one domain by the
written rules, and every candidate name conforms to the grammar or is
rejected by it.  
**Failure/blocker:** a drill item with no decided attribution or two equally
valid attributions shows a subject definition gap — stop and raise it; do
not patch the definition ad hoc in the drill record.

### Step 5 — closure review

Work: run the review matrix below, confirm the handoff checklist, and verify
the package against its task-book requirement (P0-V09), prerequisite
compatibility (W05 conventions; W12 boundaries), document links, and
downstream handoff wording. Completion is claimed only in the verification
record, with evidence, and only for what was actually run.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W13-DV01 → P0-V09 | domain registry review | inspect the registry against the plan's fourteen areas and the contract §3 required content | all fourteen domains present with subject definitions and the required boundary notes | a complete classification space exists; not that events exist |
| W13-DV02 → P0-V09 | naming and compatibility review | inspect grammar, declaration rule, empty registry, and compatibility rules against contract §4–§6 | grammar complete; registry empty; declaration and deprecation mechanics explicit; namespace version stated | names are governed as interfaces; not that any telemetry serializes them |
| W13-DV03 → P0-V09 | boundary review | inspect the §7 rules in the document and the W12 reference | one-fact-one-name rule present; channel/metrics references point at W12 ownership without restating it | duplicate naming is foreclosed; not W12's channel semantics (W12 owns them) |
| W13-DV04 → P0-V09 | attribution drill | step 4 procedure on the five hypothetical categories | one domain per item, decided by written rules; grammar conformance checked; zero registry entries | consistent classifiability of new events; not that the drill categories are real events |
| W13-DV05 → P0-V09 | discovery and link review | resolve routing row and index row from a fresh checkout | one-link reachability; truthful status; links resolve | documentation navigation; not W05's taxonomy |
| W13-DV06 → W13 closure | consumability review | read the namespace document as a P1 telemetry-design author declaring the first events | the author can form, declare, and version names without inventing policy, and knows payloads/encoding are theirs | handoff readiness; not that any P1 design is done |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. No validation here
proves P0-V01–P0-V08, P0-V10–P0-V15, and none may be reported as doing so.

## 4. Error, security, and observability model

W13 adds no runtime error path, synchronization, or `unsafe` code. Its
failure reporting is textual: a missing domain definition, a grammar gap, an
undeclared name in a later design, or an unclassifiable drill item fails the
associated review and is recorded as such.

Security positions: event names are part of the observable surface, so the
grammar's prohibition on embedding IDs, secrets-adjacent detail, and
platform-specific names (contract §4) keeps names safe to expose in logs and
tooling; the declaration rule keeps third parties from inferring behavior
from undeclared string churn. Observability is reflexive: the namespace
exists so future telemetry is consistent, and its own evidence trail
(registry state, drill record, run/not-run entries) is the only accepted
proof surface.

## 5. Handoff checklist

Before handing W13 to a reviewer, provide:

- the exact changed-file list;
- DV01–DV06 evidence paths and their run status, including the drill record
  and any blocked-by-prerequisite note for W12's boundary language;
- confirmation that the canonical-event registry has zero entries and that
  no event, field, payload, encoding, transport, or code surface was
  declared or authorized;
- the recorded convention source from step 1 and any conflict raised; and
- open items for P1+ telemetry designs (first declarations, encoding
  choices), W07/W20 (future registry-consistency check), and W05 (possible
  re-homing) — without resolving their contracts here.
