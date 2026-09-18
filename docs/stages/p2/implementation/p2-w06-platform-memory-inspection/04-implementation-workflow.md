# P2-W06 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P2-W06 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the
documents named in the parent README and inspects the actual tree: the
W02–W05 implementations this design consumes must exist with the published
contracts ([01 §2](01-scope-and-foundations.md) A1–A4) and their own
verification records. W06 is sequenced after W05 in the stage map; if any
source package is unimplemented, W06 code work is blocked — record the
blocked state per task book §2 (upstream defect); do not stub sources,
duplicate their logic, or build against invented types.

Stop and obtain direction instead of guessing when:

- a source contract diverges from the assumed shape (e.g., a stats query
  turns out to be `&mut self`) — raise a design conflict against the
  sibling design; do not adapt silently with wrapper mutations;
- P1 has delivered no boot-context record (A5) — implement with the
  `not recorded` marker path and mark the boot-context section
  blocked-by-upstream in the verification record; do not read architecture
  registers inside W06;
- making evidence appear to pass seems to require hard-coded expected
  values inside production code — that is precisely the P2-H04 defect;
  expectations belong in tests only;
- a consumer asks for a CLI, export format, or telemetry emission —
  Reserved (README Decision 7); route the request to a new design.

## 2. Ordered implementation steps

### Step 1 — section types and builders

Target: `inspect::sections` module.

Work: define the six section records and the per-source builders exactly
per [03 §3](03-code-contracts-inspection.md), with capacity constants
matched to the sources'. Rationale: sections are the only place source
values are touched, keeping the projection policy reviewable in one module.

Suggested observation: host build under the pinned toolchain; builder unit
runs over W02–W05 test fixtures that already exist in those packages' test
areas.

**Acceptance:** every section field names exactly one source field; fact
states render verbatim; no content strings are copied.  
**Failure/blocker:** a needed source accessor that does not exist is a
sibling-design conflict (§1), not a reason to reach into source internals.

### Step 2 — consistency registry

Target: `inspect::consistency` module.

Work: implement C1–C4 as pure functions per
[03 §4](03-code-contracts-inspection.md) and fix their order. Rationale:
P2-H04's divergence detection must be a named, individually assertable set
— not an ad-hoc comparison buried in rendering.

**Acceptance:** each check has a unit demonstration of pass and of its
distinct failure detail, using synthetic divergences built from real
fixture sections.  
**Failure/blocker:** a check that cannot be expressed from published
values indicates a missing source query — sibling-design conflict, stop.

### Step 3 — composition entry point

Target: `inspect::compose`.

Work: implement per [03 §2](03-code-contracts-inspection.md): fixed fill
order, fixed check order, all-or-nothing publication, no allocation.

**Acceptance:** compose over the packages' existing end-to-end fixture
chain (validated blob → facts → draft/seal → allocator → heap) returns a
report; injected divergence at any source returns the right
`ConsistencyBroken` ID with nothing published.  
**Failure/blocker:** an inconsistency discovered in the *sources* under
composition is evidence of an upstream defect — record it against the
owning package; W06 must not smooth it over.

### Step 4 — renderer

Target: `inspect::render`.

Work: implement per [03 §5](03-code-contracts-inspection.md) with the
bounded line buffer and `fmt::Write` sink. Rationale: rendering is
isolated so byte-format changes never touch semantics.

**Acceptance:** rendering the same report twice yields identical bytes;
section order matches [02 §2](02-architecture-and-state.md); no
allocation (checked by the project's allocation-failure test hooks if the
P0 baseline provides them, else by construction review).  
**Failure/blocker:** format drift versus [02 §2](02-architecture-and-state.md)
fails review; fix the renderer, and check whether any already-written W09
expectation drafts need updating.

### Step 5 — boot diagnostics wiring point

Target: the boot sequence's inspection call site (owned by the stage's
integration code; W06 provides the function only).

Work: expose one documented call point where boot diagnostics compose and
render the report after W05 initialization, binding the P0 diagnostic
channel as the sink. Rationale: W09's evidence needs a stable place where
the render appears in the boot log; W06 itself still owns no logger API.

**Acceptance:** on a host-side boot-simulation fixture, the diagnostic
point emits the render; the call site compiles target-side (execution
evidence is W09's, not W06's).  
**Failure/blocker:** if the P0 diagnostic channel (A6) is not integrated
yet, record blocked-by-upstream for the *wiring* only; the function-level
evidence (Steps 1–4) stands alone.

### Step 6 — validation and closure

Work: run the matrix in
[05-validation-and-handoff.md](05-validation-and-handoff.md), complete the
handoff checklist, record decisions and deviations in
`../p2-w06-platform-memory-inspection-record.md` and evidence in
`../../verification/p2-w06-platform-memory-inspection-verification.md`
(created when the work starts). Completion is claimed only in the
verification record, only for what actually ran.

## 3. Evidence destinations

| Evidence | Destination | Created when |
|---|---|---|
| Implementation decisions, deviations, changed files | `../p2-w06-platform-memory-inspection-record.md` | implementation starts |
| Command/output/environment per validation row | `../../verification/p2-w06-platform-memory-inspection-verification.md` | validation runs |
| QEMU render evidence (baseline + repeated boots) | W09's verification record, cross-referenced | W09 executes |

Nothing in this design creates or pre-fills those files.
