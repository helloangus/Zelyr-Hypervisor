# P2-W01 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W01 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the documents
named in the [entry README](README.md) and inspects the tracked tree
(`git ls-files`): as of this design there is no Cargo workspace, no crate, and
no P1 runtime. The package therefore starts as host-testable pure logic with
the P1-facing edges (A1–A3 of [01 §2](01-intake-boundary.md)) expressed as
injected inputs behind small trait/function boundaries, so implementation can
proceed and be unit-tested before P1 lands.

Stop and obtain direction instead of guessing when any of the following
occurs:

- a P0/P1 assumed contract cannot be represented (e.g., no address newtype
  exists yet from the P0 base crate) — record the upstream defect; do not
  substitute naked `usize` addresses (Coding Guidelines);
- implementing an edge requires choosing a workspace, crate, or target —
  that is P0-W03 scope; keep modules logical and placement-neutral until the
  workspace exists, and record the physical-placement TODO in the
  implementation record;
- a validator seems to need semantics (cells, `reg`, compatibles) — that is
  W02 scope; stop, do not grow intake;
- an authority asks to copy/relocate or release the DTB — Reserved; a
  superseding design is required, not a local edit;
- a test seems to require QEMU — QEMU evidence belongs to W09; keep this
  package's evidence host-side.

## 2. Ordered implementation steps

### Step 1 — fix the module skeletons and the injected edges

Target: the five logical modules of
[02 §1](02-architecture-and-state.md) in their platform-layer home (logical
module tree; physical placement recorded as a TODO until P0-W03 lands).

Work: define the types of [03 §1](03-code-contracts-intake.md) and the
injected-edge signatures (`HostPhysicalRead` window, image-range input) so
everything above `fdt_access` compiles host-side with no `unsafe`. Why first:
it freezes the boundary surface the rest of the package fills in.

**Acceptance:** the type set exists, no `unsafe` outside `fdt_access`, no
dependency on arch/board/SoC items, `cargo test`-shaped host entry compiles
under the P0-W02 toolchain once the workspace exists (before that: the
modules exist as reviewed source).  
**Failure/blocker:** if a type cannot be expressed without a P0 primitive
that does not exist, stop per §1; do not invent the primitive.

### Step 2 — implement `fdt_header::validate`

Target: `fdt_header` module, contract [03 §3.1](03-code-contracts-intake.md).

Work: implement header decode and span checks with checked arithmetic and
big-endian byte composition. Add fixtures for every `DtbHeaderInvalid` field
class, `DtbSizeInvalid` mismatch/cap, and the version-policy rejections.

**Acceptance:** every header failure mode in
[01 §3](01-intake-boundary.md) has a host fixture asserting its diagnostic;
valid v17 headers pass; malformed inputs never panic.  
**Failure/blocker:** a fixture that cannot be expressed without relaxing a
check is a design conflict — stop and record; do not weaken the check.

### Step 3 — implement `fdt_structure::validate` and `StructureCursor`

Target: `fdt_structure` module, contracts
[03 §4.1–§4.2](03-code-contracts-intake.md).

Work: iterative token-stream validator with caps and anomaly recording; then
the cursor whose accessors are certified by the validator. Build a small
fixture builder that emits valid DTBs (root + known-shape nodes) and mutated
invalid ones.

**Acceptance:** property-style host run over many mutated blobs produces
only diagnostics or certified cursors — no panic, no OOB (test asserts via
debug-check accessors); unknown-node cases yield anomalies or silence per
[01 §6](01-intake-boundary.md).  
**Failure/blocker:** any case where the cursor can address outside the blob
is a critical contract breach: stop implementation, fix the type design.

### Step 4 — implement `fdt_reservation::validate`

Target: `fdt_reservation` module, contract
[03 §5.1](03-code-contracts-intake.md).

Work: terminated-list validation, fixed-capacity collection, zero-address
flagging.

**Acceptance:** unterminated, capped, and span-overflow lists yield
`DtbReservationInvalid` with the right detail; valid lists round-trip
verbatim including zero-address entries.  
**Failure/blocker:** as Step 3.

### Step 5 — implement `intake::validate` and the access boundary

Target: `intake` + `fdt_access` modules, contracts
[03 §2.1, §5.2](03-code-contracts-intake.md).

Work: placement checks in the fixed order; the single `unsafe` slice
fabrication with its `SAFETY` argument; the blocked-prerequisite stop path
when A3 is absent; diagnostic funneling; handle publication.

**Acceptance:** placement fixtures (absent, misaligned, unreachable,
oversized, overlapping) assert their classes and never read a byte past the
checks (a counting fake window proves zero reads after a placement failure);
the unsafe inventory gains exactly one entry with the `SAFETY` argument; the
orchestrator's stage order matches
[03 §5.2](03-code-contracts-intake.md).  
**Failure/blocker:** any second `unsafe` site is a governance violation —
stop and redesign; an A2-contract mismatch discovered here is an upstream
defect to record.

### Step 6 — host validation pass and evidence

Target: verification record
`../../verification/p2-w01-boot-platform-description-intake-verification.md`
(created when evidence exists).

Work: run the validation matrix of
[05 §3](05-validation-and-handoff.md) (W01-DV01–DV09) host-side; record
commands, environments, results, and explicit not-run entries (QEMU paths).
Complete the implementation record
`../p2-w01-boot-platform-description-intake-record.md` with changed
artifacts, new unsafe entries, and deviations.

**Acceptance:** every matrix row has a status (passed/failed/blocked/not
run) with evidence; no completion claim beyond what ran.  
**Failure/blocker:** a failed validation is recorded as failed with
diagnosis — it is evidence, not something to make pass by weakening checks.

### Step 7 — closure review

Work: run the handoff self-review of
[05 §4](05-validation-and-handoff.md); verify W02/W07/W08/W09 consumability
by reading this design as each consumer; confirm no scope growth
(discovery, map, allocator APIs absent); confirm P2-ACR-01 untouched by this
package (it does not concern W01, but the record must not claim otherwise).

**Acceptance:** closure review notes exist in the implementation record;
completion itself is claimable only in the verification record, for what
actually ran.

## 3. Non-responsibility reminders for the implementer

No platform-fact types; no memory ranges claimed; no allocator calls; no
console writes outside the P0 diagnostic channel; no board/QEMU names; no
new dependencies; no toolchain/target changes; no edits outside this
package's modules plus their host fixtures.
