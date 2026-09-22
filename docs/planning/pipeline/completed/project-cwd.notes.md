# M9 #160 — spawn in the active project's root — Notes

- **Forge ticket:** #160 `0f7a5507-c378-4b11-a675-e5c68aea8c25` · **AAR:** `801acd3b-6101-4751-8d99-327348f4b7d6`

## Phase 1 — Plan / Phase 2 — Design (folded)
- 3 shim sites → spawn_session_in(active root); root cloned before &mut/closure use; boot sites stay.
- **AAR id:** `801acd3b-6101-4751-8d99-327348f4b7d6`.

## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 3 — Implement
- new_terminal_pane, split_focused_pane, the "new-agent" arm: spawn_session → spawn_session_in(&root, ...)
  with `let root = self.shell.active_project().root.clone()` taken BEFORE the &mut self / moved into the
  split closure. fmt; 0 err; clippy OK.

## Inspect (Phase 3.5)
Documented adversarial SELF-REVIEW (a 3-site mirror chore; the tiny-diff pattern):
- BORROW ORDER: all 3 clone the root before workspace_mut()/add_tab — no overlapping borrow; the closures
  capture the clone by move (checked: no lifetime leak past the call).
- CLOSURE CAPTURE (:2348): root joins zdotdir/cols/rows as owned captures — same shape as the existing
  locals; the FnMut is called once per split.
- MISSED SITES: grep shows exactly 2 cwd-less spawn_session callers left — the boot seed (:408) and the
  legacy-fallback (:688), both the launch-cwd project BY CONSTRUCTION (REQ-002 met). The #163 restore path
  already used spawn_session_in(root).
- SEMANTIC HONESTY: a pane spawns in the PROJECT root, not its sibling pane's live `cd` — matches the
  ticket ("cwd follows the project") and the Files/branch model; noted as intended, not a gap.
Lenses: borrow-before-&mut, closure capture, missed-site grep, semantics-vs-ticket.

## Phase 4 — Validate
- **Driven (end-to-end, REQ-001):** seeded workspace.shell with 2 projects (the Marley repo + /tmp/marley-proj-b),
  booted (#163 restored BOTH — pc_boot2.png), rail-clicked project B, ⌘T. lsof -d cwd on marley's zsh children:
  the restored B terminal 28381 AND the NEW ⌘T zsh 33091 both at /private/tmp/marley-proj-b (the boot PTY 28379
  at the Marley repo). pc_after2.png: titlebar /tmp/marley-proj-b, the prompt segment `marley-proj-b ❯`,
  terminal 2 under B. Before #160 the new tab would read the launch cwd.
- **REQ-002:** grep-clean — the only cwd-less spawn_session callers are the 2 boot seeds (launch-cwd by construction).
- **Validation detour (harness, not the app):** my first seed regex-stripped only line 1 of the app's
  MULTI-LINE `shell = """…"""` TOML value → the mangled file failed to parse → defaults → legacy boot.
  LESSON: never line-regex a TOML file that holds multi-line strings — rewrite the whole file.
- **Gate:** GREEN [diff] 15/15.

## Phase 5 — Complete
- CHANGELOG + app_shell #160 note; forge #160 → done. Sprint #21: only #96 left. LESSONS: never line-regex multi-line-string TOML; re-assert frontmost in the SAME bash as drive verbs.
