# P4-W05 Implementation Workflow

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W05 detailed design](README.md).

## 1. Preconditions and failure boundary

Before coding, the implementer verifies the Coding Guidelines preflight and
the [P4-W01](../p4-w01-entry-contract-reconciliation/README.md) entry review
status for rows R06 (QEMU environment), R18 (toolchain/target), R19/R20
(newtypes/diagnostics governance where applicable to the Guest), plus the
consumer-side contracts M1–M8 of [01 §2](01-scope-and-foundations.md). The
Guest crate's structure and console/bootinfo logic can be developed before
the hypervisor path is runnable; on-target scenario evidence is gated on the
W02/W03/W04 paths existing.

Stop and record instead of improvising when:

- the Guest needs a facility outside the boot-info/console/entry convention
  (for example a timer, a random source, a second output channel) to express
  a scenario — that scenario does not belong to P4; record it as an out-of-
  scope finding rather than extending the boot contract;
- the flat-binary route cannot express a needed construct (for example
  position-independent relocations) — the Guest must be linkable to run at
  `IMAGE_LOAD_IPA` with base-relative code only; if a construct defeats
  that, redesign the scenario, not the route (route change = W03 design
  change);
- expected outcomes in the scenario table disagree with observed W04 exit
  classes — the table and the classification contract are reconciled by
  joint review (M6), and the table version is bumped; never re-label a
  marker to match an unreviewed behavior;
- a planned scenario (VG-008/VG-009/VG-011) cannot be delivered — apply the
  task book deferral rule in the implementation record's deferral section;
  never silently skip it;
- automation (W08) asks for marker or ordering changes — that is a protocol
  version change with joint review (D8), not a local edit.

## 2. Ordered implementation steps

### Step 1 — crate skeleton and build route

Target: the Guest crate (`guests/validation-aarch64` as the recorded
location, per the workspace design) and its flat-binary build.

Work: set up the `no_std` binary crate at the location the P0 workspace
design assigns to the Validation Guest (assumption M1 — the scaffold
directory is not by itself an authority for crate layout), linker placement
at the load convention (M2), `_start` naked entry, and the artifact route
consumable by W03's embedding.

**Acceptance:** the build produces the flat binary artifact; `entry` runs
far enough (under a host-side harness or on target) to be demonstrably
positioned correctly.  
**Failure/blocker:** target/build mismatch (M1) blocks; record; do not
adopt unstable features or a different toolchain.

### Step 2 — console and fmt

Target: `console`, `fmt`.

Work: implement the polled writer over the console-page convention (M4) and
minimal formatting; add the marker-helper grammar wrappers.

**Acceptance:** line-oriented output with CRLF; marker helpers enforce the
grammar; no allocation anywhere.  
**Failure/blocker:** console mapping absent (M4) — console-dependent
scenarios block; proceed with validator/table work and record the blocked
row.

### Step 3 — boot-info validator

Target: `bootinfo`.

Work: implement validation per [03 §2](03-code-contracts-guest.md) with the
full error-class test suite (host-side harness where feasible).

**Acceptance:** every error class has a test; truncation/bit-flip property
checks pass; accessors are bounds-checked at validation time.  
**Failure/blocker:** block-format drift (M3) is resolved by a joint version
bump with W03 — never by tolerance flags on either side.

### Step 4 — scenario framework and mandatory scenario bodies

Target: `scenario`, `halt`, `panic`.

Work: implement dispatch and the mandatory bodies (VG-001–VG-007, VG-010,
VG-012) per [02 §5](02-guest-architecture-and-scenarios.md) and
[03 §4](03-code-contracts-guest.md), with the marker-before-trigger
discipline (D5).

**Acceptance:** every mandatory scenario compiles with its markers and
trigger in place; table/id agreement with W04's validation table holds
(review).  
**Failure/blocker:** a trigger whose outcome is not in W04's class table is
a joint-review item (M6) before proceeding.

### Step 5 — planned scenario bodies

Target: VG-008, VG-009, VG-011.

Work: implement per the table. VG-009 coordinates with W04's re-entry proof
path (its episode sequencing is defined jointly with the W04 run loop).

**Acceptance:** planned scenarios present with markers and triggers; any
deferral recorded per the task book rule (reason, downstream owner,
exit-criterion effect).  
**Failure/blocker:** infeasibility is recorded as the deferral, never as a
silent reduction of the table.

### Step 6 — integration runs on target

Target: on-target scenario evidence via W02/W03/W04 paths, recorded by
[P4-W08](../p4-w08-qemu-integration-regression/README.md).

Work: run each scenario end to end; verify markers, exit classes, stop
results, and EL2 liveness after each run.

**Acceptance:** mandatory scenarios produce their expected marker sequences
and classified exits; repeat runs are stable.  
**Failure/blocker:** mismatches diagnose through the W04 frame and W06;
record as failed with diagnosis; never adjust markers or triggers to mask a
fault-path defect.

### Step 7 — closure review

Work: run the [05](05-validation-and-handoff.md) matrix and handoff
checklist; record facts for W09 (asset location, protocol/table versions,
delivered vs deferred scenarios, maintenance rules).

**Acceptance:** matrix reviewed; handoff complete; no completion claims
outside the verification record.

## 3. Evidence destinations

- Implementation facts (versions, deferral section, asset location):
  `../p4-w05-validation-guest-record.md` (created when work starts).
- Commands, environments, results, run/not-run:
  `../../verification/p4-w05-validation-guest-verification.md`.
- No `unsafe` is expected in the Guest beyond the naked `_start` and the
  MMIO writer; both require SAFETY notes and inventory entries (Guest-side
  inventory linked from the implementation record).
