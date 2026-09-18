# P0-W17 Artifact Naming Baseline — Verification Evidence

**Status:** Complete evidence recorded; W17 closure claimed.
**Date:** 2026-09-18 (Asia/Shanghai)
**Environment:** Documentary review against branch `p0/w17-artifact-naming`
(baseline: merge of PR #27).

## Results

| ID | Review | Result | Evidence / reason |
|---|---|---|---|
| W17-DV01 → P0-V14 | Grammar review | **passed** | Grammar, character set, and parseability rules present: fields are `[a-z0-9]` (`version` adds `.`), `-` separator forbidden in values, `+` only for `+dirty`; parsing is split-on-`-` against the per-class declared subset; the hyphen-free token rule carries the display-name mapping tables (`qemuvirt`→QEMU virt, `orangepi3b`→Orange Pi 3B). |
| W17-DV02 → P0-V14 | Identity-mapping review (W16 cross-review) | **passed** | The §5 mapping table traces every name field to a W16 schema field (arch→target_architecture, platform→platform, profile→build_profile, version→project_version, revision→source_revision, +dirty→dirty); `class` is declared category-not-identity; `name` is the pending ADR-054 token; `capability_summary` and the three reserved compatibility positions are explicitly not name-encodable. "A name is a view of identity, never a second authority" — W16's §5.2 obligation satisfied; recorded in both records. |
| W17-DV03 → P0-V14 | Vocabulary review | **passed** | Five vocabularies present with governance: `zelyr` working token with the ADR-054 migration path; class tokens per the inventory; `aarch64` arch; `qemuvirt`/`orangepi3b` platforms with `host` deliberately not pre-declared; profile vocabulary intentionally empty with the fail-closed rule; §4.6's adding rule requires a named consumer. |
| W17-DV04 → P0-V14 | Inventory review | **passed** | Eleven categories with tokens, first consumers, applicability subsets in canonical order, status (one P0-required, ten Reserved), informative extensions, and format/generation owners; the categories-not-files rule and the non-categories list are present. |
| W17-DV05 → P0-V14 | Name-construction dry run | **passed** | Hypothetical names parsed and constructed by grammar alone (evidence below); no real artifact was named or produced. |
| W17-DV06 → P0-V09 | Discovery and link review | **passed** | Routing row reaches the naming baseline in one link; links (W16 metadata, build-choice governance, portability rules, runner entry) resolve; stage-index row truthful. |
| W17-DV07 → W17 closure | Consumability review | **passed** | Producing build designs (format declaration + §6 row), W19/W20 (evidence naming + `+dirty` semantics), W09/P1 runner consumers (placeholder superseded compatibly), ADR-054 owners (migration path) — each can act without inventing policy. |

## Name-construction dry run (W17-DV05 evidence — hypothetical, none produced)

| Hypothetical governed name | Parse | Verdict |
|---|---|---|
| `zelyr-hypervisor-aarch64-qemuvirt-<profile>-0.1.0-20cae8a` | split on `-`: 7 fields against the `hypervisor` row's full-form subset, in canonical order | **well-formed** (profile token must exist in the vocabulary first — fail-closed today) |
| `zelyr-hypervisor-aarch64-qemuvirt-0.1.0-20cae8a+dirty` | profile omitted — **not declared** by the `hypervisor` row | **ill-formed**: an undeclared omission; the row cannot drop profile |
| `zelyr-report-qemuvirt-0.1.0-20cae8a` | 5 fields vs the `report` row (name, class, platform, version, revision) | **well-formed** per row (Reserved class; grammar supports it now) |
| `Zelyr-Hypervisor-AARCH64-0.1.0-abc` | uppercase, missing fields | **ill-formed**: character set + subset violations |
| `zelyr-hypervisor-aarch64-qemuvirt-0.1.0-20cae8a.elf` | extension not part of identity | **well-formed** class-part; `.elf` is informative-only |

## Not run / not proved

- **No governed artifact exists**; the first `hypervisor`-class name instance
  is the owning build design's deliverable (expected not-run at W17 closure,
  as the plan anticipated).
- **Reproducibility of names:** stated as a determinism rule; no repeated
  production run exists yet.
