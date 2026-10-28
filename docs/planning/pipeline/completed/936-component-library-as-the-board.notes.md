# The Component Library as the board draws it — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-936-component-library-as-the-board.md
- **Pipeline spec:** 936-component-library-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order (item 11, Components).
- **Classification / tier:** feature, S. Marley crate only (`marley_workbench`).
- **Recall (§18.3):**
  - L-claude-799 (checking a long scrolling page): put the thing to check at the top, or narrow
    the page; the scenario clicks `card` so the page is short and the strip is in view.
  - #801's scenario writes templates into `$E2E_PROFILE/config/components/`; reuse its files.
  - #849's scenario clicks the find field at a measured place (`FIND_X`, `FIND_Y`); the left
    list's rows sit under it.
  - No failure or prevention rule names `component_library.rs`.
  - The brain (2026-10-28, `brain ask`): nothing on this seam; only unrelated follow-ups came back.
  - Promoted from `queued/` on 2026-10-28; every cited seam re-checked at HEAD 547e4cb860 and holds.
- **Discovery:** one file, `crates/marley_workbench/src/component_library.rs`:
  `templates_group` (the strip), `ComponentLibrary::render` (Try it's header and where the error
  goes), `render_navigation` (the chosen row), and a new handler on `ComponentLibrary` that
  writes a template's example into the scratch editor.
- **Decisions:** D1–D4 in the spec.

### Design
- `templates_group` becomes a method on `ComponentLibrary` (it needs `cx.listener` for the chip
  clicks), or takes the listener it needs. The heading row: `Label` "Your templates", a muted
  "·", and a link-styled `Button` "components/ in Marley's config folder" that runs
  `cx.open_with_system(&templates_dir())`.
- A chip: a rounded, bordered `h_flex` with an id per file name, the name in the buffer font, and
  a state `Label` (`Color::Success` "loaded" or `Color::Error` with the error). The error is cut
  with `util::truncate_and_trailoff` to about 40 characters; `.tooltip(Tooltip::text(error))`
  holds the whole. Chips sit in an `h_flex().flex_wrap().gap_2()`.
- Loaded chip click: `serde_json::to_string_pretty(&template.example_tree())` set into
  `self.scratch` with `Editor::set_text`; the existing `BufferEdited` subscription runs
  `try_scratch`, so nothing else changes. Broken chip click: the workspace opens
  `broken.path` (the library keeps a `WeakEntity<Workspace>` taken in `open`, passed to `new`).
- Try it header: an `h_flex` of "Try it", a muted hint, and, when `self.tried` is
  `Some(Err(error))`, a flex-filling `Label` with `error.to_string()` in `Color::Error`,
  single-line and truncated. The body under the field draws only `Some(Ok(tree))`.
- Chosen row: compare `self.query(cx)` (already lower-cased) with the entry's name; when equal,
  `.bg(colors.element_selected)` and the label's colour `Color::Default` (the rest keep
  `Color::Default` too, so only the fill marks it; the board's ink change is the fill's
  contrast).
- File manifest: `crates/marley_workbench/src/component_library.rs` (Marley crate). No Zed path,
  so no `docs/marley/zed-touchpoints.md` row.

### Visual check plan
- REQ-001: setup writes two good templates and a broken one; open the library from the palette,
  click `card`; `936-01-chosen` shows two "loaded" chips and one red error chip, no drawn cards.
- REQ-002: click the `deploy_status` chip (place from the first run's shot); `936-02-template`
  shows its JSON in the box and the card drawn under it.
- REQ-003: `936-01-chosen` shows the header line with the hint.
- REQ-004: clear the box, type `{"type": "badge", "text": "beta", "tone": "orange"}`;
  `936-03-error` shows the error on the header line and no callout.
- REQ-005: `936-01-chosen` shows `card`'s row filled; `936-04-day` the same in Marley Day.

### Risks
- A long error on the header line could push the hint off; the error label must shrink and
  truncate (`min_w_0`, `truncate`), never the hint.
- Writing into the scratch editor from a click while the editor has focus elsewhere: `set_text`
  needs the window; the listener has it.
- Template file names can collide with element ids of the catalog cards; prefix chip ids with
  `marley-template-chip-`.

## Phase 2 — Code
- **Built** (`crates/marley_workbench/src/component_library.rs`, Marley crate):
  - `ComponentLibrary::render_templates` replaces `templates_group`. It draws "Your templates", a
    muted "·" and the link "components/ in Marley's config folder" (a transparent accent `Button`
    that opens the folder, the full path in its tooltip). Under it is a wrapping row of
    `template_chip`s, each with its file name and "loaded" (`Color::Success`) or the error cut to
    48 characters (`Color::Error`), and the whole of it in a tooltip.
  - `draw_template` writes a loaded chip's `example_tree` (pretty JSON) into the scratch editor.
    Its `BufferEdited` subscription draws it.
  - `open_template_file` opens a refused file through the workspace, which the library now holds
    as a `WeakEntity<Workspace>` taken in `open`.
  - Try it's header is an `h_flex` with "Try it", the hint (`flex_none`, so it never shrinks) and,
    on a refusal, the error at the right (`flex_1`, `min_w_0`, truncated, the whole error in a
    tooltip). Under the box, only a tree that draws is drawn.
  - The left list fills a row with `element_selected` while `query == name`.
- **Deviations:**
  - A loaded chip's tooltip carries the template's description, which the drawn card used to
    show.
  - A chip's `debug_selector` is `marley-template-chip-<stem>` for the scenario.
- **Review:**
  - `.map().unwrap_or_else()` became `map_or_else` (pedantic `map_unwrap_or`), and a redundant
    `file_name.clone()` went.
  - The first gate found `unused_self` on `render_templates`, which became an associated
    function.
  - Re-entrancy: the chip listeners update `scratch` and the workspace, never the library itself.
- **Gate, first run:** gate:2 failed (the `unused_self` above) and gate:7 cargo-audit failed.
  - Today's advisory-database fetch brought RUSTSEC-2026-0391, -0398, -0403, -0410 (zstd and its
    crates) and -0414 (pulldown-cmark 0.13.4). `Cargo.lock` was unchanged, so they are not this
    ticket's own.
  - Fixed at the source where a fixed version is in range: pulldown-cmark 0.13.5 and zstd-sys
    2.1.2.
  - zstd 0.13.3 and zstd-safe 5.0.2/7.3.0 need major steps (zstd 0.14.3, zstd-safe 8.2.0) that no
    requirement in the tree allows. Upstream main 5e8998a2a7 (2026-10-28) locks the same
    versions, so the three ids went on `.cargo/audit.toml`'s upstream-tree list, as #904 did with
    crypto-bigint.
