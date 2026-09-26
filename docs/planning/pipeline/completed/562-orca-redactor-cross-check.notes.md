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

## Promotion (2026-09-26, at `0b53d83ec0`)
- **Seams re-read:** `crates/marley_mcp/src/redact.rs` as cited: `Rule` (19-28), `BUILT_IN`
  (30-97: `private key`, `secret` at 40-45, `bearer token` at 46-51, the two `url password`
  rules, the provider shapes, `jwt`), `BUILT_IN_RULES` (99-113), `redact` (150-166), `apply`
  (169-194) with the marker skip; `script/e2e/516-secret-redaction-for-agents.sh`'s fakes and
  lines (the one `Authorization: Bearer` line, no cookie, no hyphenated name).
- **A correction to the plan:** `AUTHORIZATION` does not join the `secret` rule's names. That rule
  runs before `bearer token`, and on `Authorization: Bearer <token>` its bare value stops at the
  space, so it would hide `Bearer` and leave the token. The `authorization` rule takes every
  `authorization` label (`authorization=<value>` too), after `bearer token`, whose marker it skips.
- **The cookie rule's place:** before `secret`, so a cookie line holding `token=<value>` is hidden
  once, as a cookie, rather than first as a secret and again as the rest of the line.
- **Brain consultation 3fd14448c0d944088047a2bc5e11893f:** nothing on this seam.

## Phase 2 — Code
- **Built** (`crates/marley_mcp/src/redact.rs`, the one file):
  - the `secret` rule's names take `API[_-]?KEY`, `PRIVATE[_-]?KEY`, `ACCESS[_-]?KEY`, `BEARER`
    and `PRIVKEY`;
  - a `cookie` rule, second in the order (after `private key`, before `secret`);
  - an `authorization` rule after `bearer token`, keeping the label and the scheme word;
  - the module's doc comment names the two new kinds; each new entry's comment says why it sits
    where it does.
- **Deviations from the plan, and why:**
  - `AUTHORIZATION` is not a `secret` name (the promotion's correction): on `Authorization: Basic
    <credential>` that rule's bare value is `Basic`.
  - The `authorization` rule's optional scheme may follow a quote (`["']?(?:basic|…)[ \t]+`), so
    `"Authorization": "Bearer <token>"` in JSON keeps its bearer marker and counts once, and
    `"Authorization": "Basic <credential>"` keeps `"Basic `; the plan's form took the whole quoted
    value as a second credential there (a double count and the scheme lost).
  - The `authorization` rule's value takes an auth-param list (`name=value` or `name="value"`,
    comma-separated) before the bare token, so a Digest header (`username="…", response="…"`)
    is hidden whole: the plan's bare value stopped at the first quote and left `response` and
    the nonces.
  - The `cookie` rule's unquoted value stops at a quote before a space or the line's end, so
    `curl -H "Cookie: …" <url>` keeps its closing quote and the URL; the plan's `[^\n]+` took the
    URL too. A cookie value cannot hold a space or a bare double quote (RFC 6265's
    cookie-octet), so nothing of a cookie is left.
- **Review of the diff:**
  - The rule order was modelled on sample lines with a scratch copy of `apply` in Python, which
    reads `BUILT_IN` from the file; its `re` and Rust's `regex` agree on these patterns. Every
    REQ line came back as designed, each hidden once.
  - #516's fakes, each against the new rules: the bearer line is hidden once (the
    `authorization` rule sees the marker and skips it), and no other line changes.
  - `curl --cookie=<value> <url>` loses the URL (the value runs to the end of the line). That is
    a false positive, which AD-claude-516 accepts; curl's own spelling, `-b` or `--cookie <value>`,
    has no `=` or `:`, so the rule leaves it alone.
  - An AWS SigV4 header (`AWS4-HMAC-SHA256 Credential=…, Signature=…`) hides only its scheme
    word, since that scheme is not on the list. The key id is still hidden by `aws key id`, and
    the signature lasts for one request. Neither redactor had a rule for it; it stays out.
  - Provenance: the patterns follow RFC 7235, 7617, 7616 and 6265. Orca's `redactor.ts` (MIT)
    was read for the comparison and nothing was copied from it. No Zed path is touched, so the
    ledger has no rows.
  - `cargo fmt -p marley_mcp -- --check` is clean, and `cargo clippy -p marley_mcp --all-targets
    -- -D warnings` is clean.

## Phase 3 — Test
- **The scenario:** `script/e2e/562-orca-redactor-cross-check.sh` (`compositor sway`, no clicks
  and no Chromium). Its setup writes `secrets.sh` from pieces, plus `fakes.txt`, `expected.txt`
  and `count.txt`. The script prints:
  - five `Authorization` forms: Basic, Token, Proxy-Authorization, a JSON body with a Basic
    credential whose base64 ends in one `=`, and a Digest header of six parameters;
  - the six added names;
  - `Cookie`, `Set-Cookie`, and a cookie in a `curl -H` argument;
  - #516's bearer line;
  - `PATH=/usr/bin:/bin` and a plain line.

  The stand-in agent reads the block with `mcp_agent terminal-read secrets.sh`. The checks: every
  expected line is a whole line of the read, no fake comes back, `[redacted: bearer token]`
  appears once, and the count line is `redacted: 15 in the read, 0 in the list`.
