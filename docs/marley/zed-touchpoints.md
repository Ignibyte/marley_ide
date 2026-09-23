# Zed touchpoints

Every place the fork differs from upstream Zed outside Marley-owned paths. An upstream merge
starts here: each row says what Marley changed, why, and how to put it back if the merge
drops or conflicts with it.

**Upstream base:** `78648aaf7d` (zed-industries/zed `main`, the 2026-09-18 fork point; the
same commit as `UPSTREAM_BASE_FALLBACK` in `.claude/hooks/lib-hook-helpers.sh`). Move both
at every upstream merge.

**Marley-owned paths**, never listed below: `crates/marley_*`, `docs/marley/`,
`docs/planning/`, `docs/marley_architecture/`, `docs/specs/`, `docs/warp_architecture/`,
`docs/zed_architecture/`, `docs/decisions/`, `docs/tickets/`, `.claude/`, `script/gates.sh`,
`script/mutation.sh`, `CONSTITUTION.md`, `CHANGELOG.md`, `deny.toml`, `.gitleaks.toml`,
`.semgrep.yml`, `.cargo/audit.toml`, `.mcp.json.example`.

## Rules

1. A change to any other path gets a row here in the same change. No row, no change.
2. The change is the smallest diff that works: a new line, a new module, a new enum variant,
   a new trait method with a default body. Never a reformat, rename or move of upstream code.
3. A code touchpoint carries a `// Marley: <why>` comment on its hunk, so
   `rg "Marley:" crates assets` finds every one after a merge conflict.
4. When a touchpoint can go (upstream grew the seam, or the logic moved into a Marley crate),
   the row and the hunk leave together: revert the hunk first, while its row still allows
   the write, then remove the row.

**Enforcement** (live since W0, TICKET-436). gate:16 in `script/gates.sh` fails on a changed
path outside the owned set that has no row, on a row whose path no longer differs, on
duplicate rows and rows for owned paths, and on any upstream file the owned set would claim.
`.claude/hooks/enforce-commit-gate.sh` runs the same check at every `git commit`, Rust or
not, and `.claude/hooks/enforce-zed-ledger.sh` blocks a Write or Edit to a Zed path until
its row exists. A Bash edit gets past the write hook but not past the commit. The owned
list above documents `marley_owned_path` in `.claude/hooks/lib-hook-helpers.sh`, which is
the authority, so change the two together. A row counts when a backticked path opens its
first column.

## Touchpoints

| Path | What changed | Why | On merge |
|---|---|---|---|
| `Cargo.toml` | Seven `crates/marley_*` workspace members; `marley_fleet`, `marley_rail` and `marley_workbench` in `[workspace.dependencies]` | The Marley crates build in Zed's workspace | Re-add the members in sorted order. Keep a `[workspace.dependencies]` entry only for a crate something depends on, or cargo-shear fails |
| `Cargo.lock` | Entries for the Marley crates and their dependencies, and `zed`'s dependency on `marley_workbench` | Generated | Regenerate; never hand-merge |
| `.rules` | A "Marley" section above Zed's rules | Every agent session reads it first | Keep the section on top; take upstream's rules below it verbatim |
| `.config/typos.toml` | `extend-exclude` entries for Marley's reference docs | They transcribe Warp and gpui-era text verbatim | Re-add the block |
| `clippy.toml` | `allow-unwrap-in-tests` and `allow-expect-in-tests` | The Marley crates deny `unwrap_used` and `expect_used` outside tests (rustal's lint table, #447); Zed's crates enable neither lint, so the keys change nothing for them | Keep the two keys; take upstream's other settings verbatim |
| `.gitignore` | `.mcp.json`, `/mutants.out`, `/mutants.out.old`, `/mutants.diff` | Local MCP config carries bearers; the end-of-sprint mutation run (`script/mutation.sh`) writes the rest | Re-add the block |
| `README.md` | The two `> [!IMPORTANT]` review lines at the top | Zed's `.rules` self-review rule | Temporary. Chad removes them; an agent never does |
| `crates/paths/src/paths.rs` | `APP_NAME` is `"Marley"`; unit tests check that the config, data and state directories end in `marley` and that `crates/zed/RELEASE_CHANNEL` stays `dev` | The fork keeps its settings, database and logs apart from a stock Zed install; `APP_NAME` is Zed's documented switch for forks. On any channel but `dev` the fork would share stock Zed's keyring items, updater and app id, so the channel stays `dev` until TICKET-445 gives Marley its own | Keep `"Marley"` and the `dev` channel. If upstream moves or renames `APP_NAME`, carry the value and the tests there |
| `crates/zed/Cargo.toml` | `default-run` and the main `[[bin]]` are `marley`; the `marley_workbench` dependency | `main.rs` asserts at compile time that the binary's name matches `APP_NAME`; the app crate installs the Marley layout | Re-apply both names over upstream's and keep the dependency; leave its other bins alone |
| `crates/settings_content/src/marley.rs` | New file: `MarleySettingsContent { layout }` and `MarleyLayout { Zed, Marley }` | The `marley` settings block; the settings derive macros only resolve inside this crate | Keep the file; if upstream reorganizes the content modules, move it with them |
| `crates/settings_content/src/settings_content.rs` | The `marley` module and `pub use`, the `marley` field on `SettingsContent`, and `marley` in `flattened_deserialize!` | The key has to exist for the settings schema and the merge | Re-add the three pieces |
| `crates/settings/src/vscode_import.rs` | `marley: None` in the exhaustive `SettingsContent` literal | The literal names every field | Re-add the line |
| `crates/zed/src/zed.rs` | `marley_workbench::init(cx)` as the first line of `initialize_workspace`; the deferred callback that built Zed's sidebar calls `marley_workbench::register_sidebar`, and the `sidebar::Sidebar` import it no longer needs is gone; `"marley"` in `test_action_namespaces` | `initialize_workspace` runs once at startup before any window opens, and zed's own tests run it too, so the layout switch is installed before any window builds its sidebar; the layout setting decides which sidebar a window gets; the namespace test lists every action namespace, `marley` included | Diff upstream's version of that callback against the Zed arm of `register_sidebar` in `crates/marley_workbench/src/marley_workbench.rs` and carry every upstream change there, then point the callback at `register_sidebar` again; keep the `init` call first in `initialize_workspace` and the namespace in the list |

## At an upstream merge

1. Merge, then run `rg "Marley:" crates assets` and walk every row above against the result.
2. Move the upstream base here and in `.claude/hooks/lib-hook-helpers.sh`.
3. Regenerate `.cargo/audit.toml` and re-check `deny.toml`; both are tuned to upstream's
   lockfile.
4. Re-check the traps [workbench-shell.md](workbench-shell.md) depends on: the `sidebar`
   crate's actions are still bound in the default keymaps (D1), `reload_keymaps` still clears
   bindings added at init (D7), and `test_action_namespaces` in `crates/zed/src/zed.rs` still
   lists every action namespace. Also `rg "register_sidebar\(" crates` (a new upstream caller
   bypasses the layout switch) and read the `workspace::Sidebar` trait for new default methods,
   which the rail would take on silently.
