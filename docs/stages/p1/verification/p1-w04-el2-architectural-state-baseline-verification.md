# P1-W04 verification record

**Status:** Local implementation validation; runtime evidence deferred.
**Scope:** W04 mechanism, not completed P1-V07.
**Version:** v0.1
**Owner/change context:** P1-W04, 2026-09-25.
**Supersedes:** None.

Implementation: [record](../implementation/p1-w04-el2-architectural-state-baseline-record.md).
Environment: pinned Rust 1.98.1, Linux host, AArch64 softfloat target, branch
`p1/w04-el2-baseline` based on merged W03 commit `4c1f554`.

## Evidence

| Check | Outcome / meaning |
|---|---|
| W04-DV01 facts | Passed inspection: only W03 query seam, optional VHE not required; exact mapping in record |
| W04-DV02 write specs | Passed source/Arm field review and five source-shared host tests |
| W04-DV03 readback/guards | Passed host mask/mismatch tests plus terminal-route/optional-skip source review |
| W04-DV04 boundary | Passed first-review inspection: two unsafe primitives, no unsafe declaration cell; second soundness review recorded by PR reviewer |
| W04-DV05 order | Passed source review: C1→C8, one HCR write, ISB after every write, no retry |
| W04-DV06 100 boots | Not run: W09 integration and W10 regression own execution |
| W04-DV07 consumers | Passed interface review: category/value/control-status queries, no hardware reread by consumers |

The five new host tests cover all owned-bit mismatch cases, zero/all-ones/
alternating residue, preserving unowned fields, full-constant semantics,
timer read-only ISTATUS, optional-timer designation, monotone category order,
unique controls, W02 ownership exclusion and VTCR RES1. No untrusted parser,
allocator/resource failure, or concurrent runtime exists in this package.
Repeated phase invocation is rejected before any register write (source review).

Host results prove value/mask semantics only. They do not prove EL2 accesses,
instruction/barrier effects, optional timer accessibility, QEMU boot or hardware.

## Local commands

All following commands/checks passed locally on 2026-09-25 with no warnings
or findings; host tests reported 11 passed, zero failed/ignored:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --exclude hypervisor --all-targets -- -D warnings`
- `cargo clippy --target aarch64-unknown-none-softfloat -p hypervisor -- -D warnings`
- `cargo test --workspace --exclude hypervisor`
- `cargo build --workspace --exclude hypervisor`
- `cargo build --target aarch64-unknown-none-softfloat -p hypervisor`
- Repository QG-DOCS body from `.github/workflows/ci.yml`
- `git diff --check`

Initial target Clippy rejected an interior-mutable named const used for atomic
initialization. It was replaced with a const constructor, without suppression;
the corrected target Clippy and host tests passed (11 total: five W03,
five W04, one infrastructure).

No QEMU execution is claimed: W09 has not invoked this mechanism yet. No
manual test-only boot wiring is committed. Disassembly/runtime validation and
cold-boot consistency remain W09/W10 evidence, including live verification of
the optional timer on a supporting CPU model. No Guest/hardware test was run.
