# P4-W03 Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W03 detailed design](README.md).

## 1. Preconditions and failure boundary

Before coding, the implementer verifies the Coding Guidelines preflight and
the [P4-W01](../p4-w01-entry-contract-reconciliation/README.md) entry review
status for rows R07–R12 (P2 platform/memory/allocation), R04 (P1 Host
Stage-1), R18–R20 (P0 toolchain/newtypes/logging). Per
[01 §2](01-scope-and-foundations.md), host-testable steps may proceed against
assumed interfaces (M1/M2 signatures), but acceptance evidence that depends on
missing upstream delivery is recorded as blocked, never as passed.

Stop and record instead of improvising when:

- the delivered P2 allocator cannot provide a contiguous page run (M2/D1) —
  record the conflict; do not build a scatter-gather loader in W03 (that is a
  design change);
- the ownership-accounting extension point does not exist (M3) — apply the
  recorded degradation and add the W01 gap-list entry; do not invent an
  accounting scheme;
- Host Stage-1 does not provide writable access to the allocated range (M4) —
  record the dependency; do not add W03-owned Host mapping code;
- any consumer (W02/W04/W05) duplicated layout constants instead of citing
  `GuestLayout` — that is a review failure in the consumer, fixed by citation,
  not by relaxing the record;
- the image grows past the layout bound during Guest development — raise the
  bound through a layout version bump and consumer review (it is a test
  contract, W01 A4); never relax a validation rule silently.

## 2. Ordered implementation steps

### Step 1 — layout record

Target: `gm-layout`.

Work: define the `GuestLayout` constants and validation per
[03 §1](03-code-contracts-guest-memory.md), sizing defaults against the
P2-declared reference RAM facts (M7), and the invariant test suite.

**Acceptance:** `validate` passes on the recorded values and fails on each
mutated invariant (disjointness, alignment, containment) in tests.  
**Failure/blocker:** a value that cannot satisfy invariants given reference
RAM facts is a design issue — re-derive values and record the derivation, do
not drop an invariant.

### Step 2 — Guest RAM object

Target: `gm-ram` allocate/init_zeroed/mapping_grants/release and the write
view.

Work: implement per [03 §3](03-code-contracts-guest-memory.md) against the
assumed allocator interface; add the boundary unit tests and the `unsafe`
inventory entry for the write view.

**Acceptance:** allocation/OOM/accounting tests pass; zero-init and re-init
are byte-deterministic; release requires the unmap proof; write-view
boundary refusals hold.  
**Failure/blocker:** allocator-interface mismatch (M2) stops this step per
§1; the vocabulary-level work (steps 1 and 4) continues unblocked.

### Step 3 — image view and validation plan

Target: `gm-image`.

Work: implement the view type and `validate_image_plan` per
[03 §4](03-code-contracts-guest-memory.md) with the full negative suite.

**Acceptance:** every error class has a test; property checks classify
arbitrary (size, offset) inputs; no rejection path performs writes.  
**Failure/blocker:** a needed error class not in the model is a design
addition (record, then implement), not an ad-hoc error value.

### Step 4 — boot-info block

Target: `gm-loader::BootInfo`.

Work: implement the explicit-layout block, checksum, and round-trip tests
per [03 §2](03-code-contracts-guest-memory.md). Share the format with
[P4-W05](../p4-w05-validation-guest/README.md) by citation (its Guest parser
validates independently).

**Acceptance:** serialize→independent-validate round trips pass; truncation
and corruption are detected Guest-style; determinism (no varying fields).  
**Failure/blocker:** a field-width mismatch discovered against the Guest
implementation is resolved by a layout-version bump and joint review, not by
loosening one side.

### Step 5 — loader procedure

Target: `gm-loader::load_guest`.

Work: implement the fixed-order procedure per
[03 §5](03-code-contracts-guest-memory.md) and the end-to-end host tests
(stubbed RAM), including determinism (two loads byte-identical) and
all-or-nothing failure behavior.

**Acceptance:** host-side load suite passes; `GuestInput` matches the W04
consumer contract.  
**Failure/blocker:** consumer-contract drift (W04 shape change) is resolved
by joint design note, not by informal parameter passing.

### Step 6 — build embedding route

Target: build plumbing that delivers the built Guest artifact as the
`GuestImage` byte slice (M5).

Work: coordinate with the build/target governance owner (W01 R18) so the
flat-binary artifact of [P4-W05](../p4-w05-validation-guest/README.md) is
embedded and addressable; keep the mechanism out of Core.

**Acceptance:** a build produces a Hypervisor image whose `GuestImage` view
length matches the Guest artifact byte length; identity metadata flows for
diagnostics.  
**Failure/blocker:** embedding mechanism unavailable (M5) — the route
decision D3 must be revisited in design (recorded design change), never
worked around with runtime loading.

### Step 7 — on-target construction evidence

Target: QEMU-run construction evidence (P4-V03 inputs) through the
integrated W02+W04+W05 path and recorded via
[P4-W08](../p4-w08-qemu-integration-regression/README.md).

Work: exercise the full construction order of
[02 §4](02-architecture-and-state.md) on target; confirm deterministic
initialization across repeat runs (with W07 scenarios).

**Acceptance:** RAM-origin (allocator-only) and forbidden-overwrite defenses
demonstrated by the negative scenarios; deterministic reinitialization shown
for the declared repeat form.  
**Failure/blocker:** on-target failures diagnose through W06 fault context;
record as failed with diagnosis; never loosen validation to pass.

### Step 8 — closure review

Work: run the [05](05-validation-and-handoff.md) matrix and handoff
checklist; write implementation-record facts (route selected, layout values
as test facts, limitations for W09).

**Acceptance:** matrix reviewed; handoff complete; no completion claims
outside the verification record.

## 3. Evidence destinations

- Implementation facts (route decision, layout version, limitations):
  `../p4-w03-guest-memory-image-record.md` (created when work starts).
- Commands, environments, results, run/not-run:
  `../../verification/p4-w03-guest-memory-image-verification.md`.
- Unsafe inventory delta (write view): P0 inventory location (W01 R20),
  linked from the implementation record.
