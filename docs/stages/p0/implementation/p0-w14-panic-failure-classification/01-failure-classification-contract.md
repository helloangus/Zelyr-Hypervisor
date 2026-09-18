# P0-W14 Failure Classification Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W14 detailed design](README.md).

## 1. Logical artifact groups and ownership

W14 is governance work, so its logical modules are authoritative artifact
groups, not Rust modules.

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Failure-classification document | `docs/security/failure-classification.md` | §12/§19, ADR-007/013/035/051, plan class list, W10/W12/W13 boundaries by subject, this design | the sole normative home of the class taxonomy, propagation rules, diagnostic/review embedding, and thresholds; it defines no error type, handler, or fault mechanism |
| Documentation routing | one row in `docs/README.md` routing table | classification document location | discoverability; it does not restate policy |
| Stage implementation index | `docs/stages/p0/implementation/README.md` | design status | a truthful status row; it never claims completion |
| Implementation record | `docs/stages/p0/implementation/p0-w14-panic-failure-classification-record.md` (created when work starts) | actual decisions taken | changed artifacts, deviations; no command logs |
| Verification record | `docs/stages/p0/verification/p0-w14-panic-failure-classification-verification.md` (created when evidence exists) | actual commands and review output | run/not-run evidence per the validation matrix; not part of the design |

The artifact named in the second column is the sole authoritative home for
the statement in its row.

## 2. Classification document form

The document is normative, carries the status header required by
`docs/README.md`, and must contain exactly the sections of §3–§7 below. Each
class section follows the same field order so classes are comparable:
definition; typical detection points; containment scope; allowed responses;
prohibited responses; diagnostic treatment; informative examples.

## 3. The five failure classes

### 3.1 FC-INVARIANT — hypervisor invariant violation

- **Definition:** the hypervisor detected that one of its own guarantees no
  holds (memory ownership, capability soundness, exception-return
  correctness, scheduler state, or another invariant an approved design
  declares).
- **Detection:** internal assertion-equivalent checks at the points the
  owning design declares — never triggered by external input alone (see §4's
  classification test).
- **Containment:** none — by definition the guarantee is broken. The only
  correct behavior is a fatal exit.
- **Allowed responses:** terminate execution through the fatal path with the
  full failure classification, site, and build identity ([W12](../p0-w12-logging-diagnostic-baseline/README.md)
  minimums, by reference).
- **Prohibited responses:** downgrade to a recoverable error, log-and-
  continue, retry, or swallow. The hypervisor must not operate without the
  guarantees it claims.
- **Diagnostic treatment:** the fatal/crash channel, exclusively.
- **Examples (informative):** a page discovered with two owners; a capability
  table slot resolving to the wrong object generation; a return to an
  exception level with a corrupted saved state.

### 3.2 FC-GUEST — guest-caused fault

- **Definition:** a failure caused by guest-controlled input or guest
  behavior: invalid addresses, lengths, indices, register state, descriptor
  content, hypercall arguments, or prohibited operations (ADR-007's untrusted
  set).
- **Detection:** validation and permission checks on guest-influenced paths;
  synchronous fault handlers for guest-triggered conditions.
- **Containment:** the faulting VM's context only — the vCPU, the VM, or the
  involved virtual device. Never another VM, never hypervisor-global state.
- **Allowed responses:** inject a fault to the guest; terminate the offending
  vCPU or VM; refuse the operation. The owning design selects per path.
- **Prohibited responses:** global panic or hypervisor shutdown (the §19
  MUST, operationalized); silent retry loops that hide hostile patterns;
  sharing the failure across VM trust boundaries.
- **Diagnostic treatment:** VM-scoped diagnostics plus trace/metric facts
  under W12/W13 governance; never the fatal channel.
- **Examples (informative):** a Stage-2 fault from an unmapped guest access;
  an out-of-range hypercall buffer; a malformed virtio descriptor.

### 3.3 FC-RESOURCE — resource exhaustion

- **Definition:** a request cannot be satisfied because a governed resource
  is unavailable or a limit is reached: pages, capability slots, event
  slots, queue depth, CPU budget.
- **Detection:** allocator and pool boundaries; admission checks.
- **Containment:** the requesting context; other tenants keep their
  entitlements.
- **Allowed responses:** refuse, queue with backpressure, or degrade the
  requester, per the owning design's policy; the refusal is observable.
- **Prohibited responses:** panic; silent success; revoking another context's
  resources to satisfy the request outside a designed reclamation policy.
- **Diagnostic treatment:** metric facts (pressure, refusal counts) plus the
  operation's own refusal result; fatal only via the §5 escalation rule.
- **Examples (informative):** a page-frame allocation failing under memory
  pressure; a VM creation refused at a configured limit.

### 3.4 FC-UNSUPPORTED — unsupported feature / hardware capability

- **Definition:** a requested operation or required mechanism is absent:
  a platform capability the hardware lacks, a feature the build excludes, or
  an operation no design has implemented.
