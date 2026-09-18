# P0-W04 Governance Contract

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W04 detailed design](README.md).

## 1. Logical artifact groups and ownership

W04 is policy-documentation work, so its logical modules are authoritative
artifact groups, not Rust modules.

| Artifact group | Authoritative owner | Inputs | Output / non-responsibility |
|---|---|---|---|
| Build-choice governance document | `docs/development/build-profile-governance.md` | ADR-047/ADR-037/ADR-046 semantics, task-book Reserved list, this design | the sole normative home of switch classes, the classification procedure, the profile registry, review questions, prohibited cases, and change thresholds; it does not define runtime configuration schemas, feature code organization, gate commands, or CI |
| Documentation routing | one row in `docs/README.md` routing table | governance document location | discoverability of the governance document; it does not restate policy |
| Stage implementation index | `docs/stages/p0/implementation/README.md` | design status | a truthful status row; it never claims completion |
| Implementation record | `docs/stages/p0/implementation/p0-w04-build-profile-feature-governance-record.md` (created when work starts) | actual decisions taken | changed artifacts, drill outcome summary, deviations; no command logs |
| Verification record | `docs/stages/p0/verification/p0-w04-build-profile-feature-governance-verification.md` (created when evidence exists) | actual drill and review evidence | run/not-run evidence per the validation matrix; not part of the design |

The artifact named in the second column is the sole authoritative home for the
statement in its row. Other documents may link to it but must not duplicate or
contradict it.

## 2. Prerequisite state

W01 is completed (root navigation and conventions exist and are the entry
surface this document is routed from). The ADR baseline exists and is the
semantic authority. No other prerequisite is required: W04 introduces no
manifest, no tool, and no code, so it neither consumes nor waits on W02/W03.
If a Cargo manifest with features or profiles were to appear in the tracked
tree before W04 implements (currently none exists; W03's baseline has none),
the governance document still lands unchanged and applies to that manifest at
review; W04 must not modify the manifest.

## 3. Switch classes (required document content)

The governance document must define exactly three switch classes. Nothing
else is a build-time switch.

### 3.1 Binary capability — Cargo feature

A compile-time switch that includes or excludes a capability of the produced
binary. Valid when:

- its presence or absence changes *what the binary can do*, not a quantity,
  binding, or selection that must vary at runtime;
- it is implementable as additive, named, documented feature(s) whose default
  state is declared;
- the capability has an owning module/design into which it is compiled.

ADR-047 semantics: a feature expresses "the binary has this capability".
Examples of the *shape* (informative; none exist in P0): including an
experimental virtio backend; compiling in an extended telemetry surface.

### 3.2 Build profile

A named, coherent build selection for a deployment or research class:
which capabilities are compiled in, and the build's optimization/diagnostic
posture. Valid when:

- it selects among capabilities and build characteristics, but never defines
  new runtime policy;
- every profile builds the same architecture: differences are the feature sets
  selected, never divergent Core behavior (the non-fork rule, §5.2);
- it is one of the registered profiles (§5) or its introduction went through
  the §8 thresholds.

Cargo's built-in `dev` and `release` configurations are build configurations,
not members of the ADR-047 profile set; the governance document must state
this so the vocabulary cannot silently merge.

### 3.3 Runtime resource/policy

A quantity, binding, or selection that must remain adjustable per boot, per
VM, or per deployment: counts, sizes, affinities, device and policy choices.
These are never compile-time switches. Their home is the future runtime
configuration model under ADR-037's layering (build configuration controls
binary capability; boot and VM/runtime configuration are separate objects).
In P0 no such model exists, which is *why* no switch of this class may be
created: there is nowhere correct to put it.

### 3.4 Precedence rule

When a proposal seems to fit more than one class, the runtime-policy reading
wins. A switch that mixes a capability with a runtime quantity (for example
"feature: support up to N VMs") must be split into its capability part and its
runtime part or rejected; it must never be recorded as a feature. An
unclassifiable switch is a design defect: the proposing design is sent back,
not waved through.

## 4. Classification procedure (required document content)

The governance document must contain this ordered procedure; each question
yields a verdict or the next question:

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
             the §8 thresholds for a new or activated profile.
      no  -> not a switch: it is a design decision to record in the owning
             design, not a build-time control.
Any Q1-yes answer combined with a feature proposal: prohibited combination —
split or reject (precedence rule).
```

Every accepted switch must record its class, owning design, and default state
where the switch is declared, once declarations exist (none exist in P0).

## 5. Profile registry (required document content)

### 5.1 Reserved profiles

The governance document must record all six names from ADR-047 and the task
book, each with status **Reserved** and a one-line P0-level purpose and
applicability statement. The statements are elaborations of ADR-047, marked as
refinable by the design that first implements the profile:

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
from drifting into unrelated meanings.

### 5.2 Non-fork rule

The governance document must state: profiles select; they do not fork. All
profiles build the same architecture from the same sources. Hypervisor Core
code must never branch on profile identity; capability differences enter only
through features that a profile selects. A proposed profile that requires
Core-conditional behavior is rejected at review or escalated per §8.

## 6. Review questions for new switches (required document content)

The governance document must provide a review checklist for any design or code
change that introduces or modifies a switch:

1. Which class is it (per §4)? Is the classification recorded at the switch?
2. If a feature: is the default state declared, documented, and additive?
3. If a profile: is it in the registry, and does it select without forking?
4. Does any answer to Q1 hide inside a feature or profile name (renamed
   runtime policy is the most common violation)?
5. Does the switch's absence leave the binary correct and buildable (features
   default off, or their default is a recorded decision)?
6. Does the switch need a dependency or code organization decision? If so,
   route to W18's governance and the owning design — do not settle it here.

## 7. Prohibited cases (required document content)

The governance document must prohibit, explicitly, making any of the following
a feature, a profile-intrinsic constant, or any other compile-time switch:

- VM count or limits;
- memory sizing (RAM amounts, region sizes);
- vCPU count or topology per VM;
- CPU affinity or pinning choices;
- device selection for a specific VM or deployment.

The document may add informative hypothetical examples of violations (marked
informative so they are never read as existing switches), for example a
`max-vms` feature, a `ram-mb` feature, or a profile that hard-codes a device
set for a named VM. The prohibition's authority is ADR-047's final clause and
ADR-037's layering; the document cites them rather than re-deriving them.

## 8. Change thresholds (required document content)

- **Routine maintenance** (ordinary PR review): adding a switch classified per
  this document; refining a Reserved profile's purpose statement when the
  implementing design arrives; documentation corrections.
- **Policy decision required** (recorded issue and owner decision before the
  change): introducing custom Cargo `[profile.*]` sections; activating a
  Reserved profile; adding a profile outside the registered set;
  reclassifying an existing switch; changing a feature's default state.
- **ADR required:** any change that would make runtime policy compile-time,
  allow profile-specific architecture forks, or otherwise alter ADR-047 or
  ADR-037 semantics. The document must name the first two thresholds
  explicitly and give the ADR rule by reference to `docs/adr/README.md`.

## 9. Explicitly excluded code interfaces

There are no Rust types, functions, traits, modules, crates, Cargo features,
`[profile.*]` sections, manifest edits, build scripts, CI jobs, or public APIs
in this design, and no runtime configuration schema is fixed. The document
governs future switches; it does not create any. Adding any of these artifacts
under W04 is a scope conflict requiring the applicable detailed design (at
minimum W03 for manifests/targets, W18 for dependencies, W07 for gates) and
must be stopped at review.