- **Gate, second run:** `just gate-diff` GREEN, 17 of 17, receipt written.

## Phase 3 — Test
- **Scenario:** `script/e2e/936-component-library-as-the-board.sh` (sway). It uses `just build`,
  then `just e2e`, with shots in the scratchpad. Three runs:
  - The first run measured the chip (570, 840) and the scratch box (1000, 260); its guessed
    coordinates missed.
  - The second run's `type_text` of the badge JSON stopped after `{"type": "badge", "`, so the box
    showed serde's syntax error, not the wrong tone. The scenario now pastes the tree through the
    clipboard (#799's `paste_into_scratch`), which is also what the hint says to do.
  - The third run is the one read below.
- **`936-01-chosen` (REQ-001, REQ-003, REQ-005):**
  - The find field holds `card`. Its row under "Layout" is filled in the selected colour, and the
    page is narrowed to the card entry.
  - Try it's header reads "Try it" then, muted, "Paste JSON; it draws as you type."
  - "Your templates · components/ in Marley's config folder" (the link in the accent colour) sits
    over three chips: `deploy_status.json loaded` and `release_note.json loaded` in green, and
    `bad_one.json The body uses the input `missing`, which `inputs…` in red, cut short. No drawn
    template cards and no callouts.
- **`936-02-template` (REQ-002):** after a click on `deploy_status.json`, the box holds the
  template's example as pretty JSON. Under it is the drawn card: "staging", Version 1.4.0,
  Deploys today 4, and the api/up and worker/slow table, with a green border.
- **`936-03-error` (REQ-004):** the box holds `{"type": "badge", "text": "beta", "tone":
  "orange"}`. The header line's right end reads, in red, "$.tone: Unknown variant `orange`,
  expected one of `default`, `muted`, `accent`, `success`, `warning`, `error`." The hint stays
  whole, and there is no callout under the field.
- **`936-04-day` (REQ-001, REQ-003, REQ-005):** the same view in Marley Day.
  - The `card` row is filled in a light orange.
  - A crop of the strip shows "loaded" in green on both good chips and the error in red; the
    folder link is in the accent.
- **Not reached:**
  - A broken chip's click (opens its file) and the folder link's click (opens the system file
    manager). The first opens an editor tab; the second leaves the run's sway. Neither is in an
    acceptance criterion.
  - The error's tooltip needs a hover held over the chip; the truncated text is what the
    criterion names.
- **Noise:** the third run ended with `wl_display_dispatch: Broken pipe`. That is the scenario's
  `wl-copy` losing the clipboard when sway stopped, not Marley.
- **Bottom row:** it already shows "Component Library", the red `bad_one.json` chip and "15
  components · 2 templates", which #909 built. That matches the spec's Out item.

## Phase 4 — Complete
- **Documented (§21):**
  - `CHANGELOG.md`: an Added entry for #936, and a Changed entry for the advisories with the lock
    bumps.
  - `docs/marley/look-plan.md`: the Component Library row names #936.
  - `docs/marley_architecture/marley_workbench.md`: the Component Library section has the chosen
    row and a "Templates and Try it as the board" entry.
  - No Zed path was touched, so no `zed-touchpoints.md` row.
- **Knowledge (§19):** L-claude-936-paste-json-into-an-editor-do-not-type-it-001. No `F-` block:
  no real bug was found in Code or Test. No `AD-` block: the advisories followed `.cargo/audit.toml`'s
  existing policy.
- **Brain:** consultation 733e887e96434dbfb2078323f0414ffe closed with `brain no-decision`. The
  spec's decisions held, and the audit entries follow existing policy.
- **Ticket:** closed and moved to `tickets/closed/`. No row was left in BACKLOG.md.
