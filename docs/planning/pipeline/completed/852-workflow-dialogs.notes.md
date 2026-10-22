# Save as Workflow and Run as the Workflows board draws them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-852-workflows-and-launch.md
- **Pipeline spec:** 852-workflow-dialogs.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #851; split while planning: the two workflow dialogs
  here, the Launch menu, remote terminals, the block filter and keys TICKET-882.
- **Classification / tier:** feature, S. Marley crate only.
- **Recall (§18.3):** #558's scenario: its hover points are stale (its editor shot shows no
  editor); the block is selected with Ctrl Up and the palette's Save as Workflow instead.
- **Discovery:** the editor already has Name, Command, a row per parameter (default and
  description), the global box, the file, and Cancel and Save; the run prompt has its fields and a
  line on the keys.
- **Decisions:** D1, D2 in the spec.

### Design
- `workflows.rs`: `template_parts(text)` splits a command into text and `{{name}}` parts;
  `render_template` draws them in the buffer font with accent chips. The editor gains the line,
  the count, and a footer (`Saved to <file>`, `render_button` Quiet Cancel and Primary "Save ·
  Enter"). `ParameterPrompt` gains `template` (the command and its arguments), a notify on each
  field's edits, the filled preview (`substitute`, or the part as written when it fails) and a
  footer with the keys and a Primary Run.
- **File manifest:** `workflows.rs`; the scenario.

## Phase 2 — Code
- **Built:** `workflows.rs`: `template_parts` (plain text and `{{name}}` parts, a non-name brace
  pair kept as text) and `render_template` (the buffer font; a parameter filled from the values or
  an accent chip). `WorkflowEditor`: "Save as workflow", `render_found` (the command's chips over
  the count line), the footer ("Saved to <file>", #870's Quiet Cancel and Primary "Save · Enter").
  `ParameterPrompt`: `template` (command and arguments joined), a notify on each field's edits,
  the filled preview, and a footer with the keys and a Primary Run.
- **Fix in Test:** the preview put a gap between parts that carry their own spaces; the gap went.
- **Gate:** green on the first run, 17 of 17, after the fix.
- **Review:** saving, running, Escape and the global box are unchanged; the buttons call the same
  actions as Enter.


## Phase 3 — Test
- **Scenario:** `script/e2e/852-workflow-dialogs.sh` (sway): the block selected with Ctrl Up and
  saved from the palette; run from Zed's task picker (Alt Shift T).
- `852-01-save`: Name "serve --port", the command `serve --port {{port}} {{path}}`, the chips
  line with `port` and `path` in the accent, "2 parts became parameters.", the rows (4180, web),
  the global box, "Saved to …/.zed/tasks.json", Cancel and "Save · Enter". REQ-001 met.
- `852-02-run`: "Run serve --port", port 5000 typed, path web, the preview `serve --port 5000
  web`, "Enter runs it · Esc runs nothing" and Run. REQ-002 met.
- `852-03-day`, `852-04-save-day`: Marley Day, the run prompt and the save card. REQ-001 and
  REQ-002 met.


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md`, look-plan W5 row.
- Knowledge: none new. Brain: `brain decide` on consultation d9c1af10.
- Ticket closed; pipeline archived; TICKET-882 holds the rest of the boards.
