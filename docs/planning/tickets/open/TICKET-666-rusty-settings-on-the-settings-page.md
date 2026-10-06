# TICKET-666 — Rusty's settings on the Marley settings page

- **Ticket:** LOCAL #666 (feature, Rusty in Marley R8)
- **Owner:** claude-opus-5-5, 2026-10-06 (Chad: "we need to look into then adding the last things missing", after the R1 to R7b batch)
- **Pipeline doc:** (none yet)
- **Source ticket:** `docs/marley/rusty-in-marley.md` (R-D3, the slices table); the read-only survey of
  Rusty at `13249a8` and Marley on 2026-10-06
- **Status:** open

## Summary
The Rusty's Server page shows one of Rusty's settings, the embedding provider. Rusty's app shows ten known keys (`brain_vault_path`, `notes_path`, `embedding_provider`, `embedding_model`, `ollama_url`, `pin_timeout_minutes`, `skills_enabled`, `skills_path`, `brain_auto_enrich`, `default_workflow`) with descriptions, the other stored keys (masked where `settings_list` masks them), a field to add a key, and the embedding status (`brain_semantic_status`). This ticket brings those to the Rusty's Server page over `settings_list` and `setting_set`, never `setting_get` (it returns credentials unmasked).

## Acceptance
Every setting Rusty's app shows is on the Rusty's Server page, an edit reaches Rusty and reads back, and a credential-looking value shows masked.
