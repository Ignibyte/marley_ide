---
pipeline_id: b110f49b-4276-4a85-84cb-efaa86e4ccdb
ticket: docs/planning/tickets/open/TICKET-562-orca-redactor-cross-check.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Orca's redactor as a cross-check for #516's rules"
type: chore
slice: prong 2, the MCP server's tools (#516's redactor); the Orca second pass, smaller item 3
references: [docs/planning/design-notes/orca-second-pass-2026-09-25.md, docs/planning/pipeline/completed/516-secret-redaction-for-agents.spec.md, docs/orca_architecture/07-engineering-and-changelog.md]
---

## Title
#516's Test phase compared Marley's redactor with Orca's in a night and took two rules from it.
This chore does the comparison rule by rule, writes the table down, and adds what Marley still
lacks: an `Authorization` header whose scheme is `Basic`, `Token` or `Digest` (only the `Bearer`
form is hidden today, and `Basic dXNl…` leaks its credential after the scheme word), hyphenated
secret names (`x-api-key`, `private-key`, `access-key`, which `API_?KEY` and its siblings do not
match), the labels `authorization`, `bearer` and `privkey`, and a `cookie` rule that hides a
cookie header's whole value. Every kind Marley names today keeps its marker, so #516's golden
scenario stays true.

## Scope
### In
- **The comparison**, the table below, kept in `docs/marley_architecture/marley_mcp.md`'s
  Redaction section at Complete.
- **`crates/marley_mcp/src/redact.rs`**:
  - the `secret` rule's name alternation takes `API[_-]?KEY`, `PRIVATE[_-]?KEY`, `ACCESS[_-]?KEY`,
    and the names `AUTHORIZATION`, `BEARER` and `PRIVKEY`;
  - a new `authorization` rule, after the `bearer token` rule: `(?i)(\b(?:proxy-)?authorization["']?[ \t]*[=:][ \t]*(?:(?:basic|bearer|token|digest|negotiate|ntlm|apikey)[ \t]+)?)("[^"\n]*"|'[^'\n]*'|[^\s"',;]+)`,
    keeping group 1 (the label and the scheme word) and hiding the credential as
    `[redacted: authorization]`; a credential already hidden by an earlier rule is left alone,
    as `apply` does for every rule;
  - a new `cookie` rule: `(?i)(\b(?:set-)?cookie["']?[ \t]*[=:][ \t]*)("[^"\n]*"|'[^'\n]*'|[^\n]+)`,
    keeping the label and hiding the rest of the line as `[redacted: cookie]`, since a cookie
    line is a list of credentials.
- **The proof**: `script/e2e/562-orca-redactor-cross-check.sh`, machine checks through the
  stand-in agent's `terminal-read` (as #516's), the fakes put together at run time; and
  `516-secret-redaction-for-agents.sh` rerun unchanged.

### Out (explicitly deferred)
- Orca's `.env` rule (every `NAME=value` line hidden): #516's reason stands, `env` output would
  lose `PATH` and every other harmless value.
- Orca's PEM rule for every `BEGIN … END` block (certificates, public keys, requests): they are
  not secrets, and an agent reading a certificate chain needs it. Marley hides private keys.
- Orca's attribute-key drops (`env`, `headers.authorization`, `install_id`): structured
  telemetry, not text.
- Splitting `api key` into Orca's `anthropic-key` and `openai-key` tags: the kind tells the agent
  something was there, the vendor changes nothing it can do, and #516's golden scenario names
  `api key`.
- A rule on a value's shape alone for AWS secret keys (40 base64 characters with no label):
  neither redactor has one; it would hit every base64 run.
