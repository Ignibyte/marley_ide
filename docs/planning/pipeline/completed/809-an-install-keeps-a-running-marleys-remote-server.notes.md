# An install keeps the remote server of a Marley still running — Notes

- **Local ticket doc:** ../../tickets/closed/TICKET-809-install-keeps-the-running-builds-remote-server.md
- **Pipeline spec:** 809-an-install-keeps-a-running-marleys-remote-server.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-19 (`/goal`): "lets do 812, 814, 809, 806". Filed 2026-10-17 after
  Open Remote Folder failed with "no Marley remote server is staged for linux-x86_64 at commit
  3f1b2d6" from a Marley three installs old.
- **Classification / tier:** bug; `script/install-marley` (Marley-owned, #502); no Rust.
- **Recall (§18.3):**
  - AD-claude-445: the server is staged by commit, newest three kept; on `dev` the client uses or
    uploads only the staged server of its own commit (`staged_dev_remote_server`,
    `auto_update.rs`).
  - Zed's own `cleanup_remote_server_cache` runs only on the download path, which `dev` never
    takes, so the install's prune is the only remover.
  - The brain (consultation 6bebff32d0b64be69aa2e56e1b7df2e8): nothing on this seam.
- **Discovery:** `script/install-marley`, `stage_remote_server`: after the `mv` into
  `$platform_dir/$commit.gz`, `find … -printf '%T@ %p\n' | sort -rn | tail -n +4 | cut … | xargs
  -r rm -f --`. The script's header says "The newest three are kept."
- **Decisions:** D1, D2 in the spec.

### Design
- Approach: a `running_commits <platform_dir>` function prints, one a line, each staged commit
  (the `.gz` names) that the binary of a running Marley holds: for each `pgrep -x marley` pid,
  `grep -aoF -f <(the names) /proc/<pid>/exe`, then `sort -u`. The prune reads the paths past the
  newest three and removes each whose name is not in that list (`grep -qxF`). The header's
  sentence names the kept servers.
- File manifest: `script/install-marley` (Marley-owned). No Rust, no Zed path.

### Visual check plan
- REQ-001, REQ-002: a scratch `XDG_DATA_HOME` with
  `marley/remote_servers/dev/linux-x86_64/` holding four fake `.gz` files: the running Marley's
  commit (`/proc/<pid>/exe` of Chad's Marley, pid 7745, built at `ba5aacce78…`) as the oldest,
  and three made-up 40-hex names, newer. `XDG_DATA_HOME=<scratch> script/install-marley
  --remote-server-only` stages HEAD's server there and prunes. Listing before and after: HEAD's
  server, the two newest fakes and the running commit's stay; the oldest fake goes. The old
  prune, run alone on a copy of the same fixture, removes the running commit's file.
- REQ-003: `just shot`.

### Risks
- `grep` reads each running binary once (about 2.3 GB for a release build): seconds per Marley,
  once per install.
- A pid that exits between `pgrep` and `grep` is skipped (`2>/dev/null || true`).

## Phase 2 — Code
- Built to the manifest in `script/install-marley`: `running_commits` before `stage_remote_server`,
  and the prune's `xargs rm` replaced by a loop that keeps a path whose name `running_commits`
  printed. The header's "The newest three are kept" now names the running Marleys' servers too.
- Review: under `set -euo pipefail`, `pgrep` with no match and `grep` with none both end in
  `|| true`, and an empty `running` keeps nothing extra (`grep -qxF` on an empty string fails);
  a name is a 40-hex commit, so `grep -F` and the unquoted `for pid` are safe; `shellcheck` clean.
  The Mac branch keeps its own newest three (out of scope).
- The gate: `just gate-diff` green, 17 of 17.

## Phase 3 — Test
- The fixture: a scratch `XDG_DATA_HOME` with four fake staged servers: `ba5aacce78…` (the commit
  Chad's running Marley, pid 7745, was built at; its `/proc/7745/exe` reads `(deleted)` and still
  holds the commit once) dated 2026-10-11, then `1111…` (10-12), `2222…` (10-13), `3333…` (10-14).
- The control: the old prune line alone, on a copy of the fixture with a newest file added, left
  `2222…`, `3333…` and the new one, and removed `ba5aacce78…`: the bug.
- The fix: `XDG_DATA_HOME=<scratch> script/install-marley --remote-server-only` built the musl
  server (2m 13s), staged `9a533d872936…gz` (39 MB, "with changes not committed"), and left
  `2222…`, `3333…`, `9a533d87…` and `ba5aacce78…`; `1111…` went (REQ-001, REQ-002). Chad's real
  `~/.local/share/marley/remote_servers` was not touched.
- REQ-003: `just shot 809-starts` refused on Hyprland while Chad's Marley is open, and ran under
  `COMPOSITOR=sway`: `809-starts` shows the debug Marley drawn with the scratch project's trust
  prompt over it, its rail and status bar in place. Focus report: "1 Marley windows before the
  run, 1 after".
- The scratch fixture and its staged server were removed after the run.

## Phase 4 — Complete
- Docs: `CHANGELOG.md` (Fixed, #809); `docs/marley/guide.md`, the remote server's paragraph and
  the data-folder table row; the script's own header. No Zed path; no slice status moves.
- Ledger appends: `F-claude-809-an-install-pruned-the-running-marleys-remote-server-001`,
  `L-claude-809-a-running-marleys-commit-is-in-its-binary-001`. No prevention rule: the class (an
  age-pruned cache a running reader needs) has this one instance, now fixed at its source.
- Brain: consultation `6bebff32d0b64be69aa2e56e1b7df2e8` closed with `brain decide`,
  `decisions/an-install-keeps-the-staged-remote-server-of-every-running-marleys-commit-read-from-its-binary`
  (follow-up 2026-11-19).
- Ticket closed; its `BACKLOG.md` row left at promotion.
