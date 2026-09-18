# P2-W02 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W02 detailed design](README.md).

## 1. Preconditions and failure boundary

The implementer loads the documents named in the
[entry README](README.md) and inspects the tracked tree. Expected state: no
workspace, no W01 code yet; W02 is developed host-first against a small
fixture builder that assembles DTBs in memory (reusing W01's validator to
certify fixtures, so tests exercise the real intake→discovery chain).

Stop and obtain direction instead of guessing when:

- a needed W01 cursor capability is missing (subtree skip, sibling
  iteration) — a W01 contract revision is required; record a design
  conflict, do not reach around the cursor;
- a fixture needs a binding not listed in
  [01 §5](01-scope-and-foundations.md) — extending the supported bindings
  is a design change with a rationale, not a local edit;
- host tests would need allocation — the no-heap policy
  ([01 §3](01-scope-and-foundations.md)) is binding; restructure the test;
- anything suggests board-specific behavior — prohibited
  ([01 §8](01-scope-and-foundations.md)); stop.

## 2. Ordered implementation steps

### Step 1 — `dt_cells` decoders with the width policy

Target: `dt_cells` module, contracts
[03 §1](03-code-contracts-decoders.md).

Work: `CellsParams` resolution, `decode_reg_entries`, `read_string_prop`,
`compatible_matches`, with checked composition and bounded copies. Fixture
set: cell-width boundaries, entry multiple checks, string cap/termination,
compatible near-misses.

**Acceptance:** all decoder units pass on host; width policy (1–2)
enforced at construction; zero heap.  
**Failure/blocker:** a decode case needing width > 2 or heuristics is a
design change request — stop per §1.

### Step 2 — CPU inventory and boot-CPU match

Target: `discovery::cpu`, contract [03 §3](03-code-contracts-decoders.md).

Work: `/cpus` walk, status/enable-method defaults, capacity enforcement,
boot-CPU matching recorded as a result for normalization to enforce.

**Acceptance:** fixtures for enabled/disabled/unknown entries, missing
reg, capacity overflow, matched and unmatched boot CPU; DT order
preserved.  
**Failure/blocker:** any temptation to make an individual CPU entry fatal —
that contradicts
[01 §6](01-scope-and-foundations.md); only collection-level facts are
fatal.

### Step 3 — memory and reserved walkers

Target: `discovery::memory`, `discovery::reserved`, contracts
[03 §4–§5](03-code-contracts-decoders.md).

Work: `/memory@*` and `/reserved-memory` walks, rsvmap passthrough,
zero-length handling, source tagging.

**Acceptance:** multi-bank, zero-size, missing-reg, and capacity fixtures
pass; records verbatim; no alignment/overlap judgment present (a review
check — that is W03's).  
**Failure/blocker:** any range claim or overlap resolution appearing here
is scope growth into W03 — remove it.

### Step 4 — singleton fact walks

Target: `discovery::devices` (GIC, timer, PSCI), `discovery::chosen`,
contracts [04 §3](04-code-contracts-facts.md).

Work: four walks with their state outcomes; alias resolution (one hop);
initrd artifact extraction.

**Acceptance:** every outcome class of each walk has a fixture
(Usable/Unsupported/Unusable/Absent per fact); no walk can return a fatal
error; first-candidate rule observable.  
**Failure/blocker:** a required-but-missing detail that seems fatal —
check [01 §6](01-scope-and-foundations.md): only CPU/memory/capacity facts
are fatal.

### Step 5 — normalization assembly

Target: `normalize`, contract [04 §4](04-code-contracts-facts.md).

Work: pipeline order, fatal-fact enforcement, capability projection,
counters, assembly.

**Acceptance:** fatal cases each produce their diagnostic with nothing
published; capability projection mirrors facts exactly (property-style
cross-check over the fixture corpus); two runs over identical fixtures
produce identical records (determinism, W02-DV10).  
**Failure/blocker:** a projection/fact mismatch is a contract breach — fix
the projection, never the test.

### Step 6 — board-name and layering review pass

Target: whole module set.

Work: mechanical review per
[01 §8](01-scope-and-foundations.md) — no platform/board/QEMU strings, no
arch registers, no SoC constants; only fact states drive differences; the
only hardware-facing dependency is the W01 handle.

**Acceptance:** review note recorded in the implementation record; zero
violations.  
**Failure/blocker:** any hit is fixed by redesigning the fact, not by
renaming the constant.

### Step 7 — host validation pass and evidence

Target: verification record
`../../verification/p2-w02-platform-discovery-normalization-verification.md`
(when evidence exists) and implementation record
`../p2-w02-platform-discovery-normalization-record.md`.

Work: run the matrix of [06 §3](06-validation-and-handoff.md)
(W02-DV01–DV10); record statuses incl. not-run (QEMU, real fixtures).

**Acceptance:** every row statused with evidence; no completion claim
beyond what ran.  
**Failure/blocker:** failures are recorded as failures with diagnosis.

### Step 8 — closure review

Work: read this design as W03 (can I get banks/reservations/artifacts?),
W06 (can I render every fact?), W07 (can I run walkers on fixtures?),
P3-via-W10 (are CPU/PSCI/boot-CPU facts sufficient and honest?); confirm
handoff checklist of [06 §4](06-validation-and-handoff.md).
