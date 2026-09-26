# Secrets hidden from what agents read — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-516-secret-redaction-for-agents.md
- **Pipeline spec:** 516-secret-redaction-for-agents.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-09-25: Warp's Secret Redaction, "love it need it".
- **Classification / tier:** feature; Marley crates, plus two fields in the Marley settings file of
  `settings_content` and an item on the Marley page in `settings_ui` (both rows exist).
- **Recall (§18.3):** plan D15 (the browser's readers redact URL values, `redact_url`); #491's
  bearer-never-logged rule; #496's pick text never carries a field's value (F-claude-496-...). No
  terminal-side redaction exists.
- **Discovery:** `crates/marley_workbench/src/mcp.rs` 319 to 420 (`terminal_blocks`, `block_entry`,
  `terminal_read`, `tail`); `browser_tools.rs` 91 and 375 (`browser_console`, `entries`);
  `crates/marley_browser/src/observe.rs` 21 to 30 and 287 (`SECRET_NAMES`, `redact_url`);
  `marley_workbench.rs` 161 (`MarleySettings`); `crates/settings_ui/src/marley_page.rs`.
- **Checklist (no task tool):** done.

### Design
- **The redactor** (`marley_mcp::redact`): a `Rule { kind, regex, keep }` list built once (a
  `LazyLock`), where `keep` is the capture group to leave in place (the variable name, the
  `Bearer ` prefix, the URL's scheme and host); user patterns compiled into rules of kind
  `pattern`; `Redactor::redact` applies rules in order on the text and returns the new text and
  the count. Pure, so the scenario is its proof.
- **The settings:** `MarleySettingsContent` gains `redact_secrets_for_agents: Option<bool>` and
  `redaction_patterns: Option<Vec<String>>`; `MarleySettings` gains `redact_secrets: bool`
  (default true) and `redaction_patterns: Vec<String>` (so it is `Clone`, no longer `Copy`). A
  global `AgentRedaction` holds the compiled `Redactor` and is rebuilt when settings change; a
  pattern that fails to compile is left out and shown in a toast on each workspace.
- **The tools:** `terminal_blocks` and `terminal_read` run the command and output through it and
  add `redacted`; `browser_console` runs each entry's text through it.
- **The page:** an Agents section with the toggle; the patterns list stays in the settings file
  ("Edit in settings.json").
- **File manifest:** `crates/marley_mcp/src/redact.rs` (new), `marley_mcp.rs` (the module),
  `crates/marley_mcp/Cargo.toml` (`regex`); `crates/marley_workbench/src/marley_workbench.rs`
  (settings, the global), `mcp.rs`, `browser_tools.rs` (Marley crate);
  `crates/settings_content/src/marley.rs`, `crates/settings_ui/src/marley_page.rs` (Zed crates, rows
  exist and are widened).
- **E2E plan:** as the spec's UI proof; the stand-in agent gains a `terminal-read` command if it
  lacks one.
- **Risks:** false positives on secret-named variables (`TOKEN_LIMIT=5` becomes redacted), which
  cost an agent a value, never leak one; the rules favour that side.

## Phase 2 — Code
- **Built:** `marley_mcp::redact` (the `Redactor`: eleven built-in rules run in order, then the
  user's patterns as `[redacted: pattern]`; a value that is already a marker is left alone, so a
  token assigned to `GITHUB_TOKEN=` counts once); `marley.redact_secrets_for_agents` (default true)
  and `marley.redaction_patterns` in `settings_content` and `default.json`; `MarleySettings` reads
  both (`Clone`, no longer `Copy`); the `AgentRedaction` global in `mcp.rs`, rebuilt on a settings
  change only when the two values changed, with an app notification naming a pattern that did not
  compile; `terminal_blocks`, `terminal_read` and `browser_console` run what they return through
  it and the two terminal tools add `redacted`; the registry's three descriptions and two output
  schemas say so; the Marley page's Agents section with the toggle.
- **Deviations:** the `Rule` gained `keep_after` (a URL's password keeps its trailing `@`, so the
  host still reads); `agent_redactor` answers the built-in rules when the global is missing
  (before `start` ran), so a read never leaves unredacted by accident.
- **Review:** `terminal_read` cut the output to its tail and then redacted it, so a private key
  whose BEGIN line fell before the cut would pass with its body intact. It now redacts the whole
  output and then cuts; `redacted` counts over the whole output, and the schema says so. The
  console entries' `source` already passes `redact_url`. No entity is read while updated: the
  tools read the terminal and the global through `&App`. Nothing here is from Warp's source; the
  token shapes are their issuers' published formats.
- **Checks:** `cargo fmt`; clippy on `marley_mcp`, `marley_workbench`, `settings_content` and
  `settings_ui` clean before the review fix; the gate's clippy runs again in Test.

## Phase 3 — Test
- **Scenario:** `script/e2e/516-secret-redaction-for-agents.sh` (`compositor sway`), with the
  browser fixture's stand-in agent, which gained `terminal-read <text>` (the newest block whose
  command holds the text: its command, both counts, its output). The scenario's setup puts the
  fakes together from pieces at run time (a private key block, a secret-named assignment, a
  bearer header, a URL password, and one each of an AWS key id, a GitHub, Slack, Stripe, Google
  and `sk-ant-` key, and a JWT; a line only a user pattern matches; a plain line), so the file
  holds none for gitleaks. The terminal runs `DEPLOY_TOKEN=<fake> sh secrets.sh`.
- **Checks in the run log** (the scenario fails on any):
  - REQ-001, on by default: every kind comes back as `[redacted: <kind>]`, no fake as printed,
    the command as `DEPLOY_TOKEN=[redacted: secret] sh secrets.sh`; `redacted: 12 in the read, 1
    in the list`; the pattern's line as printed (no built-in rule takes it). **Pass.**
  - REQ-002, `marley.redact_secrets_for_agents: false` written to the profile copy's settings
    while Marley runs: every fake as printed, no marker, `0` and `0`. **Pass** (the global
    rebuilds on the settings change, with no relaunch).
  - REQ-003, `true` again and `marley.redaction_patterns: ["zq-[0-9]{4}-internal", "("]`:
    `build id [redacted: pattern]`, `13` in the read. **Pass.**
  - The console: a page logging two fakes; `browser_console` answers `deploying with
    [redacted: github token]` and `postgres://marley:[redacted: url password]@db.example.test/app`.
    **Pass.** After the toggle is clicked off, the settings file holds
    `"redact_secrets_for_agents": false` and the console answers both as printed. **Pass.**
- **Shots, read:**
  - `516-00-printed`: the terminal shows every fake as printed, with the block's bar and pill:
    the buffer stays exact (D2).
  - `516-02-bad-pattern`: the notification "Marley left out redaction patterns that are not
    regular expressions: (: regex parse error: … unclosed group", bottom right.
  - `516-01-setting-on` (REQ-004): the Marley page, Agents section, "Redact Secrets for Agents"
    with its description and the toggle on, between Layout and Privacy.
  - `516-03-setting-off`: the same toggle off after the click at (1548, 377), with the reset
    arrow that marks a value unlike the default.
- **Found in Test, not in scope:** the first run opened the Settings window before the off
  read. sway tiled it beside the main window, the terminal narrowed from about 120 columns to 30
  and rewrapped, and the block's bar and pill vanished; `terminal_read` then answered rows that
  began inside the command line (`ake sh secrets.sh`) and stopped mid-line. The block anchors do
  not follow a rewrap (the plan's D2, "the known weak spot"). Minted as TICKET-544 with a Queue
  row; the scenario now reads the terminal before the Settings window opens and proves the
  toggle through the console, which has no rows to lose.
- **Also changed in Test:** from the Orca comparison, the private-key rule takes the PGP `BLOCK`
  form and a block with no END line yet (hidden to the end); a bare token as a URL's userinfo is
  a `url password`; Slack's `xoxe-`/`xoxo-` prefixes. A stale doc comment in `registry.rs` said
  a setting can take `browser.write` away; none does, and it now says so.
- **Focus report:** sway, headless; "hyprland: 0 Marley windows before the run, 0 after; the run
  added no rule and did not reload it".
- **Gate:** `just gate-diff`. The first run was red on rustfmt alone (the Slack rule's line grew
  past the width with the Orca prefixes, after the last `cargo fmt`); formatted, the second run
  ended `GATE GREEN [diff]`, 16 passed, 0 failed, clippy on every target of the scope included.
- **Pre-existing, not in scope:** TICKET-544 (above).
- **Verdict:** PASS.

## Phase 4 — Complete
- **Docs:** `CHANGELOG.md` (Added: secrets hidden from what agents read);
  `docs/marley_architecture/marley_mcp.md` (a Redaction section: `redact.rs`);
  `docs/marley_architecture/marley_workbench.md` (the Marley page's Agents section,
  `MarleySettings`' two fields, the `AgentRedaction` global and the three tools);
  `docs/marley/zed-touchpoints.md` (the `settings_content` row: `default.json`'s `marley` block now
  exists and names no layout; the `default.json` and `marley_page.rs` rows were widened in Code);
  the Warp once-over note (item 2 shipped as #516). The slice is a once-over item, not a plan
  slice, so `three-prong-plan.md` is unchanged.
- **Knowledge:** F-claude-516-a-cut-before-redaction-leaks-the-cut-secret-001,
  F-claude-516-a-rewrap-moves-every-blocks-rows-001,
  PR-claude-redact-the-whole-text-before-cutting-it-001,
  L-claude-516-a-second-window-in-sway-narrows-the-terminal-under-test-001,
  L-claude-516-fake-secrets-are-put-together-at-run-time-001,
  AD-claude-516-redact-at-the-tool-boundary-on-by-default-001.
- **Brain:** Plan skipped the consultation; asked at Complete (consultation
  e5319b69205442818388d6651922d57f: nothing on redaction, only unrelated follow-ups due) and
  decided: `decisions/marley-hides-secrets-in-what-its-tools-hand-agents-on-by-default`, follow-up
  by 2026-10-09.
- **Found here and ticketed:** TICKET-544 (blocks keep their rows across a rewrap), with a Queue
  row, in this commit.
- **Ticket:** closed; the pair archived to `completed/`.
