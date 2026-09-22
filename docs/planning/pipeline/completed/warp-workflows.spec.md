---
pipeline_id: a6335ce5-d0dd-4b0d-b137-0d5247fa5aa5
ticket: forge#204 (cb29fc31-4d22-4121-9375-1fc376a5c84c) · local docs/planning/tickets/open/TICKET-204-warp-workflows.md
aar_id: 23fd1e42-fe64-4c50-bc02-b5d19865da72
status: Phase 5 — Complete PASS
title: command-palette workflows — save + re-run named commands (bounded v1)
type: feature
milestone: M12.2
references: []
---

## Title
Command-palette workflows (Warp Workflows analog, forge #204): save a command under a name (with
optional `{{param}}` placeholders) and re-run it from the command palette. A bounded v1 — the pure
substitute engine + Workflow model + a marley_settings round-trip + palette save/list/run — with the
fancy interactive param-prompt modal deferred.

## Scope
### In
- **Pure `workflows.rs`** (NEW, cov/MSI 100): `struct Workflow { name, command, params }` +
  `substitute(template, args) -> Result<String, SubstituteError>` (replace `{{name}}` tokens; Err on a
  token with no supplied arg; non-`{{}}` braces left literal) + `params_of(template) -> Vec<String>`
  (the distinct token names, in order — auto-derives a workflow's params on save).
- **marley_settings round-trip** (mirrors #87 `RemoteHosts`): a `Workflows: Vec<Workflow>`
  `define_setting!` + `AppliedSettings.workflows` + `persist_workflows(manager, &[Workflow])`, tolerant
  of an absent/malformed table (→ empty, no boot crash).
- **Shim** (app.rs, masked): the palette lists saved workflows as `WORKFLOW_BASE` dynamic commands
  (like #199 `THEME_BASE`); invoking one INSERTS its command into the focused prompt buffer (a no-param
  workflow via `substitute`; a param workflow inserts the template with its `{{tokens}}` to fill inline
  + run — the prompt buffer is the param entry in v1). A "Save command as workflow…" palette entry
  takes the last submitted command + prompts for a NAME (the `renaming_tab`-style inline draft), Enter
  commits → append + `persist_workflows`; `params` auto-derived via `params_of`.

### Out (explicitly deferred → follow-up)
- A **dedicated interactive multi-param prompt modal** (collect each `{{param}}` value in a modal, then
  substitute + auto-run). v1 fills params inline in the prompt buffer.
- A workflow **edit / delete UI** (v1: save + run; remove via the settings file). 
- Reordering / import / export of workflows.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — `substitute(template: &str, args: &BTreeMap<String,String>) -> Result<String,
  SubstituteError>`** with `enum SubstituteError { MissingParam(String) }`. A `{{name}}` token → 
  `args[name]`; a token with no arg → `Err(MissingParam(name))`; a single/non-`{{}}` brace (`{`, `}`,
  `{x}`) is left literal. `params_of(template) -> Vec<String>` returns the distinct token names in
  first-seen order. (Design pins the exact tokenizer; the ticket's "undeclared param" = a token not
  covered by args → the same MissingParam error, since a workflow's `params` are auto-derived from its
  tokens.)
- **D2 — `Workflow { name: String, command: String, params: Vec<String> }`** in `workflows.rs`, deriving
  `Debug, Clone, PartialEq, Eq, Serialize, Deserialize` (mirrors `marley_remote::RemoteHost`).
- **D3 — Persist via `Workflows: Vec<Workflow>` `define_setting!`** (mirrors #87 `RemoteHosts` at
  settings.rs:42) + `persist_workflows` (mirrors `persist_theme`/`persist_grid`) + the
  absent/malformed → empty tolerance.
- **D4 — Palette: a `WORKFLOW_BASE: u32` dynamic-command range** (mirrors #199 `THEME_BASE` / #87
  `CONNECT_BASE`) for the saved workflows + a static "Save command as workflow…" command. Dispatch maps
  the id → workflow index (a pure `workflow_pick_index`, like #199 `theme_pick_index`).
- **D5 — Invoke → insert into the cooked prompt buffer** (`buffer.edit(caret..caret, &cmd, Human)`, the
  #59/#65/#200 rule); no-param workflows are `substitute`d, param workflows insert the template. Save →
  the `renaming_tab`-style inline name draft (`naming_workflow: Option<String>`).
- **D6 — Clean-room (§20).** Warp Workflows is the concept; this is an original implementation.
- **D7 — `substitute` uses a `str::find` scan, not a char-Vec index scan** (design) — `find`/`trim`/
  `get`/`push_str` are method calls (unmutated), so the only viable mutants are the two `+ 2`s + the
  Result/String body defaults — a small, cleanly-killable surface.
- **D8 — Rename `palette::theme_pick_index` → `dynamic_command_index`** (design) — it is already generic
  (`id, base, count`); now that it serves the theme AND workflow ranges, the honest name is reused for
  both. A behavior-identical pure-module rename.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a template has `{{param}}` tokens and `args` supplies them, `substitute` shall return the command with each token replaced by its arg. | unit: `substitute("git push {{remote}}", {remote:origin})` == `Ok("git push origin")`; multi-token + token-at-start/end. |
| REQ-002 | WHEN a `{{param}}` token has no supplied arg, `substitute` shall return `Err(MissingParam)`. | unit: `substitute("git push {{remote}}", {})` == `Err(MissingParam("remote"))`. |
| REQ-003 | WHERE the template contains single or non-`{{}}` braces, `substitute` shall leave them literal. | unit: `substitute("echo {hi} }}{{", {})` leaves `{hi}` etc. intact (design pins the exact edge). |
| REQ-004 | WHEN workflows are saved then reloaded, the `Workflows` setting shall round-trip identically; a malformed workflows table shall load as an empty list. | unit: settings round-trip identity + malformed→empty (mirrors the #87 tests). |
| REQ-005 | WHEN a command is saved as a named workflow, it shall appear in the command palette and persist across a reload. | driven: save `deploy = git push origin` → it shows as "Workflow: deploy" in the palette (+ re-load proof via the settings round-trip unit). |
| REQ-006 | WHEN a saved workflow is invoked from the palette, its command shall be inserted into the focused prompt ready to run (a no-param workflow substituted; a param workflow's template inserted to fill inline). | driven: invoke "Workflow: deploy" → `git push origin` appears at the prompt; a `{{param}}` workflow inserts the template. |

## Phase Plan
- **P2 Design** — pin the `substitute`/`params_of` tokenizer + `SubstituteError`; the `Workflows`
  setting + `persist_workflows` + `AppliedSettings` wiring; the palette `WORKFLOW_BASE` registration +
  dispatch + the buffer-insert + the `naming_workflow` save draft; `cargo mutants --list` for the real
  set. CONFIRM the deferred param-modal cut (or add a minimal single-param prompt if the inline-buffer
  is too clunky).
- **P3 Implement** — workflows.rs + settings.rs + the app.rs shim.
- **P3.5 Inspect** — critics vs the diff (the tokenizer edge cases, the round-trip, the palette
  dispatch id math, the buffer-insert, clean-room); fix real findings.
- **P4 Validate** — the exact-value matrix + the round-trip test + RUN; driven captures (save → palette
  → invoke → prompt); gate green.
- **P5 Complete** — CHANGELOG + app_shell.md; AAR; archive; close; file the param-modal follow-up.
