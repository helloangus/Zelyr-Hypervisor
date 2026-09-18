# P4-W05 Scope, Foundations, and Resolved Decisions

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W05 detailed design](README.md).

## 1. Scope classification detail

### Required (P4-D01–D08, P4-J)

- Guest crate: `no_std`, bare-metal EL1 entry, flat-binary artifact
  consumable by the W03 route, own stack setup, no firmware/libc/firmware-
  protocol dependence.
- Runtime services: defensive boot-info parsing, console writer over the
  console-page mapping, minimal formatting (hex/decimal), panic handler.
- Scenario framework: validated scenario id → dispatch → begin/end/fault
  marker emission with the versioned marker protocol.
- The P4 scenario set per [02 §5](02-guest-architecture-and-scenarios.md)
  §5: mandatory VG-001–VG-007, VG-010, VG-012; planned VG-008, VG-009,
  VG-011.
- Maintenance boundary: protocol/table versioning, change rules, ownership
  statement.

### Required-if-feasible (explicit deferral path)

- VG-008, VG-009, VG-011: planned P4 scenarios. Deferral is allowed only by
  an explicit stage-review record naming the AArch64-specific reason, the
  downstream owner, and the exit-criterion effect (task book §6). This
  design schedules them as normal work and provides the deferral record path
  (`../p4-w05-validation-guest-record.md` deferral section) without
  pre-deciding a deferral.

### Reserved (must not be precluded; not implemented in P4)

- Scenario growth for P5+ (hypercall/fault-boundary suites —
  [P5-W07](../../../p5/plans/p5-w07-validation-guest-isolation-suite.md) is
  the named future consumer), timer/interrupt scenarios (P6), scheduler
  scenarios (P7).
