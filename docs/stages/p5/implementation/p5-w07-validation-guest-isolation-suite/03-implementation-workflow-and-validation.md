# P5-W07 Implementation Workflow and Validation

**Status:** Proposed detailed design; implementation and validation are not
claimed.  
**Parent:** [P5-W07 detailed design](README.md).

## 1. Preconditions and failure boundary

Before changing any file, the implementer verifies it has loaded the
documents named in the parent README and inspects the actual state:

- the W02, W03, W05, and W06 detailed designs and implementations exist and
  their records confirm the outcome classes used in
  [01](01-scenario-matrix.md);
- the P4-W05 Validation Guest asset and P4-W08 automation entry point exist
  as implemented, evidenced artifacts (P4-W09 handoff), including the debug
  channel the markers use;
- the delivered W03 rules for P5VG-050/054/057 are transcribed into the
  matrix rows;
- the W05 grant and revoke contracts are implementable from the test
  bootstrap side (Hypervisor-authored grants for two contexts).

Stop and obtain direction instead of guessing when: a prerequisite
design/record is missing or contradicts the matrix (blocked prerequisite or
`Architecture Change Request`); the P4 foundation cannot boot the
two-context composition ([02 §2](02-two-context-isolation-and-harness.md)
failure boundary — no weaker substitute); the delivered supported-call set
cannot express a required scenario family (record the gap against W02/W06;
do not invent calls); or implementing the suite appears to require a Guest
SDK, inter-VM communication, or machine-ABI surface (scope violation —
stop).

## 2. Ordered implementation steps

### Step 1 — reconcile prerequisites and fix the expectation baseline

Target: implementation record
(`../p5-w07-validation-guest-isolation-suite-record.md`, created in this
step).

Work: confirm each prerequisite above; transcribe the delivered W03 rules
into P5VG-050/054/057; fix the test-bootstrap constants (test slot values,
initial generation, the A0 handle constant) per
[02 §2.2](02-two-context-isolation-and-harness.md); record the declared
environment (QEMU version, CPU count, boot composition).

**Acceptance:** the record contains the confirmed prerequisite list, the
transcribed rows, and the fixed constants; every matrix row has a defined
expected class.  
**Failure/blocker:** a missing or contradicting prerequisite stops the
affected groups and is recorded per §1.

### Step 2 — implement the single-context scenario set

Target: the Validation Guest asset (P5 scenario extension) and the
Hypervisor-side test bootstrap (grants, test objects, revoke/re-grant and
destroy/re-create steps the matrix requires).

Work: implement groups A–G ([01](01-scenario-matrix.md)) as deterministic
scenario programs emitting the marker grammar; implement the bootstrap
steps each row's precondition requires (A2 without authority, observe-only
grant, revocable capability, forbidden-state object, destroyed/re-created
objects). Keep each scenario's expected class fixed from the matrix.

**Acceptance:** every group A–G scenario builds, has one marker, and no
scenario contains pointer values or free-text fields; the bootstrap performs
no Guest-negotiated grants.  
**Failure/blocker:** a scenario that cannot be expressed without new
hypervisor surface is a prerequisite gap — record against the owning
package; do not extend the boundary from W07.

### Step 3 — implement the two-context composition

Target: boot composition, per-instance scenario programs, and the B-side
scenarios (P5VG-100–103).

Work: compose two instances on the P4 boot path per
[02 §2](02-two-context-isolation-and-harness.md); bind disjoint grants;
embed (or harness-pass) the A0 constant; implement the negative control.

**Acceptance:** P5VG-100–103 exist exactly as specified, including the
in-boot negative control; the composition uses no management surface and no
identity-based grant.  
**Failure/blocker:** the two-context failure boundary of §1/[02 §2] —
blocked prerequisite plus `Architecture Change Request`; no simulation
substitute.

### Step 4 — wire the harness expectations

Target: expectation list and run procedure extending the P4-W08 entry point
([02 §4](02-two-context-isolation-and-harness.md)).

Work: encode scenario order, expected classes, repeat counts, and timeouts
from the matrix; implement the non-success classes verbatim; ensure verdicts
are per-scenario and mechanical.

**Acceptance:** a dry run against a deliberately broken expectation yields
`marker-mismatch`, not a pass; timeout and unsupported-environment paths
yield their classes.  
**Failure/blocker:** any need to reinterpret markers at match time is a
design violation — fix the expectation, never the matcher.

### Step 5 — marker-discipline and boundary review

Target: review evidence in the verification record.

Work: review every emitted marker and scenario constant against the
discipline of [02 §3](02-two-context-isolation-and-harness.md): no Host
pointer or Guest buffer content, no ABI promise, no free text; confirm the
suite reveals no Host information even under the negative scenarios; confirm
scenario additions followed the maintenance rule.

**Acceptance:** recorded review with per-item findings; any disclosure
finding fixed before validation runs.  
**Failure/blocker:** a disclosure-class finding is a security-relevant
defect — stop scenario runs until resolved and record.

### Step 6 — validation runs

Target: verification record
(`../../verification/p5-w07-validation-guest-isolation-suite-verification.md`).

