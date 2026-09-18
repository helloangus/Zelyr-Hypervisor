# P0-W14 Panic/Failure Classification — Implementation Record

**Status:** Implemented on branch `p0/w14-failure-classification`; verification
evidence in [the verification
record](../verification/p0-w14-panic-failure-classification-verification.md).
**Date:** 2026-09-18 (Asia/Shanghai)
**Design:** [W14 detailed implementation
design](p0-w14-panic-failure-classification/README.md)

## Changed artifacts

| Artifact | Change |
|---|---|
| `docs/security/failure-classification.md` (new) | Normative failure taxonomy v0.1: five classes (FC-INVARIANT, FC-GUEST, FC-RESOURCE, FC-UNSUPPORTED, FC-PLATFORM) in a uniform field order, propagation/classification rules incl. the guest-triggerability test and escalation rule, the panic-worthiness rule, design/code checklists (D1–D4, C1–C4), informative ADR §13 cross-map, thresholds |
| `docs/README.md` | One routing row for failure-path design/fatal-path review |
| `docs/security/README.md` | One pointer line (no policy restated) |
| `docs/stages/p0/implementation/README.md` | W14 status row updated truthfully |
| This record; the verification record | Decisions and review evidence |

## Deviations from the design

None. No error type, handler, fault mechanism, code, gate, or CI workflow was
added. The unsafe policy's `failure-class` placeholder (contained /
VM-local / hypervisor-invariant violation) now resolves to this taxonomy at
the next audit trigger; no inventory entry exists yet, so no re-audit was
required.

## Handoff notes for downstream packages

- **W12:** owns output content minimums per channel; this taxonomy owns
  which classes may produce what (fatal channel = FC-INVARIANT exits only).
- **W10:** `SAFETY`-comment failure-class values now follow this taxonomy;
  the re-audit rule covers any entry written before delivery (none exist).
- **W13:** trace/metric facts for FC-GUEST/FC-RESOURCE/FC-UNSUPPORTED consume
  the namespace W13 delivers.
- **P5 (management ABI):** owns final class assignment for
  management-domain untrusted input (§2 open item recorded).
- **P1+:** every failure path names its class at detection (D1); the
  panic-worthiness rule is the first review check on any new fatal path.
