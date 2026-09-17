# P5-W05 — Capability, rights, bootstrap, and revocation

**Status:** Planned work package; implementation not claimed
**Parent:** [P5 task book](../task-book-v0.1.md)
**Prerequisites and consumers:** [P5 plan index](README.md)

## Goal

Establish the P5 authorization foundation: caller-associated capability and
operation-level rights, explicit bootstrap grant, permission check, and basic
revoke semantics without identity-derived privilege.

## Scope

Cover P5-T07–T12 and the capability portion of T23: object reference plus
rights plus caller association; distinguish observe/control/modify/destroy/
delegate classes; explicit initial grants; valid/no-right/no-capability/type/
state cases; no VM-0/first-VM/role bypass; and grant/use/revoke/reject-old
authority.

## Out of scope

Freezing rights or capability encoding, a delegation tree, attenuation,
parent-child revocation, authentication, RBAC, persistent policy, Control
Domain, policy broker, or complete management operations.

## Work sequence

1. Inspect W02's ABI/error route, W04's object-reference lifecycle, and
   ADR-013 authority constraints.
2. Produce an approved detailed design for authority ownership, rights checks,
   initial-grant authority, revocation, failure recovery, and concurrent-state
   responsibility.
3. Define the planned operation-level permission and bootstrap scenarios,
   including absence, insufficiency, wrong caller, wrong type, invalid object,
   and destroyed/revoked authority.
4. Integrate the planned check with W06's dispatch sequence without making a
   handle, role, VM ID, or first-VM status equivalent to authority.
5. Review future extension to interrupt, IPC, service, and device objects while
   retaining P5's basic grant/check/revoke boundary.
6. Record implementation/evidence status and hand the authority contract to
   W06–W08 and P6 planning.

## Acceptance and closure

P5-V06–V08 require evidence that an explicitly granted caller succeeds only
within its rights, identity shortcuts never authorize, and revoked authority
cannot be reused. Passing does not prove delegation, a policy engine, or a
Control Domain.

## Handoff

W06 receives the required authorization ordering and controlled denial classes;
W07/W08 receive isolation, revocation, and stress scenarios. P6 receives only
evidenced extensible authority facts.
