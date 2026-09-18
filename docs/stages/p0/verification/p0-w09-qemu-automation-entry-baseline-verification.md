# P0-W09 QEMU Automation Entry Baseline — Verification Evidence

**Status:** Complete evidence recorded; W09 closure claimed.
**Date:** 2026-09-18 (Asia/Shanghai)
**Environment:** Documentary review against branch `p0/w09-qemu-runner-entry`
(baseline: merge of PR #19). **No QEMU execution occurred in this package and
none was required**; every validation below is documentary.

## Results

| ID | Review | Result | Evidence / reason |
|---|---|---|---|
| W09-DV01 → P0-V13 | Contract completeness review | **passed** | `docs/testing/qemu-runner-entry.md` contains §§1–8: single-entry rule with the human-investigation carve-out, responsibility boundary (runner vs caller), grammar sketch with five rules, seven reserved parameter classes with owners, four runtime-behavior requirements, exactly six exit statuses, the four-part evidence content set, and consumability constraints. No QEMU flag recipe, runner language, image, or timeout default is fixed. |
| W09-DV02 → P0-V13 | Placeholder marking review | **passed** | §8 states, in bold, the interface-only status: no runner exists, the entry cannot be invoked, no QEMU execution occurred, nothing proves EL2/guest/hardware behavior; the routing row and stage-index row carry the same placeholder status; no artifact implies a run. |
| W09-DV03 → P0-V13 | Grammar walkthrough | **passed** | Four paths traced (evidence below); each ends in a fixed status class with a complete evidence content set and no undocumented decision. |
| W09-DV04 → P0-V09 | Discovery and link review | **passed** | Routing row reaches the contract in one link; `docs/testing/README.md` pointer line added without restating policy; contract links (ADR baseline) resolve; stage-index row truthful. |
| W09-DV05 → P0-V13 | Single-source review | **passed** | Tracked-file scan: no script, workflow, or document embeds a QEMU command line for automated use (`scripts/`, `tests/` hold only markers; all QEMU mentions are informational references); the contract is the designated single home. |
| W09-DV06 → P0-V09 | Prerequisite-reference review | **passed** | W02 (delivered pin), W03 (delivered build surface), W05 (delivered conventions), W17 (not delivered — placeholder naming rule with recorded supersession path) are each stated with their assumption and boundary in the implementation record and contract §6. |
| W09-DV07 → W09 closure | Consumability review | **passed** | W19 (placeholder boundary quotable), W20 (future/non-P0 classification prevents misreading absence as a silent skip), P1-W10 (reference invocation, verdict logic, and evidence set implementable without renegotiating semantics), W17 (supersession path recorded), W07 (future-class promotion path) — each can act without inventing policy. |

## Grammar-walkthrough trace (W09-DV03 evidence)

Hypothetical future invocation (P1-style):
`<runner> run --profile p1-boot-smoke --param regression=cycles=3 --timeout 90s`

| Path | Contract resolution | Status | Evidence produced |
|---|---|---|---|
| Success: marker appears within 90 s, success condition met | grammar accepts (`--profile` known, `regression` reserved, finite timeout); profile supplies the QEMU invocation; serial captured byte-complete; cleanup runs | `0` | invocation record + full capture + outcome record w/ condition reference |
| Timeout: marker never appears; 90 s expire | §4 timeout rule: terminate emulator, preserve partial capture + timeout record; timeouts never success | `3` | invocation record + partial capture + timeout/outcome record |
| Launch failure: emulator missing or dies before first observable output | §5 class reserved for environment/runner health; never counted as target failure | `2` | invocation record (with emulator identity attempt) + outcome record |
| Usage error: `--param cpu=16` (unknown class) or absent effective timeout | §2 rules: unknown class is a usage error, never heuristic; infinite timeout invalid | `1` | usage error report; no run, no evidence root written |
| (Boundary) runner cannot write evidence root | §5: unexpected internal situations map to `5` | `5` | whatever survives (launch record) — consumer treats as internal error, not target outcome |

No path required inventing policy: timeout defaults stay with profiles
(calling designs); the contract fixes only the mechanism.

## Not run / not proved

- **No QEMU execution, no runner invocation, no EL2/guest/hardware
  evidence** — none exists in P0 by design; the runner's real behavior is
  proven only by its implementing package (expected P1-W10).
- **CI classification:** not exercised; W20 owns the future/non-P0 wiring.
- **Evidence-file naming:** placeholder rule only until W17 delivers.
