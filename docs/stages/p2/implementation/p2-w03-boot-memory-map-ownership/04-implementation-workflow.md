# P2-W03 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W03 detailed design](README.md).

## 1. Preconditions and failure boundary

Load the documents named in the [entry README](README.md); inspect the
tracked tree. Expected state: no workspace; W01/W02 designed but not
implemented; W03 develops host-first over injected fact records, with a
fixture factory producing synthetic `PlatformInfo`-shaped inputs.

Stop and obtain direction instead of guessing when:

- the W02 fact record shapes differ from those assumed here — that is a
  W02 contract revision (design conflict), not a local adapter hack;
- the P1 image range cannot be represented or is absent — blocked
  prerequisite per [01 §3](01-scope-and-foundations.md) A3; protection
  must not be skipped or defaulted;
- implementing the seal seems to require designing W04's metadata
  placement — stop; W03 validates plans, W04 chooses them;
- a fix would require an unprotect/reclassify operation — Reserved
  (README Decision 7); superseding design required;
- anything reaches for `MemoryObject`/`MemoryRegion` — blocked by
  P2-ACR-01 ([01 §2](01-scope-and-foundations.md)).

## 2. Ordered implementation steps

### Step 1 — frame-span newtypes and predicates

Target: `bootmap::ranges`, contracts
[03 §1](03-code-contracts-bootmap.md).

Work: `PhysFrameNum`/`PageCount`/`PhysFrameRange` (or adapted P0
primitives), `to_frames`, spatial predicates; fixture set for conversion
boundaries (unaligned base/len, overflow, adjacency).

**Acceptance:** no raw integer can become a frame span outside the
constructors (review check); all boundary fixtures pass; zero heap.  
**Failure/blocker:** if P0 already ships conflicting address types, adopt
them and record the deviation — do not maintain parallel type systems.

### Step 2 — `ProtectedSet` assembly

Target: `bootmap::classify`, contract [03 §3](03-code-contracts-bootmap.md).

Work: the five protection sources with stable identities; R6/R7/R9/R10;
capacity bound.

**Acceptance:** conflict, duplicate, zero-size, and out-of-RAM fixtures
produce the exact outcomes of the
[02 §4](02-architecture-and-state.md) table; identities preserved for P4
extension.  
**Failure/blocker:** any silent precedence between protection sources —
stop; that is Decision-3 territory requiring a design change.

### Step 3 — draft builder (RAM integration)

Target: `bootmap::build`, contract [03 §4](03-code-contracts-bootmap.md).

Work: bank conversion, sorted merge, interval subtraction (clipping),
entry assembly, invariants.

**Acceptance:** multi-bank, adjacent, overlapping, unaligned, zero-size,
and protected-interleaved fixtures all classify exactly per the policy
table; clip log records every shrink; no board/QEMU constants.  
**Failure/blocker:** an invariant that cannot hold without dropping
protection fidelity is a design breach — stop and fix the policy, not the
test.

### Step 4 — draft queries

Target: `bootmap` draft type, contract [03 §5](03-code-contracts-bootmap.md).

Work: `allocatable_spans`, `protected_ranges`, `class_at` over sorted
entries.

**Acceptance:** binary-search queries correct over random span sets
(property-style loop); the draft type has no sealing-capable surface.  
**Failure/blocker:** n/a beyond §1 rules.

### Step 5 — seal and sealed queries

Target: `bootmap::seal`, contracts
[03 §6–§7](03-code-contracts-bootmap.md).

Work: plan validation (R11), reclassification, final audit, summary,
accounting equation.

**Acceptance:** valid seals produce sealed maps whose protected superset
equals protected + plan (audit independently recomputed in tests); each
R11 violation class rejected; summary equation exact; sealed type offers
`&self` methods only.  
**Failure/blocker:** an audit that cannot be expressed without trusting
the builder's own bookkeeping — restructure so the audit re-derives from
entries, as the contract requires.

### Step 6 — negative-space review

Target: whole module set.

Work: confirm exclusions hold — no allocator calls, no memory objects, no
DTB release, no mutation APIs, no board names, no heap; confirm P2-ACR-01
restated in the implementation record.

**Acceptance:** review note in the implementation record with zero
violations.  
**Failure/blocker:** a hit is removed or escalated per §1.

### Step 7 — host validation pass and evidence

Target: verification record
`../../verification/p2-w03-boot-memory-map-ownership-verification.md`;
implementation record
`../p2-w03-boot-memory-map-ownership-record.md`.

Work: run the matrix of [05 §3](05-validation-and-handoff.md)
(W03-DV01–DV10); statuses incl. not-run (QEMU accounting is W09;
metadata-planning interplay is validated with W04).

**Acceptance:** every row statused with evidence; no completion claim
beyond what ran.  
**Failure/blocker:** failures recorded as failures with diagnosis.

### Step 8 — closure review

Work: read this design as W04 (is the allocation domain derivable and
unambiguous? is the seal handshake implementable?), W06 (are queries
sufficient for a truthful map dump?), W09 (are accounting totals
checkable?), P4-via-W10 (are identities and extension points sufficient
without objects?); confirm the handoff checklist of
[05 §4](05-validation-and-handoff.md).
