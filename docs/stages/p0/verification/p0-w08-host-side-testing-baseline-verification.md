# P0-W08 Host-Side Testing Baseline — Verification Evidence

**Status:** Complete evidence recorded; W08 closure claimed.
**Date:** 2026-09-18 (Asia/Shanghai)
**Environment:** Linux (WSL2, x86_64); executions ran in the isolated
`RUSTUP_HOME`/`CARGO_HOME` sandbox restored through the repository manifest
(W02 restoration path), pinned toolchain `1.98.1`, host target
`x86_64-unknown-linux-gnu` (machine-derived per the build-target policy).

**Proof boundary (restated per contract §7):** the results below prove
host-side entry semantics only. They do not prove EL2 behavior, MMIO or
interrupt behavior, cache/TLB effects, timing, or any real-hardware property,
and they are not QEMU, guest, or hardware evidence.

## Results

| ID | Review | Result | Evidence / reason |
|---|---|---|---|
| W08-DV01 → P0-V09 | Boundary review | **passed** | Contract §1 defines the host-verifiable classes (parsers, normalization, pure state machines, newtype arithmetic, codecs, handle logic, pure policy) and the bare-metal-bound classes, with the three stated rules (expressible-without-the-machine test; no emulator dependency; model-tests may not claim hardware verification). |
| W08-DV02 → P0-V03 | Entry existence/build | **passed** | `cargo test --workspace --exclude hypervisor` compiles every host-target member from a restored environment and reports a truthful summary; host build through the same scope succeeds; a bare whole-workspace invocation fails on the bare-metal member by construction, demonstrating the exclusion is the semantics, not a filter. |
| W08-DV03 → P0-V04 | Baseline execution | **passed** | The entry reports `test result: ok. 1 passed; 0 failed; 0 ignored` (unit) and `0 passed` (doc-tests); the placeholder test asserts only entry health. |
| W08-DV04 → P0-V04 | Failure visibility | **passed** | Deliberate local mutation (`assert!(false, …)`, never committed) made the entry report `test result: FAILED. 0 passed; 1 failed` and exit status 101; revert restored `ok. 1 passed`. Truthful reporting verified in both directions. |
| W08-DV05 → P0-V09 | Category matrix review | **passed** | Contract §5 embeds the five required categories with definitions, the conditional concurrency extension, the applicability matrix over host-verifiable unit classes, the mapping rule (per-unit statement, motivated exclusions), and the reserved extensions with owners. |
| W08-DV06 → P0-V09 | Discovery and link review | **passed** | `docs/README.md` routing row reaches the contract in one link; `docs/testing/README.md` pointer line added without restating policy; contract links (toolchain/build-target baselines) resolve; implementation-index row truthful. |
| W08-DV07 → W08 closure | Consumability review | **passed** | Read as W07 (sole binding target, stable spelling, truthful exit), W19 (test step + quotable proof boundary), W20 (CI-executable, no emulator), later module designs (matrix + mapping rule), and W09/P1 (explicit non-substitution statement); each consumer can act without inventing policy. |

## Commands and observed results

```text
# sandbox restored per W02 contract
cargo test --workspace --exclude hypervisor
  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
  test result: ok. 0 passed; 0 failed; ... (doc-tests)
cargo build --workspace --exclude hypervisor
  Finished `dev` profile ... target(s)
cargo test --workspace                       # no exclusion
  error[E0152]: found duplicate lang item `panic_impl`
  error: could not compile `hypervisor` (bin "hypervisor" test)
# failure-visibility dry run (local mutation, reverted before commit)
  test result: FAILED. 0 passed; 1 failed; 0 ignored
  cargo exit: 101
# after revert
  test result: ok. 1 passed; 0 failed; 0 ignored
```

Note: the dry run's revert used a manual file edit because the mutated file
was not yet git-tracked (`git checkout --` cannot restore untracked files);
the committed state contains only the original `assert!(true)` placeholder.

## Not run / not proved

- **CI execution:** not run; W20 owns CI wiring (P0-V03/V04's CI leg).
- **Product-logic coverage:** none exists; the placeholder asserts entry
  health only and is never product coverage.
- **QEMU/guest/hardware:** not run and not claimable; see the proof boundary
  above.
- **Non-Linux hosts:** not exercised; the entry's environment contract is
  repository-declared toolchain only.
