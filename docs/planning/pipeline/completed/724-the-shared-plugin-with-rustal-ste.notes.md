# The shared plugin with rustal-ste — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-724-the-shared-plugin-with-ste100.md
- **Pipeline spec:** 724-the-shared-plugin-with-rustal-ste.spec.md

## Phase 1 — Plan
- **Request:**
  - Chad, 2026-10-09: "for harness communication we should use this", then "adapt it".
  - The harness shipped the skill in its plugin (TICKET-115, `cda7c24`).
  - Its Done message: "To update Marley's copy, copy `crates/harness-runtime/src/claude/plugin/`
    from commit `cda7c24`. Keep the paths that `FILES` in `crates/harness-runtime/src/claude/plugin.rs`
    gives."
- **Recall:** #709's `shared_plugin.rs` (`FILES`, `digest`, `install_in`); its scenario pins the
  version and digest.
- **Design:**
  - Copy the files.
  - `FILES: [(&str, &str); 16]` in the harness's order.
  - `identity::marley_plugins` gives the shared plugin when it is set, else the skill's folder.
  - Docs.
  - The scenario is #709's with the new values and two more checks.
- **File manifest:** `marley_workbench` (`shared_plugin.rs`, the plugin files), `marley_terminal`
  (`identity.rs`), docs, the scenario.

## Phase 2 — Code
- **Built:**
  - **The 16 files** copied from rustal-harness `cda7c24` (`git archive`), mapped to the
    plugin's paths. A digest computed over them in the harness's order is
    `eebd8a515080bb5c301f40121fe9ba0de41cd4b5afb11eb8c7adabecb213b1d0`, the harness's.
  - **`shared_plugin::FILES`** has 16, in that order.
  - **`identity::marley_plugins`** gives the shared plugin, else the skill folder.
  - **The typos exclusion** covers the plugin's skill folder.
  - **The docs:** the guide, the HTML guide and the architecture note.
- **Review:**
  - Of the old six files, only `plugin.json` changed (the version 0.3.0); the tested Claude Code
    versions are unchanged, so `CLAUDE_SHARED_PLUGIN`'s range stays.
  - No host, address or home path is in the files.
- **Gate:** `724-gate-1.log` GATE GREEN [diff].

## Phase 3 — Test
- **Scenario:** `script/e2e/724-the-shared-plugin-with-rustal-ste.sh`, #709's with the new values,
  under `compositor sway`, `nice -n 19`. Both the shared plugin and #725's skill are on.
- **First run (`shots-724a`): every check passes.**
  - **724-01-on:** the stand-in `claude` in a new terminal prints:
    - `CLAUDE_CODE_PLUGIN_DIRS=…/claude-code/shared/eebd8a51…b1d0`, one folder only;
    - `plugin: rustal-harness 0.3.0` (REQ-001);
    - `folder: eebd8a515080bb5c301f40121fe9ba0de41cd4b5afb11eb8c7adabecb213b1d0`, the harness's digest
      (REQ-001);
    - the modes 400/400/700;
    - `skill: present` (REQ-001);
    - `rustal-ste folders: 0`: #725's folder stepped aside (REQ-002);
    - `report waiting: exit 0`, and the rail's Needs you shows "Claude Code · repo, Waits for you".
- Chad's Hyprland untouched.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Changed); the guide, the HTML guide and the architecture note
  (Phase 2).
- **Knowledge:** none new.
- **Ticket:** closed.
- **Gate:** `724-gate-2.log`, GATE GREEN [diff], on the tree committed.
