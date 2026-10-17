# Component templates from files — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-801-component-templates-from-files.md
- **Pipeline spec:** 801-component-templates-from-files.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-17: "Maybe even research how we could make this pluggable to add
  outside components without recompile." On the plan: "I'm not sure if the WASM plugins would be
  useful but maybe we create a ticket for that later." Templates are the first step; WASM and MCP
  Apps wait in the intake doc.
- **Discovery (drafted by /spec; re-verified at promotion):** `jsonschema` 0.51 and `schemars` 1.0
  in `[workspace.dependencies]`; `paths::config_dir()`; `Fs::watch`; the settings file watcher in
  `crates/settings` as the reload pattern.
- **Recall (§18.3):** #799's parse (the JSON path errors and limits this extends); Zed's
  `settings::watch_config_dir` (the `Fs::watch` loop this follows); the lint lessons of #798 and
  #799 (gate:14's private links, the gpui crates' `unused_results`).
- **Promoted 2026-10-17**; seams re-read on the tree #800 left.

### Design
- **`marley_components` (Marley):** `Templates::load_in(dir)` (every `*.json`, in name order: the
  file's shape, the name's rules, the inputs schema compiled with `jsonschema`, each placeholder
  named in `inputs.properties`; duplicates both refused; a template whose example does not draw
  refused), `Template::schema` and `example_tree`; `parse_with` and `parse_str_with`, where a type
  outside the catalog that names a template has its inputs checked (`$.…inputs.<path>` on an
  error), put in the body's `{"$input": …}` places, and the result parsed as any tree, templates
  inside templates to a depth of 4, a cycle refused.
- **`marley_workbench/src/component_library.rs` (Marley):** `MarleyTemplates`, a global loaded at
  start from `paths::config_dir()/components` (made if missing) and again on every change the
  folder's `Fs::watch` reports; the Templates group (each drawn from its example with its path, each
  refused file with its error, Open Folder); the scratch box parses with the templates.
- **`component_tools.rs` (Marley):** `component_list` adds the templates; `component_show` and the
  thread's drawing parse with them.

## Phase 2 — Code
- **Built:** as the design; `component_list`'s description now names templates and their shape.
- **Review of the diff:** loading reads files on a background task, never in render; a refused
  template is never offered (`component_list` lists only `templates()`); substitution touches
  only whole-value `{"$input": …}` objects; the expansion counts its components against the same
  limit as any tree.
- **Gate run 1 (`801-gate-1.log`):** red on gate:14 (the module doc linking the private
  `MarleyTemplates`) and gate:21 (`async_block_without_await` on the loader's `async move {
  Templates::load_in(&dir) }`). The loader runs through `smol::unblock(move || …)`, a closure, not
  an async block. **Gate run 2 (`801-gate-2.log`): GATE GREEN [diff].**

## Phase 3 — Test
- **Scenario:** `script/e2e/801-component-templates-from-files.sh` (`compositor sway`): the run's
  config folder holds `components/deploy_status.json`; `release_note.json` and `bad_one.json` are
  copied in while Marley runs; #800's stand-in agent CLI and scripted agent, rewritten to use the
  template. The library's end is reached with one long scroll (the page stops at its end).
- **Shots, each read (`scratchpad/shots-801/`; the run after the loader's change in
  `shots-801b/`, read by OCR, shows the same):**
  - `801-01-library`: the Templates group with Open Folder and the folder's path; `deploy_status`
    drawn from its example (the staging card, Version 1.4.0, Deploys today 4, the services table)
    under its file's path (REQ-001).
  - `801-02-added`: `release_note` below it, drawn as its success callout, with no restart
    (REQ-002).
  - `801-03-broken`: `bad_one.json is not used` and "The body uses the input `missing`, which
    `inputs` does not name…" (REQ-003).
  - `801-06-list`: `agent list` prints the built-ins, then `deploy_status release_note`; no
    `bad_one` (REQ-006, REQ-002 for `component_list`).
  - `801-04-thread`: the thread draws the expanded `deploy_status` (production, 1.5.0, 2, api up,
    search down), and under the second call "$.inputs.deploys: "two" is not of type "integer""
    (REQ-004, REQ-005); `801-05-bad-inputs` repeats it.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` ("Component templates from files"); `docs/marley/guide.md`
  ("Component templates (#801)", with a template's shape).
- **Knowledge:** L-claude-801-a-blocking-call-off-the-main-thread-under-dylint-001.
- **Brain:** no decision recorded.
