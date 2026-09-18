# P8-W05 Validation Matrix and Handoff Design

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P8-W05 detailed design](README.md).

## 1. Evidence boundaries

P8-V07 (Linux CPU compatibility run) and P8-V08 (unsupported-operation
negative test) are runtime evidence produced through the fixture
([P8-W15](../p8-w15-reproducible-linux-fixture/README.md)) and the
automated regression ([P8-W16](../p8-w16-automated-linux-regression/README.md))
routes, or by their equivalent manual runs while those packages land.
Per the plan's acceptance wording: passing these validations does **not**
prove trap/emulation implementation, CPU-feature completeness, or any
per-register behavior beyond the declared scenarios. Review items are
completable before any run; run items must not be claimed from reviews.

## 2. Validation matrix

| ID | Test or review | Suggested technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W05-DV01 → P8-V07 | consumed-contract reconciliation review | inspect the implementation record's per-contract status list against W01 and the P4–P7 records | every consumed contract listed with evidence status; shape differences recorded or resolved; blocked steps identified | the design rests on stated assumptions; not that predecessors work |
| W05-DV02 → P8-V07 | classification completeness review | map [01-classification-model.md](01-classification-model.md) §4 areas to the plan scope (Guest EL, system registers, MMU/cache/TLB, barriers, WFI/WFE, timer, interrupt masking, CPU/features) | every area covered with a class or an explicit routed placeholder; assignment rules and invariants present | the posture is total over the plan's areas; not that every register is classified |
| W05-DV03 → P8-V08 | decision-point exclusivity and fail-closed review | code review of the exit-path integration: enumerate resolution paths | exactly one resolution path; registry miss/missing handler/handler failure all land in Reject; no silent-continue path; registry immutable at runtime | the fail-closed boundary is structural; not that every Guest input was tried (that is V08's run) |
| W05-DV04 → P8-V08 | negative-matrix run | execute the unsupported-operation scenarios (unknown/reserved encodings; EL2-register access from EL1; concealed-feature use; missing-handler route) through the fixture; observe diagnostics | each scenario yields an explicit VM-scoped diagnostic (complete `GuestDiagnosticRecord`), a Guest-observable fault or contained stop, one telemetry event; no silent corruption, no hypervisor panic | controlled rejection works for the declared classes; not full encyclopedic coverage of unimplemented space |
| W05-DV05 → P8-V07 | normal-path run | execute the declared Linux boot scenarios: earlycon + kernel boot with the declared Direct/Emulate paths (EL1 controls, barriers, cache maintenance, WFI blocking with scheduler wakeup, timer/GIC/PSCI routes as their packages land) | declared paths operate under their classified behavior across the pinned fixture boot; WFI wakeups observable; no unclassified exit observed in the run | the classified normal paths support the declared Linux boot stage; not that every Linux-used operation is covered, nor any trap/emulation completeness claim |
| W05-DV06 → P8-V07/V08 | guardrail review | code review against the guardrails: Guest input untrusted at the decode boundary; no host fact in classification data; Guest EL1 preserved; P5/P6/P7 contracts consumed not redesigned; invariant path narrow; bounded exit-path work | every guardrail holds with cited code locations; no new conduit, no sixth class, no widened invariant path | the boundary rules are enforced; not runtime behavior (covered by DV04/DV05) |
| W05-DV07 → W05 closure | claim-hygiene and record review | inspect all W05 artifacts | no completion claim; diagnostics record fields complete-or-flagged; evidence recorded as passed/failed/blocked/not-run with method and date | record integrity; nothing else |

Record each validation as **passed**, **failed**, **blocked**, or **not
run** with method, input, environment (QEMU reference platform where run),
date, and reason. P8-V07 requires DV01, DV02, DV05, DV06, DV07; P8-V08
requires DV03, DV04, DV06, DV07. Blocked run items name the missing
predecessor or fixture per the W01 classes. No validation here proves
P8-V09 through P8-V26, and none may be reported as doing so.

## 3. Error, security, and observability model

**Error model.** Two distinguished classes per
[03](03-controlled-failure-and-diagnostics.md) §4: GuestFault-class
(rejects, missing handlers, handler-recoverable failures — recoverable,
VM-scoped, diagnosable) and InvariantViolation-class (containment failure,
record failure, handler postcondition violation, registry corruption —
narrow, established fatal-diagnostic path). The decision point is total:
every Guest-originated trapped operation ends in Resume,
ResumeWithEffect, VcpuStop, VmFault, or the narrow invariant path — never
in undefined behavior, never in a silently ignored exit.

**Security model.** Guest-derived values (syndrome fields, PC, FAR) are
untrusted: decoded only through the Arch boundary, range-checked before any
address-like use, never dereferenced, never copied as buffers into
diagnostics. No Guest value selects a handler; only the immutable registry
does. Classification data contains no host facts (host-independence rule).
The invariant path cannot be reached by Guest behavior by construction, and
any code change that could reach it from Guest input is a security-relevant
design conflict to raise. Linux-caused faults remain recoverable VM-facing
errors — the hypervisor never treats a Linux bug as a host emergency.

**Observability model.** Per-class/per-area counters (trimmable per the P0
trace rules); one structured event per Reject with a complete-or-flagged
`GuestDiagnosticRecord`; trace correlation per the established telemetry
contracts, degrading to exit-sequence numbering if correlation contracts
differ. The observability surface is exactly what
[P8-W13](../p8-w13-guest-fault-diagnostics/README.md) extends and
[P8-W17](../p8-w17-linux-performance-baseline/README.md) measures; W05
defines the classification and record shape, not the diagnosability
breadth or the performance KPIs.

## 4. Handoff checklist

Before handing W05 to a reviewer, provide:

- the exact changed-file list and the layer-respecting home each contract
  received, with the ADR-layering check for each;
- DV01–DV07 evidence paths and run status, including explicit blocked/not-
  run entries naming the missing predecessor, fixture, or regression route;
- the classification inventory as implemented, with routed placeholders and
  their routes visible;
- confirmation of the guardrails: no feature value or register encoding
  chosen; no sixth class; no WFE policy change; invariant path narrow; all
  Guest input validated at the decode boundary; no host fact in
  classification data; PSCI/vGIC/timer/console mechanisms untouched;
- new `unsafe` inventory with `SAFETY` commentary (expected only at the
  Arch syndrome-decode boundary, if anywhere);
- design deltas recorded against assumed contract shapes (step 1), and any
  conflict routed per the failure boundaries; and
- the open items for consumers: W06/W07/W08/W09 handler-route obligations,
  W13 diagnostic consumption, W18 negative-target list, and the W02 C2
  routed values still pending.
