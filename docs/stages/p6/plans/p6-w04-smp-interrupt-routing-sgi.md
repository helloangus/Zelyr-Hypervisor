# P6-W04 — SMP interrupt routing and SGI

**Status:** Planned work package; implementation not claimed
**Parent:** [P6 task book](../task-book-v0.2.md)
**Prerequisites and consumers:** [P6 plan index](README.md)

## Goal

Establish evidence-backed Host cross-pCPU SGI and basic SPI-routing behavior that later work can use without assuming scheduler policy.

## Scope

Cover declared SGI target forms, target-accounting and completion, supported SPI target changes, P3 notification/lifecycle integration, and diagnostic failure outcomes.

## Out of scope

P7 rescheduling policy, complete remote-vCPU kick behavior, TLB semantics, IRQ load balancing, Guest SGIs, or GIC register/API design.

## Work sequence

1. Inspect W02–W03 and the P3 pCPU lifecycle, notification, synchronization, and TLB-transport contracts.
2. Produce an approved detailed design for bounded SGI/send/receipt/completion and SPI routing-change behavior.
3. Integrate target eligibility and failure handling with online/offline pCPU state and Host IRQ lifecycle rules.
4. Define CPU0-to-CPU1, CPU1-to-CPU0, multi-target, and supported SPI re-route acceptance scenarios.
5. Review that SGI remains a Host mechanism and does not create scheduler or Guest-SMP semantics.
6. Record results and hand the trusted mechanism boundary to W11, W13, and P7.

## Acceptance and closure

P6-V04 through P6-V06 require target-attributed SGI and supported SPI routing evidence with determinate completion. Passing proves neither general RPC nor scheduling policy.

## Handoff

W11 receives Host-SGI scenario inputs; P7 may later design reschedule/kick policy over an evidenced Host mechanism.
