# Codex 0.162 in the tested range — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-771-codex-0162-in-the-tested-range.md
- **Pipeline spec:** 771-codex-0162-in-the-tested-range.spec.md

## Phase 1 — Plan
- **Request:** the queue's top row, split from #750, #752 and #753's findings (2026-10-11).
- **Classification / tier:** chore; a constant and doc lines in `marley_agent` and
  `marley_workbench`; the guide. No Zed file.
- **Recall (§18.3):**
  - #648: an integration runs on the versions checked; the agent bar's chip says why one is off
    and its click opens the setting that allows an untested version.
  - #650 and #651: the range closed at the two ends whose generated schemas agree on every field
    Marley reads; #651 compared the requests and `serverRequest/resolved` byte for byte.
  - The box: Codex is mise's (`~/.local/bin/codex` runs `mise x codex`), with 0.160.0 and 0.162.0
    installed; `marley.codex_app_server` is unset in Chad's settings (off).
  - The brain: nothing on this seam beyond #650's decision.
- **The check** (scratchpad `codex-771/`):
  - 0.158.0 installed with `MISE_DATA_DIR=<scratch>/mise mise install codex@0.158.0`, so Chad's
    mise state is untouched; 0.162.0 is the box's.
  - `codex app-server generate-json-schema [--experimental] --out …` from each, with
    `CODEX_HOME=<scratch>/home` under `unshare -rn`: stable 314 and 317 files, experimental 440 and
    447.
  - A script (`python3 -I`) took the `oneOf` alternatives of `ServerNotification`, `ServerRequest`
    and `ClientRequest` for the methods Marley handles (from `codex_events.rs` and
    `codex_server.rs`) and their responses, walked every `$ref` from them (154 definitions), and
    compared old and new: six changed, all additive (the spec's D2). Marley parses `ThreadStatus`
    with `#[serde(other)]`, reads `Turn.status` and `TurnError.message`, reads `item/started` as
    loose JSON and only a `fileChange` item's paths (`codex_events.rs` 887-898; that variant is
    unchanged), and reads no `MessagePhase`, `ResponseItem` or `CodexErrorInfo`.
  - `InitializeParams`, `InitializeResponse`, `InitializeCapabilities` and `ClientInfo` are the
    same; `ClientNotification.json` is byte-identical; `app-server --help` lists `--listen <URL>`
    with `unix://PATH`, and `--remote <ADDR>` is on `codex` and `codex resume`, at both versions.

### Design
- `marley_agent/src/versions.rs`: `CODEX_APP_SERVER.tested.before` → `0.162.1`; its doc names the
  ends 0.155.1 and 0.162.0 and what the 0.162 check compared.
- `marley_agent/src/codex_events.rs`: the module doc's versions.
- `docs/marley/guide.md`: "It runs on Codex 0.155.1 to 0.162.0".
- **File manifest:** the two Marley crate files; the guide at Complete.

### Visual check plan
| Criterion | The scenario | Shot |
|---|---|---|
| REQ-001 | `MARLEY_CODEX` = the box's 0.162.0, `marley.codex_app_server` on, `marley` on the terminals' PATH; in the repo's terminal `marley agent-cli --provider=codex`; after a settle, `pgrep -af "app-server --listen unix://"` names a process, and the run's log has no "so Codex starts without one" | `771-01-codex` |
| REQ-002 | not run: the range's lower end and the chip are #648's, unchanged | — |

### Risks
- The real Codex TUI in a scratch `CODEX_HOME` starts on its sign-in screen; the App Server and
  Marley's join do not need a sign-in, and no turn runs.
- 0.159 to 0.161 sit inside the range unchecked, as 0.156 and 0.157 did.

## Phase 2 — Code
- **Built:** `versions.rs`: `CODEX_APP_SERVER.tested.before` = 0.162.1, its doc naming 0.162.0 and
  what was compared; `codex_events.rs`: the module doc's versions.
- **Deviation:** `crates/marley_fleet/Cargo.toml` asks for serde's `derive`: `just clippy
  marley_agent` alone failed in `marley_fleet` ("cannot find derive macro `Serialize`"), which
  built only when another crate in the same build turned the feature on. Pre-existing, one line,
  fixed at its source.
- **Review of the diff:** the lower end and every other integration's range are untouched; no test
  pins the range.

## Phase 3 — Test
- **Scenario:** `script/e2e/771-codex-0162-app-server.sh` (`compositor sway`). The first run
  showed Marley start the App Server on 0.162.0, which then exited: the runner's scratch
  `CODEX_HOME` did not exist and Codex refuses one; `setup` makes it.
- `771-01-codex` (REQ-001): Codex's tab reads `codex --remote unix:///run/…`, the launch Marley
  uses only when it joins an App Server, on Codex's sign-in screen (a scratch home); `app servers:
  … codex app-server --listen unix:///run/user/1000/marley-<pid>/codex/1.sock`; `check Marley did
  not start Codex without one: pass`.
- **Seen, not in scope:** the Codex row reads as a plain terminal while Codex sits on its sign-in
  screen with no thread.
- REQ-002 by the review.
- **Gate:** `just gate-diff` GREEN, 17 passed; run again after the in-app guide's range line
  (`crates/marley_workbench/guide/index.html`, which the receipt covers) at Complete.

## Phase 4 — Complete
- **Docs:** `CHANGELOG.md` (Changed: Codex 0.162 with its App Server); `docs/marley/guide.md` and the
  in-app guide (`crates/marley_workbench/guide/index.html`, two lines) name 0.155.1 to 0.162.0;
  `docs/marley_architecture/marley_agent.md` (the range and the versions `codex_events` reads).
  The gate ran again after the in-app guide's change.
- **Ledgers:** F-claude-771-marley-fleet-built-only-beside-a-crate-that-turned-on-serde-derive-001,
  L-claude-771-check-a-new-codex-against-the-schema-it-generates-001,
  L-claude-771-the-runners-codex-home-does-not-exist-yet-001.
- **Brain:** consultation 91c2f370…, `brain decide`.
- **Open with Chad:** turning `marley.codex_app_server` on (unset in his settings, so off).
- Ticket closed, pipeline archived, committed.
