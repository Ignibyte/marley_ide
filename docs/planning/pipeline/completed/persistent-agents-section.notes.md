# persistent Agents section — Notes

- **Forge ticket:** #91 `09e43d8c-d681-4353-b32a-3ace356981e3` · **AAR:** `2890437b-be54-4b15-abe8-d8692206dfd0`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-091-persistent-agents-section.md

## Phase 1 — Plan
- **Request:** forge #91 (M2.F 2/6) — the Fleet always-visible in the #90 Agents tab.
- **Pre-flight:** the Fleet overlay (app.rs ~2260) builds the row string inline (#78/#80/#82 format) + the
  #81 clickable-row → focus; #90's Agents-section body is a placeholder to replace. agent_rows tested.
- **Decisions:** D1 agent_row_text single source; D2 rows reuse #81 click→focus; D3 empty → agents_empty_hint.
- **AAR id:** `2890437b-be54-4b15-abe8-d8692206dfd0`.

## Phase 2 — Design
- (pending)
## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 2 — Design
- **agent_view.rs (PURE):** `agent_row_text(row: &AgentRow) -> String` = `format!("{} {}{} ({}{}){}", glyph, label, ticket_suffix, status, age_suffix, last_suffix)`; ticket_suffix = row.ticket.map(|n| format!(" #{n}")).unwrap_or_default(); age_suffix/last_suffix = "" or format!(" · {}", x). `agents_empty_hint() -> &str` = "no agents — ⌘⇧A to launch".
- **app.rs SHIM:** (1) the Fleet overlay (#68 ~2277) replaces the inline suffix-build + format! with `agent_row_text(&row)` (output-identical refactor). (2) the #90 Agents-section body: `if agent_rows(&self.agents).is_empty() { placeholder(agents_empty_hint()) } else { a flex_col of each row = div().child(agent_row_text(&row)).on_mouse_down(Left → view.workspace.focus(pane); flash "jumped to {label}"; notify) }` (reuse #81; pane/label captured).
- **Mutation targets:** agent_row_text 3 suffix branches + assembly order; agents_empty_hint literal.
- **Test plan:** agent_row_text_full (a row with ticket+age+last → the whole string) + agent_row_text_bare (no ticket/age/last → "{glyph} {label} ({status})") + agents_empty_hint literal. cov/MSI 100. The Agents-section render + click masked (static live + engine).
- **Risks:** the Fleet refactor MUST be byte-identical output (agent_row_text tests pin it); ⌘⇧E overlay stays.

## Phase 3 — Implement
- **Built:** `agent_row_text(&AgentRow)` (the shared row-string: glyph+label+ #ticket + ({status}{ · age}){ · last}) + `agents_empty_hint()` (agent_view); the Fleet overlay refactored onto agent_row_text (output-identical); the #90 Agents-section body renders agent_rows as clickable rows (→ workspace.focus + flash, reuse #81) or agents_empty_hint when empty.
- **Verification:** fmt; check --all-targets 0 err; clippy OK. section_label still consumed (Forge placeholder). (agent_row_text/agents_empty_hint tests → validate.)

## Phase 3.5 — Inspect
- **Method:** self-review (a pure format extraction + a masked render reusing the proven #81 click pattern).
- **Lenses — no findings:** agent_row_text = the EXACT prior Fleet format moved into a tested pure fn (3 suffix empty-vs-format branches + the assembly order — pinned by full+bare tests); agents_empty_hint literal; the Fleet overlay + Agents section BOTH call agent_row_text (single source, refactor output-identical); the Agents rows reuse #81 (pane Copy + label clone into the move listener; workspace.focus Result ignored) but WITHOUT fleet_open=false (the section is persistent, no overlay to close); empty → agents_empty_hint. No panics. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** agent_row_text_full (all segments) + agent_row_text_bare (none) + agents_empty_hint_text. `cargo nextest` → pass; gate cov/MSI 100.
- **Self-test:** the #90 static capture already proved the 3-tab strip renders; the Agents-section content render is masked (reuses the proven #68 Fleet-row + #81 click patterns) + engine-tested via agent_row_text; switching to the Agents tab needs a synthetic click (ENV-BLOCKED all session).
- **Gate:** GREEN [diff] 15/15, cov/MSI 100.

## Phase 5 — Complete
- CHANGELOG ### Added; aar-submit(5); forge #91 → done. **M2.F 2/6.** agent_row_text + agents_empty_hint (cov/MSI 100); the Agents dock section (clickable agent_rows → focus). Fleet refactored onto agent_row_text.
