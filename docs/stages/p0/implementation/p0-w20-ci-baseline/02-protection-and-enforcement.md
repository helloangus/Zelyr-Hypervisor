# P0-W20 Protection and Enforcement Design

**Status:** Proposed detailed design; implementation not claimed.  
**Parent:** [P0-W20 detailed design](README.md).

## 1. Authority relationship to the integration policy

The [branch and pull-request integration
workflow](../../../../development/integration-workflow.md) is adopted
normative policy. W20 implements its deferred technical enforcement; it does
not restate, rewrite, summarize into divergence, or amend the policy. The
contract document's protection section **references** the policy's rules and
adds only what the policy does not carry: the concrete GitHub settings that
make the rules hold, the merge-method enablement, and the enforcement-evidence
procedure. If the policy and the configured behavior ever appear to disagree,
the policy governs and the configuration is corrected in the same change.

## 2. Protection requirements (contract content)

The contract document's protection section must fix the following as
normative requirements for the `main` branch settings:

1. **PR-only integration:** require a pull request before merging; `main` is
   the sole integration branch. Direct commits, force pushes, and deletion of
   `main` are rejected.
2. **Required status checks:** exactly the six check names of the
   [check-mapping register](01-ci-contract.md) §3, in strict mode — a check
   that has not reported success does not pass, so a missing, cancelled,
   skipped, failed, or timed-out required check blocks merge (the policy's
   rule, made mechanical).
3. **Bypass honesty:** no bypass allowance is granted to any role, app, or
   push rule beyond what the platform itself reserves. Administrators are
   included in the restrictions where the repository settings permit. Where
   a repository plan or platform limitation leaves an administrator bypass
   capability in place, that residual capability is recorded in the
   verification evidence as **policy-bound, not technically bound** — the
   policy's statement that an administrator's ability to bypass is not
   permission to bypass remains the control. The evidence must never describe
   bypass as impossible.
4. **Merge methods:** at least one enabled merge method must satisfy the
   policy's rule that the merge method preserve a reviewable connection
   between the merged change and its PR. Methods that can sever the PR
   association are disabled. The exact enablement set is recorded in the
   implementation record at implementation time; changing it is a recorded
   minor change because the acceptance rule lives in the policy, not in the
   setting.
5. **Up-to-date requirement:** the "require branches to be up to date before
   merging" option is **not** imposed at P0. Rationale: it serializes merges
   into a queue disproportionate to the project size, and base-branch drift
   is detected by the `main`-push runs of the same check set required by
   P0-V08. Escalating to "require up to date" is a design-level change under
   the [CI contract](01-ci-contract.md) §9, taken if drift produces incidents.

## 3. Settings-to-contract mirror rule

GitHub protection settings are not reviewable through pull requests; they are
changed out-of-band by a repository administrator. The reviewable authority is
therefore the contract document, and the live settings must mirror it:

- every settings change is made in the same change as the contract edit that
  justifies it, and both are recorded together in the implementation record;
- the verification record contains a settings observation (per-setting values
  as visible on GitHub) compared field-by-field against the contract
  requirements;
- drift between settings and contract discovered at any later time is a
  defect to be fixed in the change that repairs it, not a note for later.

This mirrors, at the repository-configuration level, the single-source rule
the W02 contract applies to the toolchain manifest.

## 4. Failure boundaries and blocked states

Stop and record instead of improvising when any of the following occurs:

- **No configured remote.** The W01 implementation record documents remote
  publication as pending owner inputs. Without the remote, protection cannot
  be configured and the enforcement exercise cannot run; both are recorded as
  blocked with the missing owner input named. The workflow and contract
  document may still be authored and dry-run, but W20 is not closable and no
  evidence may be simulated.
- **Administrator access unavailable.** Configuration and settings observation
  require repository administration. If the implementer lacks it, the
  configuration step is blocked and recorded; read-only settings observation
  is recorded for what it shows.
- **Plan or platform limitation.** If the repository plan does not support a
  restriction (for example administrator inclusion), configure the strictest
  available set, record the residual capability per §2 item 3, and continue;
  this is an honest evidence statement, not a failure of the exercise.
- **Missing prerequisite contract.** If the W07 register document, the W02
  manifest, or the W03/W08 entries have not delivered, the affected check
  cannot be truthfully configured. It is recorded as blocked; the required set
  may not be partially configured while presenting the package as complete —
  P0-V08 closes only when the full required set is configured, enforced, and
  evidenced.
- **Differently-delivered prerequisite.** If a delivered contract differs from
  its proposed design (different gate set, labels, or entry spelling), W20
  follows the delivered contract and reconciles the mapping in the same
  change; a semantic conflict is raised through W07's thresholds, never
  absorbed locally.

## 5. Enforcement-evidence procedure (the P0-V08 exercise)

The plan requires evidence of protection and check status through a real
GitHub pull request; a configuration review or local syntax check does not
satisfy P0-V08. The exercise consists of four observations, each recorded in
the verification record with the PR or push reference, the commit, timestamps,
and the observed output:

1. **Green path.** Open a real PR containing a bounded, coherent change (for
   example the workflow and contract document themselves). Observe: all six
   required checks are present, execute on GitHub, and report success; the
   merge control is available to a maintainer only after all six have passed.
2. **Red probe.** Add a probe commit to the same PR branch that deliberately
   fails exactly one gate — a formatting violation is the simplest honest
   probe for `QG-FMT`. Observe: the named check reports failure and GitHub
   refuses the merge while the required check is failing. This is the
   observation that proves the checks are *required*, not merely present.
3. **Restore.** Revert the probe commit. Observe: all six checks return to
   success and merge becomes available again. Both probe results are recorded
   together (W07 principle 3 discipline).
4. **Direct-push rejection.** Attempt a direct push to `main` from a
   non-administrator context and observe the rejection. If only an
   administrator context is available, record that honestly: the observation
   then demonstrates rejection by policy discipline rather than by technical
   enforcement, and P0-V08 evidence must say exactly that.

Probe discipline: the probe lives only on the exercise PR branch, is never
merged, is fully reverted, and is an evidence tool — it never edits a gate or
a workflow to make a failing state pass.

Each observation's record states what it proves and what it does not (the
validation matrix in [the implementation workflow](03-implementation-and-validation.md)
§3 carries the proof boundaries).

## 6. Explicitly excluded procedures

No protection-bypass path, emergency direct-push convention, additional
protected branch, deployment environment, secret, webhook, or external status
service is designed or authorized. Extending protection to branches beyond
`main`, or integrating external reporting, is a design-level change under the
[CI contract](01-ci-contract.md) §9. If enforcing any rule of the policy
appears to require amending the policy text, that is the stop boundary: raise
the conflict to the policy's owning change, do not adapt the policy silently.
