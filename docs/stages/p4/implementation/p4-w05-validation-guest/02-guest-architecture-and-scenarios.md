# P4-W05 Guest Architecture and Scenario Set

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P4-W05 detailed design](README.md).  
**Companion:** decisions and provenance finding in
[01-scope-and-foundations.md](01-scope-and-foundations.md).

## 1. Guest runtime module map

The Guest is a deliberately small `no_std` program; "modules" are logical
units within the one Guest crate (file placement follows the crate's own
structure — no crate splitting is authorized in P4).

| Module | Responsibility | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|
| `entry` | `_start`: establish stack, preserve x0 (scenario id), call runtime init, dispatch | registers per W04 entry convention | runtime start | boot-info contents |
| `bootinfo` | defensive validation of the boot-info block; accessors returning checked layout facts | block bytes at the fixed IPA | validated `BootInfoView` | producing the block (W03 does) |
| `console` | line-oriented writes to the console page (MMIO, polled); banner and marker emission | strings/bytes | console output | Host-side log formatting |
| `fmt` | minimal unsigned hex/decimal formatting | integers | strings | heap allocation (none exists) |
| `scenario` | dispatch table, per-scenario bodies, marker discipline, halt helper | scenario id, `BootInfoView` | scenario outcomes (markers, triggers, halts) | exit classification (W04/W06) |
| `halt` | terminal WFI loop helper used by every terminal path | — | trap that W04 classifies | stop semantics (W04 owns) |
| `panic` | Guest panic hook: marker + halt | panic message | `VG-PANIC` + WFI trap | recovery (never attempts it) |

Layering rule: only `console` performs MMIO; only `bootinfo` reads Guest RAM
parameters; only `scenario` decides control flow. The Guest contains no
timing loops, no randomness, no reliance on unspecified register values
beyond the entry convention.

## 2. Boot contract from the Guest's perspective

```text
At _start (EL1h, DAIF masked, per W04):
  x0        = scenario id (validated by W04 before entry)
  SP        = stack top from the layout
  PC        = image base (entry)
  Known IPA = boot-info block location (the Guest's only built-in constant)

Boot sequence:
  1. set SP from the entry convention (already valid; reassert defensively)
  2. validate boot-info block (magic, layout version, size, checksum, bounds)
     -> on failure: emit VG-FAIL:BOOTINFO and halt (D7)
  3. emit boot banner: protocol version, layout version, scenario id
  4. dispatch to the scenario body
```

CurrentEL confirmation: `entry` reads CurrentEL and the runtime asserts the
value is EL1 before the banner; a non-EL1 value emits
`VG-FAIL:EXCEPTION-LEVEL` and halts (defensive; W04's construction makes it
unreachable — the check documents the invariant instead of assuming it).

## 3. Console contract

- Writes are polled MMIO byte writes to the console page base (Device
  memory, per W03 D5); the writer busy-waits on the transmit-ready flag of
  the reference PL011 frame layout. The register layout is a property of the
  mapped frame, cited from the W03 convention — it is test-environment
  plumbing, not a device model.
- Writes are line-oriented (`writeln(bytes)` appends CRLF); markers are
  emitted as whole lines to keep automation parsing unambiguous.
- No output occurs between the expected-fault marker and the triggering
  instruction ([03 §4](03-code-contracts-guest.md) D5), so the last marker
  before an exit identifies the fault site precisely.

## 4. Scenario framework and marker discipline

Every scenario body follows one shape:

```text
scenario(id, bootinfo):
    emit(VG-<id>:BEGIN)
    ... steps, each with observable effect or single marked trigger ...
    on success path:  emit(VG-<id>:OK);  halt_via_wfi()
    on guest-side failure: emit(VG-<id>:FAIL:<REASON>); halt_via_wfi()
    on planned fault:  emit(VG-<id>:FAULT:<KIND>); <single triggering access>
                       // exit to EL2 happens here; W04/W06 observe
```

`halt_via_wfi()` executes WFI — W04 routes it to a classified `Wfi` exit and
its policy stops the vCPU (W04 D7); the Guest performs no further action.

## 5. P4 scenario table (VG-001–VG-012)

**Provenance:** reconstructed from the tracked governing sources (task book
§6 split; W05 plan scope enumeration) — see
[01 §3](01-scope-and-foundations.md). Table version `VG-T1`, owned by this
design, pending stage-owner confirmation against the canonical source.

