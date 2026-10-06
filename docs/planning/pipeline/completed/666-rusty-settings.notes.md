# Rusty's settings on the Marley settings page — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-666-rusty-settings-on-the-settings-page.md
- **Pipeline spec:** 666-rusty-settings.spec.md

## Phase 1 — Plan (2026-10-06)
- **Request:** the Queue's top after #665, from Chad's 2026-10-06 "adding the last things missing".
- **Classification / tier:** feature, R8's third part; Marley crates only.
- **Pre-flight:** green; no active pipeline; cargo idle; 75 GB free on the build disk.
- **Recall (§18.3):**
  - #643's page: `RustyServerView` drawn from the `Rusty` global, the provider's dropdown,
    `set_provider` then `read_settings`, a refusal kept in `refused`.
  - L-665: run `shellcheck` and `typos` on the scenario as soon as it is written.
  - AD-643 and #643's scenario: the page opens through a run-only key bound to
    `zed::OpenSettingsAt { path: "marley.rusty.server" }`.
  - The brain (consultation `8b876e011766456e8ad65559a3e048d7`): nothing on this seam.
- **Discovery:**
  - Rusty at `eb1ab51`: `settings_list` answers `[{key, value}]` with a credential-looking key's
    value as `•••` (`looks_secret`: the key holds key, token, secret, password or passwd);
    `setting_get` masks the same since TICKET-050; `setting_set { key, value }` answers `set`, and
    refuses the mask under a credential-looking key in the words `<key> holds a hidden value; "•••"
    is the mask a read returns, not a value to store`; `brain_semantic_status` answers `{ provider:
    "provider:model" | null, stats: { model, dims, pages, chunks }, stale }`.
  - Rusty's app (`SettingsPage.qml:48-58`): the ten known keys, their words and defaults; Enter
    saves a changed value; other keys with "hidden; type a new value to replace it" over a mask.
  - Marley: `rusty.rs` `RustyServerView` (a unit struct; the page renders from the global),
    `provider_row`, `set_provider`, `read_settings` on connect and on each announcement;
    `ServerSettings` (`get`, `embedding_provider`); `setting_set_arguments`.
  - The stand-in answers `settings_list` and `setting_set` from `settings.json`, unmasked.

