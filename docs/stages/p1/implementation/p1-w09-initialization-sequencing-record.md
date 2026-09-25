# P1-W09 initialization sequencing implementation record

**Status:** Ordered boot path implemented; broader negative/repetition evidence tracked separately.
**Scope:** W01–W08 phase composition, deferred marker replay, Stable handoff and W02 idle.
**Version:** v0.1
**Owner/change context:** P1-W09 integration, 2026-09-25.
**Supersedes:** The [pure lifecycle foundation record](p1-w09-lifecycle-foundation-record.md) only as the current implementation status; that record remains historical evidence.

The [W09 design](p1-w09-initialization-sequencing/README.md), its
[preflight correction](p1-w09-initialization-sequencing/00-preflight-amendment.md),
and [W08/W09 failure seam](p1-w08-host-stage1-address-space/05-failure-seam-reconciliation.md)
govern the implementation. `boot/lifecycle.rs` retains the one-word CAS tracker
and adds the target-only sequencer. Host tests still compile its pure tracker
on x86; target-only adapters are explicitly AArch64-gated. `boot/mod.rs`
records Entry/Runtime at the W02 seam, calls the sequencer, records Stable,
and enters the W02-controlled WFI loop through the closed arch wrapper. The
wrapper's [design correction](p1-w02-minimal-rust-el2-runtime/06-controlled-idle-reconciliation.md)
and [U-015](../../../security/unsafe-inventory.md) describe its bounds.

| Phase | Owning mechanism called once | Exit/failure boundary |
|---|---|---|
| Entry | W01 validated assembly transfer, then W02 glue records both events | W01 pre-transfer rejection; tracker misuse terminal |
| Runtime | W02 context publication, identity and writer readiness | P0/W02 panic route; no completion on failure |
| Capabilities | W03 `build_capability_report` | W03 explicit required-fact panic; tracker stays Entered |
| El2Baseline | W04 `establish_el2_baseline` | W04 panic with failed control; no completion |
| Exceptions | W05 `install_el2_exception_entry` | W05 early panic or guarded fatal path |
| Console | W06 `bring_up_early_console`, then replay and W03 report rendering | console readback error routes terminal panic; no partial success |
| FatalPath | W07 `arm_fatal_path` | W07 minimal readiness stop before arm, report path after arm |
| Stage1 | W08 `enable_host_stage1` | every `Stage1Error::static_message()` token routes via W07 `fail_phase(Stage1, …)`; no completion after Err |
| Stable | W09 CAS transition, W10 token, W02 idle | only after Stage1.complete; later faults use W05/W07 with Stable position |

The tracker has no event buffer: before the console phase all events are
encoded by its monotone position. After W06 publishes availability,
`materialize_replay()` emits exactly Entry.enter through Console.enter using
W06's `ZELYR P1 PHASE` framing; a one-shot guard rejects duplicate replay.
Console.complete is the first live marker. FatalPath and Stage1 enter/complete
are live. Stable emits W10's independently fixed `ZELYR P1 STABLE` token.
W03 capability lines are transported after replay without changing their
owner or content.

The source contains no new external ABI, dependency, heap, lock, rollback or
future-stage mechanism. New crate-private APIs are the W09 transition/
sequencer functions and one AArch64 WFI wrapper. The only new `unsafe` is
U-015, an arch-layer instruction boundary accepted after independent review;
its evidence limits are recorded in the inventory. Existing W01–W08 unsafe
blocks are not widened.
The pre-vector exception window and firmware WFI trapping policy remain
declared limitations, not silent prerequisites.

The [verification record](../verification/p1-w09-initialization-sequencing-verification.md)
separates host/target checks, runner-observed canonical boot, linked ELF
placement, and deferred W10/W11/hardware coverage. W10 consumes the phase
sequence and Stable token; W11 consumes the tracker phase and fatal/panic
classes; W12/P2 may describe only the stable environment evidenced there.
