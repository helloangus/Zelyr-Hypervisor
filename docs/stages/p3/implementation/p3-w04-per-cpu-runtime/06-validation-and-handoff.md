# P3-W04 Validation and Handoff

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W04 detailed design](README.md).

## 1. Validation matrix

Each item is recorded as **passed**, **failed**, **blocked**, or **not
run** with command, input, environment, timestamp, and reason in the
verification record. QEMU items appear only where W04's scope produces
them; repeated/stress evidence and the regression matrix belong to
[P3-W12](../p3-w12-smp-stress-failure-tests/README.md) and
[P3-W13](../p3-w13-qemu-smp-regression/README.md).

| ID | Requirement | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|---|
| W04-DV01 | Layout, slot ownership, reserved capacity (→ P3-V04 prerequisites) | layout review | review [03](03-code-contracts-percpu-area.md) against code; check alignment rules and fill patterns; confirm every slot names its owner | alignment holds; owners recorded; reserved region untyped and untouched; versioned layout | structure and boundaries; not consumer protocols (W07/W08/W09/W11's) |
| W04-DV02 | Allocation, header validation, access correctness (→ P3-V04) | host-side unit tests | fake page source through `allocate_all`; register-fake through `current`/`install`; error paths asserted | validation catches corrupted headers; exactly-once install; ordering per architecture §5; no fallback locality source | logic correctness on the host; not hardware register behavior |
| W04-DV03 | Independent execution per CPU (→ P3-V04) | QEMU isolation capture | per declared count: per-CPU diagnostics show correct identity, distinct areas, distinct stacks; boot CPU included | every online CPU reports its own identity and unique addresses | isolation on the reference platform per captured boot; not stack safety under load (W12), not repeated boots (W13) |
| W04-DV04 | No implicit global current-CPU/context (→ P3-V04) | boundary review + symbol search | search for global current-CPU symbols; review all locality paths; verify table is diagnostic-only in kernel code | only the register path exists for self-access; table restricted to cross-CPU reads | the prohibition holds in this package; consumer compliance is each consumer's review plus [P3-W10](../p3-w10-smp-safety-audit/README.md) |
| W04-DV05 | Foundations recorded for downstream owners (→ W04 closure) | closure review | verify implementation record (offsets, constants, register convention) and handoff checklist | slot owners can integrate without redefining the layout; reserved capacity documented, undefined | handoff readiness; not downstream correctness |

A QEMU isolation capture proves distinctness and identity on the
reference platform at boot time; it does not prove contention-freedom,
cache behavior on real hardware, or stack safety under stress. Stack
overflow protection is a recorded Reserved item, not a proven property.

## 2. Error, security, and observability model

- **Error model.** Allocation and validation failures are fatal
  boot-critical before publication; installation failures are reported
  through the W02/W03 failure paths and leave the CPU outside every
  eligible set; a post-install header mutation or duplicate install is an
  invariant violation with a fatal diagnostic. No error path frees or
  reuses memory.
- **Security.** Areas and stacks are not guest-reachable; the layout is
  internal, versioned, and never serialized. The `unsafe` surface is the
  register read/write plus the reference construction over the register
  value, each with `SAFETY` obligations tied to the install ordering. The
  eligibility gate (W03) authorizes installation — W04 adds no parallel
  permission logic. Cross-CPU typed-slot access without the owning
  package's protocol is forbidden by contract and checked in review.
- **Observability.** The isolation diagnostic (identity + addresses per
  CPU) is the package's primary output; `install_state` transitions and
  the readiness signal are the observable lifecycle points; counter
  meaning arrives with W11's catalog on the reserved block W04 provides.

## 3. Handoff checklist

Before handing W04 to a reviewer:

- the exact changed-file list, with crate/module placement traced to the
  approved workspace design;
- W04-DV01–DV05 evidence paths with run status, including explicit
  not-run entries (counts not reachable; stress deferred to W12; matrix
  deferred to W13);
- the recorded constants and layout facts: `RUNTIME_STACK_SIZE` with
  rationale, area layout offsets and version, register convention, boot
  CPU's boot-stack decision;
- confirmation that new `unsafe` is limited to the CPU-local register
  boundary and inventoried per P0 unsafe governance;
- confirmation that no lifecycle transition, rendezvous protocol, lock,
  notification/TLB protocol, exception-vector change, telemetry catalog,
  or scheduler/vCPU field was added; the reserved region is untyped;
- consumer contract confirmations: W05 (signal point), W06 (privacy
  boundary), W07/W08 (slot placement), W09 (slot + `current()` validity),
  W10 (invariants), W11 (counter block), W12/W13 (cross-CPU read path),
  W14 (reserved capacity statement);
- any recorded blocker (TPIDR_EL2 reservation; address-space coverage;
  sibling call points) with its decision owner.