Work: execute the §3 matrix rows in scope (DV01–DV09); run the declared
repeats for stability rows (P5VG-003, P5VG-103); record W06's dispatch-side
categories for the same boots where the W06 integration makes them
available; record every command, environment value, and result; enter
explicit not-run rows with reasons.

**Acceptance:** every executed row meets its pass condition with evidence;
failures recorded as failures with diagnosis.  
**Failure/blocker:** a failed isolation row (P5VG-101/102) is a stage-level
security finding — record and escalate; it may not be re-scoped by W07.

### Step 7 — records and handoff

Target: implementation record; handoff section of the verification record.

Work: complete the record (changed files, dependencies added to the Guest
asset if any — expected none beyond the established toolchain, new `unsafe`
expected none in scenario code); deliver the marker set, scenario inventory,
and non-success classes to W09; deliver the inventory and evidence links to
W10.

**Acceptance:** handoff artifacts located per the README's downstream
section.  
**Failure/blocker:** a missing artifact blocks closure review, not the
consumer.

## 3. Validation matrix

Record each validation as **passed**, **failed**, **blocked**, or **not
run** with command, input, environment, timestamp, and reason. All QEMU rows
are QEMU-only evidence in the declared environment: they prove Guest-visible
behavior of the delivered P5 mechanisms there; they do not prove AArch64
hardware semantics, real-hardware behavior, behavior of future hypercalls,
or any machine ABI.

| ID | Test or review | Technique | Passing condition | Proves / does not prove |
|---|---|---|---|---|
| W07-DV01 → P5-V11 | prerequisite and expectation review | Step 1 | prerequisites confirmed; every row has a fixed expected class; constants fixed | the suite is well-defined; not that it passes |
| W07-DV02 → P5-V11 | group A run | Step 6, QEMU boot | P5VG-001–003 markers match | discovery and the valid full chain are Guest-observable (with W06-DV03); not general management |
| W07-DV03 → P5-V11 | group B + F run | Step 6, QEMU boot | P5VG-010–016, P5VG-050–057 match | malformed/address/overflow/type containment is Guest-observable; not fuzz exhaustiveness (W08) |
| W07-DV04 → P5-V11 | group C + G run | Step 6, QEMU boot | P5VG-020–027, P5VG-060–062 match | handle/lifecycle negatives behave as specified; not object-system completeness |
| W07-DV05 → P5-V11 | group D + E run | Step 6, QEMU boot | P5VG-030–033, P5VG-040–042 match | authority, revocation, and no-identity-shortcut behavior observable (INV-P5-04/06/10); not delegation |
| W07-DV06 → P5-V12 | two-context isolation run | Step 6, declared composition | P5VG-100–103 all pass including the negative control | one context cannot use another's authority despite knowing its value; not memory isolation, scheduling, or multi-VM policy |
| W07-DV07 → P5-V15 (support) | marker-discipline review | Step 5 + harness `unexpected-output` class | no disclosure-class finding; grammar exact | safe observability of the suite; not the telemetry system (W09) |
| W07-DV08 → P5-V11/V12 | repeatability run | declared repeat boots for P5VG-003 and P5VG-103 | identical classes across declared repeats | stability in the declared environment; not statistical robustness |
| W07-DV09 → W07 closure | inventory and handoff review | Step 7 checklist | scenario inventory, evidence links, and W09/W10 artifacts located | handoff readiness; not downstream completion |

The suite's Guest-side rows are the stage-level proof surface for
INV-P5-01 through INV-P5-10 as mapped in the task book §6 and the matrix
groups; W06's DV rows prove the dispatch side of the same invariants.

## 4. Error, security, and observability model

The suite's security property is disclosure discipline: markers prove
behavior without revealing Host information, and the negative scenarios
prove authority without leaking capability internals. Its error model is
the non-success class table of
[02 §4](02-two-context-isolation-and-harness.md) — every non-pass is a
named, recorded outcome. Its observability is the marker stream plus, after
W09's integration, the dispatch-side category correlation; no new telemetry
mechanism is created here. No `unsafe` is expected in scenario code; any
exception is reported per the Coding Guidelines.

## 5. Handoff checklist

Before handing W07 to a reviewer, provide:

- the exact changed-file list (Guest asset extension, bootstrap, harness
  expectations) and the fixed test-bootstrap constants with their derivation
  note;
- evidence paths and run status for W07-DV01–DV09, including explicit
  not-run entries and every non-success class observed;
- the transcribed W03 rows (P5VG-050/054/057) and any matrix amendments
  made under the maintenance rule, with rationale;
- confirmation that markers, constants, and scenario order carry no ABI or
  machine-model claim and no Host-information disclosure;
- confirmation that isolation evidence is two-context and negative-controlled,
  or explicitly blocked with the `Architecture Change Request` recorded;
- the delivered W09 inputs (marker set, inventory, non-success classes) and
  W10 inputs (inventory, evidence links);
- open items: blocked prerequisites, Reserved extensions, and any
  `Architecture Change Request` / `ADR Required` record — without resolving
  them here.
