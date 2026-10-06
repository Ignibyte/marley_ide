# The Secrets tab — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-667-rusty-secrets-tab.md
- **Pipeline spec:** 667-rusty-secrets-tab.spec.md

## Phase 1 — Plan (2026-10-06)
- **Request:** the Queue's top after #666, from Chad's 2026-10-06 "adding the last things missing";
  the last of the Rusty-in-Marley batch.
- **Classification / tier:** feature, R8's last part; Marley crates and two small hunks in Zed's
  `context_server` crate.
- **Pre-flight:** green; no active pipeline; cargo idle; 75 GB free on the build disk.
- **Recall (§18.3):**
  - R-D7 in `docs/marley/rusty-in-marley.md`: no secret value in Marley's logs, settings or state.
  - AD-666: masked values never sent back; a secret's field emptied after its write.
  - L-664, L-665: a toggle group in a box of its own; `shellcheck` and `typos` on the scenario.
  - The brain (consultation `0263871a0bfd4cb2b12782eeef7e119e`): nothing on this seam.
- **Discovery:**
  - Rusty at `eb1ab51`: `secret_list` answers the names; `secret_pin_status` `{ set, unlocked,
    locked_out_seconds }`; `secret_pin_set { pin, token? }` `set` (a PIN of six characters or more,
    no line break; changing one needs the token); `secret_unlock { pin }` `{ token,
    expires_in_seconds }`, refusing `no PIN is set; set one in the app first`, `wrong PIN`,
    `wrong PIN 5 times; locked for a minute`, `locked for another <n> seconds`; `secret_lock`
    `locked`; `secret_reveal { key, token }` `{ key, value }` or `no secret named <key>`;
    `secret_update { key, value, token }` `updated`; `secret_set { key, value, token? }` `set` and
    `secret_delete { key, token? }` `deleted`, the token needed once a PIN is set (`the vault is
    locked; unlock with the PIN and pass the token`); a stale token `that unlock is not the live
    one; unlock again` or `the unlock has expired; unlock again`.
  - Rusty's app (`SecretsPage.qml`): its words on the vault; the PIN set twice; Unlock; Lock;
    the expiry timer; `Window.active` false relocks; rows with `••••••••`, Reveal or Hide, Copy,
    Replace, Delete asked first.
  - Zed's `context_server`: `client.rs:282` `recv: {message}` and `:341` `outgoing message:` at
    trace, `:305` `Unhandled JSON from context_server: {message}` at error; the stdio transport's
    `:102` `outgoing message:` again at trace. `Client::new` knows the server's id; the HTTP
    transport logs no body. Marley's connection is `marley-rusty`, Zed's agents' `rusty`.
  - Zed's `Editor::set_masked` (as `askpass_modal.rs:45` uses it); `cx.write_to_clipboard`;
    `cx.observe_window_activation`.

