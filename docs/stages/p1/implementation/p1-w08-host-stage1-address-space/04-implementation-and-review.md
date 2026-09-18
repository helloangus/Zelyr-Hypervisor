# P1-W08 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P1-W08 detailed design](README.md).

## 1. Preconditions and failure boundary

W08 can start only after the accepted W02/W04/W09 contracts are available
as designs and the W05/W06/W07 seams are settled designs, because the
inventory and the failure route are consumed, not invented. Before
changing any file, the implementer verifies the mandatory reading (parent
README), inspects the current tree (`git ls-files`; confirm the P0
workspace/target state and the accepted sibling designs), and records the
assumed-contract states from
[01-architecture-and-state.md](01-architecture-and-state.md) §8.

Stop and obtain direction instead of guessing when any of the following
occurs:

- the P0 layout baseline provides no extension point for the region
  symbols the inventory needs — record the upstream defect; do not fork a
  private linker script;
- a region seam (W05 vector symbols, W06 console region) is absent or
  contradicts the inventory — raise the coordination issue; the inventory
  is never silently widened or narrowed;
- the recorded architecture revision cannot confirm a descriptor or
  `TCR` field behavior the transition relies on — record the uncertainty
  and stop the affected step; encodings are never guessed;
- closure appears to require an allocator, dynamic mapping, Stage-2
  work, huge pages, or a second device window — Out of Scope (parent
  README); stop.

## 2. Ordered implementation steps

### Step 1 — confirm inventory sources and prerequisite seams

Target: implementation record
(`../p1-w08-host-stage1-address-space-record.md`, created in this step).

Work: verify the P0 layout extension points cover every inventory
symbol; verify W04's recorded values and query names for the premise
assertions; verify W05/W06/W07 seams against
[01-architecture-and-state.md](01-architecture-and-state.md) §8; record
the architecture revision, the canonical-path load-address assumption,
and the sizing arithmetic for `STAGE1_TABLES`.

Suggested observation: read the sibling designs and P0 plans; no
repository change.

**Acceptance:** the record states each seam satisfied, or names the gap
and its owner.  
**Failure/blocker:** a seam gap stops the affected step (fail-closed);
no local adaptation.

### Step 2 — implement address types, descriptors, and tables

Target: the stage-1 module (physical placement per the P0 workspace
baseline; recorded).

Work: implement the typed arithmetic (including the recorded `PhysAddr`
extension), `MappingClass`, the descriptor/table types, `build_stage1_tables`,
and `verify_stage1_tables` per
[02-code-contracts-address-types-and-tables.md](02-code-contracts-address-types-and-tables.md).

Suggested observation: host-side unit evidence of build/verify round-
trips and the checked-arithmetic failure paths where the P0 host-test
baseline permits; otherwise contract inspection.

**Acceptance:** build/verify closure holds (set equality both ways); no
naked-integer address computation; every `unsafe` block carries its
`SAFETY` justification for the P0 unsafe inventory.  
**Failure/blocker:** an inventory inconsistency stops the step; the
tables are never enabled on an unresolved verify.

### Step 3 — implement the class assignment and transition

Target: the stage-1 module.

Work: implement `MAPPED_REGIONS` with the §1 assignments, the sysreg/asm
boundary, and `enable_host_stage1()` with its recorded values, barriers,
and post-MMU verification per
[03-code-contracts-mapping-and-transition.md](03-code-contracts-mapping-and-transition.md).

**Acceptance:** the step order matches the contract; the enable is
single-shot and read-back-verified; `SCTLR_EL2` supersession is recorded
as this design's change (never a silent W04 edit).  
**Failure/blocker:** a recorded-value uncertainty stops the step per §1;
no guessed encoding.

### Step 4 — wire the continuity and failure seams

Target: the stage-1 module plus the recorded contract points in consumer
wiring (owned by those packages).

Work: confirm the premise assertion sources; confirm the `fail_stage1`
→ `fail_phase` → W07 route; confirm the NC5 continuation point with
W11's contract; confirm the vector/console continuity rows with W05/W06.

**Acceptance:** every seam is wired at its contracted point; W08-owned
code touches no other package's mechanism.  
**Failure/blocker:** a missing consumer design records the deferred link
with its owner; no stub wiring.

### Step 5 — constraint and layering review

Target: implementation record; this design's review tables.

Work: run the prohibited-attribute rules P1–P6 against the class model
and the verification gate; run the no-hidden-identity-map check (every
code access through symbols; the assumption recorded; no literal
addresses); run the layering check (no board-name branch; facts queried,
never re-read; Core-facing seams API-bound); walk the boundary inventory.

**Acceptance:** every rule passes with a pointer to code or record; any
failure is a recorded P1-V13/P1-V19 finding.  
**Failure/blocker:** an unremovable violation stops the package.

### Step 6 — acceptance evidence and closure

Target: verification record
(`../../verification/p1-w08-host-stage1-address-space-verification.md`).

Work: perform the executable reviews of §3; record the deferred
executions (post-MMU boot behavior → W10 R1; NC5 post-MMU fault → W11)
with their owning packages; complete the handoff checklist.

