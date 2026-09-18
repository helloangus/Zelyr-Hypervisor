# P0-W12 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P0-W12 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the documents
named in the parent README and inspects the current tracked tree. Useful
read-only discovery: `git ls-files` (confirm no diagnostics-baseline document
exists yet) and a search of tracked documents for existing diagnostic
statements (the ADR's observability entries, the Coding Guidelines' telemetry
rule, the sibling designs' boundary pointers). No Rust source exists, so no
logging call exists to migrate or govern.

Stop and obtain direction instead of guessing when any of the following
occurs:

- a prerequisite design (W04, W05, W06) or a consumer design (W13, W16) has
  delivered a contract that conflicts with the channel, level, visibility, or
  identity statements of this design — raise the cross-package conflict; do
  not re-decide the sibling's subject here;
- implementing appears to require a transport, buffer, macro surface, crate,
  or code of any kind — those are Reserved to P1+ designs, and reaching for
  them is a scope violation;
- a maintainer requests semantics that drop compile-time trimmability or
  runtime filterability, or remove an ADR-048 coverage area — that is the ADR
  path per [contract 01](01-diagnostic-channels-and-levels.md) §5; or
- the level set or channel set is challenged as arbitrary — the set is this
  design's resolved decision within the plan's mandate; changing it is a
  policy decision under contract 01 §5, not an implementation-time edit.

## 2. Ordered implementation steps

### Step 1 — verify prerequisite and sibling surfaces

Target: implementation record (created in this step).

Work: record which sibling designs have delivered documents (W04–W06
prerequisites; W13/W16 consumers), which convention source this
implementation follows, and any conflict surface found — in particular
whether W14's failure-classification contract exists yet (it is referenced by
subject in [the crash contract](02-crash-identity-and-consumer-constraints.md)).

**Acceptance:** the record names the convention source and every
blocked-by-prerequisite surface.  
**Failure/blocker:** an unresolved governing conflict stops the work per §1.

### Step 2 — author the diagnostics baseline document

Target: `docs/development/diagnostics-baseline.md`.

Work: write the document with the status header and all content required by
[contract 01](01-diagnostic-channels-and-levels.md) and
[contract 02](02-crash-identity-and-consumer-constraints.md): four channels
with non-substitution lists, the five-level set, three visibility classes
with the trimming rules, the fatal-information minimums, the identity
association property, the transitional early-console rule, the M/N constraint
lists, and the boundary table. No macro, type, transport, or event name may
appear as an authorized surface.

**Acceptance:** every required section is present; the document names no
code identifier, transport, or trace-event name; it contradicts no ADR,
task-book, or Coding-Guidelines rule.  
**Failure/blocker:** a contradiction is raised per §1, not absorbed by
rewording.

### Step 3 — wire discovery

Targets: `docs/README.md`, `docs/stages/p0/implementation/README.md`.

Work: add one routing row to `docs/README.md` pointing diagnostic/logging/
telemetry semantic questions at the baseline document; add the W12 design row
to the stage implementation index with status "Proposed design;
implementation not claimed" (updated truthfully as work proceeds). Change
nothing else in these files.

**Acceptance:** a contributor starting from `docs/README.md` reaches the
baseline in one link; links resolve from a fresh checkout.  
**Failure/blocker:** a broken or duplicating link fails review.

### Step 4 — constraint walkthrough

Target: verification record.

Work: execute the walkthrough of W12-DV05 against clearly hypothetical P1
scenarios, for example: (a) an early-boot stage needs to report progress
before any subsystem exists; (b) a bring-up developer needs register-level
detail on a fault; (c) an operator needs to know a VM started; (d) a
benchmark build must exclude high-frequency events; (e) a fatal exit occurs
during EL2 bring-up. For each, record which channel, level, and visibility
class the baseline assigns and which M/N constraint binds the future design —
then confirm no step required choosing a transport, macro, or event name.

**Acceptance:** every scenario resolves to exactly one governed treatment,
and no step prescribed an implementation.  
**Failure/blocker:** a scenario with no governed treatment or two competing
treatments is a gap in this design — stop and raise it; do not extend the
document ad hoc.

### Step 5 — cross-reviews with W13 and W16

Target: verification record and implementation record.

