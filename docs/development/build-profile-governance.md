# Zelyr Build-Profile / Feature Governance

**Status:** Normative build-choice governance.  
**Scope:** Classification of every build-time switch (Cargo features, build
profiles, runtime resource/policy choices), the classification procedure, the
Reserved profile registry, review questions, prohibited cases, and change
thresholds. It does not define runtime configuration schemas, feature code
organization, quality-gate commands, or CI.  
**Version:** v0.1  
**Owner/change context:** P0-W04 build-choice governance; operationalizes
ADR-047, ADR-037, and ADR-046.  
**Supersedes:** The absence of switch-classification policy.

## 1. Semantic authority

The ADR baseline owns the semantics this document operationalizes:

- **[ADR-047](../adr/adr-000-architecture-baseline-v0.1.md)** — the
  minimal/research/embedded/general/secure/full profile set is supported; a
  feature expresses "the binary has this capability"; VM counts, RAM, and
  similar runtime policy must never be abused as Cargo features.
- **[ADR-037](../adr/adr-000-architecture-baseline-v0.1.md)** — configuration
  layering: build configuration controls binary capability only; boot and
  VM/runtime configuration are separate objects owned by later stages.
- **[ADR-046](../adr/adr-000-architecture-baseline-v0.1.md)** — Cargo
  workspace from day one, without over-fine-grained early crate splitting.

## 2. Switch classes

Exactly three switch classes exist. Nothing else is a build-time switch.

### 2.1 Class 1 — Binary capability (Cargo feature)

A compile-time switch that includes or excludes a capability of the produced
binary. Valid when:

- its presence or absence changes *what the binary can do*, not a quantity,
  binding, or selection that must vary at runtime;
- it is implementable as additive, named, documented feature(s) whose default
  state is declared;
- the capability has an owning module/design into which it is compiled.

(Informative shape only; no feature exists in P0: including an experimental
virtio backend, or compiling in an extended telemetry surface.)

### 2.2 Class 2 — Build profile

A named, coherent build selection for a deployment or research class: which
capabilities are compiled in, and the build's optimization/diagnostic posture.
Valid when:

- it selects among capabilities and build characteristics, but never defines
  new runtime policy;
- every profile builds the same architecture: differences are the feature
  sets selected, never divergent Core behavior (the non-fork rule, §4.2);
- it is one of the registered profiles (§4.1) or its introduction went through
  the §6 thresholds.

Cargo's built-in `dev` and `release` configurations are **build
configurations, not members of the ADR-047 profile set**; the two vocabularies
must not silently merge.

### 2.3 Class 3 — Runtime resource/policy

A quantity, binding, or selection that must remain adjustable per boot, per
VM, or per deployment: counts, sizes, affinities, device and policy choices.
These are never compile-time switches. Their home is the future runtime
configuration model under ADR-037's layering. In P0 no such model exists,
which is exactly why no switch of this class may be created: there is nowhere
correct to put it.

### 2.4 Precedence rule

When a proposal seems to fit more than one class, the **runtime-policy reading
wins**. A switch that mixes a capability with a runtime quantity (for example
"feature: support up to N VMs") must be split into its capability part and its
runtime part, or rejected; it must never be recorded as a feature. An
unclassifiable switch is a design defect: the proposing design is sent back,
not waved through.

## 3. Classification procedure

Apply the questions in order; each yields a verdict or the next question:

```text
Q1. Does the switch set or bound a runtime quantity, binding, or selection
    (count, size, affinity, device, per-VM/per-boot choice)?
      yes -> class 3 (runtime resource/policy): it must NOT become a feature
             or profile constant; route to the runtime configuration model.
      no  -> Q2.
Q2. Does the switch change what the compiled binary is capable of, in a way
    that is legitimate to fix at build time and has an owning module?
      yes -> class 1 (binary capability): record as a documented feature with
             its default state and owning design.
      no  -> Q3.
Q3. Does the switch select a coherent named build selection over capabilities
    and build posture for a deployment/research class?
      yes -> class 2 (profile): must map onto the registered set or follow
             the §6 thresholds for a new or activated profile.
      no  -> not a switch: it is a design decision to record in the owning
             design, not a build-time control.
Any Q1-yes answer combined with a feature proposal: prohibited combination —
split or reject (precedence rule).
```

