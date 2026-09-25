# W08/W09 Stage-1 failure-seam reconciliation

**Status:** Proposed detailed-design correction; no implementation or validation claim.
**Scope:** The internal W08 mechanism result and W09 failure-dispatch ownership.
**Version:** v0.1
**Owner/change context:** P1-W08/W09 integration preflight, 2026-09-25.
**Supersedes:** W08 [address/table contracts](02-code-contracts-address-types-and-tables.md)
§7 and [transition contracts](03-code-contracts-mapping-and-transition.md) §3
where they require W08 to call the not-yet-linked W09 `fail_phase` directly;
W09's adapter contract remains authoritative for phase routing.

## Observed dependency and required foundation

The merged W08 foundation has a pure table model and a page-separated image,
but no hardware transition. The merged W09 foundation has a tracker and
failure-reason type, but no `fail_phase` or sequencer. The original W08 design
would make its transition body call W09's absent dispatcher, while W09's
workflow requires the W08 mechanism before wiring the sequencer. This is a
build-time dependency cycle, not permission to add a dummy phase or bypass a
failure. The required foundation is a fallible W08 mechanism whose errors the
W09-owned adapter can route once both packages are linked.

## Corrected internal contract

`enable_host_stage1() -> Result<(), Stage1Error>` is W08's single-shot
mechanism entry. It remains the sole owner of table construction, hardware
programming and post-MMU checks. On `Ok(())`, the post-MMU environment in
§7 holds. On `Err(e)`, it has not declared phase completion, never retries or
rolls back, and leaves the W09 tracker at `Entered(Stage1)`. The caller must
immediately enter the terminal route; normal boot must not observe the error
and continue. `Stage1Error { step, detail }` remains a static, allocation-free
carrier and preserves the failure point, including post-enable failure.

W09 alone owns `stage1_step`: call `enable_host_stage1()` exactly once and, on
`Err(e)`, call `fail_phase(Stage1, FailureReason::new(e.static_message())) -> !`.
The static message includes the step and detail using a closed vocabulary; no
heap formatting or new report transport is authorized. W09's `fail_phase`
delegates Stage1 to W07's armed fatal report. W08 has no direct dependency on
W09's dispatcher and does not print its own failure. A fault during the
transition still enters W05/W07 directly and never returns to this result
path. W09 records `Stage1.complete` only after `Ok(())`.

This is Required internal seam work. The W08 transition state machine,
attribute classes, terminal failure posture, NC5 continuation point,
external ABI, and future-stage exclusions do not change. No new `unsafe` or
dependency is authorized by this correction.

## Validation and handoff

Review W08's return paths: each premise/build/verify/program/postcheck error
must be a distinct `Stage1Error`, and no `Ok` may precede all postchecks.
Review W09's adapter: one call, `Err` always diverges through `fail_phase`,
and the tracker cannot advance to Complete on error. Host tests may exercise
the static error vocabulary; target build checks the exact interface. QEMU
and fault execution remain W10/W11 evidence. This design correction proves
none of those tests ran and does not close P1-V13–P1-V15.
