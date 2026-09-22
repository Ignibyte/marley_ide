---
pipeline_id: 784149f0-9731-441d-9d60-257ee8f9570d
ticket: forge#181 (67543670-020d-4a13-87e3-c95d97a9cd72)
aar_id: 1d238685-5698-4714-982f-24c8723bff12
---

# Notes — M12 #181 agent launch picker

## Phase 1 — Plan / Phase 2 — Design (folded; the palette pattern is well-established)
**Approach.** Mirror the #19 palette exactly (PaletteState + handle_palette_key + the overlay render), but:
(1) the list is a FIXED `launchable_kinds()` so move WRAPS (modulo) instead of clamping; (2) typing feeds a
separate `prompt` buffer, not a filter (no reset-on-edit). New pure module `agent_launcher.rs`.

**Prompt-delivery decision.** Primary: after `launch_agent` writes `{program}\r\n` to the new pane's shell,
if the prompt is non-empty write `send_payload(prompt)` to the SAME session — the PTY line discipline buffers
it until the exec'd agent reads stdin. This is the #72 send path applied to the just-launched pane. RISK: a
race if the shell hasn't exec'd the agent yet; the driven capture is the check. FALLBACK (if the capture shows
the prompt landing in the shell not the agent): stash a per-pane pending first-send and deliver it on the first
pump tick where `is_command_running` is true. Start with the primary; escalate only if validation fails.

**File manifest.**
- `agent_launcher.rs` (NEW, PURE) — launchable_kinds + AgentLauncherState (+ unit tests in Phase 4).
- `lib.rs` — `mod agent_launcher;` (+ pub use if needed).
- `app.rs` (SHIM) — the field; "new-agent" opens the picker; the key-router branch; handle_launcher_key
  (mutants::skip); the overlay render; `launch_agent(kind, prompt)` refactored out of the "new-agent" arm.

**Regression Test Plan.**
| Test | Proves |
|---|---|
| `launcher_move_wraps` | REQ-001 — move_down 0→1→0, move_up 0→1→0 (wrap both ways; kills +/-/% mutants) |
| `launcher_selected_kind` | REQ-001/003 — selected_kind = Claude@0, Codex@1; launchable_kinds order |
| `launcher_prompt_edits` | REQ-002 — push appends, backspace pops, kind unchanged by typing; new() = (0, "") |
| driven (typed) | REQ-004 — ⌘⇧A picker → type → Enter → agent in a split + prompt delivered |
| gate --diff | REQ-005 |

**Risk.** `launchable_kinds().len()` is a fixed 2 (never 0) so the `n-1` wrap can't underflow — documented, no
defensive guard (equivalent-mutant magnet). selected_kind indexes an invariant-safe index (move keeps it in
[0,n)). The key-router branch goes with the other overlays (before the ⌘F etc. raw-key handlers). "new-agent"
opening the picker keeps the keymap test (cmd-shift-a → "new-agent") valid.

## Phase 3 — Implement
Built to the manifest; `cargo check --workspace` clean (only the pre-existing `block v0.1.6` warning).
- `agent_launcher.rs` (NEW, PURE) — `launchable_kinds() -> &[Claude, Codex]`; `AgentLauncherState { selected,
  prompt }` with `new`/`selected`/`prompt`/wrapping `move_up`(`(sel+n-1)%n`)/`move_down`(`(sel+1)%n`)/
  `selected_kind` (invariant-safe index)/`push`/`backspace`. Unit tests co-located (see deviation).
- `lib.rs` — `mod agent_launcher;` (before agent_view).
- `app.rs` — imported `launchable_kinds` + `AgentLauncherState`; field `agent_launcher: Option<..>` (init None);
  the "new-agent" verb now OPENS the picker (`= Some(new())`) so both ⌘⇧A and the 🧠 icon open it; refactored
  the old inline launch into `launch_agent(kind, prompt)` (mutants::skip) — split + `launch_command(kind)` +
  AgentRun tag + last_agent, and (prompt non-empty) `write_bytes(send_payload(prompt))` as the first send; a
  key-router branch (`agent_launcher.is_some()` → `handle_launcher_key`, placed first among the overlays);
  `handle_launcher_key` (mutants::skip) — esc→None, enter→`take()` + `launch_agent`, ↑/↓ move, backspace +
  printable edit the prompt; the overlay render (🧠 title, the kinds list with the selected row accent-
  highlighted, + the prompt line / muted placeholder) after the palette overlay.

**Deviation:** the pure unit tests are co-located in agent_launcher.rs now (a new module reads best with its
tests) rather than deferred to Phase 4 — Phase 4 RUNS them + confirms the gate. Prompt delivery uses the
primary approach (send right after the launch write; the PTY buffers it) — driven capture is the check.

## Inspect (Phase 3.5)
2 parallel general-purpose critics (correctness/wrap-mutation/prompt-delivery + state/borrow/simplification/
security). Both ran the build + tests + `cargo mutants --list`; the launch_agent refactor was verified a
faithful behavior-preserving extraction. Findings + verdicts:

- **[HIGH] The modulo wrap left an EQUIVALENT mutant at n=2 → MSI < 100.** `move_up = (sel + n - 1) % n` with
  n=2: the `- 1`→`+ 1` mutant gives `(sel + n + 1) % n` ≡ `(sel + 1) % 2` = the original (±1 are identical mod 2).
  Both I and the correctness critic confirmed it empirically (a targeted run: MISSED). REAL ship-blocker.
  FIXED: refactored move_up/move_down to the explicit branch form (`if sel == 0 { n-1 } else { sel-1 }`), where
  every arithmetic mutant yields a wrong/out-of-bounds index a test catches. Re-ran `cargo mutants -f
  agent_launcher.rs` → 21 mutants, 19 caught, 2 unviable, **0 missed** (MSI 100). → failure-record + prevention rule.
