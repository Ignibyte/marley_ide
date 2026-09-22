# Live theme picker — Notes

- **Forge ticket:** #199 (2223ad76-e2f3-4f93-92e6-ff1ec27905c8)
- **AAR:** fd3b6cad-19c8-40d3-90fe-69be0de38cb4
- **Local ticket doc:** docs/planning/tickets/open/TICKET-199-warp-theme-picker.md
- **Pipeline spec:** warp-theme-picker.spec.md

## Phase 1 — Plan
- **Request:** a live theme picker in the palette. Auto-approved (/work 195-222, M12.2 polish).
- **Classification / tier:** work pipeline, small. Systems: app.rs palette command registration + dispatch
  (shim) + a small pure id↔index helper. Reuses themes.rs (registry) + set_theme + settings.rs persist.
- **Forge recall (§18.3):** #25 built `set_theme`/`toggle-theme`; #87 built the `CONNECT_BASE` dynamic
  palette-command pattern (the exact model); the persist/load round-trip (settings.rs) is tested. The
  id-arithmetic exact-value rule applies (a `checked_sub` index math needs boundary tests). AAR opened.
- **Discovery (code read — MOST INFRA EXISTS):**
  - themes.rs: `ThemeRegistry::builtin()` (Dark+Light), `themes() -> &[Theme]`, `by_name`, `default_for`,
    `dark_default`. `Theme{name, appearance, colors}`.
  - app.rs: `set_theme(&mut self, theme, cx)` sets `self.theme` + `persist_theme(manager, name)` (applies
    live — the render reads `self.theme.colors` each frame — AND persists). `active_theme()`. A
    `toggle-theme` cockpit command (#25) flips Dark↔Light; NO per-theme picker.
  - settings.rs: `theme_name: String`; `persist_theme(manager, name)` + boot-load validates the saved name
    vs the registry (unknown/missing → default). Round-trip DONE + tested.
  - The commands are assembled in the constructor (~666): `let mut commands = cockpit_commands();` then a
    loop pushing `Command{id: CommandId(CONNECT_BASE + i), title: action.label, keywords:[connect,ssh,
    remote], binding: None}` per remote host. `handle_palette_key` (~1267-1284) dispatches: `activate` →
    the selected `CommandId`; a `CONNECT_BASE`-range id → `id.0.checked_sub(CONNECT_BASE)` → the i-th remote
    action; else `action_for_command` (the static cockpit verbs). CONNECT_BASE = 1000.
- **The delta:** add a `THEME_BASE` (2000) block; a loop pushing `Command{id: CommandId(THEME_BASE+i),
  title: format!("Theme: {}", theme.name), keywords:[theme,color,appearance], binding: None}` per
  `registry.themes()`; a dispatch branch resolving a `THEME_BASE`-range id → the i-th theme → `set_theme`;
  a pure `theme_pick_index(id, base, count) -> Option<usize>` for the id↔index (exact-value tested).
- **Deferred:** live preview on highlight (optional); custom-theme editor; new themes.
- **Decisions:** D1 THEME_BASE=2000 above CONNECT_BASE=1000 (no realistic collision — confirm at design);
  D2 apply via existing set_theme; D3 a pure id↔index helper (cov/MSI 100); D4 auto-approved, document w/
  a live theme-switch capture.
- **Open questions for Design:** (1) the pure helper signature — `theme_pick_index(id: u32, base: u32,
  count: usize) -> Option<usize>` (in-range 0..count → Some, below base or ≥ base+count → None) vs reusing
  a generic `pick_index`; is the connect dispatch inline `checked_sub` or a helper? (mirror it). (2) where
  the registry is reachable at dispatch time (a `ThemeRegistry::builtin()` is cheap to rebuild, or is it
  stored on self?) — set_theme takes a `&Theme`, so resolve `registry.themes().get(i)`. (3) do the theme
  commands need distinct keywords so "theme"/"color" filters find them (yes — add keywords). (4) confirm
  the dispatch order: THEME_BASE-range checked BEFORE the static `action_for_command` fallback (like
  connect), so a theme id doesn't fall through.

## Phase 2 — Design

### Discovery verified
- **The exact connect dispatch** (`handle_palette_key`, `#[cfg_attr(test, mutants::skip)]`, app.rs 1264-1281):
  `enter` → `self.palette.activate(&filter_commands(&self.commands, query))` returns `Option<CommandId>`;
  then `if let Some(action) = action_for_command(id) { dispatch_action } else if let Some(target) =
  id.0.checked_sub(CONNECT_BASE).and_then(|i| self.remote_actions.get(i as usize)).map(|a| a.target.clone())
  { open_remote_target }`; then `self.palette_open = false`. `remote_actions` is stored on `self`. The
  connect index-math is INLINE (`checked_sub` + `.get`), untested (in the skip'd shim). → the THEME branch
  mirrors this but routes the id-math through a PURE helper (gives it a mutation home).
- **No id collision:** `CONNECT_BASE=1000` → connect ids 1000..1000+N_hosts; static cockpit ids 0/1/2/4/5;
  `THEME_BASE=2000` → theme ids 2000..2002 (2 builtin themes). N_hosts would need >1000 to reach 2000 —
  impossible. 2000 is clear.
- **`Command { id: CommandId, title: String, keywords: Vec<String>, binding: Option<KeyBinding> }`** (palette.rs:16)
  — the registration loop uses these. `binding: None` (theme commands have no chord → no keycap chip, fine).
- **palette.rs** is the pure module (`filter_commands`, `action_for_command`, `#[cfg(test)] mod tests` at 141)
  — the home for the new `theme_pick_index`, gpui-free, cov/MSI 100.
- **`set_theme(&mut self, theme: &Theme, cx)`** (app.rs:933) applies live + persists — callable from the
  handler. **`ThemeRegistry::builtin()`** is the source at BOTH the constructor (registration) + dispatch
  (resolution) → the index order aligns (same `themes()` order both times).

### Architecture / approach
- **PURE seam** (palette.rs, cov/MSI 100): `pub fn theme_pick_index(id: CommandId, base: u32, count: usize)
  -> Option<usize>` = `id.0.checked_sub(base).map(|i| i as usize).filter(|&i| i < count)` — Some(i) for an
  in-range id, None below base (`checked_sub` None) or ≥ base+count. Takes `CommandId` (palette.rs owns it)
  so it reads naturally at the call site. Exact-value tested. (Kept theme-specific rather than a generic
  `pick_index` to avoid touching the connect path — bounded; a future refactor could route connect through
  it too.)
- **SHIM** (app.rs, both in the `mutants::skip` paths):
  - constructor (after the connect loop ~679): `let theme_registry = ThemeRegistry::builtin(); for (i, theme)
    in theme_registry.themes().iter().enumerate() { commands.push(Command { id: CommandId(THEME_BASE + i as
    u32), title: format!("Theme: {}", theme.name), keywords: vec!["theme".into(), "color".into(),
    "appearance".into()], binding: None }); }`
  - dispatch (`handle_palette_key` enter, a new `else if` AFTER the connect branch): `else if let Some(index)
    = theme_pick_index(id, THEME_BASE, ThemeRegistry::builtin().themes().len()) { if let Some(theme) =
    ThemeRegistry::builtin().themes().get(index).cloned() { self.set_theme(&theme, cx); } }`. `set_theme`
    applies + persists. Then `palette_open = false` (already there).
- No new apply/persist logic; the registry + set_theme + persist_theme round-trip are reused.

### File manifest
- `crates/marley_app/src/palette.rs` — ADD `pub fn theme_pick_index(id: CommandId, base: u32, count: usize)
  -> Option<usize>` (pure; tests at validate).
- `crates/marley_app/src/app.rs` — (a) `const THEME_BASE: u32 = 2000;` (near `CONNECT_BASE`); (b) the
  theme command-registration loop (after the connect loop); (c) the dispatch `else if` branch (after the
  connect branch); (d) import `theme_pick_index` + confirm `ThemeRegistry`/`Theme` are in scope (they are —
  `set_theme` uses `Theme`).

### Regression Test Plan
| REQ | test |
|---|---|
| REQ-004 | palette.rs `theme_pick_index_maps_range` — exact: `(CommandId(2000),2000,2)`→Some(0); `(2001,2000,2)`→Some(1); `(1999,2000,2)`→None (below base, checked_sub None); `(2002,2000,2)`→None (≥ base+count); `(2000,2000,0)`→None (empty registry). Kills `checked_sub`/base, the `< count` filter (`<`→`<=` dies on (2002,2000,2)), the +offset. |
| REQ-001 | driven capture — ⌘⇧P → type "theme" → the palette lists "Theme: Marley Dark" + "Theme: Marley Light". |
| REQ-002 | driven capture — select "Theme: Marley Light" → the cockpit switches to light colors live. |
| REQ-003 | review — the dispatch calls the EXISTING `set_theme` (which calls `persist_theme`); the persist/load round-trip is already tested (settings.rs). (A full relaunch capture is optional — the round-trip is engine-proven.) |
| REQ-005 | review + capture — the static cockpit commands + the `CONNECT_BASE` connect commands + `set_theme` are unchanged; the theme commands are additive in a distinct id range. |
- **Uncoverable by unit test:** the shim registration + dispatch (`mutants::skip`) — validated by the driven
  capture. The pure `theme_pick_index` carries the id-math mutation load; `set_theme`/`persist_theme`/the
  registry are already tested.

### Risks / decisions
- **R1 — index alignment** — the registration loop + the dispatch BOTH iterate `ThemeRegistry::builtin()
  .themes()` (a deterministic builtin order: Dark, Light), so `THEME_BASE + i` at registration resolves to
  the same `themes()[i]` at dispatch. If a future theme source becomes dynamic/user-ordered, this must stay
  a single ordered source — noted.
- **R2 — dispatch branch order** — the THEME `else if` comes AFTER the CONNECT `else if`, and both after the
  `action_for_command` static-verb branch. The id ranges are disjoint (0/1/2/4/5 < 1000 < 2000), so order
  doesn't affect correctness, but the explicit `else if` chain keeps it clear.
- **R3 — `ThemeRegistry::builtin()` rebuilt at dispatch** — cheap (2 structs); avoids storing a registry on
  self. Acceptable (matches the stateless-lookup style).
- **R4 — theme command keywords** — `["theme","color","appearance"]` so a "theme"/"color" query filters
  them; the title "Theme: <name>" also matches "theme". Confirmed via `filter_commands` (matches title +
  keywords).

## Phase 3 — Implement
- **palette.rs** — added `pub fn theme_pick_index(id: CommandId, base: u32, count: usize) -> Option<usize>`
  (`id.0.checked_sub(base).map(|i| i as usize).filter(|&i| i < count)`) after `action_for_command`. Pure,
  gpui-free.
- **app.rs** — (a) `const THEME_BASE: u32 = 2000;` after `CONNECT_BASE`; (b) added `theme_pick_index` to the
  `use crate::palette::{…}` import; (c) the theme command-registration loop after the connect loop (~680):
  one `Command{ id: CommandId(THEME_BASE+i), title: "Theme: {name}", keywords:[theme,color,appearance],
  binding: None }` per `ThemeRegistry::builtin().themes()`; (d) the dispatch `else if` after the connect
  branch (~1281): `theme_pick_index(id, THEME_BASE, …len()) → get(index).cloned() → self.set_theme(&theme,
  cx)`. `ThemeRegistry`/`Theme` were already imported (line 95).
- **Deviations from design:** none. (Theme-specific `theme_pick_index` as designed; the connect inline stays.)
- `cargo fmt` + `cargo check -p marley` clean (only the pre-existing transitive `block v0.1.6` note).

## Inspect (Phase 3.5)
1 focused critic (pure-fn mutation / id-collision / index-alignment / scope) + my own trace. **1 real
test-plan gap found + closed** (a surviving mutant the planned matrix missed); rest clean.

- **[my inspect trace vs the critic's tool run — accuracy correction] I added a `(2003)` case for a
  suspected `<`→`!=` survivor; the critic RAN `cargo mutants --list` (27.1.0) and the actual mutant set is
  `< → {==, >, <=}` + a whole-body `{None, Some(0), Some(1)}` — NO `!=`, NO `>=`.** So the `<`→`!=` mutant I
  worried about is a PHANTOM (never generated); my `(2003,2000,2)→None` case is harmless but not strictly
  required. The REAL survivors the planned 5-case matrix must kill: the body-`Some(1)` mutant (killed by
  (2000,2000,2)→Some(0), the only Some(0) case) + `<`→`<=` (killed by (2002)→i==count / (2000,0)) + `<`→`>`
  and `<`→`==` (killed by the Some cases). The committed 6-case matrix (incl (2003)) kills ALL 6 actual
  mutants → MSI 100 (gate-confirmed green). Takeaway (again): trace the REAL `cargo mutants --list` set, don't
  guess the operator mutations — but keeping an extra boundary case never hurts.
- **VERIFIED — id collision / dispatch order (the KEY check).** A theme id (2000/2001) flows the dispatch
  chain: (a) `action_for_command` maps only 0/1/2/4/5/6/7/8 → None; (b) the CONNECT branch:
  `2000.checked_sub(1000)=Some(1000)` → `self.remote_actions.get(1000)`. `remote_actions` = the saved-host
  list (`remote_palette_actions(&applied.remote_hosts)`, realistically 0-10 entries) → `.get(1000)` = None →
  `.and_then` short-circuits → the connect `if let Some(target)` is false → falls to (c) the THEME branch.
  So a theme id is NEVER swallowed by connect (would require 1001+ hosts — impossible). The id ranges are
  disjoint (statics <9 · connect 1000+i · theme 2000+i). SAFE. (Implicit assumption <1000 hosts —
  documented; `THEME_BASE=2000` leaves a 999-host gap.)
- **VERIFIED — index alignment.** `ThemeRegistry::builtin()` is a DETERMINISTIC literal `from_themes(vec![
  Light, Dark])` — a fixed Vec order, identical at registration + dispatch. So `THEME_BASE+i` registers to
  `themes()[i]` which the dispatch resolves to the same `themes()[i]`. (NB the order is [Light, Dark] →
  THEME_BASE+0 = "Marley Light", +1 = "Marley Dark" — both correct; noted for the capture.)
- **VERIFIED — set_theme reuse + scope.** The diff ADDS only: the pure fn (palette.rs); the const, import,
  registration loop, dispatch else-if (app.rs). It does NOT touch `action_for_command`/`cockpit_commands`/
  the connect loop+branch/`set_theme`/`persist_theme` (grep of the diff's ±lines confirms). The dispatch
  reuses the EXISTING `set_theme` (which calls `persist_theme`) → REQ-003 (persistence) leans on the already-
  tested settings round-trip. Tokens/themes reused — no new hsla; clean-room.

**Critic's two LOWs (no action):** (1) a THEORETICAL id-collision at ≥1001 configured remote hosts (connect
id 2000 would then equal THEME_BASE) — purely hypothetical for hand-written `[[remote.hosts]]` TOML (<10
realistic); the THEME_BASE doc-comment already encodes the <1000-host invariant. No fix. (2) the dispatch
builds `ThemeRegistry::builtin()` twice (once for the count, once for `.get`) + the inner `if let Some(theme)`
is provably-Some (index already bounded) — harmless belt-and-suspenders on a cold path (a rare user action);
the `.cloned()` is correctly required to not borrow the temporary across `set_theme`. No change.

**Verdict:** no code fix needed — the implementation is correct (critic: no HIGH/MED). The `(2003)` case I
added is harmless (the `!=` mutant it targets isn't generated); the committed matrix is MSI-100 (gate-green).
No forge failure-record (no shipped bug). Lenses: mutation-resistance (critic ran the real mutant list),
id-collision/dispatch-order (the KEY check — `get(1000)`=None), index-alignment (deterministic Vec), scope/
clean-room (0 new app.rs mutants — the burden is on the pure `theme_pick_index`).

## Phase 3.5 — Inspect
- (superseded by "## Inspect (Phase 3.5)" above)

## Phase 4 — Validate
- **Unit test (REQ-004):** added `theme_pick_index_maps_range` in palette.rs — the 6-case matrix incl the
  inspect-found `(CommandId(2003),2000,2)→None` (i>count, kills the `<`→`!=` mutant): base→Some(0), base+1→
  Some(1), below-base→None, i==count→None, i>count→None, empty-registry→None. `cargo nextest run -p marley`
  = **304 passed, 2 skipped** (303 + this); isolated run confirms `palette::tests::theme_pick_index_maps_range`
  PASS.
- **Driven captures (live app; RE-BUNDLED at inspect; #198 lesson applied — `focus` before every drive):**
  - **REQ-001** `scratchpad/199-palette-themes.png` (+ `-crop`): ⌘⇧P → type "theme" → the palette lists
    "Toggle Theme" (the #25 command) + **"Theme: Marley Light"** + **"Theme: Marley Dark"** (the two new
    per-theme commands, no keycap chip since they have no binding — correct).
  - **REQ-002** `scratchpad/199-light.png`: typed " light" to narrow to the Light row, Enter → the ENTIRE
    cockpit switched to the LIGHT theme LIVE — white/bright terminal panes, light-gray sidebar + Files, dark
    text, the light accent on the active row + pane borders. A dramatic, obvious change from the dark default
    (every prior capture this session was dark). `set_theme` applied it instantly (+ persisted).
  - **REQ-003 (persistence):** the dispatch calls the existing `set_theme` → `persist_theme`; the persist/load
    round-trip is already engine-tested (settings.rs). A full relaunch capture is optional — the round-trip is
    proven; the live switch confirms the apply half.
  - **REQ-005:** the static cockpit commands + the connect commands + `set_theme` are unchanged (verified at
    inspect); the theme commands are additive in the distinct THEME_BASE range.
- **Gate:** `git add -A` + `scripts/gates.sh --diff` — GREEN (below); cov/MSI 100 on `theme_pick_index` (the
  6-case matrix with the (2003) killer).
- **Pre-existing exclusions:** none (the `block v0.1.6` note is upstream).

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG.md — #199 under [Unreleased]/Added (below #198). app_shell.md — the palette.rs
  entry (the #222 keycap bullet) extended with the #199 theme-picker note (THEME_BASE range, theme_pick_index,
  set_theme reuse, deterministic registry).
- **Knowledge (forge):** `aar-submit` fd3b6cad (completed, effectiveness 5). No `failure-record` (no shipped
  bug). **Prevention rule recorded** → `PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators-001`
  (6b03ab48): don't hand-enumerate operator/method mutations — RUN `cargo mutants --list -f <file>` for the
  definitive set (`<` → `{==,>,<=}` only; body → typed defaults incl the sneaky `Some(1)`; method calls
  UNMUTATED). I hit this class twice (#198 `.min`, #199 `!=`).
- **Lessons:** (1) the prevention rule (above) — the critic ran the tool where I guessed, twice now. (2) this
  ticket was ~80% reuse — the registry + set_theme + persist round-trip existed; the delta was one pure
  helper + mirroring the #87 CONNECT_BASE pattern. Recognizing "this is the connect pattern again" made it
  fast + low-risk. (3) the #198 self-test lesson paid off immediately — every #199 drive started with `focus`,
  so the palette-open + type + select all landed on Marley (no wrong-window failures).
- **Close/archive:** TICKET-199 open→closed; forge ticket-close #199 done; pipeline pair → completed/.
