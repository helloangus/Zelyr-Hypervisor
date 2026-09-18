# P3-W05 Architecture and State

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P3-W05 detailed design](README.md).

## 1. Objects and ownership

| Object | Count | Owner | Writers | Readers |
|---|---|---|---|---|
| `BootPhase` word | one per boot | W05 | boot CPU (both transitions, CAS) | every CPU (gate checks) |
| Per-CPU ready flags | one per candidate | W05 | the owning CPU (release-store) | coordinator (acquire-load) |
| Ready counter | one per boot | W05 | coordinator derives from flags / ready CPUs increment | coordinator, diagnostics |
| `SmpReadyState` record | one per boot | W05 | boot CPU at declaration | W12/W13/tests, boot-integration owner |
| Degraded record (failed set) | one per boot | W05 | boot CPU at declaration (from W03 registry scan) | diagnostics, W12 |

No locks exist in W05. Every object is either boot-CPU-written
pre-publication or single-writer-per-CPU afterwards; that discipline, not
mutual exclusion, is the concurrency model.

## 2. The boot phase model

```text
Bootstrap
    boot CPU: W01 intake, registry build (W03), area/stack allocation (W04),
    provisional environments (W02) — everything that is "global"
    transition Bootstrap -> GlobalInitPublished   [store-release]
GlobalInitPublished
    boot CPU: dispatch secondaries (W02), then its own local init + install
    secondaries: entry, identity, install, ready-signal
    coordinator: wait until (all ready) or (all attempted terminal)
    per ready CPU: W03 admit_online
    transition GlobalInitPublished -> SmpReady   [fenced store-release]
SmpReady
    normal SMP operation; phase word is final for the boot
```

Both transitions are once-only by CAS and by designated actor (boot CPU).
A failed transition attempt (phase already advanced) is an invariant
violation — the boot sequencer is calling twice, which is fatal per the
P0 panic policy, because a re-run of global initialization would corrupt
every boot-static structure.

## 3. The rendezvous protocol

Roles: exactly one **coordinator** (the boot CPU, per W01's designation)
and one participant per attempted secondary.

```text
Coordinator (boot CPU)                    Participant (secondary)
------------------------                  ------------------------
... global init ...                       [cannot exist yet]
publish GlobalInitPublished
dispatch (W02 calls, gate-checked)
own local init + install (W04)
loop:                                     entry -> identity -> install (W04)
  scan W03 registry for terminals             |
  count ready flags (acquire)                 v
  until all attempted ready-or-terminal   signal_local_init_complete
per ready CPU: admit_online (W03)             [release-store flag]
compute SmpReadyState (+ degraded set)
fence; store-release SmpReady
enter idle / normal operation             enter idle / normal operation
```

The wait's exit condition is **terminal-based** (decision 5 of the entry
README): every attempted CPU is either ready (flag set) or terminal in the
lifecycle registry (Failed, or RequestRejected/TimedOut recorded by W02).
This bounds the wait without a timer and gives degraded accounting a
single source (W03's registry).

## 4. Memory-ordering rules (normative)

1. **Global publication.** Every global-init write (topology freeze,
   registry, areas, tables, provisioned stacks) happens-before the
   `GlobalInitPublished` store-release on the same CPU; every observer
   acquire-loads the phase before touching global state. On AArch64,
   release/acquire atomics provide the required ordering; no additional
   fence is required for the phase publication itself.
2. **Ready signaling.** A participant's local-install writes
   happen-before its ready flag store-release; the coordinator's
   acquire-load of the flag makes them visible. One variable, one
   writer, one reader — release/acquire suffices; SeqCst is not used
   anywhere in the protocol.
3. **SMP-ready declaration.** The declaration publishes *multiple*
   variables (registry states including online admissions, ready flags,
   outcome map, degraded record). A full-barrier (`dmb ish`) store-release
   of the `SmpReady` phase makes the entire pre-declaration state coherent
   to every later observer. This is the protocol's one explicit fence;
   its justification is the multi-variable publication, and it must not
   be dropped as "redundant" without a new design decision.
4. **No other synchronization.** Any cross-CPU sharing discovered during
   implementation that does not fit rules 1–3 is either out of W05's
   scope (route it to its owning package) or a design change to record —
   never an ad-hoc lock or barrier.

## 5. Enforcement points (who checks what)

| Rule | Enforced by | Mechanism |
|---|---|---|
| No CPU_ON before publication | W02 requester | `require_published()` gate assertion before dispatch (W02's step) |
| No secondary shared-state touch before publication | secondary entry path | acquire phase check at local-init start; failure = park with diagnostic |
| No readiness signal before local init completes | W04 install | signal is the install sequence's last step (W04's order) |
| No admission without readiness signal | W05 coordinator | `admit_online` called only for signaled CPUs |
| No SMP-ready before declared condition | W05 coordinator | once-only CAS + terminal/ready computation |
| No double global init / double declaration | W05 phase word | CAS failure = fatal invariant |

## 6. Degraded semantics

`SmpReadyState` is the rendezvous's terminal value:

```text
Pending                    boot has not reached the rendezvous end
Ready    { online: n }     all attempted CPUs ready; n admitted online
Degraded { online: n, failed: [LogicalCpuId] }
                           some attempted CPUs terminal-failed; the rest ready
```

- Computing `failed` uses W03's registry states plus W02's outcome map —
  never the ready counter alone (a silent CPU could otherwise look merely
  "slow" forever; the terminal condition forbids that).
- The reference-boot default is **continue with diagnostics**: `Degraded`
  is emitted, the failed set is diagnosed CPU-by-CPU, and boot completes
  with the online set. This makes failures repeatable and observable
  (P3-V02/V05) and keeps the regression matrix meaningful on platforms
  with induced failures.
- The halt-on-degradation profile is Reserved; choosing it is a boot-policy
  decision for the boot-integration owner (recorded open question, not
  resolved here).
- `Degraded` is a terminal, diagnosable outcome — it is never silently
  converted to `Ready`, and nothing may widen `online` beyond CPUs W03
  actually admitted.

## 7. Failure and recovery model

There is no recovery: the rendezvous runs once. Failure modes are (a) a
participant never signals but is terminal — accounted, boot continues
degraded; (b) the phase word is corrupted or a transition is attempted
twice — fatal invariant; (c) the coordinator itself fails — the boot
takes the fatal path (P1-W07) with phase attribution, because there is no
backup coordinator at P3 (a failover design would be new architecture).
