# P6-W04 Code Contracts — SPI Routing Change

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P6-W04 detailed design](README.md).  
**Convention:** checklist §3 contract template. All GICD access via the
W02 register-access surface; the IROUTER class is a 64-bit register — the
surface's 8-byte-aligned accessor variant applies (per
[W02 03](../p6-w02-physical-gic-bring-up/03-code-contracts-register-access.md)
§3.1 note). Pseudocode is design logic.

## 1. `change_spi_route`

```text
Name and stability: fn change_spi_route(spi: SpiId, target: PcpuId)
  -> Result<RouteChange, RouteError> (P6-internal; the only runtime SPI
  re-routing path; W02's Phase A set the initial routes and never runs
  again)
Purpose and caller: move a supported SPI's delivery target determinately
  (P6-V06); callers are validation scenarios (W11) and approved consumer
  designs; there is no default re-routing anywhere in P6
Inputs / outputs: SPI ID + target pCPU → change record | rejection
Preconditions / postconditions:
  - target eligible (P3 online ∧ W02 LocalReady) else Err(TargetIneligible)
  - spi in the supported range else Err(IdUnsupported)
  - under the distributor lock: SPI disabled ∧ not pending ∧ not active,
    else Err(RouteEnabled)/Err(RouteBusy) with state unchanged
  - success: IROUTER<spi> holds the target affinity (IRM=0, reserved
    fields zero), ordered by dsb() before unlock; route table updated;
    route event emitted with old and new affinity
  - the SPI remains disabled after the change; re-enabling it is the
    caller's explicit next step through the W03 enable path (separation:
    W04 owns where delivery goes; W03 owns whether it is enabled)
State and ownership change: exactly one transition per call in the route
  state machine ([02](02-architecture-and-state.md) §2.4)
Concurrency/allocation context: distributor-scoped lock held for the
  check-check-write sequence; no allocation; never called from IRQ
  context (documented requirement; re-route is configuration work)
Errors: TargetIneligible, IdUnsupported, RouteEnabled, RouteBusy,
  LockDegraded (if the P3 lock contract reports a pathological state —
  recorded, propagated, not retried)
Security/authorization checks: Host-internal surface; target namespace is
  pCPU only; no VM/route coupling exists in P6
Logic: as the change_route sequence of
  [02](02-architecture-and-state.md) §2.4
Validation: W04-DV06 (re-route reaches replacement target), DV07
  (rejection paths)
```

## 2. Determinacy statement (Required review content)

The change implements this contract, stated for reviewers and consumers:

- Before the call returns `Ok`, no delivery of `spi` to the old target is
  in flight (disabled + inactive checks under the lock), and every future
  assert of `spi` is delivered to `target` (IROUTER programmed and
  ordered before any subsequent enable).
- Between the caller's disable and the route change, an assert that
  arrives is latched as pending at the GIC — the pending check rejects
  the change (`RouteBusy`), so the caller observes and handles it (its
  consumer semantics, typically: let the old-target delivery complete,
  then retry). No loss and no double-delivery window exists in the
  accepted path.
- Concurrent `change_spi_route` calls serialize; the second caller's
  checks observe the first caller's writes (lock ordering).

## 3. `query_spi_route`

```text
Name and stability: fn query_spi_route(spi: SpiId) -> Result<PcpuId,
  RouteError> (P6-internal; read from the route table, not from hardware;
  the table is kept coherent by the same lock)
Purpose and caller: let consumers/W11 record the expected target
  (P6-V06's "recorded target")
Postconditions: reflects the last accepted change; before any change it
  reports the W02 initial route (boot affinity)
Errors: IdUnsupported
Validation: consistency test table-vs-hardware in W04-DV06 evidence
```

## 4. Enable-state interplay (boundary restatement)

W04 deliberately does not enable/disable SPIs: enabling and disabling run
through W03's `enable_id`/`disable_id`
([W03 04](../p6-w03-physical-interrupt-lifecycle/04-code-contracts-dispatch-completion.md)
§3.2) under the same lock class, so the lock, not politeness, guarantees
that a route change and an enable cannot interleave. The recommended
consumer sequence is: `disable_id` → `change_spi_route` → `enable_id`,
each step's errors handled by the consumer design. This division keeps
one owner per transition (W03: enabled-state; W04: route) and is the
review standard for W04-DV06.

## 5. Explicitly not authorized here

No IRM=1 ("any CPU") route programming, no affinity rebalancing or
periodic re-routing, no trigger-type or priority changes (W03/W10
boundaries), no ITS/MSI route concepts, no Guest-visible route state, and
no route cache shared with Guest models (P8 owns any Guest-visible GIC
model separately).
