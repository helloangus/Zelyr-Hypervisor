# P6-W12 — fault isolation and robustness

**Status:** Planned work package; implementation not claimed
**Parent:** [P6 task book](../task-book-v0.2.md)
**Prerequisites and consumers:** [P6 plan index](README.md)

## Goal

Establish the P6 failure boundary for invalid virtual-IRQ operations, spurious/unknown Host IRQs, impossible internal states, declared high-rate smoke cases, and the deferred P1 NC6 unexpected-vector execution gate.

## Scope

Cover authorization and ownership rejection, target/range/priority/lifetime errors, no cross-VM or Host effect, diagnostic handling of impossible state, bounded storm smoke evidence, and a genuine asynchronous EL2 unexpected-vector event after W02/W03 Host GIC/IRQ readiness.

## Out of scope

Production-grade DoS policy, device passthrough isolation, a global recovery framework, full fault-injection platform, or detailed panic/error implementation.

## Work sequence

1. Inspect W03, W07, W09, P5 Guest-error/capability boundaries, and the applicable unsafe/diagnostic governance.
2. Produce an approved detailed design for validation, rejection, containment, diagnostic, and escalation outcomes.
3. Integrate expected failure behavior with physical-IRQ lifecycle, vIRQ/maintenance state, and VM isolation assumptions.
4. Define invalid-range/priority/target/owner/dead-object, spurious/unknown, impossible-state, high-rate smoke, and transferred NC6 acceptance cases. Reconcile the W12 detailed design with ADR-061 before implementation.
5. Review that Guest-caused errors remain local and that unexplained invariant failures are visible rather than silently corrupting state.
6. Execute the transferred NC6 case at least twice using a deterministic genuine asynchronous source, retaining complete raw captures and checking origin/category, phase, relevant frame and bounded outcome. Do not substitute a synchronous trap, direct vector branch or fabricated marker.
7. Record factual results, limits, and handoff to W13 and downstream security review.

## Acceptance and closure

P6-V20 through P6-V22 require evidence of safe spurious handling, rejection of invalid authorized operations, and declared smoke limits without observed corruption or unexplained hang. P6-V29 requires the genuine transferred NC6 execution evidence described above. If QEMU cannot produce it, P6-V29 remains blocked pending an approved alternate environment or another architecture decision. Passing does not claim production DoS resistance.

## Handoff

W13 receives evidence and limitations; P7/P8 receive the isolation boundary and any unresolved risk record.
