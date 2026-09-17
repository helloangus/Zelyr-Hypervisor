# P6-W01 — GICv3 capability discovery

**Status:** Planned work package; implementation not claimed
**Parent:** [P6 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P6 plan index](README.md)

## Goal

Establish a reviewable, capability-driven decision that the current platform can safely support the P6 GICv3 and virtualization work.

## Scope

Inspect P2 platform facts, P1 CPU feature inventory, and the ADR against the GIC, Redistributor, CPU-interface, virtualization-interface, IRQ-number, and MMIO-range inputs P6 requires. Record usable, absent, unsupported, and contradictory outcomes with their diagnostic and escalation boundaries.

## Out of scope

Initializing GIC hardware, choosing register sequences, implementing a generic irqchip framework, or treating a QEMU result as an architectural guarantee.

## Work sequence

1. Inspect the ADR, P6 task book, P0–P5 handoffs, and actual P2 PlatformInfo/evidence.
2. Reconcile required GIC, per-pCPU Redistributor, CPU-interface, virtualization-interface, IRQ, and MMIO capability inputs.
3. Define the planned acceptance and rejection outcomes for missing, unsupported, incomplete, malformed, or conflicting platform facts.
4. Review the result for capability-driven layering, Guest-untrusted inputs, and separation of platform facts from Core policy.
5. Record the entry contract, open investigations, and downstream assumptions for W02–W13.

## Acceptance and closure

P6-V01 requires a linked capability review or evidence record that identifies each required input, its source and compatibility outcome, and a safe rejection path. Passing establishes only a usable planning/implementation boundary; it does not prove GIC initialization or hardware behavior.

## Handoff

W02–W13 receive a reviewed list of required capabilities, facts, blocks, and escalation items. No GIC state, API, or platform-specific Core behavior is implemented.