| ID | Scenario (P4 test contract) | Guest actions | Expected Guest-side markers | Expected host-side observable outcome (W04 class per W04 §3.2) | Validation | Status |
|---|---|---|---|---|---|---|
| VG-001 | EL1 entry and greeting | CurrentEL check; banner; greeting line | `VG-001:BEGIN`, `VG-001:OK`, greeting incl. `Hello from EL1`, CurrentEL = EL1 | Guest executes; completion WFI → `Wfi` exit → defined stop; no Stage-2 faults | P4-V05 | mandatory |
| VG-002 | Mapped RAM read/write | write then read back a pattern at a data address from boot-info | `VG-002:BEGIN`, `VG-002:OK` | as VG-001 (`Wfi` stop); no faults | P4-V05 | mandatory |
| VG-003 | Stack execution | recursion/frames across a bounded depth; verify SP in stack window from boot-info | `VG-003:BEGIN`, `VG-003:OK` | as VG-001 | P4-V05 | mandatory |
| VG-004 | Unmapped-IPA read trigger | emit expected-fault marker; single load from the boot-info unmapped probe address | `VG-004:BEGIN`, `VG-004:FAULT:LOAD-UNMAPPED` | `Stage2Translation` exit with faulting IPA = probe; W04 stop `GuestFault` | P4-V06, P4-V07 | mandatory |
| VG-005 | Write to read-only page | emit marker; single store to the read-only window address from boot-info | `VG-005:BEGIN`, `VG-005:FAULT:STORE-RO` | `Stage2Permission` exit; faulting IPA = RO window; stop `GuestFault` | P4-V06, P4-V08 | mandatory |
| VG-006 | Execute-permission comparison | emit marker; single instruction fetch from an XN data window (via function-pointer-style indirect call) | `VG-006:BEGIN`, `VG-006:FAULT:EXEC-XN` | `Stage2Permission` exit (execute case), distinguishable from VG-005 by syndrome/access type; stop `GuestFault` | P4-V06, P4-V08 | mandatory |
| VG-007 | WFI behavior | emit marker; execute WFI | `VG-007:BEGIN` | `Wfi` exit; defined diagnosable result (W04 policy stop) | P4-V09 | mandatory |
| VG-008 | WFE behavior | emit marker; execute WFE | `VG-008:BEGIN` | `Wfe` exit; defined result | P4-V09 | planned |
| VG-009 | Fault-and-reenter sequence | VG-004-style trigger, then — driven by W04's re-entry proof path — a second, distinct trigger, demonstrating context save/restore between episodes | sequence-specific markers per episode | two classified exits with preserved context between them (W04 DV04) | P4-V04 support, P4-V06 | planned |
| VG-010 | Controlled illegal behavior | emit marker; execute a defined illegal instruction (architecture-guaranteed undefined) | `VG-010:BEGIN`, `VG-010:FAULT:ILLEGAL` | `IllegalExecution` exit; VM-facing stop; Host alive and diagnosable | P4-V06, P4-V09 | mandatory |
| VG-011 | Unknown synchronous exception | emit marker; execute SVC (supervisor call at EL1, routed to EL2 as a synchronous exception P4 does not otherwise use) | `VG-011:BEGIN`, `VG-011:FAULT:SYNC-UNKNOWN` | `UnknownSync(ESR)` exit with full syndrome retained; stop `GuestFault` | P4-V06 | planned |
| VG-012 | Completion and controlled stop protocol | full sequence: banner, one benign action, `VG-012:OK`, WFI | `VG-012:BEGIN`, `VG-012:OK` | `Wfi` stop; EL2 teardown proceeds; repeat run clean (W07) | P4-V09, P4-V10 support | mandatory |

Mandatory set = VG-001–VG-007, VG-010, VG-012 (task book §6). VG-008/VG-009/
VG-011 are planned; a deferral needs the explicit stage-review record path
([01 §1](01-scope-and-foundations.md)).

Note on VG-009 and W04's re-entry proof: the re-entry *mechanism* is W04's
(`Reenter` action, [P4-W04 §3.3](../p4-w04-vcpu-entry-exit/04-code-contracts-vcpu-run.md));
VG-009 is the Guest-side sequence that makes it observable end to end.

## 6. Test-asset maintenance boundary

- **Ownership:** the scenario table ([§5](#5-p4-scenario-table-vg-001vg-012),
  version `VG-T1`) and marker protocol (D2, version string in the boot
  banner) are owned by this design; changes bump the version and require
  joint review with W04 (classes), W06 (diagnosis), W08 (automation
  grammar).
- **Change rules:** adding a scenario = new table version + W04 validation
  table extension; changing an expected outcome = joint review first;
  changing markers = automation-breaking change, requires W08 agreement
  before merge.
- **Deferral rule:** a planned scenario (VG-008/VG-009/VG-011) not delivered
  gets an entry in the implementation record's deferral section naming the
  AArch64-specific reason, the downstream owner, and the exit-criterion
  effect — the task book's exact requirements — before W09 closeout.
- **What the asset is not:** not a Guest OS, not a production ABI consumer,
  not a hardware bring-up tool; its conventions (console page, boot info,
  scenario id) are P4 test contracts recorded as implemented facts by W09
  (W01 A4), and P5 inherits maintenance duty, not frozen semantics.