Every accepted switch must record its class, owning design, and default state
where the switch is declared, once declarations exist (none exist in P0).

## 4. Profile registry

### 4.1 Reserved profiles

All six ADR-047 names are **Reserved**: purpose recorded, mechanism
unimplemented, activation only through the §6 thresholds. The purpose
statements are P0-level elaborations of ADR-047, refinable by the design that
first implements the profile; they do not amend the ADR.

| Profile | P0-level purpose (refinable) | Applicability boundary |
|---|---|---|
| minimal | smallest capability set that still boots and is diagnosable | mechanism bring-up and footprint-sensitive builds |
| research | experimental mechanisms and extended observability enabled | research experiments; not a release posture |
| embedded | reduced-footprint selection for constrained targets | constrained deployment classes |
| general | broad default deployment capability set | default release-facing builds |
| secure | hardened selection minimizing compiled-in surface | security-focused deployments |
| full | everything enabled for testing and benchmarks | CI/benchmarks; never a default |

None of the six is implemented, selected, or named in any build path in P0.
The registry's function is to keep the set available and prevent the names
from drifting into unrelated meanings. Security-posture selections follow this
registered-profile path, never local flag bundles.

### 4.2 Non-fork rule

Profiles select; they do not fork. All profiles build the same architecture
from the same sources. Hypervisor Core code must never branch on profile
identity; capability differences enter only through features that a profile
selects. A proposed profile that requires Core-conditional behavior is
rejected at review or escalated per §6.

## 5. Review questions for new switches

For any design or code change that introduces or modifies a switch:

1. Which class is it (per §3)? Is the classification recorded at the switch?
2. If a feature: is the default state declared, documented, and additive?
3. If a profile: is it in the registry, and does it select without forking?
4. Does any Q1 answer hide inside a feature or profile name (renamed runtime
   policy is the most common violation)?
5. Does the switch's absence leave the binary correct and buildable (features
   default off, or their default is a recorded decision)?
6. Does the switch need a dependency or code-organization decision? If so,
   route to the dependency-governance document (P0-W18) and the owning design
   — do not settle it here.

## 6. Change thresholds

- **Routine maintenance** (ordinary PR review): adding a switch classified per
  this document; refining a Reserved profile's purpose statement when the
  implementing design arrives; documentation corrections.
- **Policy decision required** (recorded issue and owner decision before the
  change): introducing custom Cargo `[profile.*]` sections; activating a
  Reserved profile; adding a profile outside the registered set;
  reclassifying an existing switch; changing a feature's default state.
- **ADR required:** any change that would make runtime policy compile-time,
  allow profile-specific architecture forks, or otherwise alter ADR-047 or
  ADR-037 semantics. Lifecycle and supersession rules are in the
  [ADR index](../adr/README.md).

## 7. Prohibited cases

None of the following may become a feature, a profile-intrinsic constant, or
any other compile-time switch:

- VM count or limits;
- memory sizing (RAM amounts, region sizes);
- vCPU count or topology per VM;
- CPU affinity or pinning choices;
- device selection for a specific VM or deployment.

(Informative hypothetical violations, none of which exist: a `max-vms`
feature, a `ram-mb` feature, or a profile that hard-codes a device set for a
named VM.) The prohibition's authority is ADR-047's final clause and
ADR-037's layering.

## 8. Boundary pointers

Target classification and triples are owned by the [build-target
baseline](build-target-baseline.md); toolchain policy by the [toolchain
baseline](toolchain-baseline.md); the first dependency decision by P0-W18's
governance; build-identity metadata dimensions by P0-W16; diagnostic
visibility semantics by P0-W12. This document does not restate their rules.
