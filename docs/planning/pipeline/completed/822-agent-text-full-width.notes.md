# An agent's text runs the full width, with a toggle in its bottom row — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-822-agent-text-full-width.md
- **Pipeline spec:** 822-agent-text-full-width.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-20. Another session told him to use #798's width button for a 100%
  agent; he saw no button. A shot of his screen showed a thread tab alone in its row (so #798
  hides the button) with its text in Zed's 850 px column. His settings got
  `agent.limit_content_width: false` through settings_change (applied). He then asked for the
  default to be full width, overriding Zed, and for the control in the bottom row; he picked an
  always-shown toggle of its own over folding it into the width button. #821 (tmux) was parked
  to queued for this, he said "not yet" to it.
- **Classification / tier:** feature; small; a Zed-crate touch in `agent_ui` and the default.
- **Recall (§18.3):** #798's notes: Marley controls in this row go through `MarleyThreadHost`
  and name no Marley crate from `agent_ui`. Rusty's brain (consultation `ee7fe4fa9a0b4756a1222d20619ee663`): nothing on agent content width.

### Design
- **`assets/settings/default.json`** (Zed file): `"limit_content_width": false` with
  `// Marley: full width by default (#822); Zed's default is true.`
- **`crates/settings_content/src/agent.rs`** (Zed crate): the doc string's `Default: true`
  becomes `Default: false in Marley (#822)`.
- **`crates/agent_ui/src/conversation_view/thread_view.rs`** (Zed crate, additive):
  `render_marley_text_width(&self, cx) -> Option<AnyElement>`, `None` without
  `MarleyThreadHost`; an `IconButton` (`IconName::Maximize` while the column is on, a narrowing
  icon while full; the coder picks from `icons.rs`), tooltip "Text: full width" / "Text:
  readable column", on click `update_settings_file(fs, cx, |settings, _| settings.agent
  .get_or_insert_default().limit_content_width = Some(!limited))` with `fs` from the thread's
  project as the file's other writers take it. Added after `render_marley_panel_width` in the
  bottom row.
- Ledger: rows for `default.json`, `settings_content/src/agent.rs` and `thread_view.rs`
  (#798's row extended) in `docs/marley/zed-touchpoints.md`.

### File manifest
| File | Crate | Change |
|---|---|---|
| `assets/settings/default.json` | Zed | one default |
| `crates/settings_content/src/agent.rs` | Zed | doc string |
| `crates/agent_ui/src/conversation_view/thread_view.rs` | Zed | the toggle |
| `docs/marley/guide.md`, `docs/marley/zed-touchpoints.md`, `CHANGELOG.md` | — | at Complete |
| `script/e2e/822-agent-text-full-width.sh` | — | the visual check |

### Visual check plan
| REQ | The scenario | Shot |
|---|---|---|
| REQ-001, REQ-002 | A thread tab alone in its row with a long answer, no width setting. | `822-01-full` |
| REQ-003 | A click on the toggle; the run reads the settings file. | `822-02-column` |
| REQ-003 | A second click. | `822-03-full-again` |

### Risks and decisions
- Users who liked the column get full width after the update; the toggle is one click back, and
  the guide says so.
- A settings write while the file has a parse error fails in Zed's usual way (a notification).

## Phase 2 — Code
- **Built:** to the manifest. `assets/settings/default.json` sets `limit_content_width: false`
  with its `Marley:` comment; `settings_content/src/agent.rs`'s doc names both defaults;
  `thread_view.rs` gains `render_marley_text_width` (an `IconButton`, `Maximize` while the column
  is on, `Minimize` while full) called after the width button, gated on `MarleyThreadHost`, its
  click writing `agent.limit_content_width = !limited` through `update_settings_file` with the
  thread's project's `fs`. Ledger rows for the three Zed files extended.
- **Deviation:** `Cargo.lock` moves the wasmtime family 48.0.6 → 48.0.7 (`cargo update -p
  wasmtime --precise 48.0.7`, cranelift 0.135.6 → 0.135.7 with it). The first gate run went red
  on gate:7 for RUSTSEC-2026-0378 and RUSTSEC-2026-0379, published 2026-10-19 against 48.0.6;
  not this change's doing, fixed at source rather than ignored.
- **Review:** a click captures the `limited` read at render, so a click always flips what the
  icon shows. A settings file write reaches every window: the user settings watcher in
  `settings_store.rs` calls `cx.refresh_windows()`, so the thread redraws at the new width with
  no observer of its own. No entity is read or updated inside its own update; the click upgrades
  the project's weak handle and returns when it is gone. Nothing else found.
- **Gate:** `script/gates.sh --diff` GATE GREEN [diff], 17 passed (second run, after the
  wasmtime bump).

## Phase 3 — Test
- **Scenario:** `script/e2e/822-agent-text-full-width.sh`, `compositor sway`, #798's scripted ACP
  agent behind Marley's own agent; the run's settings copy has `agent.limit_content_width` taken
  out, so the default shows. Run with `just e2e`, four checks pass: the scripted agent ran, the
  setting unset, the first click wrote `true`, the second `false`.
- **Fixes to the scenario on the way, none to the source:** #798's `Home Down Down Return` in the
  + menu now lands on New Browser Tab, since #819 put New tmux Terminal second (`822-00-plus`
  shows New Agent... fourth); the scenario clicks New Agent... instead. The debug build's message
  box takes typed keys seconds after wtype sends them, so the scenario waits 12 s before Return.
  In the column the bottom row moves in with the text, so the second click has its own point
  (572, 950; the first is 327, 950).
- **Shots:**
  - `822-00-plus`: the repo row's + menu, New Agent... the fourth entry (set-up only).
  - `822-01-full` (REQ-001, REQ-002): no width setting; the prompt and the answer run from the
    rail's edge to the window's, the message box spans the tab, and the bottom row shows +, the
    follow toggle and the text-width toggle (the narrowing icon); no width button, the tab being
    alone in its row (#798).
  - `822-02-column` (REQ-003): after the click, the prompt, the answer and the message box sit in
    the centered 850 px column, the toggle (the widening icon) in the column's bottom row; the
    settings file says `true`. The first click's tooltip, "Text: a readable column", is still drawn
    by the old spot until the pointer moves: gpui's tooltip, not this change's.
  - `822-03-full-again` (REQ-003): full width again, the file says `false`.
- **Not reached:** the Agent Panel in a dock (the same `render_message_editor` row, read in the
  diff).
- **Seen for Chad:** the toggle moves when the column turns on, because the whole row moves in.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added), `docs/marley/guide.md` (#798's section gains the text
  width), `docs/marley/zed-touchpoints.md` rows for `default.json`, `settings_content/src/agent.rs`
  and `thread_view.rs` (written in Code, checked against what shipped). No Marley crate changed,
  so no per-crate note; no plan slice names #798's row.
- **Knowledge:** `L-claude-822-a-menu-entry-a-scenario-counts-to-moves-when-a-ticket-adds-one-001`.
  No F block: no bug in the source was found in Code or Test.
- **Brain:** `decisions/marleys-agent-text-runs-the-full-width-with-a-toggle-in-the-threads-bottom-row`
  (consultation `ee7fe4fa9a0b4756a1222d20619ee663`, follow-up 2026-11-03); #821's consultations
  closed with no decision, since #821 is parked.

