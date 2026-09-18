# P0-W14 Panic/Failure Classification — Verification Evidence

**Status:** Complete evidence recorded; W14 closure claimed.
**Date:** 2026-09-18 (Asia/Shanghai)
**Environment:** Documentary review against branch
`p0/w14-failure-classification` (baseline: merge of PR #22).

## Results

| ID | Review | Result | Evidence / reason |
|---|---|---|---|
| W14-DV01 → P0-V09 | Class review | **passed** | All five classes present with the uniform field order (definition, detection, containment, allowed/prohibited responses, diagnostic treatment, informative examples); classes are architecturally distinct (hypervisor-invariant / guest / resource / unsupported / platform) per the task-book requirement. |
| W14-DV02 → P0-V09 | Rule review | **passed** | §2 carries classify-at-detection, the guest-triggerability classification test, the no-downgrade rule, the authorization-denial rule, the input-source rule (with the P5 open item recorded), and the escalation rule; §3 states the panic-worthiness rule (only FC-INVARIANT exits may terminate the hypervisor). |
| W14-DV03 → P0-V09 | Classification-test dry run | **passed** | Six hypothetical failures classified unambiguously (evidence below), including the trap case (guest-triggerable condition misclaimed as invariant) and the escalation case. |
| W14-DV04 → P0-V09 | Checklist review | **passed** | D1–D4 design checklist and C1–C4 code checklist present; C3 hooks the unsafe policy's `SAFETY` failure-class field by reference without restating it. |
| W14-DV05 → P0-V09 | Discovery and link review | **passed** | Routing row reaches the taxonomy in one link; security README pointer added; links to the unsafe policy resolve; stage-index row truthful. |
| W14-DV06 → P0-V09 | Consumability review | **passed** | W12 (channel-class split explicit: W12 owns content minimums, W14 owns class permission), W10 (taxonomy supersedes the placeholder at next audit), W13 (trace/metric consumers named), P5 (open item assigned), P1 designers (classify-at-detection duty) — each can act without inventing policy. |

## Classification-test dry run (W14-DV03 evidence)

| Hypothetical failure | Class | Deciding rule |
|---|---|---|
| A Stage-2 fault from a guest's unmapped access | FC-GUEST | §1.2; guest-controlled input triggered it |
| A page found with two owners during a hypervisor-internal operation | FC-INVARIANT | §1.1; internal guarantee broken; fatal, no downgrade |
| A guest hypercall argument that passes a kernel pointer causing a lookup to fail | FC-GUEST | §2 classification test: guest-controllable input → FC-GUEST, never FC-INVARIANT, even though the failure surfaces inside hypervisor code |
| Page-frame allocation fails under memory pressure | FC-RESOURCE | §1.3; refusal observable, policy-owned; not fatal |
| SMMU isolation requested on a platform whose description lacks the capability | FC-UNSUPPORTED | §1.4; capability query refusal; no silent fallback |
| A PSCI firmware call returns an unexpected error during boot | FC-PLATFORM | §1.5; owning BSP design declares containment; escalation only if continuing would violate an invariant |
| Continued operation after FC-RESOURCE would corrupt the page-ownership map | FC-INVARIANT (escalation) | §2 escalation rule: fatal exit naming the threatened invariant |

## Not run / not proved

- **No runtime failure path exists**; the taxonomy is reviewable policy, not
  exercised code (P0 defines no fault mechanism).
- **W12/W13 consumers:** their channel/namespace deliveries are their own;
  this taxonomy references, not defines, them.
- **P5 class assignment:** recorded as an open item, not resolved here.
