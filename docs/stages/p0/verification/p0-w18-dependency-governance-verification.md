# P0-W18 Dependency Governance — Verification Evidence

**Status:** Complete evidence recorded; W18 closure claimed.
**Date:** 2026-09-19 (Asia/Shanghai)
**Environment:** Documentary review and rehearsal against branch
`p0/w18-dependency-governance` (baseline: merge of PR #28).

## Results

| ID | Review | Result | Evidence / reason |
|---|---|---|---|
| W18-DV01 → P0-V09 | Checklist completeness review | **passed** | All eleven dimensions present with mandatory questions: the six task-book dimensions (TCB proportionality, `no_std`, license, maintenance, unsafe footprint, platform/architecture applicability) plus the plan's additions (transitive, allocation, stability, security, alternatives); the unremovable-dimensions rule and the evidence-source rule (no "unknown", cited sources) are explicit. |
| W18-DV02 → P0-V09 | Tier-boundary review | **passed** | D1/D2/D3 are defined by linkage (what the dependency is linked into), not by prose intent; upward reclassification follows the full D3 path; D3 requires explicit maintainer approval recorded in the register; the dual-use rule routes to D3 or splits. TCB entry cannot pass through an ordinary review. |
| W18-DV03 → P0-V09 | Register schema review | **passed** | The register carries the ten-field entry schema, all seven event types (introduction, upgrade, advisory-response, deprecation, removal, exception, reclassification), maintenance rules (same-change updates, append-oriented correction, policy precedence), and the empty state with no illustrative entries. |
| W18-DV04 → P0-V09 | Completeness rehearsal | **passed** | The hypothetical-candidate walkthrough below traverses the full path with no unanswered step; the candidate remains hypothetical and appears nowhere in the register. |
| W18-DV05 → P0-V09 | Locatability review | **passed** | Routing row reaches the policy in one link from `docs/README.md`; the policy ↔ register links resolve; a tracked-docs search finds no other dependency-rule authority (single home). |
| W18-DV06 → P0-V09 | Unsafe-interface cross-review | **passed** | §5.2 references the delivered unsafe policy's boundary categories by name; it neither restates nor pre-empts the first-party unsafe mechanics; third-party unsafe is weighted (dimension 5) and never adopted into the inventory; both documents interlock with no divergence. |
| W18-DV07 → W18 closure | Consumability review | **passed** | P1 designer (the §4.1 path is complete), build-baseline consumers (register→manifest rule actionable per manifest edit), W07/W20 (the §5.1 consistency predicate is implementable as a future gate) — each can act without inventing policy. |

## Completeness rehearsal (W18-DV04 evidence)

Hypothetical candidate (rehearsal material only; never a register entry):
a future P1 design wants a mature `no_std`-capable CRC/checksum crate linked
into the hypervisor image.

1. **Classify (§2):** linked into a bare-metal image → **D3**; the design
   records the linkage rationale.
2. **Checklist (§3):** all eleven dimensions — TCB proportionality written
   justification (could a ~50-line in-house table do? answered with
   measurements), `no_std` + default-features-off behavior + allocator
   demands, Apache-2.0 compatibility, maintenance health, unsafe footprint
   (aligned against the unsafe policy's boundary categories, weighted not
   adopted), transitive count (zero, cited), AArch64 bare-metal compilation
   demonstrated, allocation paths (none on the decode path, cited), semver
   discipline and proposed version, RUSTSEC history, alternatives (in-house,
   another crate) with rejection reasons. Every answer cites a source;
   "unknown" would be incomplete.
3. **Approval (§2):** D3 → explicit maintainer approval.
4. **Record:** register entry with all ten schema fields; introduction event
   with date and decision reference.
5. **Manifest (§4.1/§5.1):** only after the register entry may the consuming
   design's manifest name the dependency, in the same change; the lockfile
   (§5.3) makes the built version traceable to the approved version.
6. **Later triggers (§4.2):** a major upgrade re-runs the full checklist; an
   advisory triggers the expedited security path with accept-with-risk
   requiring an expiry date.

No step required an undocumented decision; the fail-closed default would
reject any attempt to skip steps 1–5.

## Not run / not proved

- **No real dependency decision has occurred** — the register is empty and
  the workspace names zero dependencies (verified at register creation:
  `Cargo.toml` files contain no `[dependencies]`).
- **Automated consistency checking:** Reserved; W07/W20 future class.
- **Notice production:** Reserved until the first real dependency needs it.
