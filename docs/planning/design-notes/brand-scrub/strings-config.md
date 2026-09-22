# Brand-scrub audit — strings / config / labels / theme-names / assets

Scope: USER-FACING + CONFIG brand mentions (Warp / Zed) in the Marley **source**
(`crates/**`). String literals, theme names, settings defaults, non-`.rs` config/asset
files, and user-facing labels (palette entries, tab titles, status messages).
EXEMPT (not audited, not scrubbed): `docs/warp_architecture/`, `docs/zed_architecture/`
(reference docs). Comment-only brand mentions are OUT of this scope — they belong to the
separate comments sweep; a few are cross-listed below only so the boundary is auditable.

Audited: 2026-07-12. Method: read-only grep + targeted file reads. No code edited.

Format: `file:line | category | CURRENT | PROPOSED reword`.

---

## RESULT: ZERO in-scope brand mentions. Nothing to scrub.

No `warp`/`zed` token appears in any **string literal**, **theme name**,
**config default**, **user-facing label**, or **non-`.rs` asset/config file** anywhere
in `crates/**`. The highest-impact, load-bearing surfaces were verified clean (see KEEP
list). There are no `string` / `theme-name` / `config-default` / `label` / `asset` hits
to reword.

---

## Searches run (all in-scope surfaces, negative)

| Search | Result |
|---|---|
| `grep -rniwE '"[^"]*(warp\|zed)' crates --include='*.rs'` (brand in a quoted string) | 0 hits |
| `grep -rnoE '"[^"]*"' crates --include='*.rs' \| grep -iwE 'warp\|zed'` (permissive: brand in ANY quoted span) | 1 hit, and it is inside a `///` doc comment — see Boundary case |
| `grep -riwE 'warp\|zed' crates --include='*.toml' --include='*.json' --include='*.md'` (config/asset) | 0 hits |
| `grep -rniwE 'warp\|zed'` over `*.stderr`, `*.svg`, `assets/icons/NOTES.md` | 0 hits |
| `grep -rniwE 'zed' crates --include='*.rs'` (whole-word zed anywhere) | 0 hits — every `...zed...` is a substring (standardized, normalized, serialized, initialized, recognized, sanitized, pluralized, neutralized, oversized) |
| `grep -rnE 'warp\|Warp' crates --include='*.rs' \| grep '"' \| grep -vE '//.*[Ww]arp'` (brand on a string-bearing line, comment portion excluded) | 0 hits — every `Warp` in `.rs` lives in the comment portion, never in a string/identifier |

---

## Load-bearing surfaces verified CLEAN (KEEP — no change; would break behavior/persistence if touched)

These are exactly the theme-name / config-default / palette-action surfaces the audit
worried about. All brand-neutral already:

- `crates/marley_app/src/themes.rs:38` — theme name `"Marley Light"` — CLEAN (not "Warp Light").
- `crates/marley_app/src/themes.rs:43` — theme name `"Marley Dark"` — CLEAN (not "Warp Dark").
  The built-in `ThemeRegistry` holds exactly these two; no brand-named theme exists.
- `crates/marley_app/src/settings.rs:23` — persisted config default `ThemeName = "Marley Dark"` — CLEAN.
  (This is the `by_name` registry key + persisted value; renaming it WOULD break saved-theme
  round-trips — but no rename is needed, it's already brand-neutral.)
- `crates/marley_app/src/palette.rs:167-169` etc. — palette command titles (`"Split Pane"`,
  `"Close Pane"`, `"Toggle Theme"`) and action strings (`"new-terminal"`, `"toggle-theme"`,
  `"split-right"`, …) — all CLEAN. (Action strings are dispatch keys; brand-neutral.)
- `crates/marley_app/src/app.rs:845` — palette label `"Save Command as Workflow…"` — CLEAN.
- `crates/marley_app/src/workflows.rs:12` — palette display pattern `"Workflow: {name}"` — CLEAN.
  NB: the feature is Marley's analog of "Warp Workflows", but the user-facing word is the
  generic **"Workflow"**, never "Warp Workflow".
- `crates/marley_settings/tests/settings.rs:14` — `define_setting!(ThemeName ... = "dark" ...)` —
  test-only fixture, lowercase generic `"dark"`, CLEAN (not the app's real default).
- All `crates/**/Cargo.toml` package names/descriptions — CLEAN.
- All `crates/marley_app/assets/icons/*.svg` + `NOTES.md`, all `*.stderr` trybuild fixtures,
  the `*.png` baseline — CLEAN.

---

## Boundary case (OUT of this scope — comment, cross-listed for the comments sweep)

- `crates/marley_app/src/right_dock.rs:74` | (doc comment, NOT a string) |
  `/// text tab (chad's "icons like Warp"). One recognizable emoji per section.` |
  The permissive quoted-span grep flagged `"icons like Warp"`, but the quotes are an
  *inner quotation inside a `///` doc comment*, not a runtime string literal. It reaches
  no user and no config. Not rewordable here — hand to the comments sweep.

## Comment-only brand mentions (OUT of scope — reference only, for the comments sweep)

All remaining `Warp` tokens in `.rs` are in `//` / `///` / `//!` comments describing
"Warp parity" design intent (typography sizes, palette tuning, layout notes, block-hover
affordances). They do not reach a user or config. Locations include (non-exhaustive):
`crates/ui_components/src/{lib.rs,render/keyboard_shortcut.rs}`,
`crates/marley_app/src/{typography.rs,workspace.rs,nav.rs,workflows.rs,grid_layout.rs,color.rs,file_tree_view.rs,right_dock.rs,app.rs}`.
These are the comments sweep's responsibility, not this one.
