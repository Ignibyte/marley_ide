# Jev through Cloudflare Workers AI, as a provider setting — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-548-system-one-via-cloudflare.md
- **Pipeline spec:** 548-system-one-via-cloudflare.spec.md

## Phase 1 — Plan (2026-10-05)
- **Request:** Chad, 2026-10-05: "Start #548 now", asked once the Queue was empty; the row stood
  under Deliberate, waiting on the System One layer, which shipped in #565.
- **Classification / tier:** feature, prong 2 (S1's second provider). Marley crates plus three
  Zed-side files already in the ledger: `settings_content/src/marley.rs`,
  `settings_ui/src/marley_page.rs`, `assets/settings/default.json`. Minted (no queued pair); the
  Deliberate row removed; the ticket in progress.
- **Pre-flight:** green; no active pipeline; the README marker present; cargo idle.
- **Recall (§18.3):**
  - AD-565: the layer is a pure core (`marley_system_one`) behind an adapter
    (`marley_workbench::system_one`); off, it makes no request, reads no key, writes no file; the
    key from `MARLEY_SYSTEM_ONE_KEY`, else gpui's keychain at the provider's URL; no client SDK.
  - L-565 (Jev's answer shape): a noul's `noul` alone, a choice's `confidence` and
    `probabilities`, a score's fractional `score` and `legend`; retries on 429, 503, 529; an
    error body cut to 300 characters.
  - L-565 (the keyring): a scenario's Marley reaches the user's own Secret Service, so no
    scenario presses Set Key or Forget Key; secrets go through the environment.
  - The brain (`brain ask`, consultation `92197e8a025f426ebe762117720bd48b`): nothing on this
    seam; follow-ups due on other work only.
- **Discovery** (an Explore sweep, then the files read):
  - `SystemOneProvider` (`settings_content/src/marley.rs:383-409`), the settings struct
    (`:340-381`), `agent_permissions_by_project` (`:149-155`) as the per-project precedent,
    resolved in `agents.rs:204-282` with `system_one::folder_path`.
  - `marley_workbench/src/system_one.rs`: the constants (`:48-67`), `SystemOneSettings`
    (`:76-138`), `Key` (`:163-183`), `KeySource` (`:185-210`), the global's one key slot
    (`:214-228`), `apply_settings` (`:313-320`), `load_key` (`:333-393`), `store_key` and
    `forget_key` (`:396-423`), `needs_key` (`:455-460`), `provider_name` (`:463-470`),
    `endpoint()` (`:484-508`), `Asking` (`:511-526`), `Draft::new` (`:570`), `ask`'s dispatch
    (`:636-686`), `state_for`'s mask (`:691-712`), `send` (`:811-878`), `post` (`:892-947`),
    `read_posted` (`:951-999`; only 401 refuses the key).
  - `marley_system_one/src/request.rs`: `build` with `json!` (`:22-61`), `Answer` (`:64-95`),
    `parse` reading a top-level `answers`, `model` and `usage.input_tokens` (`:138-168`),
    `error_excerpt` (`:172`); `files.rs`: `CallRow.provider` a string (`:35-36`), replay's envelope
    (`:214`).
  - `system_one_calls.rs`: the header's provider and key lines (`:208-213`), Set Key and Forget
    Key (`:246-260`), the rows' provider (`:285-291`).
  - `settings_ui/src/marley_page.rs:903`: `system_one_section() -> [SettingsPageItem; 17]`; the
    Provider item (`:931-955`) and Endpoint (`:956-980`); the dropdown renderer
    (`settings_ui.rs:565`, `:5088-5130`) lists every `VariantArray` variant.
  - `assets/settings/default.json:1826-1869`.
  - `script/e2e/565-system-one-layer.sh`: the Python fake (`:65-118`), its setup (`:32-54`) and the
    `system_one_setting` merge helper (`:122-174`).
  - Cloudflare (published): the model page and the Workers AI REST page, quoted in the spec.

### Design
- **`settings_content/src/marley.rs`** (Zed crate, ledger row first): `SystemOneProvider::Cloudflare`
  (its doc: Jev through Cloudflare Workers AI, zero retention); `SystemOneSettingsContent` gains
  `provider_by_project: Option<BTreeMap<String, SystemOneProvider>>`, `cloudflare_account_id:
  Option<String>`, `cloudflare_api: Option<String>`; the `provider`, `endpoint` and price docs name
  Cloudflare where they list providers.
- **`assets/settings/default.json`** (Zed file, ledger row first): `cloudflare_account_id` empty,
  `cloudflare_api` with its default, `provider_by_project` `{}`, the provider comment.
- **`marley_system_one/src/request.rs`** (Marley): `CLOUDFLARE_MODEL = "typesafe/jev"`;
  `build_cloudflare(state, set) -> Value` (`{"model", "input": {"state", "questions"}}` from the
  same parts as `build`); `parse_cloudflare(text) -> Result<Answers, ParseError>`: the envelope's
  `result` through `parse`, or the first of `errors`' messages when `success` is false or `result`
  is missing.
- **`marley_workbench/src/system_one.rs`** (Marley):
  - Settings: `by_project: Vec<(PathBuf, SystemOneProvider)>`, `cloudflare_account: Option<String>`,
    `cloudflare_api: String`; `provider_for(folders) -> SystemOneProvider` (the longest folder
    holding the main folder, else `provider`); `uses(provider)` (the default or any entry).
  - Keys: the one slot becomes two (`Slot::Direct`, `Slot::Cloudflare`), each with its key, source,
    refused flag and load counter; `slot_of(provider)`; `load_key(slot)` reads
    `MARLEY_SYSTEM_ONE_KEY` or `MARLEY_CLOUDFLARE_API_TOKEN`, else the keyring at the slot's URL,
    only while on and the slot is in use; `store_key` and `forget_key` take the slot; `state_for`
    masks with every key held.
  - `ask` resolves the provider from `asking.folders` before its dispatch; `Draft::new` takes it;
    `record` stays `rules`.
  - `endpoint(provider)`: Cloudflare's `{api}/accounts/{account}/ai/run`, refused while the
    account id is empty or holds anything but letters and digits, and while `cloudflare_api` is not
    https or loopback http.
  - `send` builds Cloudflare's body and model; `read_posted` parses Cloudflare's envelope and
    refuses the Cloudflare token on 401 or 403.
- **`marley_workbench/src/system_one_calls.rs`** (Marley): the header's Cloudflare line (account,
  token source) and Set Cloudflare Token and Forget Cloudflare Token while Cloudflare is in use;
  the key editor knows which slot it writes.
- **`settings_ui/src/marley_page.rs`** (Zed crate, ledger row first): a Cloudflare Account ID item
  after Endpoint; the array's length 17 to 18; the Provider description names Cloudflare.
- **`marley_workbench/guide/index.html`** (Marley): the System One article names the provider.
- **File manifest:**
  - `crates/settings_content/src/marley.rs` — Zed crate (row `:59`).
  - `crates/settings_ui/src/marley_page.rs` — Zed crate (row `:63`).
  - `assets/settings/default.json` — Zed file (row `:67`).
  - `crates/marley_system_one/src/request.rs` — Marley crate.
  - `crates/marley_workbench/src/system_one.rs` — Marley crate.
  - `crates/marley_workbench/src/system_one_calls.rs` — Marley crate.
  - `crates/marley_workbench/guide/index.html` — Marley crate.
  - `script/e2e/548-system-one-via-cloudflare.sh` — the scenario (Test).

### Visual check plan
The scenario `script/e2e/548-system-one-via-cloudflare.sh`, `compositor sway`, set up as the spec's
UI proof says, with #565's fake extended to Cloudflare's path and envelope.

| REQ | Scenario | Shot |
|---|---|---|
| REQ-001, 002 | `marley: open settings`, search "Provider", open the dropdown | `548-01-providers` |
| REQ-003, 004 | `direct`'s terminal: `false`, `marley: system one check` | `548-02-direct` |
| REQ-003, 005, 006 | `private`'s terminal: the same; `expect` the fake's Cloudflare request | `548-03-cloudflare` |
| REQ-007, 008 | `marley: open system one calls`; `expect` the day's rows | `548-04-calls` |
| REQ-009 | `private`: `false refuse-token`, the check; the calls tab | `548-05-refused` |
| REQ-010 | `cloudflare_account_id` emptied; the check in `private`; `expect` no new request | `548-06-no-account` |
| REQ-011 to 014 | Not shot | Review: the mask, the reads' gates, the URL check, the settings scope |

The keyring buttons are not pressed (L-565); the token comes from the environment.

### Risks
- **Cloudflare's live answer is unseen.** No account is at hand; the fake follows Cloudflare's
  published pages. If a field differs (the envelope, `usage`), the first real call reads it
  unavailable with Cloudflare's message, which names it.
- **One breaker for both providers.** Five Cloudflare failures in a row hold TypeSafe's calls for
  two minutes too (D9).
- **The rate cap** (1,000 a minute, set under TypeSafe's 1,200) may sit above Cloudflare's own limit
  for Jev, which its page does not state; Cloudflare's 429 is retried once, as TypeSafe's.
- **Two keyring items** at two URLs; the Cloudflare one moves when the account id changes.
- **The key editor** on the calls tab gains a slot; the masked input stays one.

### Checklist (no TaskCreate in this harness)
- [x] Pick: TICKET-548, named by Chad.
- [x] Pre-flight green.
- [x] Recall: AD-565, L-565 twice, the brain.
- [x] Mint the pair (id `06bb94ca-ec44-4dd6-be1e-e5ff875665af`); the Deliberate row removed; the
      ticket in progress.
- [x] Prior-art sweep: Cloudflare's two pages, the Jev note, the code (`request`, `system_one`,
      `agent_permissions_by_project`, the dropdown).
- [x] Spec: scope, Reference, Prior art, UI proof, D1 to D10, 14 EARS rows, phase plan.
- [x] Design: approach, manifest, visual check plan, risks.
- [x] Phase 1 PASS: Chad started it, and the work runs under his goal.

## Phase 2 — Code (2026-10-05)
- **Built:**
  - The three ledger rows widened first (`settings_content/src/marley.rs`,
    `settings_ui/src/marley_page.rs`, `assets/settings/default.json`).
  - `settings_content`: `SystemOneProvider::Cloudflare`; `provider_by_project`,
    `cloudflare_account_id`, `cloudflare_api` on `SystemOneSettingsContent`; the doc names the
    token's variable. `default.json`: the three keys and the provider comment.
  - `marley_system_one::request`: `CLOUDFLARE_MODEL`, `build_cloudflare` (the same questions,
    through a shared `questions_json`), `parse_cloudflare` (the envelope's `result` through the
    old parse, split out as `parse_value`; `success: false` or no `result` gives
    `ParseError::Failed` with the first error's words and code), `cloudflare_error` for an error
    status's body.
  - `marley_workbench::system_one`:
    - `CLOUDFLARE_TOKEN_VARIABLE`, `CLOUDFLARE_API`; the settings' `by_project`,
      `cloudflare_account`, `cloudflare_api`, with `provider_for` (the deepest folder, as
      `agents.rs` picks), `uses_provider` and `slot_provider`.
    - `Slot { Direct, Cloudflare }` and a `Credential` per slot (key, source, refused, load),
      replacing the one key's four fields; `KeySource::Environment` names its variable.
    - `load_key` loads both slots; `load_slot` reads only while on and a provider in use sends
      with the slot; `store_key` and `forget_key` take the slot; `slot_url`.
    - `endpoint(settings, provider)` with Cloudflare's `{api}/accounts/{account}/ai/run` (the id
      letters and digits only) and `guarded_url`, shared with `compatible`.
    - `ask` resolves the project's provider; `Draft` carries it and records `typesafe/jev` for
      Cloudflare; `state_for` masks with both keys; `send` takes the slot's key and builds
      Cloudflare's body at TypeSafe's price; `read_posted` parses the envelope, refuses the
      Cloudflare token on 401 or 403, and names Cloudflare's words in the reading.
    - `load_replay` follows `uses_provider(Replay)`.
  - `system_one_calls.rs`: the Cloudflare line (account, token source) and Set Cloudflare Token
    and Forget Cloudflare Token while Cloudflare is in use; the key field knows its slot.
  - `marley_page.rs`: the Cloudflare Account ID item after Endpoint (18 items); the Provider
    description names Cloudflare and `provider_by_project`. The guide page's System One article.
- **Deviations:**
  - REQ-010: no account id reads *refused* with the reason, as `compatible` with no endpoint
    already does; no token reads unavailable. The spec says so now.
  - The default file's `uses` comment still said "Decisions" (#659's sweep missed `assets/`); it
    says System One calls now.
