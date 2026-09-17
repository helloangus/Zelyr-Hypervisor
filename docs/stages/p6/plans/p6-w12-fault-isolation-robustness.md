# P6-W12 — fault isolation and robustness

**Status:** Planned work package; implementation not claimed
**Parent:** [P6 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P6 plan index](README.md)

## Goal

Establish the P6 failure boundary for invalid virtual-IRQ operations, spurious/unknown Host IRQs, impossible internal states, and declared high-rate smoke cases.

## Scope

Cover authorization and ownership rejection, target/range/priority/lifetime errors, no cross-VM or Host effect, diagnostic handling of impossible state, and bounded storm smoke evidence.

## Out of scope

Production-grade DoS policy, device passthrough isolation, a global recovery framework, full fault-injection platform, or detailed panic/error implementation.

## Work sequence

1. Inspect W03, W07, W09, P5 Guest-error/capability boundaries, and the applicable unsafe/diagnostic governance.
2. Produce an approved detailed design for validation, rejection, containment, diagnostic, and escalation outcomes.
3. Integrate expected failure behavior with physical-IRQ lifecycle, vIRQ/maintenance state, and VM isolation assumptions.
4. Define invalid-range/priority/target/owner/dead-object, spurious/unknown, impossible-state, and high-rate smoke acceptance cases.
5. Review that Guest-caused errors remain local and that unexplained invariant failures are visible rather than silently corrupting state.
6. Record factual results, limits, and handoff to W13 and downstream security review.

## Acceptance and closure

P6-V20 through P6-V22 require evidence of safe spurious handling, rejection of invalid authorized operations, and declared smoke limits without observed corruption or unexplained hang. Passing does not claim production DoS resistance.

## Handoff

W13 receives evidence and limitations; P7/P8 receive the isolation boundary and any unresolved risk record.
