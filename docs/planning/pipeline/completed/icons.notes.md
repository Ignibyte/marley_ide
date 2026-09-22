# real icons (SVG, themeable) [M8] — Notes

- **Forge ticket:** #137 `7726f8db-4a2b-439d-9b8b-d61b73416d4a` · **AAR:** `9881ef69-2ebb-4707-9387-587cbc86ecc2`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-137-icons.md

## Phase 1 — Plan
- **Request:** forge #137 (M8 run 1/8) — emoji → themeable SVG icons (top bar + cockpit).
- **Pre-flight (gpui 0.2.2 source read):** `svg().path(SharedString)`, Svg is Styled → `.size`/`.text_color`
  tints (rasterized alpha mask filled with the color); `AssetSource::load(&self,path)->Result<Option<Cow>>` +
  `list`; `Application::with_assets(impl AssetSource)`; app entry `Application::new().run` (app.rs:3646);
  TitlebarOptions already used (helps #138). The #134 cockpit uses section_icon (emoji) + a bg-pill (emoji
  ignore text_color).
- **Decisions:** D1 svg tinted by text_color; D2 Assets AssetSource + include_bytes; D3 drop the bg-pill; D4
  clean-room authored icons.
- **AAR id:** `9881ef69-2ebb-4707-9387-587cbc86ecc2`.

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
- **assets/icons/*.svg (NEW):** files, plus, sparkle (new-agent), details, agents, forge — 24×24 monochrome filled silhouettes (gpui tints via text_color) + NOTES.md (clean-room provenance).
- **icons.rs (PURE):** enum Icon{Files,NewTerminal,NewAgent,Details,Agents,Forge}; icon_path(Icon)->&str ("icons/<name>.svg").
- **right_dock.rs (PURE):** section_icon(RightSection)->Icon (Details/Agents/Forge).
- **app.rs (SHIM):** struct Assets impl AssetSource {load: match "icons/*.svg"→include_bytes!("../assets/icons/*.svg"); list: vec![]}; Application::new().with_assets(Assets); at the emoji sites render svg().path(icon_path(i)).size(px(16)).text_color(c) — 📁→Files, 🧠→NewAgent, cockpit 📋🤖🔨→section_icon→Icon; DROP the #134 active bg-pill (svg tints → active=accent). "+" stays a text glyph (tints fine).
- **Test plan:** icon_path_distinct (each variant→its path, all distinct); section_icon_maps (each section→Icon).
- **Risks:** include_bytes! paths relative to app.rs (../assets/icons/); Svg .size + text_color; the assets/ dir must be inside the crate.

## Phase 3 — Implement
- **Built:** 6 clean-room SVG icons (assets/icons/{files,plus,sparkle,details,agents,forge}.svg) + NOTES.md. **icons.rs (PURE):** Icon enum + icon_path(Icon)->&str. **right_dock.rs (PURE):** section_icon(RightSection)->Icon (was &str emoji); the #134 test updated to Icon variants. **app.rs (SHIM):** struct Assets impl AssetSource (include_bytes! per icon path; gpui::Result); Application::new().with_assets(Assets); ICON_SIZE=16; the top-bar 📁/+/🧠 + cockpit 📋🤖🔨 render svg().path(icon_path(..)).size(16).text_color(..) — dropped the #134 active bg-pill (svg tints → active cockpit icon = accent). Imports: gpui svg + AssetSource; crate::icons.
- **DEVIATION:** made the "+" an SVG too (plus.svg) so Icon::NewTerminal is used (clippy) + all top-bar icons are consistent SVG.
- **Verification:** fmt; check 0 err; clippy OK. (SVG RENDERING is verified at P4 — the capture.)

## Phase 3.5 — Inspect
- **Method:** self-review (a pure mapping + asset infra + render swaps). Key risk: does the SVG actually render? (proven at P4 capture).
- **Lenses — no findings:** icon_path = distinct path per Icon variant; section_icon maps each RightSection→Icon. Assets::load matches "icons/*.svg"→include_bytes! (embedded, no runtime file dep), else None; list→empty. with_assets registers it. Each render = svg().path(icon_path(..)).size(16).text_color(..) — gpui rasterizes+tints. The #134 bg-pill removed (svg respects color). No unwrap/panic. Clean-room authored SVGs (§20 OK; NOTES.md records provenance). No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** icon_path_maps_each_variant (each Icon→its path, all distinct); section_icon_distinct (each RightSection→Icon). Pass.
- **Self-test:** LIVE capture (below) — THE key check: the SVG icons actually render (themed monochrome, no emoji).
- **Gate:** (running).

## Phase 5 — Complete
- CHANGELOG; forge #137 → done. **M8 1/8.** Emoji → themed SVG icons (gpui svg + AssetSource/with_assets; pure icon_path; clean-room SVGs). Active cockpit icon accent-tinted; #134 bg-pill removed. cov/MSI 100.
