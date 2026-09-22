# handlebars

> Per-crate reference (Marley round 2) for `crates/handlebars`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[Warp-derived/AGPL]` — small Warp-original arg parser (not the upstream `handlebars` crate); standard concept, cheap to clean-room. Gap. See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| | |
|---|---|
| **Subsystem** | [06 — Platform, Settings, Persistence & Infra](../subsystems/06-platform-settings-infra.md) |
| **License** | AGPL v3 (workspace `license`; no per-crate LICENSE marker) |
| **Internal deps** | 0 |
| **Used by** | 2 |

## Purpose

A **minimal, single-purpose template engine** — *not* the upstream `handlebars` crate from
crates.io, but Warp's own tiny re-implementation of the `{{name}}` placeholder subset. It
extracts variable names from a template and substitutes values from a `HashMap`. It exists
so Warp can interpolate user/cloud-defined templates (e.g. workflow commands, prompt
snippets, notebook cells) without pulling in a full Handlebars dependency, and with
careful char-index ↔ byte-index handling so it is correct for non-ASCII templates.

## Key types, modules & public API

- **`src/lib.rs`** — two free functions:
  - `get_arguments(template: &str) -> Vec<String>` — returns the **distinct** variable
    names referenced in the template (deduped via a `HashSet`).
  - `render_template(template: &str, context: &HashMap<String, String>) -> String` —
    substitutes each `{{name}}` with `context[name]`; **unknown variables are left as the
    literal `{{name}}` placeholder** rather than blanked.
- **`src/parser.rs`** — the tokenizer:
  - `ParsedArgumentsIterator<I>` (`::new(string_chars)`) — a `char`-stream iterator that
    yields `ParsedArgument`s.
  - `ParsedArgument` with `chars_range() -> Range<usize>` (the name span, braces excluded)
    and `result() -> &ParsedArgumentResult`.
  - `ParsedArgumentResult` — the `Valid { .. }` / invalid discriminant the renderer matches on.
  - `word_count()` helper.

Both `lib.rs` and `parser.rs` keep companion `*_tests.rs` files (`#[path = ...]` included).

## Depends on (internal)

None. (No external dependencies either — `Cargo.toml` declares no `[dependencies]`.)

## Used by (internal dependents)

- [./warp.md](./warp.md) — the main binary; renders templated strings in app features.
- [./cloud_object_models.md](./cloud_object_models.md) — applies templates to cloud-defined objects (workflows/prompts carrying `{{var}}` placeholders).

Total: **2** dependents.

## Related crates

- [./cloud_object_models.md](./cloud_object_models.md) — the primary content source whose objects carry templates.
- [./field_mask.md](./field_mask.md) — sibling tiny leaf utility in the same subsystem grab-bag.

## Marley relevance

**KEEP / RENAME (low priority).** Self-contained, zero-dependency, no auth and no Warp
branding in its logic — safe to keep verbatim. The one wrinkle is the **package name
`handlebars`**, which shadows the well-known crates.io crate; if Marley later wants the real
Handlebars engine, this name collision is annoying, so a rename to something like
`marley_template` (or `mini_template`) is a clean, low-risk change — only 2 dependents and
no internal deps. Directly useful to goal (1): a custom Marley panel that renders
user-defined command/prompt templates can call `render_template` as-is. Not on the de-auth
or rebrand critical path.

## Notes / gotchas

- **Name collision**: this is *not* `handlebars` from crates.io — it's a homegrown subset. Don't assume full Handlebars features (helpers, partials, conditionals); only `{{name}}` substitution is supported.
- **Unicode-correct by construction**: both functions build a `char_to_byte` index table so placeholder spans slice on byte boundaries — don't "simplify" this to direct byte indexing.
- **Unknown vars are preserved, not emptied** — `render_template` re-emits `{{missing}}` literally; callers expecting blanks must pre-fill the map.
- `get_arguments` returns names in **non-deterministic order** (it round-trips through a `HashSet`).
- Uses `edition = "2024"`.
