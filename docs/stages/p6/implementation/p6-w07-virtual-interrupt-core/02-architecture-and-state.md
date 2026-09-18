# P6-W07 Architecture, Ownership, and State

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Parent:** [P6-W07 design entry](README.md).

## 1. Position in the P6 event chain and layer placement

W07 is the controller-independent middle of the virtual-interrupt chain:

```text
producers                         W07 (this design)            consumers
-----------                       ----------------------       -------------
W06 timer expiry (Host)    --->   inject -> per-vCPU bank   -> W08 selects and
W11 scenario HVC requests  --->   (pending/active truth,       claims for List
(P5-authorized)                   dedupe, priority)            Registers
                                  claim/return protocol     -> W09 maintenance
                                  completion reports        <- reports
                                                           <- returns evictions
```

W07 is Core-domain code: no system-register access, no GIC types, no
architecture-specific layout. Its only platform-dependent input is the
declared INTID window and priority width, taken from intake at
initialization. Host physical interrupts live entirely in the W03 domain;
they never enter W07 state, and W07 state never becomes a physical IRQ.
This separation is a review gate (W07-DV08), not a convention.

## 2. Logical modules

| Logical module | Responsibility | Owned state | Inputs | Outputs | Non-responsibility |
|---|---|---|---|---|---|
| `virq-core` | Bank state machine: inject, dedupe, selection, claim/return, completion, drain | per-vCPU `VirqBank`s | producer inject calls; W08 claim/return; W09 completion | pending state; claim tokens; summaries | presentation mechanics; authorization policy decisions beyond validation |
| `virq-authorize` | Validation spine shared by both producer classes; Guest HVC request handler over the P5 boundary | none (stateless; uses P5 capability services) | inject requests with producer class | validated parameters or typed denials | P5 dispatch/capability machinery itself |
| `virq-telemetry` | Named counters/events per bank and per event class | counters (inside banks) | state transitions | diagnostics surface | collection/reporting (W13) |

There is exactly one `VirqBank` per vCPU, created with the vCPU and drained
at destruction (P4 lifecycle hook points). The bank is the only place
Guest-directed virtual-interrupt state exists.

## 3. Core objects and ownership (one owner per datum)

| Datum | Sole owner | Written by | Read by |
|---|---|---|---|
| `pending[vintid]` bit + occurrence count | the target vCPU's `VirqBank` (W07) | inject; presentation return (re-pend) | selection; drain; telemetry |
| `active[vintid]` bit | the bank (W07) | presentation return (BecameActive); completion report (clear) | selection exclusion (D6); telemetry |
| `presented[vintid]` flag | the bank (W07), but **set/cleared only through the claim/return protocol** — the LR residency itself is W08's datum | claim (set); return (clear) | selection; drain assertions |
| claim tokens | W08 holds them while presenting; identity issued by W07 | W07 issues; W08 returns | W08 return calls validate token identity |
| priority slots | the bank | inject (validated) | selection ordering |
| LR contents, VMCR, AP registers | W08 exclusively | W08 | never W07 |
| maintenance condition state | W09 exclusively | W09 | never W07 |
| Guest timer condition | W06 exclusively (`../p6-w06-guest-generic-timer/README.md`) | W06 | W06 only |

No static mutable global interrupt state exists; a bank is reachable only
through its owning vCPU object.

## 4. State model

W07's authoritative state per `(vcpu, vintid)` is the triple
`(pending, active, presented)` plus the occurrence counter. Derived
states (diagnostic names, not stored tags):

```text
                 inject (new)        claim (by W08)
   Idle ──────────────────────> Pending ─────────────────> Presented
     ^                            |    ^                      |  |
     |   completion (no pending)  |    | re-inject            |  | re-inject
     |   <------------------------+    | while presented      |  | while
     |                                 v                      v  v presented
     |                              Pending+Presented <-------+  |
     |                                 |                          |
     |             return(StillPending)|                          |
     |                                 v                          v
     |                              Pending <──────────── return(StillPending)
     |                                                            |
     |                        return(BecameActive)                |
     |                                 v                          v
     +──────────────────────────── Active <────────────── Active+Presented
              completion                    |  ^       |           |
              (clears active;                |  |       | re-inject |
              pending re-presented later)    |  |       +───────────+
                                             |  | completion
                      inject while active    |  └──────────> Pending+Active
                      sets pending again <───┘                  (pending waits;
                                                                 D6: not presented
                                                                 until completion)
```