- Masking on screen (Warp's second half) and the harness's `session_read` (#516's Out).

## The comparison (Orca's `redactor.ts` against Marley's `redact.rs`)

| Orca's rule | Marley today | This chore |
|---|---|---|
| Rule 1, `LABELED_KV`: the labels `api[-_]?key`, `token`, `secret`, `password`, `bearer`, `authorization` with `:` or `=`, the value `Bearer \S+`, `Token \S+` or `\S+`; the label goes too | The `secret` rule keeps the name and hides the value, for names holding `SECRET`, `TOKEN`, `PASSWORD`, `PASSWD`, `PASSPHRASE`, `API_?KEY`, `PRIVATE_?KEY`, `ACCESS_?KEY`, `CREDENTIALS?`, in `NAME=value`, `NAME: value` and `export NAME=value` forms, quoted or bare | Added: the hyphen forms (`api-key`, `x-api-key`, `private-key`, `access-key`), the labels `authorization`, `bearer`, `privkey`; the `authorization` rule for a two-word credential (`Basic …`, `Token …`, `Digest …`), which the `secret` rule's bare value would cut at the space |
| Rule 2, `anthropic-key` `sk-ant-…{40,}` | `api key`: `sk-(?:ant-\|proj-)?…{20,}`, wider | Same (the tag split is Out) |
| Rule 2, `openai-key` `sk-(?:proj-)?…{32,}` | The same `api key` rule | Same |
| Rule 2, `github-token` `gh[pousr]_…{36,}` | `github token`: the same, plus `github_pat_` | Same, wider |
| Rule 2, `aws-access-key-id` `AKIA…{16}` | `aws key id`: `AKIA` and `ASIA` | Same, wider |
| Rule 2, `aws-secret-access-key` `aws_secret_access_key\s*[:=]\s*…{40}` | The `secret` rule (the name holds `SECRET` and `ACCESS_KEY`) | Same |
| Rule 2, `jwt` `eyJ…{10,}\.…{10,}\.…{10,}` | `jwt`: the payload must start `eyJ` too (JSON's `{"`), `{8,}` | Same in practice |
| Rule 2, `slack-token` `xox[baprsoe]-…{10,}` | `slack token`: the same set (#516 took `e` and `o`) | Same |
| Rule 2, `pem`: any `-----BEGIN … END-----` block | `private key`: private-key blocks only, the PGP `BLOCK` form, and a block with no END yet | Left out (certificates and public keys are not secrets) |
| Rule 3, `URL_USERINFO` `(https?://)([^/@\s]+)@`: the whole userinfo | `url password`: `user:pass@` keeps the user; a bare token of 20 or more `[A-Za-z0-9_-]` before `@` | Kept as Marley's: a bare user name (`https://admin@host/`) is not a secret |
| Rule 4, `ENV_LINE` `^\s*[A-Z_][A-Z0-9_]*\s*=\s*\S.*` | Secret-named values only | Left out (#516) |
| The attribute blocklist: `cookie`, `set-cookie`, `proxy-authorization`, `headers.authorization`, `env`, `install_id`, … | None (structured data) | Added for text: the `cookie` rule; `proxy-authorization` in the `authorization` rule |
| The key-family drop `apikey\|token\|secret\|password\|authorization\|bearer\|privkey\|privatekey` | `APIKEY` through `API_?KEY`; `PRIVATEKEY` through `PRIVATE_?KEY` | Added: `AUTHORIZATION`, `BEARER`, `PRIVKEY` |
| Idempotent passes (three locations) | `apply` leaves a value that is a marker already | Same |
| Tags `[redacted:<tag>]` | Kinds `[redacted: <kind>]` | Same shape; the kind names stay Marley's |

## Reference (§20)
Warp's Secret Redaction as #516 cites it (a recommended set of regexes plus the user's own,
before anything reaches a model). Orca's redactor (report 07 §2's observability lane, "a local
NDJSON trace sink and a redactor"; the second pass, item 3): five ordered rule families with
tags, run at three locations and idempotent (`src/main/observability/redactor.ts`, MIT, read);
#516 matched its provider shapes in Test and took two rules; this chore takes what a rule-by-rule
reading still finds. Upstream Zed: nothing redacts what its agents read (#516). The rules are
regexes on the token formats their issuers publish, reimplemented, not copied.

### Prior art
- **Behavior maps and reports.** The second pass, item 3 (the tags "tell a reader what kind of
  value was hidden"; the two rules #516 took; its `.env` rule left out and why). Report 07's
  observability section. Orca's `redactor.ts`, read at `1c2cf120e3`: `LABELED_KV` (13-14, with
  the comment that `\b` stops it stealing `FOO_SECRET=` from rule 4 and that the value
  alternation eats the whole `Bearer <jwt>` segment), `PROVIDER_PATTERNS` (17-36, most specific
  first so `sk-ant-` keeps its tag; the lazy PEM body so two blocks redact apart),
  `URL_USERINFO` (39, "bare-token `<pat>@` seen in failing git stderr"), `ENV_LINE` (42),
  `CLIENT_ATTR_BLOCKLIST` (49-67), the key-family regexes (86-89), `redactString`'s order
  (99-120: labeled pairs, providers, userinfo, `.env`), and the idempotence that makes the
  three-location placement safe (5-7).
- **Published material.** RFC 7235 (HTTP authentication: `Authorization` and
  `Proxy-Authorization` carry a scheme and credentials, `Basic` per RFC 7617, `Bearer` per RFC
  6750, `Digest` per RFC 7616); RFC 6265 (`Cookie` and `Set-Cookie`); GitHub's `Authorization:
  token <pat>` form in its REST docs; the issuers' token formats #516 cites; gitleaks' rules as
  the gate's own cross-check.
- **The code we already ship.** `crates/marley_mcp/src/redact.rs`: `Rule { kind, regex, keep,
  keep_after }` (22-28), `BUILT_IN` in its order (32-97: `private key`, `secret` at 40-45,
  `bearer token` at 46-51, the two `url password` rules, `aws key id`, `github token`, `slack
  token`, `stripe key`, `google api key`, `api key`, `jwt`), `BUILT_IN_RULES` (101-113),
  `Redactor::new` (135-148) and `redact` (152-166), `apply` (171-194) with the marker skip at
  180-185. The `secret` rule's name pattern is `[A-Z0-9_.-]*(?:SECRET|TOKEN|…|API_?KEY|
  PRIVATE_?KEY|ACCESS_?KEY|CREDENTIALS?)[A-Z0-9_.-]*`: the prefix and suffix admit hyphens, the
  core alternatives do not, so `x-api-key` misses while `x_api_key` hits; the bare value
  alternation `[^\s"',;]+` stops at the space in `Basic dXNl…`; `AUTHORIZATION` is not among the
  names. The scenario `script/e2e/516-secret-redaction-for-agents.sh`: `KINDS` (26-27), the
  bearer line `Authorization: Bearer <fake>` (61, 76), the count `12 in the read, 1 in the list`
  (171), the fakes list (90); it is golden (`script/e2e/golden`). `AD-claude-516`: redact at the
  tool boundary, on by default; `PR-claude-redact-the-whole-text-before-cutting-it-001`. Does a
  crate we build own the seam? `marley_mcp::redact` owns it; the change is two rules and six
  names inside it.

## UI proof
N/A — no UI delta: what agents read changes, nothing on screen does. The proof is
`script/e2e/562-orca-redactor-cross-check.sh`: the terminal prints, from pieces assembled at run
time, `Authorization: Basic <credential>`, `Authorization: Token <token>`,
`Proxy-Authorization: Basic <credential>`, `x-api-key: <value>`, `api-key=<value>`,
`PRIVATE-KEY=<value>`, `ACCESS-KEY=<value>`, `PRIVKEY=<value>`, `BEARER=<value>`, `Cookie:
session=<value>; theme=dark`, `Set-Cookie: sid=<value>; Path=/`, and the two lines that must
stay, `Authorization: Bearer <fake>` and `PATH=/usr/bin:/bin`; the stand-in agent's
`terminal-read` answer is checked with `expect` for each marker, for no fake as printed, for
`PATH=/usr/bin:/bin` and `[redacted: bearer token]` as before, and for the count. Its e2e run
also includes `just shot`, since the change drives no UI, and `516-secret-redaction-for-agents.sh`
green and unchanged.

## Locked-In Decisions
- D1: Add what Orca finds and Marley misses, never what #516 rejected: the `.env` rule stays
  out, and so does any rule on a value's shape alone that would hit ordinary base64.
- D2: Every kind Marley names today keeps its marker and its place in the order. The
  `authorization` rule runs after `bearer token` and keeps the scheme word, so `Authorization:
  Bearer <token>` still reads `[redacted: bearer token]` once, #516's golden checks stay true,
  and an agent still sees `Basic` or `Token` without the credential.
- D3: A cookie header loses its whole value: a `Cookie:` line is a list of credentials, and
  hiding the first `name=value` up to the `;` would leave the rest.
- D4: Kinds stay coarse: `api key`, not `anthropic key` and `openai key`.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent reads a block whose output holds `Authorization: Basic <credential>`, `Authorization: Token <token>` or `Proxy-Authorization: <scheme> <credential>`, Marley shall return the label and the scheme with `[redacted: authorization]` in the credential's place. | The run log (`mcp_agent terminal-read`) |
| REQ-002 | WHEN the output holds `x-api-key: <value>`, `api-key=<value>`, `PRIVATE-KEY=<value>`, `ACCESS-KEY=<value>`, `PRIVKEY=<value>` or `BEARER=<value>`, Marley shall return the name with `[redacted: secret]` in the value's place. | The run log |
| REQ-003 | WHEN the output holds a `Cookie:` or `Set-Cookie:` header, Marley shall return the label with `[redacted: cookie]` in place of the whole value. | The run log |
| REQ-004 | WHEN the output holds `Authorization: Bearer <token>`, Marley shall return `[redacted: bearer token]` once, as before. | The run log; `516-secret-redaction-for-agents.sh` green |
| REQ-005 | WHEN the output holds `PATH=/usr/bin:/bin`, Marley shall return it as printed. | The run log |
| REQ-006 | The spec's table shall name every rule of Orca's redactor with Marley's answer. | Review |
| REQ-007 | The diff gate shall be green and #516's golden scenario shall pass unchanged. | `script/gates.sh --diff`; the scenario's exit |

## Phase Plan
- **P1 Plan:** this spec, with the comparison.
- **P2 Code:** the names and the two rules in `redact.rs`, the module's doc comment; fmt and
  clippy clean; a review of the diff against each of #516's fakes (none may match twice).
- **P3 Test:** write and run the scenario and `just shot`; run `516-secret-redaction-for-agents.sh`
  unchanged; `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; the table into `docs/marley_architecture/marley_mcp.md`; the ledger
  capture; close the ticket, archive, commit.
