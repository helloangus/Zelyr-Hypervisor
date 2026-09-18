# P0-W13 Trace Event Namespace Baseline — Verification Evidence

**Status:** Complete evidence recorded; W13 closure claimed.
**Date:** 2026-09-18 (Asia/Shanghai)
**Environment:** Documentary review against branch `p0/w13-trace-namespace`
(baseline: merge of PR #25).

## Results

| ID | Review | Result | Evidence / reason |
|---|---|---|---|
| W13-DV01 → P0-V09 | Registry review | **passed** | All fourteen plan-named domains present with subject definitions and boundary notes; the list is closed for P0; subject definitions are the stated attribution authority. |
| W13-DV02 → P0-V09 | Grammar review | **passed** | Grammar form and prohibitions stated (no prose, imperative verbs, counters, timestamps, instance IDs, platform names, unregistered abbreviations); first token must be a registered domain; encoding explicitly out of scope. |
| W13-DV03 → P0-V09 | Declaration/registry review | **passed** | Registry table present and empty; entry fields complete; undeclared-name instrumentation is a review failure; no P0 artifact contains an event name (verified by the drill below using hypothetical names only in this evidence file). |
| W13-DV04 → P0-V09 | Compatibility review | **passed** | Immutable-meaning rule, deprecation-with-successor flow, recorded interval, removal-by-decision, and the document-version bump rules present; ADR-040 subjects untouched. |
| W13-DV05 → P0-V09 | Boundary review | **passed** | One-fact-one-name rule; log/trace disputes resolve under W12, name disputes here; payloads/encodings excluded; both boundary statements are consistent with the delivered diagnostics baseline (no restatement). |
| W13-DV06 → P0-V09 | Discovery and link review | **passed** | Routing row reaches the namespace document in one link; links to the diagnostics baseline and portability rules resolve; stage-index row truthful. The W12→W13 forward reference resolves as of this package. |
| W13-DV07 → W13 closure | Consumability review | **passed** | Telemetry implementer (declaration path + fields), W12 (boundary split), P1 mechanism authors (attribution via boundary notes), reviewers (registry-as-authority check) — each can act without inventing policy. |

## Attribution drill (W13-DV03/DV05 evidence — hypothetical names, not project events)

| Hypothetical fact | Attributed domain | Deciding boundary note |
|---|---|---|
| `boot.gic_init_done` (GIC initialized during bring-up) | `irq` (fact is about IRQ state) | boot ends where the subsystem's domain begins |
| `vcpu.exit_reason_wfi` (a WFI exit) | `vcpu` | the switch/exit itself is `vcpu`; policy reasons would be `scheduler` |
| `stage2.fault_received` | `stage2` | Stage-2 faults are `stage2`, not `memory` |
| `virtio.queue_notify` | `virtio` | virtio-specific fact; a device-model state fact would stay `device` |
| `capability.revoke` (revocation during a management operation) | `capability` | management-plane *operations* are `management`; the capability mechanism fact is `capability` |
| Grammatically rejected: `irq.GIC-0239-latency!` | — | platform/hardware name, non-snake_case token, embedded instance ID, imperative punctuation — four independent grammar violations |

Each attribution resolves from the registry's boundary notes alone; no
dispute required inventing a rule. No hypothetical name was declared, and the
registry remains empty.

## Not run / not proved

- **No event exists; no telemetry code exists** — P0 defines the naming
  governance only.
- **Compatibility mechanics:** exercised only on paper; real deprecations
  arrive with real events.