- **Before the run:** the Python model of the rule order (L-claude-562-model-…) ran on the
  scenario's lines. It showed the JSON line with a one-`=` credential losing its `", "`
  (F-claude-562-a-digest-list-…), which was fixed in `redact.rs` before the build.
- **Green, debug build at the change:** all four checks pass. The read is exactly
  `expected.txt`: each credential as `[redacted: authorization]` after its label and scheme, the
  JSON body's `", "Accept": "application/json"}` kept, each name with `[redacted: secret]`, each
  cookie value as `[redacted: cookie]` with the curl line's `" https://api.example.test/v2` kept,
  the bearer line once, and `PATH` and the plain line as printed (REQ-001 to REQ-005).
- **Red, the installed build (`d0939a6fc4`, before the change, `E2E_BINARY=~/.local/bin/marley`):**
  `check each line reads as expected: FAIL`, with 13 of the 15 lines missing from the read. Only
  #516's bearer line was hidden. The Basic, Token, Digest, cookie and hyphenated-name values came
  through as printed.
- **Shots, read:**
  - `562-00-printed`: Marley's terminal after `sh secrets.sh`, every fake as printed, which is
    right because the buffer is never changed. The Digest line wraps onto a second row on screen,
    and the read still returned it as one line (L-claude-562-a-block-read-joins-…).
  - `just shot 562-no-ui-delta`: Marley drawn as before from the profile copy; the focus report
    says the user's window and workspace were as they were. The shot held the profile copy's own
    workspace and was deleted after reading.
  - Neither shot leaves the scratchpad.
- **The golden set, with 562 added (`just regress`, the debug build):** all 21 pass. #516's
  scenario is unchanged and still green: `check on: pass (redacted: 12 in the read, 1 in the
  list)`, `off` 0 and 0, `pattern` 13 and 1, and the console on and off (REQ-004, REQ-007). 562
  itself passed in 20 s.
- **The focus report:** each sway run stopped with its Marley, pointer and keyboard. The
  Hyprland check found no Marley window before or after the run, and nothing was added or
  reloaded.
- **The gate:** `script/gates.sh --diff` gave `GATE GREEN [diff]`, 16 passed and 0 failed
  (rustfmt, clippy on every target in scope, cargo-audit, cargo-deny, cargo-shear, gitleaks,
  shellcheck, no-suppressions, source-bans, docs, zed-ledger, manifests, typos, semgrep, dylint,
  the receipt). The receipt matches the tree.
- **Pre-existing, not in scope:** dylint's `SharedString` construction warnings in `sidebar` and
  `title_bar` (Zed crates, warnings, not failures).
- **Verdict:** PASS. REQ-001 to REQ-005 by the scenario's read; REQ-006 by the spec's table,
  which names every Orca rule with Marley's answer; REQ-007 by the gate and #516 in the golden
  set.

## Phase 4 — Complete
- **Documented:**
  - `CHANGELOG.md`, under Changed: "More secrets hidden from what agents read".
  - `docs/marley_architecture/marley_mcp.md`, the Redaction section: the rule order with the two
    new rules, why the order is load-bearing, and the comparison with Orca's redactor.
  - `docs/marley/three-prong-plan.md` has no row for this chore. No Zed path is touched, so the
    touchpoints ledger has no rows to check.
- **Knowledge appended:**
  - `F-claude-562-a-one-word-value-rule-would-have-hidden-the-scheme-and-left-the-credential-001`
  - `F-claude-562-a-digest-list-read-base64-padding-as-a-parameter-001`
  - `PR-claude-a-labels-values-decide-which-redaction-rule-owns-it-001`
  - `L-claude-562-model-a-redaction-rule-change-in-python-before-the-build-001`
  - `L-claude-562-a-block-read-joins-a-wrapped-line-before-redaction-001`
  - `AD-claude-562-authorization-and-cookie-headers-have-rules-of-their-own-001`
- **Brain:** consultation `3fd14448c0d944088047a2bc5e11893f` was closed by `brain decide`
  (`decisions/marley-hides-every-authorization-schemes-credential-and-a-cookie-headers-whole-value-from-agents`,
  follow-up 2026-10-26). #516's decision page keeps its follow-up of 2026-10-09 open: it asks
  whether on-by-default redaction gets in agents' way, which two weeks of use answers and this
  chore does not.
- **Closed:** TICKET-562 moved to `tickets/closed/`, and its BACKLOG row went at promotion.
