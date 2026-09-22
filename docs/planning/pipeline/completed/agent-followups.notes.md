# M12 #182 — #174 follow-ups — Notes

- **Forge ticket:** #182 `0dd1b3d4-f985-4085-b4e7-0adf59e190ba` · **AAR:** `083fab37-5311-4eaa-a577-16b49a4e1938`

## Phase 1 — Plan / Phase 2 — Design (folded)
- pure pane_project_root over the tested locate_pane; git_working_diff_in(root) extracted; the Agents ⌘-click
  diffs the agent's project (fallback active); the send arm flashes on a dead-agent write fail.

## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 3 — Implement
- tabs.rs: Workspace::pane_project_root(pane) -> Option<&Path> over locate_pane → projects[p].root.
- app.rs: git_working_diff() → git_working_diff_in(&self.project_root); the new git_working_diff_in(root)
  (masked shim, spawns git). The Agents ⌘-click resolves pane_project_root(pane) (fallback active root),
  diffs THERE, flashes "{project} diff". The send-to-agent arm gained an else: on a dead-agent write fail,
  flash "{label} has exited" (label from self.agents, fallback "agent"); the compose line stays (#72).
- fmt; 0 err; clippy OK.

## Inspect (Phase 3.5)
Self-review (a small #174-follow-up over the tested locate_pane; 1 pure fn + 3 shim edits):
- ROOT FALLBACK: pane_project_root None (an impossible unknown pane, since the row's pane came from
  agent_rows over the live map) → the active project_root — a safe default, never a panic/empty path.
- LABEL LOOKUP: the dead-flash reads self.agents.get(&target) — target IS an agents-map key (the send arm
  only runs with Some(last_agent)), so the label is present; the fallback "agent" covers a race where the
  entry was just removed.
- NO REGRESSION: git_working_diff() is byte-equivalent (delegates to _in(project_root)); the git-panel
  file-row diff + ⌘⇧D still call git_working_diff() (active project — correct, untouched).
- THE DEAD-FLASH TRIGGER: `sent=false` fires the else for BOTH a missing target (no grid holds it) AND a
  write failure (a dead pane) — both are "the agent isn't reachable", so one honest flash is right; the
  compose line is preserved in every not-sent path (#72 upheld).
Lenses: fallback safety, key-presence, delegation equivalence, not-sent honesty.

## Phase 4 — Validate
- **Tests:** pane_project_root (extended grids_and_locate_pane_across_projects): a pane in project 0 → /tmp/p0,
  a pane in project 1 → /tmp/p1 (cross-project, NOT the active one), an unknown pane → None. Full suite green.
- **Self-test (REQ-002, self-referential):** clean boot, ⌘⇧A launched a claude agent, opened the Agents
  cockpit tab (the fleet row `○ claude (waiting…)`), ⌘-clicked the row → the working-tree DIFF overlay
  rendered the AGENT's project (the Marley repo) — showing THIS ticket's own uncommitted changes:
  `git_working_diff_in(&self.project_root)` and the new `git_working_diff_in` doc comment ("… passes the
  … root here so it shows the agent's repo, not the active project's") are visible IN the diff
  (af_diff_crop.png). The ⌘-click → git_working_diff_in(pane_project_root) path works.
- **REQ-003 (dead-agent flash):** the else-branch flashes "{label} has exited" on any not-sent (target
  missing OR write fail) while keeping the compose line (#72). Reproducing a dead-BUT-KEPT agent live is a
  narrow stage (the agent must be its grid's SOLE pane AND its shell must exit, then ⌘⇧S — ⌘⇧A splits, so
  the agent is never the last pane) — validated by review: the branch mirrors the established Flash pattern
  and reads the label from the live agents map (fallback "agent"); no new pure surface. Honestly noted.
- **Gate:** GREEN [diff] 15/15, MSI 100.

## Phase 5 — Complete
- CHANGELOG (Fixed) + app_shell #182 note (+ the missing #179 entry); forge #182 → done. **M12 2/10.** LESSON: a feature that reads the working tree makes its OWN in-flight diff the deterministic driven test content (self-referential proof).
