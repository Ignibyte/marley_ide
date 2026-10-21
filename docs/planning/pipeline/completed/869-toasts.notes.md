# Toasts as the Inbox board draws them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-869-toasts-and-prompts.md
- **Pipeline spec:** 869-toasts.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #840; split again while planning (prompts → #870).
- **Classification / tier:** feature, S. One Zed hunk (`workspace/src/notifications.rs`), a Marley
  module.
- **Recall (§18.3):** the hook pattern (#834, #838, #839). The brain: nothing on this seam.
- **Discovery:** `show_toast` (notifications.rs:196), `MessageNotification` (`content_icon`,
  `secondary_content`, render at :1019); `marley: toggle dictation`'s toast (voice.rs).
- **Decisions:** D1, D2 in the spec.

### Design
- **Zed hunk:** `pub struct MarleyToastLook(pub fn(&str, &App) -> Option<MarleyToast>)` and
  `pub struct MarleyToast { pub title: SharedString, pub line: Option<SharedString>, pub dot:
  Color }`; `show_toast` asks it, and when it answers builds the notification from the title,
  `secondary_content(line)` and `content_icon(IconName::Circle, dot)`. Ledger row.
- **Marley `toasts.rs`:** `init` sets the hook; `look(message, cx)`: none outside the Marley layout;
  split at the first ". " or ": " (the earlier of the two), trimming a final period; red for "Could
  not", "failed", "Failed", the accent otherwise.
- **File manifest:** `crates/workspace/src/notifications.rs` (Zed), `toasts.rs` (new),
  `marley_workbench.rs`; the scenario.

## Phase 2 — Code
- **Paused 2026-10-21 ~13:55 at Chad's word, before any code; resumed after the reboot.**
- **Built:** the hunk in `show_toast` (`MarleyToastLook`, `MarleyToast`), asked before the
  notification's closure; `toasts.rs` (`init`, `look`, `split`); `toasts::init` in `init`.
- **Deviation:** the split skips a ". " or ": " whose title would be one word, so `commands.rs`'s
  `marley: {message}` keeps `marley:` in its title rather than titling the toast "marley"; the line
  starts with a capital ("the Marley layout places…" → "The Marley layout places…").
- **Review:** no entity is read during its update (the hook is asked with `&App` before
  `show_notification`); the Zed hunk is additive, its row written first. `No worktree agent: …`
  keeps the accent dot: D2's words do not name it, and it reads as a state, not a failure.
- **Gate:** red once (clippy `option_if_let_else` in `split`, rewritten as a `map`), then green,
  17 of 17 (`just gate-diff`), the scenario in the tree.

## Phase 3 — Test
- **Scenario:** `script/e2e/869-toasts.sh` (sway): Voice off, `marley: toggle dictation`, then
  `workspace: use classic layout`; both toasts stay (neither autohides), so one shot holds both.
- `869-01-night`: two toasts at the bottom right, each an orange dot, a title ("Dictation is off";
  "Panel Layout presets belong to Zed's layout") over a muted line ("Turn it on in the Voice
  section of the Marley settings."; "The Marley layout places its own panels."), the second with
  its Use Zed's Layout button. REQ-001 met.
- `869-02-day`: the same in Marley Day, the dots the Day accent, the lines muted. REQ-001 met.
- Not reached: a failure's red dot (no "Could not" toast can be raised on demand in a scratch
  profile); `look` picks it by the same match.
- **Focus report:** nothing else changed; no fix needed.


## Phase 4 — Complete
- CHANGELOG (Added), `marley_workbench.md` (Toasts), look-plan W3.4 status, the ledger row checked.
- Knowledge: AD-claude-869-toasts-split-one-string-001. Brain: `brain decide` on consultation 455d7f18.
- Ticket closed; pipeline archived.
