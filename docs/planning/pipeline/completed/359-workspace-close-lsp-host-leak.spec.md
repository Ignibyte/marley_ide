---
pipeline_id: 3428bb0b-7864-4193-9682-89bcb807d6f6
ticket: forge#359 (d7981725-3b80-437c-8dae-0d2183d4bbf5) · local docs/planning/tickets/open/TICKET-359-workspace-close-lsp-host-leak.md
aar_id: cea6a711-e834-41b4-a5ae-8a707f0ec365
status: Phase 5 — Complete PASS
title: Closing a workspace drops its LSP host — the last-one-out remove in close_project_at (rust-analyzer leak fix)
type: bug
milestone: M20
references: [app.rs:6381 close_project_at (the fix site) / :1412 active_root = active_project().root / :4550 lsp_hosts.insert / :187 lsp_hosts HashMap<PathBuf, LspHost> / :4558 lsp_host_exists_for_test, tabs.rs:168 Project.root / :352 add_project (no dedup) / :360 close_project / :412 Workspace::projects(), lsp_host.rs impl Drop for LspHost, headless_drive.rs:76 tick_pump / :8460 restored_editor_tab_spawns_lsp_headless / :8427 write_cargo_fixture / :8444 seed_stub_language_server, #321 state-derived creation, #320 fake-server lane]
---

## Title
Closing a workspace never dropped its LSP host, so a rust-analyzer (1–4 GB RSS) leaked for the process lifetime.
`close_project_at` cleans up every other per-project map but not `lsp_hosts`. Fix: remove the closed project's host
from the map (LAST-ONE-OUT — two projects can share a root), so `LspHost::drop` fires its shutdown+exit+reap.

## Scope
### In
- In `close_project_at` (app.rs:6381), after `shell.close_project(idx)` returns the removed project and BEFORE it
  is moved into its `drop` thread: `if !self.shell.projects().iter().any(|p| p.root == removed.root) {
  self.lsp_hosts.remove(&removed.root); }` — remove the host ONLY when no surviving project shares the root.
- The headless proof: two projects, both get hosts; close one → its `lsp_host_exists_for_test(root)` is false (Drop
  ran) and the other survives; + a same-root pair where the host survives the first close and drops on the last.

### Out (explicitly deferred)
- **Dropping the host on the last EDITOR-tab close** — a SEPARATE #321 decision ("host shutdown/reap on
  last-doc-close"), NOT this ticket. This is WORKSPACE-close only.
- Any change to `LspHost::drop` — it is CORRECT (shutdown+exit+reap) and UNTOUCHED; the fix only makes it fire.
- A real-process death assertion (did rust-analyzer actually exit?) — needs the #320 fake-server lane, not here.
  The map-entry drop IS the proof (entry gone ⟹ `LspHost` dropped ⟹ Drop ran; Drop is correct + tested elsewhere).

## Reference (§20)
N/A — Marley-specific. This is Marley's own workspace/LSP lifecycle (the `RootView.lsp_hosts` map + `close_project`).
No Warp/Zed source read. The LSP shutdown/exit handshake `LspHost::drop` performs is the LSP protocol spec
(published material, adoption), already implemented + correct.

### Prior art
1. **OUR OWN CODE (the fix is a one-guard cleanup of an existing map):** `close_project_at` (app.rs:6381) already
   cleans `collapsed_projects`/`agents`/`remotes`/`last_agent` on close — the fix adds `lsp_hosts` to that list.
   `LspHost::drop` (lsp_host.rs) does the polite shutdown; `lsp_host_exists_for_test` (app.rs:4558, added by #321)
   is the accessor. `#321` made creation state-derived (the pump), which is why the test needs `tick_pump`
   (headless_drive.rs:76, the mock-clock lesson — a `run_until_parked` alone leaves the negative assertions
   VACUOUSLY passing). `#320`'s fake-server lane would allow a real-wire death assertion — not required here.
2. **gpui / std:** the map is `HashMap<PathBuf, LspHost>` and `LspHost: Drop`; `HashMap::remove` returns the value,
   which is then dropped in place — no crate owns this seam beyond std's Drop. Checked — no other owner.

## Locked-In Decisions
- **D1-REMOVE-KEY = `removed.root` (exact match, confirmed).** The map is keyed at insert by `active_root =
  view.shell.active_project().root.clone()` (app.rs:1412) — the RAW `Project.root` field (tabs.rs:168), NO
  `.canonicalize()` / no normalization (only the open-doc PATH is resolved via `resolve_under_root`, not the key).
  `LspHost::new` canonicalizes internally but the MAP KEY stays raw. `.root` is set once (`Project::new`) and never
  reassigned. So `removed.root` is byte-identical to the insert key → `lsp_hosts.remove(&removed.root)` hits it.
- **D2-LAST-ONE-OUT (the reachability finding — NOT an unconditional remove).** Two open projects CAN share a root:
  `Workspace::add_project` (tabs.rs:352) / `open_project_path` (app.rs:6210) push a fresh `Project` with NO dedup
  and no focus-existing, so picking the same directory twice yields two live projects sharing one `LspHost`. An
  unconditional remove on the first close would tear down a server the surviving same-root project still uses (it
  would re-spawn on a later pump tick — a visible rust-analyzer restart + lost didOpen state). So remove ONLY when
  no SURVIVING project shares the root: `if !self.shell.projects().iter().any(|p| p.root == removed.root)` (the
  removed one is already out of the Vec after `close_project`). `Workspace::projects() -> &[Project<S>]`
  (tabs.rs:412).
- **D3-DROP-UNTOUCHED.** `LspHost::drop` is correct + unchanged; the fix's only effect is that the map entry is
  removed → the `LspHost` value is dropped → Drop fires.
- **D4-PROOF-VIA-MAP + tick_pump.** The headless proof is `lsp_host_exists_for_test(root)` (the map entry). Hosts
  are created on the PUMP (#321 state-derived), so the test MUST `tick_pump` (advance_clock, mock clock — the #349
  lesson) after making a project active-with-an-open-.rs, not `run_until_parked` alone. Fixtures: `write_cargo_fixture`
  (a `Cargo.toml` gate + `.rs`) + `seed_stub_language_server` (rust → a nonexistent binary → the host is CREATED but
  no real process spawns — hermetic). A 2-project single-shell seed does NOT exist and is built by extending
  `seed_one_project`.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-CLOSE-DROPS-HOST | WHEN a workspace with an LSP host is closed and no other open project shares its root, the system shall remove its host from `lsp_hosts` (so `LspHost::drop` shuts down + reaps). | headless: 2 projects at DISTINCT roots, both hosts created (active + open `.rs` + `tick_pump`), close A → `!lsp_host_exists_for_test(root_a)`. |
| REQ-OTHER-SURVIVES | WHEN a workspace is closed, the system shall NOT drop the host of a DIFFERENT still-open project. | same drive: after closing A, `lsp_host_exists_for_test(root_b)` is still true. |
| REQ-LAST-ONE-OUT | WHEN two open projects share a root, closing ONE shall keep the shared host; closing the LAST shall drop it. | headless: 2 projects at the SAME root, host created; close one → `lsp_host_exists_for_test(root)` still true; close the last → false. |

## Phase Plan
- **P2 Design** — ratify D1/D2 (the exact remove-guard expression + placement after `shell.close_project`, before
  the `drop(removed)` thread) + the 2-project seed helper (blob route vs codec) + the host-creation sequence in the
  test (active + open `.rs` + `tick_pump`) + the test plan (distinct-roots drop+survive, same-root last-one-out).
- **P3 Implement** — the guarded remove in `close_project_at`; the 2-project seed helper.
- **P3.5 Inspect** — ★ the key form matches insert (D1); the last-one-out guard is correct (a surviving same-root
  project keeps the host; no OTHER host wrongly dropped); Drop untouched; `removed.root` read BEFORE the move into
  the drop thread (borrow); the test's `tick_pump` isn't vacuous.
- **P4 Validate** — the headless drives (+ `tick_pump`) + the `--diff` gate. The `close_project_at` arm is
  `mutants::skip`/cov-excluded → the drives carry it; if the guard is a pure helper it's cov/MSI 100.
- **P5 Complete** — CHANGELOG + the `close_project_at`/`LspHost` doc + `app_shell.md`.
