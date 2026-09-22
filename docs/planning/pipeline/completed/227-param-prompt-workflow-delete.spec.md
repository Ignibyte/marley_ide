---
pipeline_id: fbb9995b-fcc0-4629-a829-7f5d8f82b457
ticket: docs/planning/tickets/open/TICKET-227-param-prompt-modal-workflow-edit.md
status: Phase 5 — Complete PASS
title: "#204 follow-ups: the guided param-prompt modal + workflow delete (full rebuild)"
type: feature
milestone: M12.2
references:
  - docs/planning/pipeline/completed/palette-dispatch.notes.md
---

## Title
(a) Invoking a `{{param}}` workflow stops inserting the RAW template: a guided
MODAL (the Recipe C naming-card family) collects each param's value — type,
Enter advances, Enter on the last substitutes and INSERTS the filled command
at the prompt (still no auto-run — the v1 review posture holds). The pure
`substitute`/`params_of` (#204) drive it unchanged. (b) A "Delete Workflow…"
palette verb opens a finder-family picker over the saved workflows; Enter
deletes — with the **full palette-command REBUILD** the #204 R1 note demands
(the `WORKFLOW_BASE + index` scheme makes ids positional; an in-place remove
drifts every later id) — and persists. (c) the live-capture re-verify rides
#417 (standing session block).

## Scope
### In
- Pure `ParamPrompt` state machine (workflows.rs): fields
  `{workflow_index, name, template, params, values, current}`; ops
  `new(index, &Workflow) -> Option<Self>` (None for a param-less workflow),
  `push_char`/`backspace` (edit `values[current]`), `advance() -> Advanced |
  Done`, `filled() -> Result<String, SubstituteError>` (zip → substitute).
  cov/MSI 100, full-value pins.
- App: `param_prompt: Option<ParamPrompt>` — joins `close_transient_overlays`
  (the #415 membership rule), `text_input_blocked`, an Esc/typing/Enter key
  arm ahead of the terminal arms; renders via `naming_overlay_card` (Recipe C:
  title "Workflow: {name}", one line per param — filled values, the CURRENT
  param's line carries the caret glyph —, a muted hint line). `invoke_workflow`
  forks: params empty → today's substituted insert; params present → open the
  modal. Done → the SAME insert idiom → close.
- App: a static "Delete Workflow…" palette command (id in the static block) →
  `workflow_picker: Option<FinderState>`-style state (the finder-family card
  listing workflow names, fuzzy-filterable) → Enter deletes the picked
  workflow: `workflows.remove(i)` + `persist_workflows` +
  `rebuild_workflow_commands()` — a NEW rebuild fn (the
  `rebuild_pane_commands`:7711 precedent) that regenerates the whole
  `WORKFLOW_BASE` block from `self.workflows` (R1 honored). Picker joins the
  roster + text_input_blocked too.
- Headless drives: save → invoke-with-params → type/advance → filled insert;
  delete → len-1 + id/index re-agreement + persisted settings (tempdir).

### Out (explicitly deferred)
- RENAME / REORDER / template EDIT — recorded: the settings file remains
  their surface; the delete picker + rebuild establishes the exact pattern a
  rename would reuse. (The ticket's "consider" clause — considered, deferred.)
- Auto-run after substitution (the v1 review-then-Enter posture is a
  deliberate keep).
- The (c) live capture — #417.

## Reference (§20)
Warp (workflows + argument templating) — the behavior maps record Warp's
handlebars-style workflow ARGUMENT scheme
(`06-platform-settings-infra.md:147` — "Workflow/command argument
templating"), the class #204's `{{param}}`/`substitute` already mirrors.
Behavior matched: a parameterized workflow collects its arguments before the
command reaches the prompt (Warp fills placeholders inline in its command
bar; Marley collects in a MODAL — its own arrangement of its shipped
naming-card, then the same reviewed-insert). Delete-with-stable-commands is
Marley-specific bookkeeping (the id↔index scheme is ours). Behavior-level
only; no fork source.

### Prior art
1. **In-house, decisive (this batch built the substrate):**
   `naming_overlay_card` (Recipe C, #414) is the modal's chrome — zero new
   look; the #415 roster rule tells both new overlays exactly where they
   register; `FinderState` (the finder/history/symbols family) is the picker,
   reused verbatim; `substitute`/`params_of` (#204, cov/MSI 100) are the
   engine, untouched; `rebuild_pane_commands` (#399) is the rebuild
   precedent; the #204 R1 note is the delete constraint.
2. **Behavior maps** — the handlebars templating row (above); no deeper
   Warp UX doc for the fill flow (behavior-level knowledge suffices; the
   modal is our arrangement).
3. **Permissive deps / published** — none own the seam (a string-zip state
   machine; checked gpui — no form/modal primitive beyond what we ship).

## React-first (parity)
UI-AFFECTING — Zone A (palette/naming-card family). The NEW arrangement is
the param-prompt CARD (title + N param lines + hint). React half FIRST:
a `ParamPromptCard` component in marley-web beside `NamePaneCard.tsx`
(reusing its exact card look), reachable via a fixture "Workflow: deploy"
palette row (the POC has no workflow persistence — a fixture row models the
flow); drive at 5173, screenshot READ, THEN port 1:1 into the Rust render.
The delete PICKER reuses the shipped finder/history picker look verbatim
(the #414 chain-identity class — no new look, no React half; recorded). The
Marley-side live pixel pair rides #417 as everywhere this session.

## Locked-In Decisions
- D1 — **Insert, never auto-run** (v1 posture kept; the ticket offered both).
- D2 — **Delete = full `WORKFLOW_BASE` block rebuild** (R1) — never in-place
  id surgery; the rebuild fn regenerates from `self.workflows` and is called
  at delete AND at save (replacing the append at :17395 so ONE builder owns
  the block — append-drift dies with it).
- D3 — **Scope cut recorded:** rename/reorder/template-edit out (settings
  file remains their surface).
- D4 — **Both new overlays join the #415 roster + text_input_blocked** — the
  membership rule is law now.
- D5 — **The modal is Recipe C** — no new chrome; the React half is the
  arrangement only.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a workflow WITH params is invoked, the system shall open the param-prompt modal (not insert the raw template); WHEN one WITHOUT params is invoked, it shall insert the substituted command as today. | Headless drive both arms; ParamPrompt::new(None) unit. |
| REQ-002 | WHILE the modal is open, typed chars shall edit the CURRENT param's value, Enter shall advance (last param: substitute + insert at the focused prompt + close), Escape shall cancel with no insert. | ParamPrompt units (full-value per op incl. multibyte + empty values) + headless drive (type→advance→filled text at the prompt). |
| REQ-003 | WHEN a param value is empty at Done, substitution shall still produce the command with the empty value (explicitly allowed — the user reviewed) — no MissingParam is reachable from the modal (values are zipped for every param). | Unit: filled() with empty values == template with empties; the zip-totality pin. |
| REQ-004 | WHEN "Delete Workflow…" picks a workflow, the system shall remove it, persist, and REBUILD the whole workflow command block such that every surviving workflow's palette id maps to its NEW index (R1). | Headless drive: save 3, delete the middle, invoke the (new) id of the third → the THIRD's command inserts; settings round-trip asserts len 2. |
| REQ-005 | Both new overlays shall be members of `close_transient_overlays` and shall block text input while open. | The roster grep + choke-point-style flip asserts (headless). |
| REQ-006 | The full suite + gate `--diff` shall be green. | RUN at validate. |

## Phase Plan
- **P2 Design** — signatures + key-arm placement + the rebuild fn shape +
  the React fixture wiring; file manifest.
- **P3 Implement** — React card first (drive + READ), then Rust: pure machine
  → overlays → rebuild.
- **P3.5 Inspect** — critics (id-remap correctness, roster/key-arm order,
  state-machine edges, the R1 rebuild-at-save conversion).
- **P4 Validate** — units + headless drives RUN; suite; gate `--diff`.
- **P5 Complete** — CHANGELOG + app_shell workflows paragraph; ledger;
  archive; close; commit (both repos).
