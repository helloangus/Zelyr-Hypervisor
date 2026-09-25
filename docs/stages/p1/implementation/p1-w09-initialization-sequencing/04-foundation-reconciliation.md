# P1-W09 lifecycle foundation reconciliation

**Status:** Proposed detailed-design correction; no sequencer or boot evidence claim.
**Scope:** Closed phase vocabulary, tracker encoding and W07 consumer seam.
**Version:** v0.1
**Owner/change context:** P1-W09 foundation preflight, 2026-09-25.
**Supersedes:** Interface-contract clauses naming `Unknown(u16)` and an
interior-mutable `BootPhaseTracker::NEW` constant; clarifies Stable indexing.

The tracker is backed by `AtomicUsize`. Reducing an invalid encoded word to
`Unknown(u16)` would lose evidence and contradict the design's own raw-value
preservation rule. The decoder therefore returns `Unknown(usize)`, retaining
the entire observed word without truncation. This changes no legal encoding:
PreBoot=0, Entered/Completed for phases 1–8=1–16, Stable=17.

`InitPhase::Stable` is a terminal outcome but also a named label. An `index`
method claiming a value in 1–8 cannot be total on Stable. The foundation uses
`sequence_index() -> Option<usize>` and returns None for Stable; ordinary
enter/complete methods reject it without changing tracker state. The separate
`mark_stable()` transition accepts only Completed(Stage1)=16.

The accepted design sketches `BootPhaseTracker::NEW` as a constant value.
Because `AtomicUsize` has interior mutability, copying such a constant would
produce fresh trackers and trigger the pinned Clippy policy. A `const fn
new()` constructs each tracker explicitly; only the one static `TRACKER`
is authoritative in target code. Host tests construct isolated instances.

This foundation intentionally supplies only `InitPhase`,
`LifecyclePosition`, `FailureReason`, and `BootPhaseTracker` in a source file
shared with host tests. It is not linked into the target boot module until
W07 and the full W09 sequencer consume it. W09 still owns marker emission,
phase enter/complete wrappers, adapters, `fail_phase`, Stable emission and
the actual boot call. No success stub or fabricated readiness is permitted.
