# P8-W09 Implementation Workflow and Acceptance Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W09 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the parent
README, the [Coding Guidelines](../../../../development/coding-guidelines.md)
(preflight included), and the sibling contracts cited there. Useful
read-only discovery: `git ls-files` (confirm the actual crate layout) and a
search for any existing console/device statement this design must not
contradict.

Stop and obtain direction instead of guessing when any of the following
occurs:

- the W02 gate has not approved `<CONSOLE-MMIO-BASE>` /
  `<CONSOLE-SPI-INTID>` / the ID-register constants as machine facts — no
  value may be embedded; record the blocker;
- W07's `vgic_request_irq` cannot deliver the console SPI — block at the
  W07/W09 seam ([01 §7](01-architecture-and-state.md)); a Guest-polling
  fallback is explicitly not authorized (it would change the declared device
  behavior);
- W04's DTB console node disagrees with the machine facts or the compatible
  string this design assumes — the P8-V05 consistency review fails; fix in
  the machine contract, not in this code;
- W03/W15 boot-input facts are unavailable — Stages B–D of
  [04 §5](04-single-vcpu-boot-path.md) cannot run; record the blocked stage,
  not a skipped one;
- implementation seems to need virtio, DMA registers, flow control, a second
  instance, or backend rebind — Reserved/Out of Scope (README
  classification).

## 2. Ordered implementation steps

### Step 1 — machine facts registration

Target: console device facts (window base, INTID, ID constants) registered
from approved machine-contract data; consistency check against W04's DTB
node facts.

Work: register facts as immutable VM-creation inputs; fail creation on
absence/inconsistency; declare the RX-queue and log capacities as documented
implementation constants ([03 §3](03-code-contracts-console-backend-and-input.md)).

**Acceptance:** all Guest-visible console facts derive from one approved
source (DV01).  
**Failure/blocker:** missing gate approval is a recorded blocker (§1).

### Step 2 — frontend register model (M1)

Target: [02](02-code-contracts-console-frontend.md) — dispatch, register
table, TX/RX/IRQ-state behavior.

Work: implement the access dispatcher, the register subset exactly as
declared (RAZ/WI elsewhere), TX store-and-forward, RX FIFO, and the
interrupt-state recompute. Host-side unit tests drive it without a Guest.

**Acceptance:** Stage A register-level checks pass: readback/gating/RAZ-WI
behavior exact; no allocation in access paths; bounded work per access.  
**Failure/blocker:** a semantics question against the PL011 TRM stops that
register for review.

### Step 3 — backend and log (M2/M3)

Target: [03 §2, §4, §5](03-code-contracts-console-backend-and-input.md) —
seam, host backend, retained log.

Work: implement the seam with the P8 host backend behind it; fixed-capacity
log with oldest-first retention and drop counter; Host stdout mirror as a
Host-policy switch.

**Acceptance:** send never blocks or allocates; log retention bounded and
snapshot-consistent; drop behavior counted (DV04/DV05).  
**Failure/blocker:** any need for rebind/recovery semantics is Reserved —
stop and record.

### Step 4 — RX input path and interrupt integration (M4)

Target: [03 §3](03-code-contracts-console-backend-and-input.md) +
[02 §3.3](02-code-contracts-console-frontend.md).

Work: implement Host injection, bounded queue, and the W07-bridge
assertion/deassertion of the RX interrupt on `<CONSOLE-SPI-INTID>`.

**Acceptance:** round trip at register level (inject → IRQ assert → DR pop →
deassert); overflow drops counted; no storm (level semantics).  
**Failure/blocker:** a W07 seam mismatch stops per §1.

### Step 5 — single-vCPU boot path integration (M6 + [04](04-single-vcpu-boot-path.md))

Target: Stages A–D of [04 §5](04-single-vcpu-boot-path.md).

Work: integrate with the boot inputs (W03/W04/W15) and the sibling
mechanisms (W06/W07/W08); implement the marker table and observer; run the
stages in order, each gated on the previous.

**Acceptance:** each stage's condition in
[06 §1](06-validation-and-handoff.md) holds before the next is attempted;
Stage D ends in the interactive-shell round trip (M7).  
**Failure/blocker:** a failing stage blocks the next (that is the
localization discipline); a missing sibling mechanism records the blocked
stage.

### Step 6 — containment hardening and telemetry (M5)

Target: [02 §5](02-code-contracts-console-frontend.md) containment table and
telemetry events.

Work: verify/complete the W05-classified outcomes, flood-dropping, and
diagnostic-slot population; wire tx/rx/irq telemetry for W17.

**Acceptance:** every containment row has a declared, testable outcome;
telemetry events present and prunable.  
**Failure/blocker:** a containment gap is a design failure to fix in the
state model, never by rate-limiting the Guest ad hoc.

### Step 7 — scenario execution and closure review

Work: run the [06](06-validation-and-handoff.md) matrix as far as current
integration permits; evidence to
`../../verification/p8-w09-virtual-console-single-cpu-linux-verification.md`;
decisions to `../p8-w09-virtual-console-single-cpu-linux-record.md`.
Completion claims only in the verification record, only for what ran.

**Acceptance:** matrix entries passed/failed/blocked/not run with evidence;
P8-V12/V13 rows marked run have real output including the M7 round trip.  
**Failure/blocker:** a failed containment or interaction row is recorded and
fixed through the design's contracts, never by weakening the scenario.

## 3. Ordering rationale

Device (Steps 2–4) precedes boot integration (Step 5) because the console is
the observation channel for everything else; containment/telemetry
(Step 6) is hardened before scenario execution so every observed outcome is
attributable and contained.
