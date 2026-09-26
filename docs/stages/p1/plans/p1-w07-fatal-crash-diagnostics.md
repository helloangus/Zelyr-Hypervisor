# P1-W07 — Fatal crash diagnostics

Status: Planned work package; implementation not claimed  
Parent: [P1 task book](../task-book-v0.2.md)
Prerequisites and consumers: [P1 plan index](README.md); requires W05/W06 and feeds W08, W11 and W12.

## Goal

Define a bounded fatal path whose output preserves enough information to locate
early failures and does not itself become an unobservable recursive crash.

## Scope

Build/version, CPU/EL, PC/return state, syndrome, fault address, startup phase,
panic message and useful general-register context; normal panic, EL2 fault and
early-init failure behavior.

## Out of scope

Recovery of invariant violations, persistent crash storage, Guest fault policy,
remote logging and a production observability pipeline.

## Work sequence

1. Derive required fields from W05 and P0 panic/diagnostic contracts.
2. Define fatal-path capture, output ordering and terminal behavior.
3. Integrate pre-MMU and post-MMU diagnostic availability.
4. Review recursion, partial-init and unsupported-capability cases.
5. Define crash-report and panic acceptance evidence.
6. Hand off the diagnostic contract to MMU, negative validation and docs.

## Acceptance and closure

P1-V11 and P1-V12: reports contain the required context and panic/fault paths
remain bounded, useful and non-recursive in the defined reference conditions.

## Handoff

W08 must preserve the diagnostic path across MMU enablement; W11 consumes the
fault classes. This package does not authorize automatic recovery.