- **Review:**
  - Every state is masked with both keys; a Cloudflare request carries only the Cloudflare
    token.
  - The Cloudflare token is read only while on and Cloudflare is the default or some project's.
  - Clippy's `doc_markdown` took "TypeSafe's" in a doc comment for an item; reworded.
- **Gate:** the first run went red on gate:1 (a `map_or_else` rustfmt reflows) and the receipt
  (formatted mid-run); after `rustfmt`, GREEN, 17 of 17.

## Phase 3 — Test (2026-10-05)
- **Scenario:** `script/e2e/548-system-one-via-cloudflare.sh` under `compositor sway`, against the
  debug `marley`: `direct` opened, `private` handed to the same window (#613's `hand_off`), one
  Python fake answering TypeSafe's shape and Cloudflare's envelope, both secrets made up in the
  environment. The final run exited 0 with its nine checks passing: direct asked the compatible
  fake once and Cloudflare nothing; private asked Cloudflare once, at the account's path, with
  the token as the bearer, `typesafe/jev` and `input` holding `state` and `questions`, and no
  trace of the direct key; one `compatible` row and one `cloudflare` row, the latter with model
  `typesafe/jev`; the refused request reached Cloudflare; no request without an account. The
  fake's log holds exactly three requests: one at `/v1/systemone`, two at `.../ai/run`.
- **Runs:** two. The first measured the Provider dropdown (at (1513, 254), not the guessed
  (1500, 200)); the rail's rows and the settings search field were where #613 and L-659 put them.
  The measuring shot of the rail was dropped.
- **Shots**, each read (final run):
  - `548-01-search` (REQ-002): the Settings window searched for "Cloudflare": Provider, its
    description naming Cloudflare and `provider_by_project`, set to Compatible; Cloudflare
    Account ID with its description and an empty field (the run emptied it a step earlier).
  - `548-01-providers` (REQ-001): the Provider dropdown open: Typesafe, Compatible (ticked),
    Rules, Replay, Cloudflare.
  - `548-02-direct` (REQ-003, 004): `direct`'s terminal after `false`: the toast "command failed:
    yes (0.92) · compatible · 1,200 tokens".
  - `548-03-cloudflare` (REQ-003, 005, 006): `private`'s terminal: the toast "… · cloudflare ·
    1,200 tokens", read from the envelope's `result`.
  - `548-04-calls` (REQ-007, 008): System One calls: "Provider: compatible · jev-1.13.0", "Key:
    environment (MARLEY_SYSTEM_ONE_KEY)", "Cloudflare: account 0123…cdef · token environment
    (MARLEY_CLOUDFLARE_API_TOKEN)", Set and Forget Cloudflare Token beside Set and Forget Key, a
    note for each variable; the rows "check · private · cloudflare · typesafe/jev … 0.00504¢" (the
    cost at Jev's price) and "check · direct · compatible · jev-1.13.0".
  - `548-05-refused` (REQ-009): the third row in red, "Unavailable: Cloudflare refused the token:
    Authentication error (10000)"; the Cloudflare line's token "none (the provider refused the
    Cloudflare token from environment …)"; the same words in the toast.
  - `548-06-no-account` (REQ-010): the toast "Refused: set marley.system_one.cloudflare_account_id
    for the cloudflare provider"; no new request.
- **Not reached by a shot:** REQ-011 (the mask) and REQ-012 (the token read only when used), by
  review and the check that no direct key reached Cloudflare; REQ-013 (`cloudflare_api`'s
  scheme) by review, the scenario's loopback http being the allowed case; REQ-014 by the
  settings' scope (`marley` lives in the user's `SettingsContent`, never a project's).
- **Focus:** the keys reached each project's terminal, the palette and the Settings window's
  search; nothing reached the user's keyring.

## Phase 4 — Complete (2026-10-05)
- **Documented:**
  - `CHANGELOG.md`: Added (Jev through Cloudflare Workers AI, for every project or some).
  - `docs/marley/guide.md`: the providers table's `cloudflare` row; Cloudflare's setup, token,
    refusal and `cloudflare_api`; a provider per project with its example; the environment
    table's `MARLEY_CLOUDFLARE_API_TOKEN`. `walkthrough.md`: Part 10's closing paragraph.
  - `docs/marley_architecture/marley_workbench.md`: the System One section's Cloudflare and
    per-project bullet. `marley_system_one.md`: `request`'s Cloudflare build and parse.
  - `docs/marley/three-prong-plan.md`: S1 names #548.
  - `docs/marley/zed-touchpoints.md`: the three rows describe what shipped; the defaults file's
    row also names the `uses` comment fixed.
- **Knowledge:** F-claude-548-the-defaults-file-still-said-decisions-after-659-001,
  PR-claude-548-a-user-visible-rename-sweeps-assets-too-001,
  L-claude-548-workers-ai-takes-the-model-in-the-body-and-wraps-the-answer-001,
  L-claude-548-format-before-the-gate-not-during-it-001,
  AD-claude-548-cloudflare-is-a-provider-chosen-per-project-with-its-own-token-001.
- **Brain:** `brain decide` on consultation `92197e8a025f426ebe762117720bd48b`, follow up by
  2026-11-05:
  `decisions/marleys-system-one-reaches-jev-through-cloudflare-as-a-provider-chosen-per-project-with-its-own-token`.
- **Closed:** the ticket in `tickets/closed/`; no BACKLOG row (removed at promotion); this pair in
  `completed/`.
- **Gate:** run again on the final tree.
