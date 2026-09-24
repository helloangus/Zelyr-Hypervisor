# P1-W03 capability inventory verification

**Status:** Local mechanism evidence; runtime P1-V05/P1-V06 proofs pending W09–W11.
**Scope:** W03 validation; no stage-completion claim.
**Owner/change context:** W03, 2026-09-24.
**Record:** [implementation](../implementation/p1-w03-aarch64-capability-inventory-record.md).

Environment: Linux x86_64; pinned Rust 1.98.1; AArch64 target
`aarch64-unknown-none-softfloat`; branch `p1/w03-capability-inventory`, base
`a05fca6`. All commands below use the repository root.

| Check | Result / evidence |
|---|---|
| W03-DV01 fact set | Passed source/contract review: 13 records; all bit ranges and classification rationales in record; corrected TGran16/VHE semantics |
| W03-DV02 taxonomy | Passed: independent classification/observation and deterministic ALL ordering |
| W03-DV03 negative policy | Passed five product unit tests: normal, all granule field encodings, missing required/optional-unreadable ordering, reserved and frequency boundaries, complete bounded rendering |
| W03-DV04 boundary | Passed author and root independent systems review, 2026-09-24: six isolated MRS blocks, W01 preconditions, exclusive cell writer and release/acquire immutable publication. U-003/U-004 accepted |
| W03-DV05 consumers | Passed design review: W04 query and W08 transitive dependency; no current executing consumers. Their final implementation audit remains owned by those packages |
| W03-DV06 executed report/NC2 | Not run: W09 wiring and W10/W11 executed scenarios own this proof. Host tests do not substitute |
| W03-DV07 handoff | Passed source/contract review: build/query/render API, fixed vocabulary, retained report and no P2 discovery API |

## Local commands

`cargo test --workspace --exclude hypervisor`: passed, 5 product tests plus
1 separately identified entry-health test; zero failures/ignored tests.
`cargo clippy --target aarch64-unknown-none-softfloat -p hypervisor -- -D warnings`:
passed. The complete local gate set passed on 2026-09-24:

- `cargo fmt --all -- --check` (QG-FMT).
- `cargo clippy --workspace --exclude hypervisor --all-targets -- -D warnings`
  and the target Clippy command above (QG-LINT).
- `cargo build --workspace --exclude hypervisor`, host tests above, and
  `cargo build --target aarch64-unknown-none-softfloat -p hypervisor`, all
  zero-warning (QG-WARN, QG-TEST-HOST, QG-BUILD-TARGET).
- QG-DOCS Python body extracted verbatim from `.github/workflows/ci.yml`
  with `sed` and executed by `python3`: links, reachability, headers OK.

Online required checks remain the PR integration gate; local results do not
substitute for them. An initial target lint exposed an unused Observation
re-export before W04 exists; the corrected, narrowly scoped seam allowance
is tracked for removal by W09. Final local runs above passed.

Host evidence proves pure decoder/continuation/format semantics only. It does
not prove EL2 register access, firmware behavior, publication execution,
console transport, MMU, interrupts, SMP or real hardware. QEMU report output
and missing-required real CPU scenarios have not run in this package because
the W02 unlinked-W09 boundary remains. No W03-complete claim is made around
that integration boundary.
