# dock-state persistence + M2.F finale — Notes

- **Forge ticket:** #95 `8d2c5397-6c55-4586-844f-5fc7b94e1deb` · **AAR:** (below)
- **Local ticket doc:** docs/planning/tickets/open/TICKET-095-dock-persistence.md

## Phase 1 — Plan
- **Request:** forge #95 (M2.F 6/6, THE FINALE) — persist + restore the right dock's active tab.
- **Pre-flight (the #26/#87 settings pattern):** settings.rs has define_setting! (theme/docks/term/remote) +
  AppliedSettings + applied_from (pure) + applied_defaults + persist_* (manager.set); app.rs boots via
  applied_from (~256) + keeps `settings: Option<SettingsManager>` for writes + persists on change (persist_
  theme ~1300). The #90 tab click is at ~1731 (`view.right_section = section`).
- **Decisions:** D1 lowercase key + unknown→Details; D2 round-trip; D3 best-effort persist.
- **AAR id:** (recorded below).

## Phase 2 — Design
- (pending)
## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 2 — Design
- **right_dock.rs (PURE):** `right_section_key(RightSection)->&str` (Details→"details"/Agents→"agents"/Forge→"forge") + `right_section_from_key(&str)->RightSection` (match "agents"→Agents/"forge"→Forge/"details"|_→Details).
- **settings.rs:** `define_setting!(RightSectionSetting: String = String::from("details"), "cockpit.right_section")`; import RightSection + right_section_key/from_key from crate::right_dock; AppliedSettings gains `right_section: RightSection`; applied_from adds `right_section: right_section_from_key(&manager.get::<RightSectionSetting>())`; applied_defaults adds `right_section: RightSection::Details`; add `persist_right_section(manager, RightSection) -> Result` = `manager.set::<RightSectionSetting>(right_section_key(section).to_string())`. Update the 2 AppliedSettings test literals (applied_from_reads_saved_values + applied_defaults) with `right_section: RightSection::Details`.
- **app.rs:** the import adds persist_right_section; new() boot `right_section: applied.right_section` (replace the hardcoded Details); the #90 tab on_mouse_down after `view.right_section = section` adds `if let Some(m)=view.settings.as_mut() { let _ = persist_right_section(m, section); }`.
- **Mutation targets:** right_section_key arms, right_section_from_key arms + the unknown→Details fallback; applied_from right_section resolution.
- **Test plan:** right_dock `right_section_round_trip` (each key↔enum identity; "agents"→Agents; "xyz"/""→Details); settings `applied_from` gains a cockpit.right_section assertion (a toml with cockpit.right_section="agents" → right_section Agents) + the fixture default updates. cov/MSI 100.
- **Risks:** persist is best-effort (#26/#87 — a write error is swallowed, never crashes); a stale/hand-edited unknown key boots Details (tolerant, like #87 theme).

## Phase 3 — Implement
- **Built:** right_section_key/right_section_from_key (right_dock.rs, round-trip + unknown→Details); RightSectionSetting (cockpit.right_section) + AppliedSettings.right_section + applied_from resolution + applied_defaults(Details) + persist_right_section (settings.rs); app.rs boots `right_section: applied.right_section` + persists on the #90 tab click (best-effort via self.settings). Updated the 3 AppliedSettings test literals; the applied_from_reads_saved_values toml gained cockpit.right_section="agents" → asserts Agents (kills the resolution mutant).
- **Verification:** fmt; check --all-targets 0 err; clippy OK; all 5 tests pass (round-trip + 4 applied).

## Phase 3.5 — Inspect
- **Method:** self-review (a pure key round-trip + a settings field following the proven #26/#87 persist pattern verbatim).
- **Lenses — no findings:** right_section_key (3 arms) + right_section_from_key (agents/forge/_→Details) round-trip proven for all sections + unknown/""→Details; applied_from resolves right_section (the "agents" toml → Agents assertion kills an always-Details mutant); applied_defaults + the load-error path use Details; persist_right_section = manager.set (mirrors persist_theme/dock #26); the boot `applied.right_section` + the click-persist are best-effort (the settings write is swallowed — never crashes, the #26/#87 rule); the 3 fixtures updated. No panics. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** right_section_key_round_trip (each key↔enum + "agents"→Agents + unknown/""→Details) [right_dock]; applied_from_reads_saved_values now asserts a saved cockpit.right_section="agents" → right_section Agents; the 2 default fixtures → Details. `cargo nextest` → 5 passed.
- **LIVE boot-load self-test (PROVEN end-to-end):** backed up chad's ~/.marley/config/settings.toml, wrote `[cockpit] right_section = "agents"`, rebuilt + `open` + screencapture (WIN 13826) → READ the PNG: the right dock BOOTED ON the **Agents tab** (accent-highlighted) showing "no agents — ⌘⇧A to launch" (the #91 hint), footer "no sprint · no agents · focus: terminal" — the persisted tab was restored on launch. Then RESTORED chad's config. Capture: scratchpad/boot95_agents.png. (The click-persist mirrors the proven #26/#87 pattern; a synthetic tab-click to write is env-blocked but the round-trip + boot-load are engine + live proven.)
- **Gate:** GREEN [diff] 15/15, cov/MSI 100 (after the persist_right_section coverage fix).
- **Gate finding + fix:** the first gate run was RED (MSI 80%, settings.rs 96.7% cov) — persist_right_section was UNCOVERED (its delete-mutant survived). Fixed at source: settings_round_trip_survives_reload now persists RightSection::Agents + asserts it reloads → covers persist_right_section + kills the mutant. Re-gate → **GREEN [diff] 15/15, cov/MSI 100** (5 caught / 0 missed).

## Phase 5 — Complete
- CHANGELOG ### Added; aar-submit(5); forge #95 → done. **M2.F 6/6 — CLOSES M2.F.** right_section_key/from_key + RightSectionSetting + persist_right_section (cov/MSI 100). **LIVE BOOT-LOAD PROVEN** (boot95_agents.png: wrote cockpit.right_section=agents → the dock booted on the Agents tab). Gate caught persist_right_section uncovered → fixed via the round-trip test.
