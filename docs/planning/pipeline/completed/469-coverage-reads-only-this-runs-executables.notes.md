# gate:4 reads only this run's executables — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-469-coverage-reads-stale-executables.md
- **Pipeline spec:** 469-coverage-reads-only-this-runs-executables.spec.md

## Phase 1 — Plan (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] promote · [x]
  prior-art sweep · [x] spec · [x] design · [x] present (autonomous; the queue's top row).
- **Classification / tier:** chore, small; a gate-is-test change (§7): verified by the gate's
  exit codes and a negative smoke.
- **Pre-flight:** #465 committed (`748dcddd28`); no other active pipeline; cargo idle.
- **Recall (§18.3).**
  - `F-claude-465-gate4-counted-lines-from-a-stale-executable-001`: the failure and the one
    executable that caused it.
  - `L-claude-438-the-coverage-floor-counts-lines-per-function-001`: gate:4 counts a line missed
    when some function's regions on it never ran, which is how a stale copy's zero counts
    become missed lines.
  - Brain: consultation `f8fcab7345b842548a8c85ec123269bb`, nothing on this seam.
- **Discovery.** `rust_cov` (`script/gates.sh:393`); cargo-llvm-cov 0.9.0's `object_files` and
  `pkg_hash_re` (`src/report.rs:240`, `:578`). The coverage target is
  `/mnt/fast/target/llvm-cov-target`, from `cargo metadata`'s `target_directory`. Its
  `debug/deps` held today: test executables for seven Marley packages from runs at 04:44,
  08:51, 12:37 and 13:02, beside proc-macro `.so` files, `.rlib`, `.rmeta` and `.d` files.

### Design
- In `rust_cov`, after the package list and before the run:
  - the coverage target from `cargo metadata --no-deps` (`target_directory` +
    `/llvm-cov-target/debug/deps`), as `touched_packages` reads metadata;
  - when it exists, `find <deps> -maxdepth 1 -type f -perm -u=x ! -name '*.*' -delete`: the
    test executables (`<name>-<hash>`), and nothing with an extension.
- CONSTITUTION: gate:4's row notes the removal.
- **File manifest.** `script/gates.sh`, `CONSTITUTION.md`. No Rust.

### Test plan
| REQ | Test |
|---|---|
| 001 | negative smoke: build `marley_workbench`'s instrumented tests with `shell_integration.rs` shifted by a comment block, restore the file, then run `marley_terminal`'s coverage as `rust_cov` does, without the removal (missed lines expected) and with it (none) |
| 002 | the smoke's second run starts from a directory with no test executable |
| 003 | `script/gates.sh --diff` |

### Risks
- **cargo rebuilds a deleted executable** only if its fingerprint notices the missing output;
  cargo-llvm-cov's own clean of the packages it runs makes that moot for this run, and the
  smoke shows the rebuild.

## Phase 2 — Code (2026-09-23)
- **Built.** `rust_cov` reads the target directory from `cargo metadata --no-deps`, and when
  `<it>/llvm-cov-target/debug/deps` exists removes the regular, user-executable files with no
  extension there before `cargo llvm-cov nextest`. CONSTITUTION's gate:4 row says so.
- **Deviations:** none.
- **Review.**
  - A failed `cargo metadata` leaves the path at `/llvm-cov-target/debug/deps`, which does not
    exist, so nothing is removed.
  - The pattern keeps every file with an extension (`.so` proc macros, `.rlib`, `.rmeta`, `.d`)
    and every directory; `-maxdepth 1` keeps the removal to `deps` itself.
  - gate:11's shellcheck command is clean on the script.

## Phase 3 — Test (2026-09-23)
- **Negative smoke (REQ-001).** `shell_integration.rs` shifted by 20 comment lines; the
  instrumented `marley_workbench` tests built against it (`cargo llvm-cov nextest -p
  marley_workbench --no-report -E 'none()'`); the file restored by checksum. Then
  `marley_terminal`'s coverage, as `rust_cov` runs it:
  - without the removal: exit 1, `shell_integration.rs` 63 of 226 lines missed (the stale copy's
    line map adds lines the file does not have), the #465 failure reproduced;
  - with the removal (9 test executables removed, the 72 `.so` files kept): exit 0,
    `shell_integration.rs` 163 lines at 100%, `TOTAL` 100%.
- **REQ-002.** The second run started with no test executable in `deps` and rebuilt the one
  it needed.
- **Gate.** See below.
- `script/gates.sh --diff`: `GATE GREEN [diff]`, 20 passed, the receipt matching the tree.
  With no Marley crate touched, gate:4 skips cleanly in this mode, so the smoke above is its
  test.

## Phase 4 — Complete (2026-09-23)
- **Docs (§21).** `CHANGELOG.md` (Fixed); CONSTITUTION's gate:4 row.
- **Ledger (§19).** `PR-claude-a-gate-step-starts-from-no-output-an-earlier-run-left-001`, for
  `F-claude-465-gate4-counted-lines-from-a-stale-executable-001`.
- **Brain.** Consultation `f8fcab7345b842548a8c85ec123269bb` closed by
  `decisions/gate4-removes-the-coverage-targets-test-executables-before-it-runs`, follow-up
  2026-10-07.
- **Ticket** closed; the pipeline archived; one commit.