Work: if the W13 and W16 contracts exist as documents, cross-review this
baseline's channel boundaries against W13's namespace rules and the identity
association property against W16's minimum identity set; record agreement or
the exact conflict surface. If either sibling has not delivered, record the
review as **blocked-by-prerequisite** with the surface named — that is a
truthful closure state, not a failure of this package.

**Acceptance:** each cross-review is recorded as agreed, conflicting, or
blocked-by-prerequisite with specifics.  
**Failure/blocker:** a substantive conflict is raised as a cross-package
design conflict; where an ADR-level property is touched, it is labelled
`ADR Required`.

### Step 6 — closure review

Work: run the review matrix below, confirm the handoff checklist, and verify
the package against its task-book requirement (P0-V09, P0-V14), prerequisite
compatibility, document links, and downstream handoff wording. Completion is
claimed only in the verification record, with evidence, and only for what was
actually run.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W12-DV01 → P0-V09 | channel and level review | inspect the baseline document against contract 01 §2–§3 | four channels with non-substitution lists; five levels with fixed semantics and retention defaults | governed semantics exist; not that any logging code obeys them |
| W12-DV02 → P0-V09 | visibility and trimming review | inspect contract 01 §4 statements in the document | three classes mapped to levels/channels; trimming via W04-classified selections; filterability stated as binding | trimming governance exists; not that a build implements it |
| W12-DV03 → P0-V09/P0-V14 | fatal-minimum review | inspect contract 02 §1 statements in the document | panic and crash minimums present; untrusted-data rule present; no register list or format fixed | minimum principles exist; not that P1-W07 satisfies them |
| W12-DV04 → P0-V14 | identity association review | inspect contract 02 §2; check the cross-review record | association property stated; inline-vs-associable split stated; W16 cross-review agreed or blocked-by-prerequisite with surface | the property is checkable; not that W16's set is final (its contract owns it) |
| W12-DV05 → P0-V09 | constraint walkthrough | step 4 procedure on the five hypothetical scenarios | each scenario has exactly one governed treatment; no implementation prescribed | the baseline constrains P1 without pre-deciding telemetry; not that P1 designs exist |
| W12-DV06 → P0-V09 | discovery and link review | resolve routing row and index row from a fresh checkout | one-link reachability; truthful status; links resolve | documentation navigation; not W05's taxonomy |
| W12-DV07 → W12 closure | consumability review | read the baseline as W13 (do I own naming?), W16 (does my set satisfy the property?), P1-W06 (what does my console design inherit?), P1-W07 (what must my crash dump carry?) | each consumer can act without inventing policy and knows what is not W12's | handoff readiness; not that downstream packages are done |

Record each validation as **passed**, **failed**, **blocked**, or **not run**
with command, input, environment, timestamp, and reason. A step-5 review
blocked by an undelivered sibling contract is recorded as blocked-by-
prerequisite and does not by itself fail W12; a substantive conflict found
does. No validation here proves P0-V01–P0-V08, P0-V10–P0-V13, or P0-V15, and
none may be reported as doing so.

## 4. Error, security, and observability model

W12 adds no runtime error path; its subject is the semantics future error and
diagnostic paths must follow. Three security positions are part of the
governance and must appear in the document: the untrusted-data rule for
fatal output (contract 02 §1.3, applying ADR-007 to diagnostics); the rule
that diagnostics never bypass failure handling (a log line is never the
handling of an error); and information-disclosure restraint — diagnostic
content is designed to be collected and read, so secrets, credentials, and
unexposed addresses must not become diagnostic content (a property the
owning designs implement; this baseline states it as a constraint).

Observability is reflexive here: the baseline is itself the observability
governance, and its evidence trail (review outputs, walkthrough, cross-review
statuses, run/not-run entries in the verification record) is the only
accepted proof surface.

## 5. Handoff checklist

Before handing W12 to a reviewer, provide:

- the exact changed-file list;
- DV01–DV07 evidence paths and their run status, including explicit
  blocked-by-prerequisite entries for the W13/W16 cross-reviews if those
  contracts were not yet delivered;
- confirmation that no transport, buffer, macro surface, crate, module,
  trace-event name, identity field, or CI workflow was added or authorized;
- the recorded constraint source from step 1 and any conflict raised; and
- open items for W13 (namespace rules), W16 (identity set cross-review),
  W14 (taxonomy binding for the fatal channel), P1-W06 (transitional console
  rule and transport binding), and P1-W07 (fatal minimums) — without
  resolving their contracts here.