- **Detection:** capability/property queries against the platform
  description ([W11](../p0-w11-platform-portability-guardrails/README.md)
  semantics); feature gating; explicit unimplemented markers in designs.
- **Containment:** the requesting path.
- **Allowed responses:** refuse, disable the dependent path, or report the
  gap; the refusal states what is unsupported.
- **Prohibited responses:** silent fallback that pretends support; panic;
  ad-hoc probing that bypasses the capability model.
- **Diagnostic treatment:** capability/property records and, where useful,
  trace facts; a repeated unsupported request is observable, not fatal.
- **Examples (informative):** a stage without an SMMU capability requested
  for isolation; an operation intentionally out of a stage's scope.

### 3.5 FC-PLATFORM — platform / firmware failure

- **Definition:** the platform or firmware misbehaves or fails: discovery
  data inconsistent, a firmware call failing, a controller the platform
  depends on not responding, a quirk's assumed condition violated.
- **Detection:** discovery validation, firmware-call status, timeout and
  sanity checks at SoC/BSP boundaries.
- **Containment:** the affected platform function; the containment scope is
  the owning BSP/SoC design's to declare.
- **Allowed responses:** the platform handling its own failure, disabling
  the affected function, or reporting the failure upward — per the owning
  design.
- **Prohibited responses:** ignoring a failed discovery/firmware call and
  continuing as if it succeeded; panic as the first response for a
  non-integrity-threatening failure.
- **Diagnostic treatment:** platform/BSP-scoped diagnostics; escalation only
  via §5.
- **Examples (informative):** a PSCI call returning an error; discovery data
  describing memory that then faults; a mandatory controller not responding.

## 4. Propagation and classification rules

- **Classify at detection.** Every failure path in a future design names its
  class at the detection point. A path without a class is a design defect.
- **Classification test.** If guest-controllable input can trigger a
  condition, that condition is FC-GUEST (plus, where applicable, a robustness
  defect to fix), never FC-INVARIANT. A design claiming a guest-triggerable
  condition as an invariant violation has misclassified it.
- **No downgrade of FC-INVARIANT.** An invariant violation is never converted
  to a recoverable error, a warning, or a metric.
- **Authorization denial is an expected refusal,** recorded and returned per
  the authorization model (ADR-013/ADR-051) — not a failure class. A bypassed
  or corrupted authorization mechanism is FC-INVARIANT.
- **Input source decides input-failure class.** Guest-derived → FC-GUEST;
  hypervisor-internal misuse → a defect surfacing under FC-INVARIANT rules;
  management-domain untrusted input → FC-GUEST's containment principle by
  analogy, with final class assignment owned by the P5 hypercall/management-
  ABI error-boundary design (open item).
- **Escalation rule.** A non-invariant failure may escalate to fatal only
  when continuing would violate a hypervisor invariant; the escalation is an
  FC-INVARIANT exit that names the threatened invariant. Ordinary
  unavailability is not escalation.

## 5. Panic-worthiness rule

Only FC-INVARIANT exits (including §4 escalations) may terminate the
hypervisor. All other classes must terminate only within their containment
scope. This rule is what W12's fatal-channel reservation enforces and what
review checks first on any new fatal path.

## 6. Diagnostic and review embedding

- **Diagnostic treatments** are stated per class in §3 and consume W12's
  channel semantics by reference; W12 owns output content minimums, W14 owns
  which classes may produce what.
- **Design-review checklist** (required content of the document):
  - D1: every new failure path names its class and containment scope.
  - D2: no path lets guest-controlled input reach the fatal path.
  - D3: resource refusals are observable and policy-owned.
  - D4: unsupported capabilities surface through capability/property queries,
    not silent fallbacks.
- **Code-review checklist:**
  - C1: no panic/abort construct on a guest-influenced path.
  - C2: fatal exits carry the W12 minimums.
  - C3: unsafe segments' SAFETY statements name the failure-class consequence
    ([W10](../p0-w10-unsafe-rust-governance/README.md) hook).
  - C4: containment violations (cross-VM effect of an FC-GUEST response) are
    review failures.

## 7. ADR §13 cross-map (informative) and change thresholds

- Informative cross-map: GuestFault→FC-GUEST; InvariantViolation→FC-
  INVARIANT; ResourceExhausted→FC-RESOURCE; HardwareFailure→FC-UNSUPPORTED/
  FC-PLATFORM; InvalidInput/PermissionDenied are outcome vocabulary inside
  §4's source rule and authorization rule. §13 is a crate-boundary
  suggestion; this mapping amends no ADR decision, and finer error-type
  granularity remains future designs' freedom within the class semantics.
- **Routine:** clarifying wording; adding informative examples.
- **Policy decision** (recorded issue and owner decision): adding a class;
  redefining a class's containment or response boundary; changing the
  escalation rule.
- **ADR required:** permitting guest-caused faults to escalate to global
  panic; allowing invariant downgrades — each contradicts the §19 invariants
  and follows `docs/adr/README.md`'s process.