### Design
- **`marley_rusty/src/settings.rs`** (Marley): `MASK`; `looks_secret(key)` (Rusty's rule);
  `KnownSetting { key, about, fallback }` and `KNOWN` (the ten, Rusty's app's words);
  `ServerSettings::others()` (the stored keys outside the ten, in Rusty's order);
  `value_to_write(stored, typed)` (none when unchanged, when the field over a mask is empty, or when
  the typed text is the mask); `BRAIN_SEMANTIC_STATUS`, `SemanticStatus` (with `SemanticStats`)
  and `status_line()`.
- **`rusty.rs`**: the `Rusty` global gains `semantic`; `read_settings` also reads
  `brain_semantic_status`; `set_provider` becomes `set_setting(key, value)`; the page's "Rusty's
  Settings" section: the status line, the provider's row as now, a row per other known key, Other
  stored keys, and the add row. Each field is `window.use_keyed_state` keyed by the key and a hash
  of the stored value, so a value read back makes a fresh field; each row's `RustySetting menu`
  context turns Enter into `menu::Confirm`, which writes `value_to_write`'s value.
- **The stand-in**: `settings_list` and `setting_get` mask as Rusty does, `setting_set` refuses the
  mask, `brain_semantic_status` answers from a `semantic.json` in its state folder (or full-text
  only without one).
- **File manifest:** `crates/marley_rusty/src/settings.rs`, the stand-in,
  `crates/marley_workbench/src/rusty.rs` (all Marley); `script/e2e/666-rusty-settings.sh` (Test).
  No Zed crate.

### Visual check plan
| Criterion | What the scenario does | Shot |
|---|---|---|
| REQ-001, 002, 007 | The run-only key opens the Rusty's Server page | `666-01-settings` |
| REQ-003 | Click `pin_timeout_minutes`' field, select all, type 10, Enter | `666-02-saved` |
| REQ-004 | Click the masked field, Enter | `666-03-masked-kept` |
| REQ-005 | Type a new value there, Enter | `666-04-masked-replaced` |
| REQ-006 | The add row's key and value, Set | `666-05-added` |
| REQ-007 | `embedding_model` changed, the stand-in's `semantic.json` with it | `666-06-status` |
| REQ-008 | Not shot | Review: the refusal line #643 shot |

### Risks
- **The settings window's height**: ten keys, the others and the add row make a long page; Zed's
  settings window scrolls its page.
- **A field over a value Rusty changes while it is being edited** becomes a fresh field holding
  Rusty's value; the edit is lost, which is what reading Rusty's new value means.

### Checklist (no TaskCreate in this harness)
- [x] Pick, pre-flight, recall, the brain; discovery.
- [x] Mint the pair; the BACKLOG row removed; the ticket in progress.
- [x] Prior-art sweep; spec; design; visual check plan; risks.
- [x] Phase 1 PASS under Chad's request ("continue on tickets until finished").

## Phase 2 — Code (2026-10-06)
### Built
- **`marley_rusty/src/settings.rs`**: `MASK`, `looks_secret` (Rusty's rule), `KnownSetting` and
  `KNOWN` (the ten in Rusty's app's order and words), `ServerSettings::others`, `value_to_write`,
  `BRAIN_SEMANTIC_STATUS`, `SemanticStatus` with `SemanticStats` and `line`.
- **`rusty.rs`**: the `Rusty` global's `semantic`, read with the settings in `read_settings`;
  `set_provider` became `set_setting(key, value)`, which the provider's menu uses too; the page's
  settings section (`settings_section`): the status line, a row per known key (the provider's
  dropdown for `embedding_provider`, a field with Rusty's words and default for the rest), Other
  Stored Keys, and the add row (`add_row`: key, value, Set or Enter). Each field
  (`setting_row`) is `window.use_keyed_state` under `ElementId::NamedInteger` of the key and a
  hash of Rusty's value; Enter in its `RustySetting menu` context writes `value_to_write`'s value,
  and empties a masked key's field after writing, since Rusty answers the mask again.
- **The stand-in**: `settings_list` and `setting_get` mask a credential-looking key's value,
  `setting_set` refuses the mask in Rusty's words, `brain_semantic_status` answers from the state
  folder's `semantic.json` or as full-text only. A smoke run over stdio passed each.
- **The guide page**: the settings article names the nine sections (it said six) and the Rusty's
  Server page's settings.

### Deviations
- None from the design.

### Review
- Clippy: two first doc paragraphs too long, split.
- A typed secret: the field over a masked value is emptied once it is written, so the secret does
  not stay on screen; the mask itself is never sent.

### Gate
`just gate-diff`: GATE GREEN [diff], 17 of 17.

## Phase 3 — Test (2026-10-06)
Scenario `script/e2e/666-rusty-settings.sh` under `compositor sway`: the stand-in's `settings.json`
with `default_agent`, `embedding_provider`, `notes_path`, a made-up `openai_api_key` and
`pin_timeout_minutes`, and its `semantic.json`; a run-only key opens the Rusty's Server page. Four
runs (one a probe with a shot after the scroll); the last passed every check.

### What each shot shows (last run)
- `666-01-settings` (REQ-001, 002, 007): "Embedding with ollama:nomic-embed-text: 42 pages in 130
  chunks have vectors; 3 pages waiting."; the ten known keys in Rusty's app's order, each with its
  value (`notes`, `5`) or its default as the hint (`~/.rusty/skills`, `deep`), Rusty's words and
  "Default: …"; the provider's dropdown in its place.
- `666-02-saved` (REQ-003): `pin_timeout_minutes` reads 10, as Rusty stores it.
- `666-03-masked-kept` (REQ-002, 004): scrolled down: Other Stored Keys, `default_agent` with
  `claude`, `openai_api_key` empty with "hidden; type a new value to replace it" after Enter; the
  stand-in's value unchanged.
- `666-04-masked-replaced` (REQ-005): after a new value and Enter, the field empty again with its
  hint; the stand-in holds the new value.
- `666-05-added` (REQ-006): `tour_note` with "added on the tour" under Other Stored Keys, the add
  row emptied.
- `666-06-status` (REQ-007): back at the top: "Embedding with ollama:nomic-embed-text-v2: 0 pages in
  0 chunks have vectors; 42 pages waiting.", `embedding_model` reading `nomic-embed-text-v2`.
- REQ-008 by review: a refusal sets `refused`, shown as #643 shot it.

### Fixes, each run again
- **The scenario's positions**: the Settings window holds the screen's right half, so the fields
  start at x 1282; the masked field and the add row at y 875 and 920 once scrolled to the end. A
  click off a field and Enter takes the Settings window back a page, which the first run showed.

No change to the code in this phase.

## Phase 4 — Complete (2026-10-06)
- **Docs:** `CHANGELOG.md` (Added); `docs/marley/rusty-in-marley.md` (R8);
  `docs/marley/guide.md` (Rusty's settings); `docs/marley/walkthrough.md` (stop 2.15h);
  `docs/marley_architecture/marley_rusty.md` (`settings`, the stand-in's masking and status);
  `docs/marley_architecture/marley_workbench.md` (the page's settings section). No Zed crate
  touched, so no zed-touchpoints row.
- **Knowledge:** L-claude-666-a-field-per-row-from-use-keyed-state-001,
  AD-claude-666-rustys-settings-are-written-only-when-changed-and-never-as-the-mask-001. No F
  block: no fault in the code was found.
- **Brain:** `brain decide` on consultation `8b876e011766456e8ad65559a3e048d7`, follow up by
  2026-11-06: `decisions/rustys-settings-in-marley-are-written-only-when-changed-and-never-as-the-mask`.
- **Closed:** the ticket in `tickets/closed/`; this pair in `completed/`.