- **[MED] Initial-prompt delivery is a type-ahead race** — launch_agent writes `{program}\r\n` then immediately
  `send_payload(prompt)`; the prompt could be dropped (agent TUI tcflush, or ZLE read-ahead) before the exec'd
  agent reads stdin. Both critics: not corruption/panic; state critic "acceptable as-is", correctness critic
  "worth a note". DECISION: keep the primary approach; RESOLVE EMPIRICALLY in Phase 4's driven capture (the
  literal check). If the capture shows a drop, escalate to the documented fallback (stash a pending first-send,
  deliver on the first pump tick where the agent is running). The `!prompt.is_empty()` guard is correct
  (send_payload("") = a bare CR). Noted in the code comment.
- **[LOW] The 🧠 mouse-click opens the picker WITHOUT the key router's overlay guards** → a stacked palette/
  finder/history underneath (both critics; pre-existing property of every top-bar dispatch, newly visible now
  that new-agent opens an overlay). FIXED (cheap hardening): the "new-agent" arm now clears palette_open/
  finder_open/history_open/find_open before opening the launcher, so the picker is the sole overlay.
- **[LOW] n=0 underflow** in the wrap / index — not reachable (fixed non-empty `&[Claude, Codex]`), documented.
  SIGN-OFF (a guard would be an equivalent-mutant magnet).
- **[LOW] `launch_command(*kind)` as the row label** — no AgentKind Display/label helper exists; DRY and correct.
  SIGN-OFF (add AgentKind::label() only if menu labels ever need to diverge from the CLI names).

Clean (verified): the launch_agent extraction (byte-identical for claude/no-prompt); state integrity (agent_launcher
only Some in new-agent, None on esc + enter-take; not persisted; no stuck state on split-fail); security (no user
input reaches spawn — the program is a fixed &'static str; the prompt goes only to a running agent's stdin, Enter
can't inject a newline); borrow soundness (take() → owned launcher); router modality (checked first, returns);
fmt exit 0; module registered in lib.rs. Post-fix: `cargo check --workspace` clean, `cargo mutants -f
agent_launcher.rs` 0 missed.

## Phase 4 — Validate
**Pure tests** (co-located in agent_launcher.rs): launchable_kinds order; new() = (0, ""); move_wraps_over_kinds
(both directions); prompt_edits_leave_kind_untouched. `cargo mutants -f agent_launcher.rs` → 21 mutants, 19
caught, 2 unviable, **0 missed** (the branch-form wrap kills what the modulo form couldn't).
**cargo nextest run --workspace**: 753 passed, 5 skipped (in-transcript).

**Driven live-app capture (REQ-004)** — all proven live:
- ⌘⇧A opens the picker overlay: "🧠 Launch agent", claude (accent-selected) + codex, "› type an initial
  prompt…" placeholder (/tmp/mly181_picker.png).
- Typing updates the prompt line: "hello181" rendered (/tmp/mly181_probe.png) — push works live.
- Enter launches the selected kind in a split: a new pane + claude process spawned (process-tree verified),
  footer "1 agent · 1 working" (/tmp/mly181_launched.png).
- The initial prompt is DELIVERED: two claude panes showing their delivered prompts — "hi" and "probe181here"
  in claude's input area (/tmp/mly181_short.png). ← the pending-delivery fix (below).

**Escalation — pending-prompt delivery (the inspect MED, resolved).** The first launch capture confirmed the
simple same-frame send DROPS the prompt (claude sat at its trust prompt with no sign of it — the TUI's startup
input-flush ate it, exactly as both critics predicted). Per the inspect decision I escalated to the documented
fallback: `launch_agent` now STASHES `(pane_id, prompt)` in a new `pending_agent_send` field instead of sending
inline; the pump delivers it via the proven #72/#174 across-grids send path once the pane's session is
`is_command_running` (1+ tick after launch, past the flush), then clears it (or drops it if the pane vanished).
Re-validated live: both "hi" and "probe181here" landed in their claude's input. **Self-review** (shim-only —
coverage-excluded, mutants-skip'd; no gate surface): borrow safety compiler-verified (read is_command_running
into a `let`, dropping the immutable grids() borrow before the mutable grids_mut() write); delivery reuses the
send-to-agent (#72/#174) pattern verbatim; the None arm drops a vanished-pane pending (no leak); cleared on
delivery (no repeat send); the is_command_running gate lands past the tcflush window (validated).

**Harness note:** the GUI driver drops chars on long type sequences + a trailing Enter (only "prob" of
"probe181here" registered in one run; the enter was lost) — a self-test timing limitation, NOT a code bug (the
picker + push clearly work). Short prompts + separate calls drove reliably; the process tree + the delivered-
prompt captures are the ground truth.

**Gate**: `scripts/gates.sh --diff` → **GATE GREEN [diff]**, 15/15. gate:4 coverage 100% (whole-workspace,
incl. the new module). gate:5 mutation reported "no mutable lines in the diff" — because agent_launcher.rs is a
NEW UNTRACKED file and `git diff HEAD` (which --in-diff reads) omits untracked files, so --diff skips it. The
module's mutation IS verified two other ways: my manual `cargo mutants -f agent_launcher.rs` (0 missed) + the
FULL whole-workspace gate at /commit. (Nuance to remember: --diff mutation does not cover a brand-new untracked
module; the full gate is the backstop.)

**Pre-existing exclusions**: none. (Restored ~/.marley/config/settings.toml from the pre-test backup.)

## Phase 5 — Complete
(pending)
