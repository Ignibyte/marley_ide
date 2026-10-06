---
pipeline_id: 9e1ad2a2-9ee4-4aeb-afeb-9a38d5eeff12
ticket: docs/planning/tickets/open/TICKET-666-rusty-settings-on-the-settings-page.md
status: Phase 4 — Complete PASS
title: "Rusty's settings on the Marley settings page"
type: feature
slice: Rusty in Marley R8 (docs/marley/rusty-in-marley.md)
references: [docs/marley/rusty-in-marley.md, docs/planning/pipeline/completed/643-rusty-switch-and-connection.spec.md]
---

## Title
Rusty's settings on the Marley settings page. The Rusty's Server page shows one of Rusty's
settings, the embedding provider. Rusty's app shows ten known keys with what each does and its
default, the other stored keys (a credential-looking one masked), a row to add a key, and its
embedding status. This ticket brings all of that to the Rusty's Server page over
`settings_list`, `setting_set` and `brain_semantic_status`.

## Scope
### In
- **The known keys.** `brain_vault_path`, `notes_path`, `embedding_provider` (the dropdown it has
  now), `embedding_model`, `ollama_url`, `pin_timeout_minutes`, `skills_enabled`, `skills_path`,
  `brain_auto_enrich`, `default_workflow`: each with a field holding Rusty's value (empty, with the
  default as its hint, when Rusty stores none) and Rusty's words on what it does. Enter in a field
  writes it when it changed.
- **Other stored keys.** Every key Rusty stores that is not one of the ten, with its value; a value
  Rusty masks shows as an empty field with "hidden; type a new value to replace it", and nothing
  is written unless a new value is typed.
- **Adding a key.** A key and a value field with Set.
- **The embedding status.** One line from `brain_semantic_status`: the provider and model in use and
  how many pages and chunks have vectors, with the pages waiting; or that brain search is
  full-text only.
- **After a write.** The settings and the status read again; Rusty's refusal (writing the mask back,
  say) stays on the page, as the provider's does now.
- **`marley_rusty::settings`**: the known keys with their words and defaults, the mask, the status;
  the stand-in masking credential-looking keys and refusing the mask written back, as Rusty does,
  and answering `brain_semantic_status`.

### Out (explicitly deferred)
- **Deleting a key** (Rusty's tools have no delete).
- **Validating a value** before Rusty does (a path, a number); Rusty stores what it is given.
- **Rusty's own restart** after `brain_vault_path` changes; the words say to restart the service.

## Reference (§20)
N/A — Marley-specific: Rusty's own app is the reference for the behaviour
(`crates/rusty-app/qml/SettingsPage.qml:48-61` and its settings section in Rusty's repository at
`eb1ab51`: the ten known keys with their words and defaults, Enter saves a changed value, the
other stored keys with a masked one's hint, a key and value with Set). Zed's own settings page
items, editors and dropdown draw it.

### Prior art
- **Behaviour maps:** none hold another program's settings inside an editor's settings page.
- **Published material:** Rusty's tools (`crates/rusty-mcp/src/main.rs:1058-1067` `settings_list`,
  `:1982-2005` `setting_get` and `setting_set`, `:929-951` `brain_semantic_status` and its
  `SemanticStatus` `:608-615`) and its store (`crates/rusty-core/src/engine/settings_manager.rs`:
  `MASK`, `looks_secret` on key, token, secret, password and passwd, `refuse_the_mask`;
  `brain/semantic.rs:301-310` `SemanticStats`).
- **The code we ship:** the Rusty's Server page (`rusty.rs`: `RustyServerView`, `provider_row`,
  `set_provider`, `read_settings`, the `Rusty` global's `settings` and `refused`);
  `Window::use_keyed_state` for per-row editors; `marley_rusty::settings::ServerSettings`.

## UI proof
`script/e2e/666-rusty-settings.sh` (`compositor sway`: clicks in the Settings window's fields). Setup:
the stand-in with `settings.json` holding `embedding_provider`, `pin_timeout_minutes`,
`notes_path`, an extra key `default_agent` and a credential-looking `openai_api_key`; a run-only
key opens the Rusty's Server page (as #643's does). Shots:
- `666-01-settings`: the page: the ten known keys with their words and values or defaults, the
  embedding status, Other stored keys with `default_agent` and the masked `openai_api_key`, and
  the add row.
- `666-02-saved`: `pin_timeout_minutes` changed and Enter: the new value read back.
- `666-03-masked-kept`: Enter in the masked field with nothing typed: nothing written.
- `666-04-masked-replaced`: a new value typed there and Enter: written, masked again.
- `666-05-added`: a new key and value with Set: listed under Other stored keys.
- `666-06-status`: the embedding status after `embedding_model` changes.

## Locked-In Decisions
- D1 — **The page stays one view** (`RustyServerView`), its fields made with
  `window.use_keyed_state` keyed by the key and the value Rusty stores, so a value read back gives
  the field its new text and an edit in progress is kept until then.
- D2 — **Enter writes a changed value only**; an empty field over a masked value writes nothing;
  the mask is never sent (Rusty would refuse it).
- D3 — **Rusty's words for the known keys**, taken from its app, with the default as the empty
  field's hint.
- D4 — **One status line** from `brain_semantic_status`, read with the settings.
- D5 — **The stand-in masks and refuses as Rusty does** (`looks_secret`), so the scenario sees the
  mask.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE Marley is connected to Rusty, the Rusty's Server page shall show the ten known keys with Rusty's words, each with Rusty's value or, when unset, its default as a hint. | Shot `666-01-settings` |
| REQ-002 | WHILE Rusty stores keys outside the ten, the page shall list them with their values, a masked value as an empty field with its hint. | Shot `666-01-settings` |
| REQ-003 | WHEN Enter is pressed in a field whose text differs from Rusty's value, the system shall write it and show the value read back. | Shot `666-02-saved` |
| REQ-004 | WHEN Enter is pressed in a masked value's field with nothing typed, the system shall write nothing. | Shot `666-03-masked-kept`; the stand-in's log |
| REQ-005 | WHEN a new value is typed over a masked one and Enter pressed, the system shall write it. | Shot `666-04-masked-replaced`; the stand-in's file |
| REQ-006 | WHEN a key and a value are given and Set clicked, the system shall write the key and list it. | Shot `666-05-added` |
| REQ-007 | WHILE connected, the page shall show Rusty's embedding status in one line. | Shots `666-01-settings`, `666-06-status` |
| REQ-008 | IF Rusty refuses a write, THEN the page shall show Rusty's words. | Review (the provider's refusal line, shot in #643) |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `marley_rusty::settings`' known keys, mask and status; `rusty.rs`'s page, writes and
  status read; the stand-in; a review; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG, the plan's R8, the architecture notes, the guide and the
  walkthrough, ledger capture, the brain decision, close, archive, commit.
