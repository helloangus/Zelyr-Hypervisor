# P0-W12 Logging/Diagnostic Baseline — Implementation Record

**Status:** Implemented on branch `p0/w12-diagnostics-baseline`; verification
evidence in [the verification
record](../verification/p0-w12-logging-diagnostic-baseline-verification.md).
**Date:** 2026-09-18 (Asia/Shanghai)
**Design:** [W12 detailed implementation
design](p0-w12-logging-diagnostic-baseline/README.md)

## Changed artifacts

| Artifact | Change |
|---|---|
| `docs/development/diagnostics-baseline.md` (new) | Normative diagnostics baseline v0.1: four channels with non-substitution lists, five levels with retention defaults, three visibility/trimming classes, minimum fatal information (panic message, crash dump, untrusted-data rule), identity association property, M1–M6/N1–N5 constraint lists, transitional early-console rule, sibling boundaries, thresholds |
| `docs/README.md` | One routing row for diagnostic channel/level/visibility questions |
| `docs/stages/p0/implementation/README.md` | W12 status row updated truthfully |
| This record; the verification record | Decisions and review evidence |

## Forward references (recorded)

The baseline references its siblings' normative homes by pointer, as the
design requires: `trace-event-namespace.md` (P0-W13) and
`version-build-metadata.md` (P0-W16). Both packages are P0 scheduled next in
this stage; the links resolve when their contracts merge, well before final
stage validation. Until then they are forward references of the same kind the
documentation baseline records for unimplemented stage designs.

## Cross-review status (identity association)

W12's identity-association property (§5) requires a two-way cross-review
against W16's delivered contract. W16 is not delivered at W12 closure; the
obligation is recorded as **pending the W16 delivery**, with the cross-review
to be executed and recorded on the W16 side (its design carries the same
obligation). Not blocked-by-conflict — no substantive mismatch can be
assessed until W16's minimum set exists.

## Deviations from the design

None. No code surface, logging API, channel implementation, or metadata field
was defined.

## Handoff notes for downstream packages

- **W13:** owns trace-event naming; the trace channel references canonical
  names, defines none.
- **W14:** owns failure classes; the fatal channel is reserved for
  invariant-fatal exits (delivered contract; cross-link in place).
- **W16:** owns identity fields; must satisfy §5's association property
  (cross-review obligation recorded).
- **P1-W06/P1-W07:** inherit M2 (transport binding) and §4 (fatal minimums)
  respectively; the transitional early-console rule (§6.3) governs P1 start.