- A semihosting-style channel beyond the console page (re-entry if a future
  stage needs bidirectional host-guest test I/O; would ride P5's ABI work).

### Out of Scope

Linux/libc/firmware boot, production Guest ABI, Guest DTB, PSCI calls,
virtio, virtual interrupts/timer, scheduler interactions, hypercall usage,
and any permanent repository-layout decision (the `guests/validation-aarch64`
location is recorded as an implemented fact by W09 if delivered).

## 2. Assumed upstream contracts and failure boundaries

Per the [P4-W01](../p4-w01-entry-contract-reconciliation/README.md) entry
review; divergence becomes a recorded conflict (W01 §4), and the affected
step stops.

| ID | Assumed contract | Source | Relied-on behavior | Failure boundary if delivered differently |
|---|---|---|---|---|
| M1 | Build baseline: AArch64 bare-metal target, pinned stable toolchain, no unstable features | P0-W02/W03; W01 R18 | the Guest crate builds to a relocatable-at-load flat binary | target/toolchain absent → Guest build blocks; no unstable-feature workaround is permitted |
| M2 | Flat-binary route: entry at image base, embedded at build time | [P4-W03](../p4-w03-guest-memory-image/README.md) D3 | Guest entry code runs first at `IMAGE_LOAD_IPA` with a clean register file | route change (e.g. ELF) → Guest linker layout redesign, joint note; not a code patch |
| M3 | Boot-info block at fixed IPA with magic/version/size/checksum | W03 D4/§2 | Guest reads parameters only from the validated block | block drift → joint version bump and both-side update; never one-sided parsing tolerance |
| M4 | Console page: Device RW-XN mapping of the reference PL011 frame; EL2 console-quiet during the run segment | W03 D5 | Guest output via MMIO writes is visible to automation; interleaving only at exit boundaries | mapping absent/changed → console scenarios block; fault scenarios remain feasible; record limitation |
| M5 | Entry convention: x0 = validated scenario id; PC = entry IPA; SP = stack top; EL1h, DAIF masked | [P4-W04](../p4-w04-vcpu-entry-exit/README.md) §2.1 | `_start` finds its parameters in registers and stack per this convention | convention change → joint note (Guest `_start` and W04 constructor change together) |
| M6 | Exit classes: WFI/WFE trap, Stage-2 translation/permission faults, illegal-execution, unknown-sync produce classified exits with defined stop results | W04 §3.2/D7 | scenario triggers are designed against these classes; expected outcomes are stated in the table | class/behavior drift → scenario-table revision (versioned), joint review with W04/W06 |
| M7 | Stage-2 layout facts: RAM bounds, read-only data window, execute window, unmapped probe address | W03 layout record + W02 mapping plan | fault targets are data-driven from the boot-info echo, not invented | layout change → table revision via boot-info (D4 of [01 §4](#4-resolved-design-decisions)); no Guest-side constants beyond the block location |
| M8 | Logging/trace and QEMU automation boundaries consume Guest output as data | P0-W12/W13; P1-W10; W08 | markers are plain bytes on the console; no Guest-side telemetry protocol exists | automation contract change → W08's consumption adapts; the marker grammar is W05-stable |

## 3. Authority analysis for contested areas

- **Scenario provenance (governing finding):** the P4 task book §6 requires
  "VG-001 through VG-012 from the source task book," but the superseded
  root-level source document is not tracked in this repository (verified
  across full git history — the baseline commit contains only the organized
  stage tree). The governing sources that *are* tracked state: the
  mandatory/planned split (task book §6) and the scenario-class enumeration
  (W05 plan scope: "observable EL1 marker, CurrentEL confirmation,
  RAM/stack/code execution, unmapped and permission-fault triggers, WFI/WFE
  behavior, controlled illegal behavior"). This design therefore
  reconstructs the twelve scenarios from those tracked sources, labels the
  table a P4 test contract owned by this design, and records the provenance
  gap as an open question for the stage owner (W01 ledger item A9). If the
  canonical table is later supplied, the reconstruction is reconciled
  against it through a versioned table revision — definitions never silently
  diverge.
- **How the Guest proves "Hello from EL1":** with no device model, no
  hypercall ABI, and no virtual console in P4, the only in-scope output path
  is the W03 console-page mapping (W03 D5). This design consumes it as a
  temporary test convention and does not generalize it into a console
  subsystem.
- **Guest trust posture:** the Guest is authored here but must be designed
  as *potentially malicious from the hypervisor's perspective* — the Guest
  never performs an operation whose containment is untested, and the fault
  scenarios deliberately exercise the containment. Conversely, the Guest
  treats its own boot inputs as untrusted-until-validated, both for honesty
  of the test and as the defensive-parsing pattern P5's guest-copy framework
  will generalize.
- **WFI as the completion signal:** because P4 has no virtual timer or
  wakeups, the natural terminal action for a completed scenario is a WFI
  whose trap W04 classifies and stops (W04 D7). The marker protocol makes
  completion observable *before* the WFI, so the stop result is a second,
  independent confirmation — automation can require both.

## 4. Resolved design decisions

| ID | Decision | Rationale | Authority basis |
|---|---|---|---|
| D1 | One Guest image for all scenarios; scenario id in x0 (W04 M5) echoed in boot-info; dispatch on the validated value | single maintained asset (plan goal); automation selects scenarios without rebuilding; avoids N embedded images | plan scope ("maintained test asset"); W04/W03 contracts |
| D2 | Marker protocol: `VG-<id>:BEGIN`, `VG-<id>:OK`, `VG-<id>:FAULT:<KIND>`, `VG-<id>:FAIL:<REASON>`, `VG-PANIC:<REASON>`, plus a boot banner with the protocol version; fixed ASCII, line-buffered writes | stable, grep-able automation surface (W08); versionable; no formatting ambiguity | P4-V05/V06/V08/V09 evidence needs; W01 A8 discipline analog for the Guest side |
| D3 | Boot-info validated defensively before any use (magic, layout version, block size, checksum, field bounds) | Guest honesty + the defensive pattern P5 generalizes; a corrupt block yields `VG-FAIL:BOOTINFO`, never a wild write | ADR-007 applied to the Guest's own inputs; W03 §2 |
| D4 | All fault targets and layout facts come from the boot-info echo; the Guest holds no layout constants except the boot-info location | scenario semantics survive layout revisions without Guest code changes; reviewable data-driven triggers | W03 layout record (M7); W01 A4 |
| D5 | Fault triggers are single-instruction, single-access, at a marked PC: each faulting scenario writes its expected-fault marker *before* the triggering access | makes captured frame values (W04 DV09) and W06 correlation unambiguous; no multi-access ambiguity about the fault site | W06 diagnostic need (W01 R02); W04 frame fidelity |
| D6 | Scenario completion uses WFI (trap → classified exit → W04 `Stop(Controlled)`); fault scenarios end at their trigger; VG-012 exercises the full completion→stop path explicitly | defined stop per task book P4-V09 with W04 D7 policy; marker-then-WFI gives two independent confirmations | task book §6 matrix; W04 action policy |
| D7 | Guest panic handler emits `VG-PANIC:<REASON>` then halts in a WFI loop; Guest `Fail` conditions emit `VG-<id>:FAIL:<REASON>` then also WFI-halt | every Guest terminal state is diagnosable and classified; no silent hangs in automation | P4-V09; W08 determinate-result requirement |
| D8 | The scenario table and marker protocol are versioned artifacts owned by this design; changes require a table-version bump and joint review with W04/W06/W08 | "maintained test asset" with reviewable change rules; prevents silent marker drift breaking automation | plan goal; W09 records facts |
