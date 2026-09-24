# W09 preflight design correction

**Status:** Proposed detailed design; implementation not claimed.
**Scope:** Single completion owner and precise tracker/replay transitions.
**Version:** v0.2
**Owner/change context:** P1 preflight, 2026-09-24.
**Supersedes:** Interface-contract §6 console completion wording and ambiguous
entry/replay predecessor arithmetic.

## Baseline and required foundation

W02 has an explicit unlinked W09 seam and no production tracker. Required:
exactly one transition owner and unambiguous replay. Reserved: P2 extension
after Stable. Out of scope: service lifecycle, rollback or subsystem state.

## Corrected ownership and sequence

The sequencer alone calls enter/complete for Capabilities through Stage1,
including Console. The console adapter calls bring_up_early_console then
materialize_replay after availability. It never calls phase_complete.
The sequencer then completes Console once and emits its first live marker.
W02-owned glue alone records Entry/Runtime and, after sequencer return, Stable.

For index i in 1..=8, advance_enter expects 2*(i-1), including zero for Entry.
Completed(Runtime) is the sequencer entry precondition, not Entry's predecessor.
Replay at Entered(Console) includes both events of every earlier phase followed
by Console.enter, never Console.complete. Stable is its own encoding and is
rejected by ordinary enter/complete APIs.

Straight-line adapter ownership ensures one replay invocation. If the internal
helper remains separately callable, a private emission guard must reject
duplicate replay without changing authoritative lifecycle position. No second
phase state is introduced.

## Acceptance

W09-DV01/DV02 inspect the complete encoded sequence, rejected duplicates leaving
state unchanged, exact replay prefix and one live Console.complete.
W09-DV03 checks mechanism-only adapters plus the authorized replay call.
W10's oracle must reject duplicates/out-of-order markers. Results belong in
separate implementation/verification records. No phase order, ABI, allocation
or architectural policy changes.

