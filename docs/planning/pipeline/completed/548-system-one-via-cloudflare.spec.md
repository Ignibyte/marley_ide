---
pipeline_id: 06bb94ca-ec44-4dd6-be1e-e5ff875665af
ticket: docs/planning/tickets/closed/TICKET-548-system-one-via-cloudflare.md
status: Phase 4 — Complete PASS
title: "Jev through Cloudflare Workers AI, as a provider setting"
type: feature
slice: prong 2, the System One layer (S1 of docs/marley/three-prong-plan.md), its second provider of Jev
references: [docs/marley/three-prong-plan.md, docs/planning/design-notes/jev-system-one-2026-09-25.md, docs/planning/pipeline/completed/565-system-one-layer.spec.md]
---

## Title
Jev through Cloudflare Workers AI, as a provider setting. Marley's System One layer reaches Jev
through TypeSafe's own API, which keeps requests "for as long as necessary"; Cloudflare Workers AI
resells the same model and states zero retention. A `cloudflare` provider, chosen for every project
or for listed projects, sends a project holding other people's data through Cloudflare while the
rest go direct, with no code change. Chad picked the ticket up on 2026-10-05.

## Scope
### In
- **The provider.** `cloudflare` in `marley.system_one.provider`, beside `typesafe`, `compatible`,
  `rules` and `replay`.
- **Per project.** `marley.system_one.provider_by_project`, a map of folder (`~/` for home) to
  provider; a local project whose main folder is the folder or inside it takes that provider, the
  longest folder winning; every other project takes `provider`. Only the user's own settings set it.
- **Cloudflare's request.** `POST {cloudflare_api}/accounts/{cloudflare_account_id}/ai/run` with
  `Authorization: Bearer <token>` and `{"model": "typesafe/jev", "input": {"state", "questions"}}`,
  the questions as Jev's own API takes them; the answer read from the REST envelope's `result`, its
  `errors` read when `success` is false.
- **The settings.** `cloudflare_account_id` (also on the settings page) and `cloudflare_api`
  (`https://api.cloudflare.com/client/v4` by default; https, or http on this machine, as
  `compatible`'s endpoint).
- **The token.** `MARLEY_CLOUDFLARE_API_TOKEN`, else the system keyring at Cloudflare's URL for the
  account; read only while the layer is on and some project uses Cloudflare. Its source shows on
  the System One calls tab, never its value; Set Cloudflare Token and Forget Cloudflare Token
  there. Every state sent is masked with it, as with the direct key.
- **Failures.** No account id or no token reads `Unavailable` with the reason and sends nothing; a
  401 or 403 from Cloudflare marks the token refused until it changes; Cloudflare's own error
  message is kept, cut and masked, as TypeSafe's body is.
- **The log.** Each call's row names its provider (`cloudflare`) and the model asked
  (`typesafe/jev`); the check's toast names the provider.
- The settings default file, the guide page and the docs.

### Out (explicitly deferred)
- **Cloudflare AI Gateway** (its own URL shape and caching); `cloudflare_api` can point at another
  base, but only the Workers AI REST path is built.
- **A per-provider breaker, rate cap or budget.** One gate stands in front of every provider: the
  day's budget is the whole layer's, and five failures in a row hold every provider's calls.
- **The project lists on the settings page.** `projects`, `metadata_only_projects` and
  `provider_by_project` stay JSON, as #565's lists are.
- **Choosing the model on Cloudflare.** Cloudflare names one model, `typesafe/jev`; the `model`
  setting stays TypeSafe's version pin.

## Reference (§20)
N/A — Marley-specific: no editor or terminal behaviour to match. The published contract is
Cloudflare's: the model page (developers.cloudflare.com/ai/models/typesafe/jev/, read 2026-10-05:
`POST https://api.cloudflare.com/client/v4/accounts/$CLOUDFLARE_ACCOUNT_ID/ai/run`, `Authorization:
Bearer $CLOUDFLARE_API_TOKEN`, body `{"model": "typesafe/jev", "input": {"state", "questions"}}`,
the answer's `model`, `answers` and `usage.input_tokens`, zero data retention, $0.042 per million
input tokens) and the Workers AI REST page (developers.cloudflare.com/workers-ai/get-started/
rest-api/: the generic `/ai/run` takes the model in the body and answers `{"result", "success",
"errors", "messages"}`).

### Prior art
- **Behaviour maps:** none; `docs/warp_architecture/`, `docs/zed_architecture/` and
  `docs/orca_architecture/` hold no model gateway. The Jev note
  (`docs/planning/design-notes/jev-system-one-2026-09-25.md:74-77`, `:273`) names Cloudflare for its
  retention.
