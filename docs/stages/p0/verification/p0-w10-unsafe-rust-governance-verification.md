# P0-W10 Unsafe Rust Governance — Verification Evidence

**Status:** Complete evidence recorded; W10 closure claimed.
**Date:** 2026-09-18 (Asia/Shanghai)
**Environment:** Documentary review against branch `p0/w10-unsafe-governance`
(baseline: merge of PR #20). **No unsafe was introduced by this package and
none was authorized**; every validation is documentary.

## Results

| ID | Review | Result | Evidence / reason |
|---|---|---|---|
| W10-DV01 → P0-V11 | Policy completeness review | **passed** | Policy contains §§1–8: the four binding safe-first rules, the `SAFETY` template with all four required contents (precondition/establishment/failure-class placeholder/inventory id), six review rules, six categories with per-category placement constraints, seven forbidden patterns, three escalation thresholds, the gate predicate, and the single-home inventory pointer. Consistent with ADR-006 and the Coding Guidelines' unsafe rules (which it preserves and wraps). |
| W10-DV02 → P0-V11 | Inventory existence and location review | **passed** | `docs/security/unsafe-inventory.md` exists at the declared location, carries the pointer line to the policy, and holds zero entries with the no-speculative-entries rule stated. |
| W10-DV03 → P0-V11 | Schema auditability review | **passed** | All fifteen schema fields are defined (inventory contract §3) with content, filler, and lifecycle; same-change rule, pre-merge review, four audit triggers, no-silent-edits rule, and history retention are explicit. |
| W10-DV04 → P0-V11 | First-unsafe walkthrough | **passed** | Positive and negative traces (evidence below) resolve without undocumented decisions. |
| W10-DV05 → P0-V11 | Zero-unsafe confirmation | **passed** | `git grep unsafe -- '*.rs'` finds nothing at implementation start and again at closure; no pre-authorization statement exists anywhere; the governance landed before the first unsafe. |
| W10-DV06 → P0-V09 | Discovery and link review | **passed** | Routing row reaches policy and inventory in one link; security README pointer lines added without restating policy; all links resolve; stage-index row truthful. |
| W10-DV07 → W10 closure | Consumability review | **passed** | P1 arch designer (record draftable end-to-end), W07/W20 (predicate checkable, promotion path defined), W18 (first/third-party split explicit), W11 (placement hooks name layers), W14 (failure-class placeholder with re-audit rule) — each can act without inventing policy. |

## First-unsafe walkthrough (W10-DV04 evidence)

**Positive trace** — hypothetical P1 design adds an EL2 system-register
read inside the arch layer:

1. The approved design names the segment, category `arch-register`, and the
   necessity statement (no safe Rust can read the register) → review rule 1.
2. Source carries `// SAFETY: <precondition> — <establishment> — <failure
   class: contained/VM-local/invariant> — U-001` → rule 2, template §2.
3. The same change creates inventory entry `U-001` (`proposed`) with all
   fifteen fields → rule 3, same-change rule.
4. Second reviewer with arch context reviews soundness explicitly; `proposed`
   → `accepted` at merge → rules 4, pre-merge review rule.
5. Minimality review: smallest boundary; no forbidden pattern → rule 5.
6. Change states host-test categories and the QEMU/hardware validation
   covering the wrapped read → rule 6.

Every step has a fixed rule; no decision is invented.

**Negative trace** — a contributor adds `static mut COUNTER: u32` in Core to
avoid a lock:

- Forbidden pattern §5 (`static mut`), plus placement violation (Core), plus
  no design naming it (rule 1). Three independent blocking rules; the path is
  rejection, or a policy-decision threshold if the owner wants to permit a
  named case — with the alternative-considered record required. The
  institution resolves the case without ambiguity.

## Not run / not proved

- **No real unsafe change has passed through the institution** — none exists;
  P0-V11 is satisfied by documentary evidence per its definition.
- **Gate enforcement:** the predicate is defined, not wired (W07/W20 future
  class).
- **W14 taxonomy values:** placeholder wording governs until W14 delivers;
  re-audit rule recorded.
