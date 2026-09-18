# P0-W12 Logging/Diagnostic Baseline — Verification Evidence

**Status:** Complete evidence recorded; W12 closure claimed.
**Date:** 2026-09-18 (Asia/Shanghai)
**Environment:** Documentary review against branch
`p0/w12-diagnostics-baseline` (baseline: merge of PR #24).

## Results

| ID | Review | Result | Evidence / reason |
|---|---|---|---|
| W12-DV01 → P0-V09 | Channel and level review | **passed** | Four channels (log, trace, metrics, fatal) each with purpose, audience, content semantics, and a non-substitution list; five levels with fixed meanings and retention defaults; cross-level rules separate system impact from failure classes. |
| W12-DV02 → P0-V09 | Visibility and trimming review | **passed** | Three visibility classes mapped to levels/channels; trimming only via W04-classified build selections; compile-out-without-side-effects and runtime-filterability stated as binding semantic requirements; per-class/per-channel decisions, never per call site. |
| W12-DV03 → P0-V09/P0-V14 | Fatal-minimum review | **passed** | Panic-message minimum (classification, site, build identity), crash-dump minimum (machine state as requirement not register list, identity set, correlable context), and the untrusted-data rule present; no format or register list fixed. |
| W12-DV04 → P0-V14 | Identity association review | **passed (cross-review pending W16 delivery, recorded)** | The association property and the inline-vs-associable split are stated; W16 is not delivered, so the two-way cross-review is recorded as pending on the W16 side (its design carries the same obligation). No substantive conflict assessable yet. |
| W12-DV05 → P0-V09 | Constraint walkthrough | **passed** | Five hypothetical scenarios each resolve to exactly one governed treatment with no implementation prescribed (evidence below). |
| W12-DV06 → P0-V09 | Discovery and link review | **passed** | Routing row reaches the baseline in one link; stage-index row truthful; sibling links to delivered contracts resolve. Two forward references (W13, W16 normative homes) recorded in the implementation record; they resolve when those P0 packages merge. |
| W12-DV07 → W12 closure | Consumability review | **passed** | W13 (owns naming; channel references only), W16 (property it must satisfy, cross-review obligation), P1-W06 (M2 transport binding + §6.3 transition rule), P1-W07 (§4 fatal minimums) — each knows what is and is not W12's. |

## Constraint walkthrough (W12-DV05 evidence)

| Hypothetical scenario | Governed treatment | Not prescribed |
|---|---|---|
| A P1 design wants to print "VM 3 created" to the console | Info-level human log (§2); channel/level/visibility stated per M6; visibility class removable-by-build-selection | No macro or API named |
| A Stage-2 fault handler reports a guest-triggered fault | VM-scoped diagnostic per the failure classification + trace/metric facts; never the fatal channel; never unvalidated-as-fact (N5) | No fault-report format |
| A benchmark build should exclude high-frequency walk traces | Compiled-out-by-default class; excluded via a classified build selection; runtime filter when present (§3) | No cfg name or flag |
| A fatal exit occurs at an EL2 exception site | Fatal channel only; panic message minimum (§4.1) + crash dump minimum (§4.2); classification per W14 | No register list or dump layout |
| A contributor adds `print!` direct console writes after the logging design lands | Prohibited by N1 (parallel path); permitted only pre-adoption under §6.3's transitional rule | — (rule resolves it) |

## Not run / not proved

- **No logging code exists**; P0 defines no code surface.
- **W16 cross-review:** pending W16 delivery (recorded; symmetric obligation).
- **Forward-reference links:** W13/W16 normative homes pending their
  packages; recorded, resolve within P0.
