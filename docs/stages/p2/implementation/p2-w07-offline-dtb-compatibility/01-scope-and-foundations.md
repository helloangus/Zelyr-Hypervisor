# P2-W07 Scope, Foundations, and Policies

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P2-W07 detailed design](README.md).

## 1. Package outcome

Given a tracked fixture DTB image, one call — `check`
([02 §3](02-readiness-report-model.md)) — produces a `ReadinessReport`
whose every row is a mechanical mapping of a W01 intake outcome or a W02
fact state, plus a verdict ("P2-discoverable" / "not P2-discoverable")
that follows deterministically from the rows and the fixture's binding
expectations. The outcome in the strong sense: running the checker on a
blob and booting the same blob exercise the *same* validation and discovery
code, so a PASS row predicts the boot outcome for that fact, and a FAIL row
predicts a boot stop — the offline check "exposes P2 platform-description
gaps before EL2 boot" (plan goal).

## 2. Assumed prerequisite contracts and failure boundaries

| # | Assumed contract | Source | W07 relies on | Failure boundary if delivered differently |
|---|---|---|---|---|
| A1 | W01 intake validators and cursor are host-runnable over byte slices, with the diagnostic taxonomy of the W01 design | [W01 intake boundary](../p2-w01-boot-platform-description-intake/01-intake-boundary.md) (README Decision 8) | Reuse unmodified; placement rules parameterizable | If a placement rule is hard-wired to boot-only inputs, a W01 contract revision is required — sibling-design conflict; W07 must not fork the validator |
| A2 | W02 `normalize` is host-runnable, deterministic, all-or-nothing, with the five-state fact model and fatal set | [W02 foundations §6–§7](../p2-w02-platform-discovery-normalization/01-scope-and-foundations.md) | Reuse unmodified; map outcomes to report classes | Same rule: divergence or a boot-only dependency is a sibling-design conflict, not a W07 workaround |
| A3 | Host test entry, formatting/lint gates, and test organization from the P0 baselines | P0-W07/W08 plans (unimplemented) | Where the checker host target lives and how it runs | If absent, blocked upstream defect; W07 still fixes the artifact contracts |
| A4 | Dependency governance: no new crate without P0-W18 approval | P0-W18 plan | The checker adds zero dependencies (it reuses workspace code + `core`/`alloc`-free host logic) | A perceived need for a DT library offline is the same Reserved path as W01's Decision 2 |

Fixture *provenance* inputs (a QEMU `virt` DTB dump; an RK3566 vendor DTB)
are implementation-time artifacts obtained and recorded per
[03 §2](03-fixture-and-expectation-matrix.md); their absence blocks the
corresponding fixture step, not the mechanism.

## 3. Offline input model

Boot intake consumes `(dtb_phys, dtb_len)` plus machine state (A2 window,
A3 image range). Offline checking substitutes explicit parameters:

| Boot rule | Offline treatment | Report class if violated |
|---|---|---|
| Presence (`DtbAbsent`) | File length 0 / empty input | FAIL |
| Size sanity (`DtbSizeInvalid`: `< 40`, `> max`, `total_size` mismatch) | Runs unchanged; length = file length | FAIL |
| Base alignment (`DtbMisaligned` for base) | Not applicable offline (no base address) — skipped with an informational row | NOT-P2 (informational) |
| Window reachability (`DtbUnreachable`) | Not applicable offline (no A2 window) — skipped with an informational row | NOT-P2 (informational) |
| Image overlap (`DtbImageOverlap`) | Not applicable offline (no A3 range) — skipped with an informational row | NOT-P2 (informational) |
| Header/structure/reservation validation | Runs unchanged | FAIL on any class |
| W02 normalization incl. fatal set | Runs unchanged | FAIL on fatal diagnostics; WARN/FAIL per fact state otherwise |

Binding rules: the three skipped rules are recorded per run as NOT-P2
informational rows so the report is honest about what offline checking did
not exercise (README Decision 2). A future offline mode with a declared
virtual image range is Reserved.

## 4. Single-semantics policy

- Checker code contains zero DT parsing, zero compatible matching, zero
  cell decoding. It calls W01/W02 entry points and reads their outputs.
- A semantics change in W01/W02 automatically changes checker behavior;
  the corpus's binding expectations are what force the change to be
  *reviewed* (a binding delta fails the fixture test until the expectation
  file is consciously updated with rationale).
- Checker code may not name a platform or board; fixtures and expectations
  are data files. The fixture *manifest* names boards as data labels.

## 5. Placement and build boundary

The checker is host tooling: it compiles for the host under the P0-W08
testing baseline and is never linked into the EL2 image. Physical crate
placement is pending P0-W03's workspace (recorded open item); logically it
is a host-only consumer of the platform/discovery library code. It adds no
dependency, no `unsafe` of its own, no filesystem walking (inputs are
explicit byte images from the fixture directory listed in the manifest),
and no CI configuration (P0-W20 owns wiring; W08 consumes the corpus).
Output surfaces: the in-memory `ReadinessReport`, its fixed-text render
for evidence, and the delta list against expectations — nothing else.
