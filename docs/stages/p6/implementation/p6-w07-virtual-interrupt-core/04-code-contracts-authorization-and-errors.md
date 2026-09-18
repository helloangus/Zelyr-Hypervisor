# P6-W07 Code Contracts — Authorization, Validation, and Error Taxonomy

**Status:** Proposed detailed design; implementation and validation are not
claimed.
**Parent:** [P6-W07 design entry](README.md). Contracts follow the checklist
§3 template; all signatures are pseudocode. Prerequisites:
[01](01-scope-and-foundations.md) (decisions D4, D7),
[02](02-architecture-and-state.md) §6,
[03](03-code-contracts-virq-lifecycle.md) §1–§2.

## 1. Producer classes

```text
Name and stability: ProducerClass — enum { HostMechanism { source:
  MechanismSource }, GuestRequest { auth: P5AuthorizationContext } }.
  Internal to W07; stable within P6.
Purpose and caller: distinguish the two authority paths (D4) without
  giving either a private bypass. MechanismSource in P6 is the W06 timer
  source (TimerExpiry); the variant is exhaustive by design so a new
  producer is a compile-time, design-reviewed addition — never a
  catch-all.
Preconditions / postconditions: the P5AuthorizationContext exists only
  after the P5 dispatch produced it; W07 never constructs one from raw
  Guest input.
Security/authorization checks: this type is the seam where ADR-013
  authority meets injection; both branches pass the same validation
  spine ([2]) with different authority evidence.
Validation: W07-DV05.
```

## 2. Validation spine

### 2.1 `authorize_and_validate()`

```text
Name and stability: fn authorize_and_validate(request: RawVirqRequest)
  -> Result<VirqRequest, VirqDenial> — the single pre-state gate for
  every injection; called before virq_inject ([03] §2.1). Internal to
  W07; stable within P6.
Purpose and caller: make "authorized, well-formed" a property every
  producer passes through, in one reviewed place.
Inputs / outputs: raw request (typed candidate values + producer class)
  -> validated VirqRequest or typed denial.
Preconditions: for GuestRequest, the P5 dispatch has already identified
  the caller and produced its authorization context (assumed P5
  contracts: `../../../p5/plans/p5-w02-hypercall-abi-error-boundary.md`,
  `../../../p5/plans/p5-w04-handle-lifecycle-type-safety.md`,
  `../../../p5/plans/p5-w05-capability-rights-bootstrap-revocation.md`);
  postconditions: on success, all typed invariants hold ([03] §1).
Concurrency: stateless; O(1); safe in any context.
Errors: the denial taxonomy of §4; no state changed.
Security/authorization checks (in order):
  1. Handle validity/generation (P5 service) — stale handles fail.
  2. Rights: caller holds the inject-class right on the target vCPU
     object (P5 rights check). No VM-ID, role, or first-VM equivalent
     exists in this path (ADR-051).
  3. Same-VM rule: the target vCPU belongs to the caller's VM
     (cross-VM injection is not a P6 operation; cross-VM signaling is
     P12 Reserved).
  4. Class rule (D4): GuestRequest may target the SGI range (0–15)
     only; PPI/SPI vINTIDs from a GuestRequest are a ClassNotAllowed
     denial. HostMechanism may target any declared range its own design
     authorizes (W06: its own vCPU's timer INTID).
  5. Range/priority: by constructor ([03] §1.1) against the declared
     window/width.
  6. Priority rule (D7): a GuestRequest priority other than the VM
     default is a ClassNotAllowed denial in P6 (recorded bounded
     behavior; W10 may amend).
Logic (pseudocode):
    match request.class {
      GuestRequest { auth } => {
          target = p5_resolve_vcpu(auth.handle)?         // generation
          p5_require_rights(auth, target, InjectRight)?
          require_same_vm(auth.caller_vm, target.vm)?
          require_range(target = sgi_only(request.vintid))?
          require_priority_is_default(request.priority)?
      }
      HostMechanism { source } => {
          target = source.authorized_target()?           // W06: own vCPU
          require_range(request.vintid in source.class_range())?
      }
    }
    VirqRequest { target, class, vintid: VirqId::new(..)?,
                  priority: VirqPriority::new(..)? }
Validation: W07-DV05 (all denial classes exercised).
```

## 3. Guest HVC request handler

### 3.1 `handle_virq_hvc_request()`

```text
Name and stability: fn handle_virq_hvc_request(dispatch_context) ->
  HvcResult — the P6 handler registered for the (P5-designed) injection
  call class; called from the P5 hypercall dispatcher in VM-exit context.
  Internal to W07; stable within P6.
Purpose and caller: turn a Guest's request into authorized pending work,
  or a structured denial — the path P6-W11's VG-IRQ scenarios exercise.
Inputs / outputs: dispatch context (per the P5 ABI design; W07 consumes,
  never re-parses Guest bytes itself) -> structured result in the P5
  result convention.
Preconditions: P5 dispatch validated call routing and copy-safety;
  postconditions: on success, virq_inject performed with GuestRequest
  class; on denial, no state change; every path returns a distinguishable
  structured outcome (valid / supported-but-denied / malformed / guest
  fault / internal error) per the P5 error boundary.
Concurrency: VM-exit context; bounded O(1); no allocation.
Errors: mapped 1:1 from VirqDenial (§4); malformed Guest payloads are
  GuestFault-class (P5 boundary decides fault vs. error code per its
  design); a W07 invariant violation on this path is a Host-attributed
  internal error, surfaced, never charged to the Guest.
Security/authorization checks: everything runs through [2]; the Guest
  cannot observe or forge authorization context.
Logic (pseudocode):
    raw = p5_decode_injection_args(dispatch_context)?   // P5-owned
    request = authorize_and_validate(raw.into())?       // §2
    outcome = virq_inject(request)?                     // [03] §2.1
    telemetry(record outcome); return structured_ok(outcome)
Validation: W07-DV05; VG-IRQ scenario behavior is W11's evidence.
```

## 4. Error taxonomy (W07-owned, P5-mapped)

| W07 denial / error | Meaning | Guest-path mapping (per P5 taxonomy) | State effect |
|---|---|---|---|
| `InvalidRange` | vINTID outside declared window | guest error (recoverable, structured) | none |
| `InvalidPriority` | priority outside declared width | guest error | none |
| `InvalidTarget` | dead/unknown vCPU object | guest error | none |
| `ClassNotAllowed` | SGI-only rule (D4) or default-priority rule (D7) | guest error | none |
| `InsufficientRights` | P5 rights check failed | authorization denial (P5 semantics) | none |
| `BankDraining` | destroy race | guest error; Host path counts it | none (bank untouched) |
| `ProtocolViolation` | claim/return misuse | never Guest-caused | forced consistent + counted |
| `InternalInvariant` | impossible state | never Guest-caused | forced consistent + counted; Host diagnostic |

Guarantees that hold for every row: no panic reaches a Guest-caused error;
no denial mutates any bank; denials are counted with their reason
(P6-V21 input for W12); a Guest cannot distinguish Host internals from
denial reasons beyond the structured classes.

## 5. Intake

```text
Name and stability: fn virq_intake(capability_declaration) ->
  Result<VirqWindow, IntakeError> — one-time window/width validation from
  the W01-reconciled declaration (assumed contract [01] §5). Stable
  within P6.
Preconditions: W01 conclusions exist (entry review, workflow step 1);
  postconditions: window immutable; banks sized from it.
Errors: DeclarationConflict / MissingDeclaration — Guest delivery stays
  disabled with diagnosis (same pattern as W06 intake).
Validation: W07-DV01.
```