### Design
- **Zed, `crates/context_server/src/client.rs`** (Zed crate, additive): `QUIET_SERVERS`, a
  `parking_lot::Mutex<Vec<ContextServerId>>`, `pub fn log_messages_by_size(id)` adding to it, and
  `fn logs_by_size(id)`; `Client::new` reads it once and passes `quiet` to `handle_input` and
  `handle_output`, whose trace lines and the unhandled-JSON error line log the size when quiet.
  **`transport/stdio_transport.rs`** (Zed crate): the outgoing trace line logs the size for every
  server (the client's own line logs the body for the others). A `// Marley:` comment on each hunk;
  rows in `docs/marley/zed-touchpoints.md` first.
- **`marley_rusty/src/secrets.rs`** (Marley): the ten tool names; `PinStatus`, `Unlock`, `Revealed`;
  `names_from_answer`; `SecretWrite` (`Set { key, value, token }`, `Update`, `Delete`, `PinSet {
  pin, token }`) with `tool` and `arguments` (the token left out when there is none); parsers that
  report a failure without the answer's text.
- **`rusty/secrets_tab.rs`** (new, Marley): `OpenSecrets`; `BrainSecretsView` with the shared link
  and reads (`secret_list` and `secret_pin_status` together), the token and its expiry task, the
  revealed key and value, masked editors (PIN, again, unlock PIN, key, value, a replace value), the
  window-activation subscription that locks. Lock drops the token, the shown value and the timer,
  and sends `secret_lock`. Writes carry the token; a refusal shows in the tab; Delete asks first;
  Copy writes the clipboard. Nothing in it logs a PIN, a token or a value.
- **`rusty.rs`**: `mod secrets_tab;`, its `init`, and `context_server::client::log_messages_by_size`
  for `marley-rusty` and `rusty` at `init`.
- **`brain.rs`**: a Secrets button (`IconName::LockOutlined` or the nearest) after Skills.
- **The stand-in**: a PIN in `pin` and secrets in `secrets.json` in its state folder; the ten tools
  with a token that expires after `pin_timeout_minutes` (or `pin_timeout_seconds`, the scenario's
  short timeout), five wrong PINs locking for a minute, and Rusty's refusals; writes announced.
- **The guide page**: a Secrets article.
- **File manifest:** `crates/context_server/src/client.rs`,
  `crates/context_server/src/transport/stdio_transport.rs` (Zed); `crates/marley_rusty/src/
  {secrets.rs, marley_rusty.rs}`, the stand-in, `crates/marley_workbench/src/rusty.rs`,
  `crates/marley_workbench/src/rusty/{secrets_tab.rs, brain.rs}`, the guide page (Marley);
  `script/e2e/667-rusty-secrets-tab.sh` (Test).

### Visual check plan
| Criterion | What the scenario does | Shot |
|---|---|---|
| REQ-002, 003 | Palette: `rusty: open secrets` | `667-01-no-pin` |
| REQ-004 | The PIN twice, Set PIN | `667-02-pin-set` |
| REQ-005 | A wrong PIN, Unlock; then the PIN | `667-03-wrong-pin`, `667-04-unlocked` |
| REQ-006 | Reveal on a name | `667-05-revealed` |
| REQ-007 | A key and a value, Set; Replace, a value, Enter | `667-06-added`, `667-07-replaced` |
| REQ-008 | Delete, Delete in the prompt | `667-08-delete-prompt`, `667-09-deleted` |
| REQ-009 | Focus another window; unlock with a short timeout and wait | `667-10-locked-on-blur`, `667-11-expired` |
| REQ-001 | The run's log at `context_server=trace` | Checks: size lines present, no PIN, token or value |
| REQ-010 | Not shot | Review |

### Risks
- **Focus to another window under sway**: the scenario opens a second window (a terminal's own, or
  Marley's Settings window) to move the focus; if that cannot move it, the expiry shot still proves
  the lock path and the blur is read from the code.
- **A user's own Zed trace logging of other servers** is unchanged; only Rusty's servers go quiet.

### Checklist (no TaskCreate in this harness)
- [x] Pick, pre-flight, recall, the brain; discovery.
- [x] Mint the pair; the BACKLOG row removed; the ticket in progress.
- [x] Prior-art sweep; spec; design; visual check plan; risks.
- [x] Phase 1 PASS under Chad's request ("continue on tickets until finished").

## Phase 2 — Code (2026-10-06)
### Built
- **Zed, `context_server/src/client.rs`** (rows first in `docs/marley/zed-touchpoints.md`):
  `QUIET_SERVERS` (names, since the client's `ContextServerId` is crate-private),
  `pub fn log_messages_by_size(id: &str)`, `logs_by_size`; `Client::new` reads it once and passes
  `quiet` to `handle_input` and `handle_output`, whose `recv:`, `outgoing message:` and
  `Unhandled JSON` lines log the size when quiet. **`transport/stdio_transport.rs`**: the
  duplicate outgoing line logs the size for every server. Each hunk carries a `// Marley:` comment.
- **`marley_rusty/src/secrets.rs`**: the nine tool names; `PinStatus`; `Unlock` and `Revealed`
  with a `Debug` that leaves out the token and the value; the answers' parsers, whose error never
  quotes the answer; `SecretWrite` (`Set`, `Update`, `Delete`, `PinSet`) with `tool`, `arguments`
  (a token not held left out) and a `Debug` naming only the tool and the key.
- **`rusty/secrets_tab.rs`**: `OpenSecrets`; `BrainSecretsView` with the shared link and reads
  (`secret_list` and `secret_pin_status` together), the token and its expiry task, the revealed
  value, the replace target, masked fields (`set_masked`) emptied once sent; Set PIN (twice alike),
  Unlock, Lock, Change PIN, Set, Reveal or Hide, Copy, Replace, Delete asked first; locking on the
  expiry, on Lock, on the window losing the focus (`observe_window_activation`) and on release
  (`on_release` sends `secret_lock`).
- **`rusty.rs`**: `MARLEY_SERVER`; `log_messages_by_size` for it and for `rusty` at `init`;
  `mod secrets_tab` and its `init`. **`brain.rs`**: a Secrets button (`Lock`) after Skills.
- **The stand-in**: `secrets.json` and `pin` in its state folder; the nine tools with an
  in-process token that expires after `pin_timeout_minutes` (or its own `pin_timeout_seconds`), five
  wrong PINs locking a minute, and Rusty's refusals; vault writes announced. A smoke run over stdio
  passed each.
- **The guide page**: a Secrets article.

### Deviations
- The registry is keyed by the server's name, not `client::ContextServerId`, which is
  `pub(crate)` there (clippy's `private_interfaces`).

### Review
- Clippy: `private_interfaces` (above), a redundant closure, `unused_self` (`delete`),
  `needless_pass_by_value` (`send` takes `&SecretWrite`, `reveal` `&str`), a missing `;`, an unused
  binding.
- No PIN, token or value reaches a log: the client's lines are size-only for both of Rusty's
  servers, the tab logs only `secret_lock`'s answer, and every parse error and `Debug` leaves the
  secret out. The token stays in the view; the clipboard receives a value only on Copy.

### Gate
`just gate-diff`: GATE GREEN [diff], 17 of 17.

## Phase 3 — Test (2026-10-06)
Scenario `script/e2e/667-rusty-secrets-tab.sh` under `compositor sway`: the stand-in's
`secrets.json` with `github_token` and `openai_api_key` (made-up values) and no PIN; the run with
`ZED_LOG=info,context_server=trace`. Four runs; the last passed every check, the two log checks
included.

### What each shot shows (last run)
- `667-01-no-pin` (REQ-002, 003): Rusty's words on the vault, Set a PIN (two masked fields), the set
  row (Set greyed until a PIN or an unlock), "2 secrets", each name with `••••••••` and Delete.
- `667-02-pin-set` (REQ-004): the PIN field and Unlock, "Unlocking shows, edits, copies and deletes
  values for a few minutes.", "PIN set.", and "Unlock to set a value." by the set row.
- `667-03-wrong-pin` (REQ-005): Rusty's "wrong PIN" in red.
- `667-04-unlocked` (REQ-005, 006): Unlocked, Lock, Change PIN; each name with Reveal, Replace and
  Delete.
- `667-05-revealed` (REQ-006): github_token's value in the accent colour, Hide and Copy.
- `667-06-added` (REQ-007): "Set.", tour_token listed, both fields empty.
- `667-07-replaced` (REQ-007): "Replaced.", the new value shown after Reveal.
- `667-08-delete-prompt`, `667-09-deleted` (REQ-008): "Remove tour_token from the vault?"; then
  "Deleted." and two secrets.
- `667-10-locked-on-blur` (REQ-009): after the Settings window took the focus and closed: the PIN
  field again, "Locked.", the values hidden.
- `667-11-expired` (REQ-009): unlocked with the stand-in's four-second timeout, eight seconds
  later: locked again.
- **The log** (REQ-001): `marley-rusty`'s messages traced by size only, 69 received and 133 sent
  lines of `… bytes`; none of the PIN, the wrong PIN, the four values or the tokens the stand-in
  issued. The five lines with a body belong to `marley`, Marley's own MCP bridge, which nobody marks
  and which carries no secret.
- REQ-010 by review: the state line is the Memory tab's.

### Fixes, each run again
- **The scenario's positions**: the unlock field at y 172; once unlocked, the set row at 207 and the
  names from 286, 43 apart, with Reveal, Replace and Delete at x 1193, 1256 and 1318; a write's
  notice line moves the names down 31.

No change to the code in this phase.

## Phase 4 — Complete (2026-10-06)
- **Docs:** `CHANGELOG.md` (Added; Fixed for the log); `docs/marley/rusty-in-marley.md` (R-D7, the
  screens row, R8 complete, the batch line); `docs/marley/zed-touchpoints.md` (the two
  `context_server` rows, written before the hunks and matching what shipped); `docs/marley/guide.md`
  (The Secrets tab); `docs/marley/walkthrough.md` (stop 2.15i, `rusty: open secrets` in Appendix B);
  `docs/marley_architecture/marley_rusty.md` (`secrets`, the stand-in's vault);
  `docs/marley_architecture/marley_workbench.md` (The Secrets tab, the log marks).
- **Knowledge:** L-claude-667-zeds-mcp-client-logs-whole-messages-at-trace-001,
  AD-claude-667-rustys-secrets-never-reach-a-log-and-the-token-lives-in-the-tab-001. No F block:
  the logging was Zed's as it stands, met before the tab could expose it.
- **Brain:** `brain decide` on consultation `0263871a0bfd4cb2b12782eeef7e119e`, follow up by
  2026-11-06: `decisions/rustys-secrets-never-reach-a-log-and-the-unlock-lives-in-marleys-secrets-tab`.
- **Closed:** the ticket in `tickets/closed/`; this pair in `completed/`.
