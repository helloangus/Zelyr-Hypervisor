# P1-W07 Implementation Reconciliation

**Status:** Proposed detailed-design correction; no execution claim.
**Scope:** Failure-class, guard, and bounded-renderer preflight for P1-W07.
**Version:** v0.1.
**Owner/change context:** P1-W07 coding preflight, 2026-09-25.
**Supersedes:** The conflicting statements identified below in this W07 design only.

## Failure classification

The parent README's statement that every P1 failure is FC-INVARIANT is
incorrect under the normative [P0 failure classification](../../../../security/failure-classification.md)
§1–§3. A missing hardware capability is detected as FC-UNSUPPORTED; a false
reference-platform or firmware premise is FC-PLATFORM. Neither class may be
sent directly to the fatal channel merely for being unavailable. If the
capability or platform function is a declared prerequisite of safe P1 EL2
operation, continuing would violate the named boot invariant "do not run the
EL2 runtime without the required mechanism". The detecting owner records its
original class and the threatened invariant, then explicitly escalates to an
FC-INVARIANT terminal exit under §2. W07 renders that terminal decision; it
does not reclassify detection, create a new refusal policy, or make an ordinary
unsupported feature fatal. This rule governs the report-model degradation
paragraph and every W09/W11 route consuming W07.

## Guard and readiness storage

The actual W02 panic guard is `AtomicBool`, not the plain unsafe static
assumed by the W07 design. W07 transfers the single handler and its guard
semantics using safe atomics. Its readiness flag may use a safe atomic too.
Neither storage primitive needs an unsafe-inventory entry; only actual
register/assembly boundaries do. The guard remains terminal and never resets.
The W05 exception-entry guard is distinct and remains owned by W05.

## Formatter and implementation seams

The report model prints x00–x30 and SP on separate bounded lines. A fixed
128-byte line buffer therefore suffices if every long field is truncated at
a UTF-8 boundary with an explicit suffix and the final line count remains
fixed. The earlier "32 × 18 characters plus labels" calculation describes
aggregate report size, not one stack line; the implementation record must
state both maximum line size and total maximum line count. No reachable
`unwrap`/`expect` is allowed on the fatal path. W05's frame is still being
implemented on its separate branch; W07 may implement its own pure model but
must not claim exception wiring until that seam is present and reviewed.

These corrections preserve the plan's goal, P1-V11/P1-V12, and the report's
three-kind shape. They neither add a new architecture policy nor claim
implementation or verification.
