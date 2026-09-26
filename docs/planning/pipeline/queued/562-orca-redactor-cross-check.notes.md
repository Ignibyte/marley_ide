# Orca's redactor as a cross-check for #516's rules — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-562-orca-redactor-cross-check.md
- **Pipeline spec:** 562-orca-redactor-cross-check.spec.md

## Phase 1 — Plan
- **Request:** the Orca second pass of 2026-09-25, the five smaller details, item 3: Orca's
  redactor (`src/main/observability/redactor.ts`, 308 lines) as a cross-check for #516; #516
  recorded a first comparison in its spec's prior art and took two rules. Chad decided on
  2026-09-26 that every remaining Orca and Warp finding gets built.
- **Classification / tier:** chore, S. One Marley file (`crates/marley_mcp/src/redact.rs`), one
  scenario, one doc.
- **Recall (§18.3):**
  - AD-claude-516-redact-at-the-tool-boundary-on-by-default-001: the rules favour hiding; a
    false positive costs an agent a value and never leaks one.
  - PR-claude-redact-the-whole-text-before-cutting-it-001 and F-claude-516: redaction runs over
    the whole text before any cut; unchanged here.
  - L-claude-516-fake-secrets-are-put-together-at-run-time-001: the scenario assembles its fakes
    at run time so gitleaks (gate:10) finds none in the file.
  - #516's Test entry: "from the Orca comparison, the private-key rule takes the PGP `BLOCK` form
    and a block with no END line yet; a bare token as a URL's userinfo is a `url password`;
    Slack's `xoxe-`/`xoxo-` prefixes."
  - Brain: not consulted in this drafting pass; promotion runs `brain_ask` (#516's decision page
    `decisions/marley-hides-secrets-in-what-its-tools-hand-agents-on-by-default` has a follow-up
    due 2026-10-09, which this chore can close).
- **Discovery (opened and checked, 2026-09-26):**
  - `crates/marley_mcp/src/redact.rs` (195 lines), read whole: the rules and lines in the spec's
    prior art. The `secret` rule (40-45): `(?i)(\b(?:export[ \t]+)?[A-Z0-9_.-]*(?:SECRET|TOKEN|
    PASSWORD|PASSWD|PASSPHRASE|API_?KEY|PRIVATE_?KEY|ACCESS_?KEY|CREDENTIALS?)[A-Z0-9_.-]*["']?[ \t]*
    [=:][ \t]*)("[^"\n]*"|'[^'\n]*'|[^\s"',;]+)`. The `bearer token` rule (46-51):
    `(?i)(\bbearer[ \t]+)[A-Za-z0-9._~+/-]{16,}=*`. `apply` (171-194): the kept prefix, the marker
    skip when the secret starts with `[redacted` after quotes, `keep_after`.
  - `/srv/stacks/orca-refs/orca/src/main/observability/redactor.ts` (309 lines), read whole: the
    lines in the spec's prior art.
  - `script/e2e/516-secret-redaction-for-agents.sh`: `KINDS` (26-27); the bearer fake `"Fake" * 6`
    (61) printed as `curl -H "Authorization: Bearer <fake>" https://api.example.test/v1` (76);
    the fakes list (90); the count checks (171-179); the command line `DEPLOY_TOKEN=<fake> sh
    secrets.sh` (213). Each of its fakes was read against the new rules: the `authorization`
    rule matches only the bearer line, whose credential is a marker already after the `bearer
    token` rule, so it is skipped; the `cookie` rule matches nothing there; the widened names
    match nothing new. The count stays 12 and 1.
  - `script/e2e/golden`: 516 is in the set.
  - `docs/marley_architecture/marley_mcp.md`: the Redaction section (#516) to carry the table.
- **Decisions:** D1 to D4 in the spec.

### Design
- **Approach.** In `BUILT_IN`, the `secret` rule's core alternation becomes
  `SECRET|TOKEN|PASSWORD|PASSWD|PASSPHRASE|API[_-]?KEY|PRIVATE[_-]?KEY|ACCESS[_-]?KEY|CREDENTIALS?|
  AUTHORIZATION|BEARER|PRIVKEY`. After the `bearer token` entry, the `authorization` entry
  (`keep: Some(1)`, no `keep_after`) and then the `cookie` entry (`keep: Some(1)`). Order in the
  table is the order of the run: the `secret` rule runs first and takes `authorization: xyz` as a
  secret-named value when the value is one bare word, which is right (the label says what it
  is); the `authorization` rule then takes only the two-word forms and quoted values, whose
  credential the `secret` rule cut short, and skips a credential that is a marker already. The
  module's doc comment names the two new kinds.
- **The scenario.** As #516's: a `secrets.sh` written at setup from pieces (`"Basic " +
  base64("user:" + random)`, `"Token " + random(30)`, hex values for the names, cookie lines with
  `theme=dark` and `Path=/` after the credential), run in the terminal, read with `mcp_agent
  terminal-read secrets.sh`; `expect` per marker, `grep -qF` of each fake failing, the two
  lines that must stay, and the count (`redacted: N in the read`, N fixed by the fakes' number).
- **File manifest.** `crates/marley_mcp/src/redact.rs`; `script/e2e/562-orca-redactor-cross-check.sh`;
  at Complete `docs/marley_architecture/marley_mcp.md` and `CHANGELOG.md`.
- **Ledger rows.** None.

### E2E plan
| REQ | Scenario part | Log |
|---|---|---|
| REQ-001 | three `Authorization` lines (`Basic`, `Token`, `Proxy-Authorization: Basic`) | `[redacted: authorization]` three times; the scheme words present; no credential |
| REQ-002 | `x-api-key: …`, `api-key=…`, `PRIVATE-KEY=…`, `ACCESS-KEY=…`, `PRIVKEY=…`, `BEARER=…` | six `[redacted: secret]`, each name present, no value |
| REQ-003 | `Cookie: session=…; theme=dark` and `Set-Cookie: sid=…; Path=/` | two `[redacted: cookie]`; `theme=dark` and `Path=/` absent |
| REQ-004 | `Authorization: Bearer <fake>` | `[redacted: bearer token]` once; no `[redacted: authorization]` on that line |
| REQ-005 | `PATH=/usr/bin:/bin` and a plain line | as printed |
| REQ-007 | `just gate-diff`; `just e2e script/e2e/516-secret-redaction-for-agents.sh` | the exits |

Nothing is out of a scenario's reach: the redactor is pure and the stand-in reads it through the
same tool an agent uses.

### Risks
- `AUTHORIZATION_SERVER_URL=https://…` is now hidden as a secret-named value (a URL lost, never
  leaked): #516's side of the trade, and the user's patterns cannot un-hide, so the name list
  should not grow further without a case.
- Prose that says `cookie: the browser keeps it` loses its clause: harmless; the `cookie` rule
  wants a `:` or `=` right after the word, which prose rarely has.
- A JSON body with `"Authorization": "Basic …"` is quoted: the `authorization` rule's quoted
  alternation takes the whole quoted value, scheme included, as the credential; the marker then
  replaces `Basic …` entirely. Acceptable, and the review checks the count on #516's fakes.
- The order matters and is easy to get wrong in a later edit: the module's comment says why the
  `authorization` rule follows `bearer token`, and #516's golden scenario catches a regression.