Transition rules with no exceptions:

1. `inject`: `pending = true`, occurrence count +1 (saturating). Never
   creates a second queue entry. Priority is stored on first pending set
   and unchanged by coalescing (re-priority is not a P6 operation).
2. `claim` (W08): valid only from a state with `pending && !active` (D6);
   sets `pending = false`, `presented = true`, issues the token.
3. `return(token, outcome)`: `presented = false`; outcome `StillPending`
   → `pending = true`; `BecameActive` → `active = true` (pending remains
   whatever it was); `Completed` → `active = false` (if pending, the event
   awaits re-selection).
4. `report_guest_completion(vintid)`: `active = false`; if `pending`, the
   event is simply still pending (no duplicate injection is fabricated).
5. An impossible triple (e.g. `presented` without a live claim, or claim
   against `active`) is an invariant violation: counted, diagnosed,
   recovered to the nearest consistent state (return forced), never silent
   ([06](06-validation-and-handoff.md) §2).

The bounded P6 semantics encoded here: pending survives as long as
necessary (P6-V12); repeated arrivals coalesce (P6-V14); active state is
released only by a Guest-visible completion report (no false completion —
the P6-V13 wording that masking does not imply completion also holds
because masking lives at W06/W10 level, never in W07 bits).

## 5. Concurrency model

- **Per-bank lock:** each `VirqBank` has one irq-save spinlock following the
  P3 synchronization contract
  (`../../../p3/plans/p3-w06-concurrency-synchronization.md`). All state
  transitions take exactly one bank lock. There is no lock ordering problem
  because no operation takes two bank locks (multi-target injection calls
  per-target operations sequentially, each O(1)).
- **Producer contexts:** inject may run in IRQ context (W06 PPI conversion),
  VM-exit context (HVC request path), or mainline context. All are bounded
  O(1): bit set, counter increment, priority store. No allocation in inject
  (banks are fixed-size at vCPU creation; the SPI window is a declared
  constant size).
- **Selection context:** `select_next_pending` runs at presentation time
  (vCPU entry/LR refill), not in IRQ context where avoidable; it is
  O(window width) with the window width a small declared constant. W08's
  refill-from-maintenance may call it in maintenance context — bounded by
  the same width (recorded as acceptable per the Coding Guidelines bound:
  finite, small, constant).
- **Cross-vCPU isolation:** an inject targeting vCPU B never touches vCPU
  A's bank (single-bank lock rule makes cross-vCPU races structurally
  impossible in W07).
- **Claim tokens:** opaque typed tokens carry (bank identity, vintid,
  generation); a stale or foreign token return is an invariant violation
  path, counted and rejected — it can only result from a consumer bug, not
  from Guest input (Guests never see tokens).

## 6. Security model

- Every Guest-visible request passes the P5 authorization spine before
  touching state: capability validity/generation, rights on the target
  vCPU object, same-VM rule, class restriction (SGI-only in P6, D4). No
  VM-ID, role, or first-VM shortcut exists anywhere in the path (ADR-051).
- Guest-controlled integers (vintid, target handle, any HVC payload) are
  untrusted: validated by range/liveness/type before use; failures produce
  typed denials mapped to the P5 error taxonomy — never panics, never
  Host-state mutation.
- The Host-mechanism path (W06) is trusted Host code but passes the same
  validation spine with a different authority branch; a Host bug therefore
  cannot silently bypass range checks (defense in depth, cheap O(1)).
- No Guest-visible memory holds W07 state; nothing in W07 is reachable by
  Guest address.

## 7. Telemetry surface (names fixed by this design)

Per-bank saturating counters and trace events; P6-W13 consumes:

| Counter | Meaning |
|---|---|
| `inject_accepted_count` / `inject_coalesced_count` / `inject_rejected_count{reason}` | producer outcomes |
| `presented_count` / `returned_still_pending_count` / `returned_active_count` | claim/return protocol |
| `completed_count` | Guest-visible completions reported |
| `deferred_presentation_count` | selections served after an absence window (correlates with W06 deferred counts) |
| `drain_dropped_count` | pending events dropped at destruction |
| `invariant_violation_count{kind}` | impossible-state recoveries (Host-attributed) |
| trace: `virq_injected`, `virq_presented`, `virq_completed` | per the P0-W13 trace namespace; W13 owns collection |

Rejection reasons are part of the interface (P6-V21 input for W12): see
[04](04-code-contracts-authorization-and-errors.md) §4.
