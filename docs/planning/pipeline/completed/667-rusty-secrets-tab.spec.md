---
pipeline_id: 5afe7708-b8fc-4d2a-9f14-0239d4a06b91
ticket: docs/planning/tickets/open/TICKET-667-rusty-secrets-tab.md
status: Phase 4 — Complete PASS
title: "The Secrets tab"
type: feature
slice: Rusty in Marley R8 (docs/marley/rusty-in-marley.md)
references: [docs/marley/rusty-in-marley.md, docs/planning/pipeline/completed/666-rusty-settings.spec.md]
---

## Title
The Secrets tab. Rusty keeps secrets (keys for providers and services) behind a PIN:
`secret_list` gives the names only; `secret_pin_status`, `secret_pin_set`, `secret_unlock` (a
token for `pin_timeout_minutes`), `secret_lock`, `secret_reveal`, `secret_update`, and `secret_set`
and `secret_delete`, which take the token once a PIN is set (Rusty's TICKET-049). Marley shows
none. Zed's MCP client writes every message whole to its trace log, so a PIN, a token or a revealed
value would reach Zed's log file once trace logging is on. This ticket closes that first, then adds
the tab.

## Scope
### In
- **No secret in a log.** Zed's MCP client logs the messages of a server Marley marks by their size,
  never their content: the received and sent lines and an unhandled message's error line; the stdio
  transport's duplicate outgoing line logs the size for every server. Marley marks Rusty's servers
  (its own connection, and `rusty` for Zed's agents). Marley itself never logs a PIN, a token or a
  value, and a secret tool's answer that does not parse is reported without its text.
- **The tab.** `rusty: open secrets`, and a Secrets button after Skills in the Brain view, open a
  center tab: Rusty's words on what the vault is, the lock, a row to set a secret, and the names,
  each with its value hidden.
- **The lock.** With no PIN: set one (twice; six characters or more, Rusty's rule). With a PIN,
  locked: the PIN and Unlock (Rusty's refusals, a wrong PIN and the lockout, in its words). Unlocked:
  Lock and Change PIN. The tab locks when the unlock expires, when Lock is clicked, and when Marley's
  window loses the focus, as Rusty's app does.
- **The values.** Unlocked, each name has Reveal (Hide once shown), Copy while shown, Replace (a new
  value, Enter saves) and Delete (asked first). Setting a new secret needs the unlock once a PIN is
  set. Every PIN and value field is masked, and is emptied once sent.
- **Live.** Reads the names and the lock's state when it opens, when Rusty connects, on Rusty's
  announcement, after each write; off or not connected says so.
- **`marley_rusty::secrets`** and the stand-in's ten tools with a PIN, a token that expires, the
  lockout and Rusty's refusals.

### Out (explicitly deferred)
- **Importing secrets** from a file or another vault.
- **Showing who read a secret**; Rusty keeps no such record.

## Reference (§20)
N/A — Marley-specific: Rusty's own app is the reference for the behaviour
(`crates/rusty-app/qml/SecretsPage.qml` in Rusty's repository at `eb1ab51`: the vault's words, the
PIN set twice, Unlock and Lock, the unlock's expiry and the window's focus relocking, a key and a
masked value with Set, each name hidden with Reveal or Hide, Copy, Replace and Delete asked first).
Zed's own masked editors, buttons, prompt and clipboard draw it.

### Prior art
- **Behaviour maps:** none hold a PIN-locked vault inside an editor.
- **Published material:** Rusty's tools (`crates/rusty-mcp/src/main.rs:1856-1980`; `SecretSetParams`
  and `SecretDeleteParams` `:347-365` with the token; `PinSetParams`, `PinParams` `:469-482`;
  `SecretRevealParams`, `SecretUpdateParams` `:486-502`) and its lock
  (`crates/rusty-core/src/engine/pin_lock.rs`: `PinStatus { set, unlocked, locked_out_seconds }`,
  `Unlock { token, expires_in_seconds }`, the refusals, five wrong PINs and a minute's lockout).
- **The code we ship:** Zed's `context_server` client (`client.rs:282`, `:305`, `:341`) and stdio
  transport (`stdio_transport.rs:102`), which log message bodies at trace and error level; Zed's
  `Editor::set_masked`, as its askpass modal uses it; the Memory and Skills tabs' link and reads.

## UI proof
`script/e2e/667-rusty-secrets-tab.sh` (`compositor sway`: clicks on buttons and fields). Setup: the
stand-in with two made-up secrets and no PIN, the run's log at `context_server=trace`, never the
user's Rusty or vault (R-D8). Shots:
- `667-01-no-pin`: `rusty: open secrets`: Rusty's words, Set a PIN, the two names hidden.
- `667-02-pin-set`: a PIN twice: locked, the PIN field and Unlock.
- `667-03-wrong-pin`: a wrong PIN: Rusty's "wrong PIN".
- `667-04-unlocked`: the PIN: Lock, Change PIN, each name with Reveal, Replace and Delete.
- `667-05-revealed`: Reveal: the value, Copy, Hide.
- `667-06-added`: a key and a value with Set: listed, the fields empty.
- `667-07-replaced`: Replace, a new value, Enter: Reveal shows the new value.
- `667-08-delete-prompt`, `667-09-deleted`: Delete, asked, done.
- `667-10-locked-on-blur`: the focus moved to another window: locked again.
- `667-11-expired`: unlocked with a short timeout, waited: locked again.
And the log check: the run's log holds trace lines for Rusty's server, by size, and none of the
PIN, the tokens or the values.

## Locked-In Decisions
- D1 — **Close the log hole in Zed's client, by server**: a small registry in
  `context_server::client` (`log_messages_by_size(id)`), checked when a client starts; the stdio
  transport's duplicate line logs the size for all. Additive hunks with ledger rows.
- D2 — **Marley marks Rusty's two server ids** at `rusty::init`: Marley's own connection and Zed's
  `rusty` context server.
- D3 — **The token lives only in the tab**, never in a global, a setting or a log; it goes with each
  write once a PIN is set.
- D4 — **Lock on expiry, on Lock, and on the window losing focus**, as Rusty's app does; locking
  sends `secret_lock` and drops the token and any shown value.
- D5 — **Masked fields, emptied once sent**; a revealed value shows until Hide, the lock, or another
  Reveal.
- D6 — **A secret tool's unparsed answer is reported without its text**, since serde's message can
  quote the value.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE trace logging is on, the system shall log Rusty's MCP messages by their size and never their content. | The run's log; the review of each log line |
| REQ-002 | WHEN `rusty: open secrets` runs or the Brain view's Secrets button is clicked, the system shall open the Secrets tab or bring it forward. | Shot `667-01-no-pin` |
| REQ-003 | WHILE the tab shows, it shall list the secrets' names with their values hidden. | Shot `667-01-no-pin` |
| REQ-004 | WHEN a PIN is given twice alike, the system shall set it with Rusty; IF Rusty refuses, THEN its words shall show. | Shot `667-02-pin-set` |
| REQ-005 | WHEN the PIN is given and Unlock clicked, the system shall unlock with Rusty and keep the token for the tab only; IF Rusty refuses, THEN its words shall show. | Shots `667-03-wrong-pin`, `667-04-unlocked` |
| REQ-006 | WHILE unlocked, each name shall offer Reveal, Replace and Delete, and a revealed value Copy and Hide. | Shots `667-04-unlocked`, `667-05-revealed` |
| REQ-007 | WHEN a key and a value are set, or a value replaced, the system shall write it with the token and empty the field. | Shots `667-06-added`, `667-07-replaced` |
| REQ-008 | WHEN Delete is confirmed, the system shall delete the secret with the token. | Shots `667-08-delete-prompt`, `667-09-deleted` |
| REQ-009 | WHEN the unlock expires, Lock is clicked, or Marley's window loses the focus, the tab shall lock and drop the token and any shown value. | Shots `667-10-locked-on-blur`, `667-11-expired` |
| REQ-010 | WHILE Rusty is off or not connected, the tab shall say so and list nothing. | Review (the Memory tab's state line) |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the ledger rows, then the `context_server` hunks; `marley_rusty::secrets`;
  `rusty/secrets_tab.rs`; the marks at `rusty::init`; the Brain view's button; the stand-in; the
  guide page; a review; `script/gates.sh --diff` green.
- **P3 Test** — the visual check and the log check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG, the plan's R8, the architecture notes, the touchpoints rows, the guide
  and the walkthrough, ledger capture, the brain decision, close, archive, commit.
