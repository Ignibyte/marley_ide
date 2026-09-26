---
pipeline_id: 333dffec-31fd-443b-9923-21229dd14b4f
ticket: docs/planning/tickets/open/TICKET-516-secret-redaction-for-agents.md
status: Phase 4 — Complete PASS
title: "Secrets hidden from what agents read"
type: feature
slice: prong 2 (the MCP server's terminal tools) with prong 3's console; the Warp once-over item 2
references: [docs/planning/design-notes/warp-once-over-2026-09-25.md, docs/planning/pipeline/completed/515-marley-settings-page.spec.md]
---

## Title
What Marley's MCP tools give agents from terminals (`terminal_blocks`, `terminal_read`) and from
the browser's console (`browser_console`) passes through a redactor that replaces secrets with
`[redacted: <kind>]` and counts them; it is on by default, with a toggle on the Marley settings
page and user patterns in the settings file.

## Scope
### In
- `crates/marley_mcp/src/redact.rs` (new, pure): `Redactor` with the built-in rules and the
  user's patterns; `redact(&str) -> Redacted { text, count }`. Built-in rules: PEM private-key
  blocks; AWS access key ids; GitHub tokens (`ghp_`, `gho_`, `ghu_`, `ghs_`, `ghr_`,
  `github_pat_`); Slack tokens; Stripe secret keys; Google API keys; OpenAI and Anthropic keys
  (`sk-`, `sk-proj-`, `sk-ant-`); JWTs; `Authorization: Bearer` values; passwords in URLs; and
  values assigned to secret-named variables (`*TOKEN*`, `*SECRET*`, `*PASSWORD*`, `*API_KEY*`,
  `*PRIVATE_KEY*`, `*ACCESS_KEY*`) in `NAME=value`, `NAME: value` and `export NAME=value` forms.
- Settings: `marley.redact_secrets_for_agents` (default true) and `marley.redaction_patterns` (a
  list of regexes, default empty) in `crates/settings_content/src/marley.rs`; `MarleySettings`
  resolves them; a pattern that does not compile is left out and named in a toast.
- `crates/marley_workbench/src/mcp.rs`: `terminal_blocks` redacts each block's command,
  `terminal_read` its command and output, each answer carrying `redacted: <count>`;
  `crates/marley_workbench/src/browser_tools.rs`: `browser_console` redacts each entry's text.
- The toggle, "Redact Secrets for Agents", in an Agents section of the Marley settings page
  (`crates/settings_ui/src/marley_page.rs`).

### Out (explicitly deferred)
- Masking secrets on screen, in the terminal Chad reads (Warp's second half): M, later.
- Redacting what agents read outside Marley's tools (Claude Code reading a file itself, the Zed
  Agent's own file tools): those are the agent's, not Marley's.
- The harness's `session_read`: a request to rustal-harness when #534 lands.

## Reference (§20)
Warp's Secret Redaction (docs.warp.dev/support-and-community/privacy-and-security/secret-redaction/,
cited in the once-over note): a recommended set of regexes plus the user's own, applied before
anything reaches a model, off by default there. Marley matches the behavior for what its own
tools hand agents, and turns it on by default: Marley's agents are always a model away. Marley's
own prior art is the browser's `redact_url` (plan D15). Orca's grab redacts secret-looking
attribute values and query strings (docs/orca_architecture/03 §2.4).

### Prior art
- **Code we already ship.** `crates/marley_browser/src/observe.rs` (`redact_url`, `SECRET_NAMES`,
  `HIDDEN`), the browser tools' answers (`crates/marley_workbench/src/browser_tools.rs`,
  `entries`), the terminal tools (`crates/marley_workbench/src/mcp.rs`, `terminal_blocks`,
  `block_entry`, `terminal_read`, `tail`), `MarleySettings` in `marley_workbench.rs`, the Marley
  page (#515). The `regex` crate is a workspace dependency already.
- **Published material.** The token formats their issuers document (GitHub's `ghp_`/`github_pat_`,
  AWS's `AKIA`/`ASIA` key ids, Stripe's `sk_live_`, Slack's `xox?-`), and gitleaks' default
  ruleset as a cross-check of the shapes (the gate already runs gitleaks). Its private-key rule
  also takes the PGP form, `PRIVATE KEY BLOCK`, which Marley's rule takes too.
- **Orca** (MIT), `src/main/observability/redactor.ts`, compared in Test after the second Orca
  pass named it: the same provider shapes, and two Marley lacked, now in the rules: a token
  standing alone as a URL's userinfo (`https://<token>@host`, as git prints one) and Slack's
  `xoxe-`/`xoxo-` prefixes. Its `.env` rule, which hides every `NAME=value` line, was left out:
  `env` output would lose `PATH` and every other harmless value; Marley hides secret-named ones.
- Rejected: redacting in `marley_terminal` for everything it stores: the terminal's own drawing
  stays exact, and only what leaves for a model is changed.

## UI proof
UI-AFFECTING (what agents see, and a setting). `script/e2e/516-secret-redaction-for-agents.sh`
(`compositor sway`): the scenario's shell prints fake secrets assembled at run time (so the
scenario file holds none for gitleaks to flag). The stand-in agent (`mcp_agent`) reads the block
through Marley's server: the run log shows each secret as `[redacted: <kind>]` and the count
(REQ-001). With `redact_secrets_for_agents: false` written to the profile while Marley runs, the
same read returns the text as printed (REQ-002). With a user pattern set, a line matching only it
is redacted (REQ-003), and a pattern that is not a regular expression is named in a notification
(`516-02-bad-pattern`). The Marley settings page shows the Agents section with the toggle on
(`516-01-setting-on`, REQ-004); clicked off (`516-03-setting-off`), a page's console reaches the
agent as printed. The terminal is read before the Settings window opens (TICKET-544).

## Locked-In Decisions
- D1 — On by default: a model is always on the other end of Marley's tools.
- D2 — Redact at the tool boundary, never in the terminal's buffer: what Chad sees stays exact.
- D3 — Name the kind, not the value: `[redacted: github token]` tells the agent what was there
  without it.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent reads a terminal block whose output holds a secret of a built-in kind, Marley shall return `[redacted: <kind>]` in its place and the number of redactions. | The run log (the stand-in's `terminal_read`) |
| REQ-002 | WHERE `marley.redact_secrets_for_agents` is false, Marley shall return the text as printed. | The run log after the setting changes, with no relaunch |
| REQ-003 | WHEN `marley.redaction_patterns` holds a regex, text it matches shall be redacted as `[redacted: pattern]`. | The run log |
| REQ-004 | WHILE the Marley settings page shows, it shall show "Redact Secrets for Agents" in an Agents section. | Shot `516-01-setting-on` |

## Phase Plan
- **P1 Plan** — this spec.
- **P2 Code** — the redactor, the settings, the tools, the page item; fmt and clippy.
- **P3 Test** — the scenario; the gate.
- **P4 Complete** — docs, knowledge, close, archive, commit.
