# cmd-P finder Enter inserts into the cooked buffer — Notes

- **Forge ticket:** #65 `e410d50c-a919-4544-b75c-50af2feb865e`
- **AAR:** `fc267cda-016c-4d30-bd64-8c46542eaad4`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-065-finder-cooked-buffer.md

## Phase 1 — Plan
- **Request:** forge #65 (M2.C, auto-approved) — the finder-Enter cooked-buffer bug (a #59 follow-up).
- **Classification:** work pipeline, `bug`, SHIM-ONLY (app.rs handle_finder_key). No new pure surface. UI.
- **Confirmed (grep'd):** handle_finder_key Enter does `state.session.write_bytes(&bytes)` (the #59 bug);
  the #59 file-click already uses `state.buffer.edit(caret..caret, …, Human)` + caret advance (app.rs:1058)
  — the pattern to mirror.
- **Decisions:** D1 buffer.edit not write_bytes; D2 mirror #59; D3 no unit tests, the self-test is the
  proof (+ #65 finally makes the finder Enter self-testable — Enter is a keycode).
- **AAR id:** `fc267cda-016c-4d30-bd64-8c46542eaad4`.

## Phase 2 — Design

### SHIM — `app.rs` `handle_finder_key` Enter (726–737), replace verbatim
```rust
            "enter" => {
                let text = {
                    let results = self.finder.results(&strs);
                    self.finder
                        .chosen(&self.project_files, &results)
                        .map(|path| path.to_string_lossy().into_owned())
                };
                if let Some(text) = text {
                    if let Some(state) = self.workspace.focused_state_mut() {
                        // Insert into the COOKED prompt buffer (#65/#59), NOT the PTY — a raw write
                        // reaches the shell's ZLE but not Marley's rendered line.
                        state
                            .buffer
                            .edit(state.caret..state.caret, &text, EditOrigin::Human);
                        state.caret =
                            CharOffset::from(state.caret.as_usize() + text.chars().count());
                    }
                }
                self.finder_open = false;
            }
```
Only the middle changes: `into_bytes()`→owned String; `write_bytes(&bytes)`→`buffer.edit` + caret. Same
`chosen(&project_files, &results)` selection, same `finder_open = false`. `EditOrigin`/`CharOffset` are
already imported (used by #59/#60).

### File manifest
- MODIFY `crates/marley_app/src/app.rs` — the one `handle_finder_key` Enter arm. (No other files; no new pure surface.)

### Mutation / coverage
- ZERO new pure lines. `handle_finder_key` is `mutants::skip` + app.rs is cov-excluded → the gate has
  nothing new to mutate/cover. `buffer.edit`/`CharOffset` are marley_editor-tested (existing).

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | cmd-P → Enter → the top file's path APPEARS at the prompt (visible cooked-buffer insert) | self-test (drive cmd-P + Enter → capture) |
| REQ-002 | `cargo nextest --workspace` green (no regression) + gate GREEN | gate |

Uncoverable: `handle_finder_key` (masked, live loop) — proven by REQ-001. NO new unit tests (SHIM-only bug
fix; the decision reuses marley_editor's tested buffer.edit/CharOffset). This ticket finally makes the
finder Enter self-test-verifiable — Enter is a keycode (unlike #57's type-to-filter, which needed key_char).

### Risks / decisions
- D-2.1 mirrors #59's file-click EXACTLY (buffer.edit at the caret + advance by char count) — the
  established, self-test-proven pattern. D-2.2 the finder's `results`/`chosen` selection is unchanged (an
  empty query ranks all files, top selected → Enter inserts that). D-2.3 no run-on-Enter (inserts like
  #59/#60 — you edit/send it yourself).

## Phase 3 — Implement
- **Built (SHIM, app.rs — masked):** the `handle_finder_key` Enter now inserts the chosen path via
  `state.buffer.edit(caret..caret, &text, EditOrigin::Human)` + `state.caret` advance (the #59 cooked-
  buffer pattern), replacing the invisible `session.write_bytes(&bytes)`. Same selection + close.
- **Deviations:** none — mirrors #59's file-click verbatim. No new pure surface.
- **Verification:** `cargo fmt`; `cargo check -p marley` 0 err; clippy `-D warnings` OK; `cargo nextest
  -p marley` 132 pass (no regression). Zero new pure lines → the gate has nothing new to cover/mutate.

## Phase 3.5 — Inspect
- **Critic:** 1 (diff read + a 4-site pattern compare + a write_bytes grep + a masking check). Verdict:
  **PASS — ship.**
- **Confirmations:** (a) mirrors #59/#60 EXACTLY — 4 sites now share `buffer.edit(caret..caret, &X, Human)`
  + `caret = CharOffset::from(caret.as_usize() + X.chars().count())` (history #60, finder #65, paste,
  file-click #59); the advance is `chars().count()` (CHARS, multibyte-safe), not len(). (b) NO write_bytes
  remains in `handle_finder_key` (write_bytes survives only legitimately: ctrl-L redraw, agent-launch #62,
  paste-while-running, encode_key typing). (c) no panic (all Option-total: text/focused_state_mut →
  no-op on None); `caret..caret` = insert-at-cursor; EditOrigin::Human matches the siblings. (d) fully
  masked — handle_finder_key carries mutants::skip + app.rs cov-excluded → ZERO gate:5/cov exposure.
- **Finding:**
  | # | Sev | Finding | Action |
  |---|---|---|---|
  | L1 | LOW (out of scope) | The finder (+ #59 file-click) insert UNCONDITIONALLY into the cooked buffer, whereas PASTE guards on `is_command_running()` (writes to the PTY when a program runs). So cmd-P-picking a file while vim/a REPL runs lands the path in the cooked prompt, not the running program. | Accept — this is #59's SAME accepted behavior (the ticket says "mirror #59 exactly"), NOT introduced here. A future ticket could guard the finder/file-click inserts on is_command_running like paste. Noted for chad. |
- **No code change** — SHIP as-is.

## Phase 4 — Validate
- **Tests:** NONE new (SHIM-only bug fix; the decision reuses marley_editor's tested buffer.edit/CharOffset).
- **No-regression (actual):** `cargo nextest run -p marley` → 132 passed (incl. marley's integration tests).
- **SELF-TEST (UI — REQ-001, drove the LIVE app — CLOSES the #57 gap):** cmd-P → the finder (top =
  `.cargo/audit.toml`) → Enter → the prompt now shows **`❯ Marley .cargo/audit.toml`** (the path INSERTED
  into the cooked buffer, visible + editable) (`scratchpad/finder_enter.png`) — vs the old invisible
  write_bytes. First visual verification of the finder Enter (Enter is a keycode, unlike #57's key_char
  type-to-filter).
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, cov 100%, MSI 100%. app.rs masked (0 new pure lines).
- **Pre-existing:** none.

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- **Docs:** CHANGELOG `### Fixed`.
- **Knowledge:** aar-submit (5); reused PR-claude-insert-at-prompt-goes-to-cooked-buffer (the #59 lesson). Filed follow-up #71 (guard finder/file-click inserts on is_command_running like paste).
- **Ticket:** forge #65 → done; archived. **6/6 of M2.C — SPRINT COMPLETE.**
