# The shell tests install Marley's scripts in a scratch directory — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-475-shell-tests-scratch-data-dir.md
- **Pipeline spec:** 475-shell-tests-scratch-data-dir.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-01)
- **Request:** wave 5 of `design-notes/remaining-work-2026-09-30.md` (Chad, 2026-09-30); the
  ticket was Deliberate until the tests run again, which #634 does.
- **Recall (§18.3):** #474's Phase 3 found the tests writing the user's scripts; AD-claude-483
  kept the tests in the tree, building, run by no gate; L-claude-465-zeds-clippy-bans-std-process-in-tests-too-001.

## Phase 1 — Plan (promoted 2026-10-01)
- **Classification / tier:** chore, test hygiene; Zed crates `terminal` and `terminal_view`
  (additive, test-only), Marley crate `marley_workbench` (test-only).
- **Recall (§18.3):** no knowledge entry on test data directories; #474's Phase 3 found the
  writes. The brain had nothing on the question (its consultation listed only follow-ups due).
- **Discovery (Explore):** the install runs only for a local interactive terminal whose shell is
  `System` or `Program` naming bash, zsh or fish (`terminal.rs` around 1328, `for_program`). It is
  reached by three unit-test binaries: `terminal` (`marley_bash_reports_each_typed_command_as_a_block`,
  `marley_bash_is_titled_without_the_integrations_arguments`, the two zsh PTY tests, the two
  direct `marley_shell_integration_*` tests, Zed's `test_terminal_closes_after_nonzero_exit` on
  the system shell); `terminal_view` (six title tests through `create_terminal_shell`, sixteen
  panel tests through `add_terminal_shell` and friends, all on the default `"shell": "system"`);
  `marley_workbench` (fourteen `routing_tests.rs` tests through `open_center_terminal` and
  `seed_first_terminal`). Every one is a `#[cfg(test)]` module of its crate's lib; no `tests/`
  folder is involved. No test calls `set_custom_data_dir` yet; no crate of the three has `ctor`.
  `marley_workbench` reads and writes `paths::data_dir()` in more places (`mcp.rs`, `guide.rs`,
  `browser.rs`, `harness.rs`, `system_one.rs`), so its tests are moved whole.
- **Before:** the user's `shell_integration` held `marley.bash`, `ssh-remote-command` (both
  2026-09-30 07:41:59) and `zsh/.zshenv` (07:41:59), and no `fish/` yet: this tree's #466 has
  not run in the installed Marley, so a test run before the fix would add `fish/` there.

### Design
- **Approach:** `terminal::marley_use_test_data_dir()` under
  `#[cfg(any(test, feature = "test-support"))]` sets `paths::set_custom_data_dir` on
  `<the test binary's folder>/../marley-test-data` (`std::env::current_exe`; the build profile's
  folder). Each binary calls it from a `#[cfg(test)] #[ctor::ctor(unsafe)]` function: `terminal`
  in its `mod tests`, `terminal_view` in `terminal_view.rs`'s `mod tests`, `marley_workbench` in
  `marley_workbench_tests.rs`. A failure to find the binary's folder panics with its reason in
  the ctor, which aborts the binary before any test: loud, never a silent fallback to the user's
  folder.
- **File manifest:**
  - `crates/terminal/src/terminal.rs` (Zed): the helper; the ctor in `mod tests`.
  - `crates/terminal/Cargo.toml` (Zed): `ctor` among the dev-dependencies.
  - `crates/terminal_view/src/terminal_view.rs` (Zed): the ctor in `mod tests`.
  - `crates/terminal_view/Cargo.toml` (Zed): `ctor` among the dev-dependencies.
  - `crates/marley_workbench/src/marley_workbench_tests.rs` (Marley): the ctor.
  - `crates/marley_workbench/Cargo.toml` (Marley): `ctor` among the dev-dependencies.
  - `docs/marley/zed-touchpoints.md`: the four Zed rows extended before the edits.

### Visual check plan
| REQ | What runs | What shows it |
|---|---|---|
| REQ-001 | `cargo test` of the three binaries' tests that start a shell (by name filter), with `XDG_DATA_HOME` on an empty scratch folder | `find` on that folder after the run: empty; the user's `shell_integration` listing with times, unchanged |
| REQ-002 | the same run | `marley-test-data/shell_integration` holds `marley.bash`, `zsh/.zshenv`, `fish/…`, `ssh-remote-command`; the bash and zsh PTY tests' result lines |
| REQ-003 | `git diff` | no `fn` added under a test attribute, no test body changed |
| §7 | `just shot` | Marley starts and draws, unchanged |

A test that fails for a reason other than the data directory goes to #634's triage, named here.

### Risks
- Processes of one nextest run share `marley-test-data`; the install writes a changed file in
  place, so on the first run after a script changes, one process can truncate a script while
  another's shell reads it. The tests shared the user's folder the same way; if #634 sees it, the
  fix is an atomic write in `install_in` (a temporary file and a rename).
- State a test leaves in the folder outlives the run, as it did in the user's folder.
- If `ctor`'s expansion trips `marley_workbench`'s `unsafe_code = "deny"` after all, the fallback is
  a `std::sync::Once` in the crate's shared test setup, without suppressing the lint.