- **Published material:** the two Cloudflare pages above. L-565 records Jev's answer shape from
  Chad's own runs; Cloudflare's page gives the same `answers` and `usage`.
- **The code we ship:** `marley_system_one::request` builds Jev's question set and parses its
  `answers` (`request.rs:22-61`, `:138-168`), reused whole: Cloudflare's `input` is the same
  `state` and `questions`. `marley_workbench::system_one` already posts with `http_client`, a
  deadline and a retry (`system_one.rs:892-947`), reads the key from the environment or gpui's
  keychain (`:333-393`), and validates a loopback `http` endpoint (`:484-508`).
  `agent_permissions_by_project` (`settings_content/src/marley.rs:149-155`,
  `marley_workbench/src/agents.rs:204-282`) is the per-project pattern: a folder map, `~/`
  expanded by `system_one::folder_path`, the longest folder winning. The settings page's dropdown
  lists any new `SystemOneProvider` variant itself (`settings_ui.rs:565`, `:5088-5130`). No crate
  we build talks to Cloudflare; Cloudflare's SDKs are JavaScript and Python, and #565 rejected
  bringing a client SDK's runtime into a gpui app (AD-565).

## UI proof
`script/e2e/548-system-one-via-cloudflare.sh` (`compositor sway`: it opens the settings dropdown).
Setup: two scratch repositories, `direct` and `private`, both in `marley.system_one.projects`;
`provider` `compatible` at a loopback fake's `/v1/systemone` (TypeSafe's own URL cannot be pointed
anywhere else), `provider_by_project` sending `private` through `cloudflare`, `cloudflare_api` at
the same fake's `/client/v4`, a made-up 32-hex `cloudflare_account_id`; `MARLEY_SYSTEM_ONE_KEY` and
`MARLEY_CLOUDFLARE_API_TOKEN` made-up values in the environment, never the keyring (L-565). The
fake (Python, as #565's) answers TypeSafe's shape at `/v1/systemone` and Cloudflare's envelope at
`/client/v4/accounts/<id>/ai/run`, logs each request's path, headers and body, and answers 403 with
Cloudflare's error envelope when the state holds `refuse-token`. Shots:
- `548-01-search`: the Settings window searched for "Cloudflare": Provider and the Cloudflare
  Account ID item.
- `548-01-providers`: the Provider dropdown open: Typesafe, Compatible, Rules, Replay, Cloudflare.
- `548-02-direct`: in `direct`'s terminal, `false`, then `marley: system one check`: the toast's
  provider `compatible`.
- `548-03-cloudflare`: the same in `private`'s terminal: the toast's provider `cloudflare`; the
  fake's log holds one Cloudflare request with the account's path, the token's bearer and
  `typesafe/jev` (an `expect`).
- `548-04-calls`: `marley: open system one calls`: the header's provider line and a Cloudflare line
  (the account, the token from `MARLEY_CLOUDFLARE_API_TOKEN`); the two rows, `compatible` and
  `cloudflare`.
- `548-05-refused`: in `private`, `false refuse-token`, then the check: the toast reads unavailable
  with Cloudflare's message; the calls tab's Cloudflare line reads refused.
- `548-06-no-account`: `cloudflare_account_id` emptied from outside; the check in `private`: the toast
  reads refused, naming the missing account id; the fake's log holds no new request (an
  `expect`).