**Acceptance:** the verification record distinguishes passed reviews,
deferred executions, and not-run items.  
**Failure/blocker:** a failed review is recorded as failed with
diagnosis; completion is not claimed around it.

## 3. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W08-DV01 → P1-V13 | Inventory review | inspect architecture §2 against the W02–W07 sources and the P0 layout points | every mapped region traced to a seam; the not-mapped list recorded; sizes from symbols/constants only | the inventory is complete for P1's runtime; not that no code ever touches an unmapped address (NC5's subject) |
| W08-DV02 → P1-V13 | Class-model review | inspect the class table, `MappingClass`, and descriptor encoding against the recorded revision | six closed classes; semantics total; encodings reserved-bit-respecting; typed arithmetic throughout | attributes are explicit and encodable as recorded; not hardware translation behavior |
| W08-DV03 → P1-V14 | Transition review | inspect `enable_host_stage1()` against §3 of [mapping and transition](03-code-contracts-mapping-and-transition.md) | premise-asserted; ordered; barriers per the recorded reasoning; single enable; read-backs; no retry | a controlled transition by design; not integrated boot success (W10) |
| W08-DV04 → P1-V14 | Continuity review | inspect architecture §6 rows and the §3 wiring | every consumer window mapped pre-enable; VBAR unchanged; failure route armed | continuity by contract; not post-MMU liveness itself |
| W08-DV05 → P1-V13, supports P1-V19 | No-RWX / no-hidden-identity-map review | P1–P6 walk; tree search for literal addresses and unmapped-region access; boundary inventory | no ambiguous attribute expressible; verification gate rejects them mechanically; identity assumption recorded with symbol discipline | the containment and assumption properties as designed; not hardware fault behavior (NC5) |
| W08-DV06 → P1-V13/P1-V14 | Post-MMU environment and consumability review | read the outputs as W09 (phase body + route), W10 (R1 markers), W11 (NC5 point), W12 (assumptions), P2 (supersession obligations) | each consumer can act without inventing W08 policy; the §7 environment statement is reviewable | handoff readiness; not downstream or stage completion |
| — (deferred) → P1-V13/P1-V14 | Executed evidence | W10 R1 (boot reaches stable post-MMU; markers continue) and W11 NC5 (unmapped access faults into the armed report) | per those packages' matrices | the executed half of the acceptance wording; owned by W10/W11 |

Record each validation as **passed**, **failed**, **blocked**, or **not
run** with command, input, environment, timestamp, and reason. The
boot-dependent proofs are deferred to W10/W11 by contracted wiring, not
omitted; until they exist, P1-V13's and P1-V14's executed halves are
unproven and no W08 artifact may report otherwise. No validation here
proves P1-V15 through P1-V21.

## 4. Error, security, and observability model

**Errors.** W08 has one failure posture: `Stage1Error` → terminal fatal
report with `stage1` attribution and a step reason — no retry, no
rollback, no partial enable. A fault during the sequence reaches the
same terminal state through W05's vectors and the armed report. The
degraded state after a failure is the pre-MMU world, unchanged: the
tables were built but nothing was enabled that could half-run.

**Security.** The mapping classes are the stage's containment hardware:
no writable code or vectors, no executable data or stack, no Device-X,
no RWX — enforced by the class model, mechanically checked by table
verification, and never bypassed by a code path (no literal-address
access). Unmapped access by P1 code is a fault into the armed report,
which is the desired loud failure. `unsafe` is confined to the closed
sysreg set and the barrier/TLB/I-cache asm wrappers, each with `SAFETY`
justifications in the P0 unsafe inventory; table memory itself is safe
Rust over private statics. Capability-driven configuration (facts
queried, no platform names) keeps the layering invariant.

**Observability.** The transition is observable through the phase
records (W09 markers, pre- and post-MMU), the failure route's step
reasons (the transition-state vocabulary), and the post-`stage1` marker
as the console-liveness proof. W08 adds no prints of its own and no
telemetry; mapping dumps or region inspection are later-stage tools
(P2-W06's named interest, via W12's handoff).

## 5. Handoff checklist

Before handing W08 to a reviewer, provide:

- the exact changed-file list and the module locations of every
  contracted item, including the recorded `PhysAddr` extension;
- the assumed-contract table as observed
  ([01-architecture-and-state.md](01-architecture-and-state.md) §8),
  including any recorded blocker or seam deviation;
- W08-DV01..DV06 evidence paths and run status, including the explicit
  deferred/not-run entries (integrated post-MMU boot → W10 R1; NC5 →
  W11);
- the implementation-selected values: architecture revision and recorded
  descriptor/`TCR`/`SCTLR` values, table sizing arithmetic, load-address
  assumption, step-reason statics, the W03 trigger exercise note;
- confirmation that all new `unsafe` is confined to the named sysreg and
  asm primitives with `SAFETY` justifications filed in the P0 unsafe
  inventory process;
- confirmation that no allocator, map/unmap service, Stage-2 register
  work, huge page, second device window, literal-address access, or
  public ABI was introduced;
- open items: P2 supersession obligations (relocation, discovered
  memory) recorded for W12 — not designed here.
