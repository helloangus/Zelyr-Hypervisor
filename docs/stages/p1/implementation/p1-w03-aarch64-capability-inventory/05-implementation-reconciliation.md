# P1-W03 implementation reconciliation

**Status:** Detailed-design amendment for P1 implementation; no validation claim.
**Scope:** Hardware decoding corrections and concrete realization of W03.
**Owner/change context:** W03 current-state audit, 2026-09-24.
**Supersedes:** Incorrect granule/VH interpretations in the original W03 proposal.

## Current-state foundation ledger

| Required outcome | Observed baseline at a05fca6 | Bounded deliverable / owner | Evidence |
|---|---|---|---|
| EL2 inventory | W02 runtime exists; no inventory | W03 architecture module inside existing hypervisor member | DV01–DV04 |
| Pure policy validation | Existing host-test member contains only entry-health test | Compile the exact pure production file in that member's integration test; unit cases remain beside the module | DV01–DV03 |
| Published consumer seam | W09 adapter is a design contract, not executing code | W03 build/query/render functions; W09 owns invocation and removal of narrowly scoped seam allowances | DV07; W10 execution |
| Correct fact semantics | Proposed TGran16 and VH semantics contradict Arm definitions | Correct the field decodes; preserve the 13-record taxonomy and required set | DV01 |

## Hardware facts and authority

The reference vocabulary is Armv8-A as described by Arm ARM DDI 0487J.a,
including FEAT_LPA/LPA2 register encodings. P1 does not implement those
extensions' translation mechanisms. Supporting primary sources are the
[Arm Cortex-A53 TRM DDI 0500](https://documentation-service.arm.com/static/6040c321ee937942ba301626),
[Arm feature names 109697_0100_02](https://documentation-service.arm.com/static/668bf39069e89f01e39c46dd),
and Arm's [TF-A v2.12 register definitions](https://github.com/ARM-software/arm-trusted-firmware/blob/v2.12.0/include/arch/aarch64/arch.h).

TGran16 zero means unsupported, unlike TGran4 and TGran64. TGran4 0/1,
TGran16 1/2, and TGran64 0 are recognized as supported. Other encodings
are conservatively Absent. PARange 0–6 maps to 32/36/40/42/44/48/52 bits;
newer encodings outside the stated revision are conservatively Absent.
ASIDBits 0/2 maps to 8/16; GIC 1/3 identifies recognized CPU interfaces.
VH 1 identifies VHE; other encodings are Absent. This corrects the misleading
`Stage2Support` identifier to `VirtualHostExtensions`, including W04's reference.
EL2 profile is retained as future Stage-2 context; no absence of VHE implies
absence of Stage-2. No required fact or continuation policy changes.

Identification-register accessibility assumes the W01 trusted firmware
contract. Later EL3 feature traps may violate that assumption; W03 does not
claim that every possible firmware configuration makes these reads trap-free.

## Concrete boundaries and lifecycle

`hypervisor/src/arch/aarch64/capabilities/facts.rs` owns pure raw-snapshot
decoding, `FactId`, orthogonal classification/observation, immutable report,
required verification and bounded rendering. `mod.rs` owns exactly six reads
and the publication cell. W03 defines no external ABI or new crate.

`CapabilityReport::from_registers(&RawRegisters)` creates a complete stack
draft. `verify_required()` returns the first rejection in ALL order. Only
successful drafts enter `CAPABILITIES`. The cell claims publishing before
writing, release-publishes after writing, and permits immutable reads only
after acquire-observing published. Repeated publication/early query routes
through panic before touching storage. There is no reset, retry, allocation,
blocking, or partial-publication continuation. The atomics strengthen the
single boot CPU contract without adding SMP functionality. Raw values are
identification facts, not semantic addresses.

W09 calls `build_capability_report()` then later `render_report(&mut sink)`;
W04 consumes `query(FactId) -> FactRecord` and `Observation`. The existing
W02 unlinked-W09 termination remains until its owning package wires it.
Temporary dead-code allowances are restricted to those three seam functions,
the unused observation import, and the architecturally unavailable Unreadable
variant. W09 removes seam allowances when it adds the actual callers.

## Diagnostic declaration

Every `cap <label>=<value> (<classification>)` line encodes one canonical
`cpu.capability.fact` structured event. Channel: structured trace; level: none;
visibility: low-frequency boot facts retained in the P1 bring-up build.
Thirteen lines are emitted once, associated with the W09/W06 boot identity.
This does not introduce high-frequency tracing, a filter framework, or a build
switch. W06 supplies framing/transport; W03 never writes UART registers.
Labels are kebab-case FactId names, values unsigned decimal or exactly
`Absent`/`Unreadable`, classifications exactly `Required`/`Optional`/`Future`.
Lines contain no newline and fit 96 bytes. Fatal rejection remains the fatal
channel with its frozen `cap-reject fact=<label> <reason>` vocabulary.

## Validation mapping and handoff review

Decode and policy tests cover normal, all granule encodings, reserved values,
frequency boundaries, missing required facts, optional/future unreadability,
and deterministic rejection ordering. Rendering covers maximum u64 and all
13 lines. Stateful publication is reviewed for early/repeated access and
release/acquire ordering; privileged execution belongs to W09/W10 and NC2 to
W11. No resource recovery is supported; all fatal paths are terminal. No
guest, hardware-board, or concurrent boot execution is claimed by host tests.

The skill checklist review finds each foundation owned, all three external
seams explicit, single publication ownership, and future runtime proofs assigned
to W10/W11. There is no architectural/ADR change or added dependency. Actual
results belong exclusively in the verification record.