## Locked-In Decisions
- D1 — **`cloudflare` is a fifth `SystemOneProvider`.** The dropdown lists it with no settings-UI
  code. Rejected: `compatible` with a Cloudflare URL (Cloudflare's body, path and envelope differ
  from TypeSafe's).
- D2 — **Per project by a folder map, `provider_by_project`,** the longest folder winning, as
  `agent_permissions_by_project` does; any provider may be named. Only the user's settings set it.
  Rejected: a `cloudflare_projects` list (it names one provider; the map reads the same and covers
  a project kept on `rules`); a project's `.zed/settings.json` (a cloned repository could route its
  owner's state).
- D3 — **The account id is a setting, the token a secret.** `cloudflare_account_id` names the path
  and is no secret; the token comes from `MARLEY_CLOUDFLARE_API_TOKEN`, else the keyring at the
  account's `/ai/run` URL. Not `CLOUDFLARE_API_TOKEN`: a wrangler token there may hold far more than
  Workers AI.
- D4 — **Two key slots.** The direct key (`typesafe`, `compatible`) and the Cloudflare token are
  loaded, shown, refused and masked apart; each is read only while the layer is on and some project
  or the default uses its provider. Every state is masked with both.
- D5 — **The body is Jev's own.** `request::build`'s `state` and `questions` go inside `input`, with
  `model: "typesafe/jev"`; `model` (TypeSafe's version pin) is not sent to Cloudflare. The row
  records `typesafe/jev`.
- D6 — **The envelope is unwrapped in `marley_system_one::request`** (`parse_cloudflare`): `result`
  goes to the same parse; `success: false` gives the first error's message. `parse` itself is left
  alone, since replay builds its answers through it.
- D7 — **A 401 or a 403 from Cloudflare refuses the token**; TypeSafe keeps 401 alone. Cloudflare's
  API answers a bad or under-scoped token with 403 ("Authentication error") as well as 401, and
  either means the next call would fail the same way.
- D8 — **`cloudflare_api` is a setting**, `https://api.cloudflare.com/client/v4` by default, checked
  as `compatible`'s endpoint is (https, or http on this machine). It lets a scenario reach a fake
  with no test-only code path.
- D9 — **One gate for every provider:** the budget, the breaker, the rate cap and the repeat check
  stay the layer's. Cloudflare's price is TypeSafe's, $0.042 per million input tokens.
- D10 — **The calls tab shows both.** The header keeps the default provider's line and key, and
  adds a Cloudflare line (account, token source) with Set Cloudflare Token and Forget Cloudflare
  Token while any project or the default uses Cloudflare. No scenario presses a keyring button
  (L-565).

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method: a named shot of the visual check,
the review of the diff, or the gate's exit code.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The settings page's Provider shall offer Cloudflare beside Typesafe, Compatible, Rules and Replay. | Shot `548-01-providers` |
| REQ-002 | The settings page's System One section shall hold a Cloudflare Account ID item. | Shot `548-01-search` |
| REQ-003 | WHERE `provider_by_project` names a folder holding a project's main folder, the system shall ask that project's questions through the named provider, the longest such folder winning. | Shots `548-02-direct`, `548-03-cloudflare`; the fake's log |
| REQ-004 | WHERE no entry of `provider_by_project` holds a project, the system shall ask through `provider`. | Shot `548-02-direct`; the fake's log |
| REQ-005 | WHEN a question goes through Cloudflare, the system shall POST to `{cloudflare_api}/accounts/{cloudflare_account_id}/ai/run` with the token as a bearer and a body of `model` `typesafe/jev` and `input` holding the state and the questions. | The fake's log (`expect`) |
| REQ-006 | WHEN Cloudflare answers with `success` and a `result`, the system shall read the reading from `result`'s `answers` and the cost from its `usage.input_tokens`. | Shot `548-03-cloudflare`; shot `548-04-calls` |
| REQ-007 | Each call's row in the day's file and on the calls tab shall name its provider and model. | Shot `548-04-calls`; the day's file (`expect`) |
| REQ-008 | WHILE some project or the default uses Cloudflare, the calls tab shall show the account and where the token came from, never the token. | Shot `548-04-calls` |
| REQ-009 | IF Cloudflare answers 401 or 403, THEN the system shall read the call unavailable with Cloudflare's message, mark the token refused, and send nothing through Cloudflare until the token changes. | Shot `548-05-refused` |
| REQ-010 | IF no account id is set, THEN a question through Cloudflare shall read refused with the reason, as `compatible` with no endpoint does; IF no token is found, THEN it shall read unavailable with the reason; and no request shall be sent. | Shot `548-06-no-account`; the fake's log (`expect`) |
| REQ-011 | The system shall mask every state it sends with the Cloudflare token as well as the direct key. | Review |
| REQ-012 | WHILE the layer is off or no project and not the default uses Cloudflare, the system shall not read the Cloudflare token from the environment or the keyring. | Review |
| REQ-013 | `cloudflare_api` shall be refused unless it is https, or http on this machine, with the reason as the call's. | Review |
| REQ-014 | A project's `.zed/settings.json` shall not set `provider_by_project`. | Review (the field's settings scope) |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the touchpoint rows widened first; `SystemOneProvider::Cloudflare` and the three
  settings with `default.json`; `request::build_cloudflare` and `parse_cloudflare`; the workbench's
  provider per project, the two key slots, the Cloudflare endpoint, refusal and model; the calls
  tab's Cloudflare line and buttons; the settings page's item; the guide page; a review;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), the guide, the walkthrough, the plan's S1
  row, ledger capture (§19), the brain decision, close the ticket, archive, commit.
