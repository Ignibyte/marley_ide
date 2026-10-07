# The Rich Input button hides it too — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-677-rich-input-button-hides-it.md
- **Pipeline spec:** 677-rich-input-button-hides-it.spec.md

## Phase 1 — Plan (2026-10-07)
- **Request:** Chad, 2026-10-07 (quoted in the spec).
- **Classification / tier:** feature; `marley_workbench` only.
- **Pre-flight:** green; #676 committed; cargo idle.
- **Recall (§18.3):** #481's scenario (the stand-in by `exec -a claude`); #649 (`agent_editor`
  takes the key when the agent's editor runs in a tab, where the button keeps opening).
- **Discovery:** `agent_bar.rs` `rich_input_button`; `rich_input.rs` `open`, `open_for`, `close`,
  `Prompts`, `Target`.

### Design
- **`rich_input.rs`**: `is_open_for_agent(view, cx)` (the prompt is open and its target an agent);
  `hide(view, window, cx)`, Escape's `close`.
- **`agent_bar.rs`**: `rich_input_button` takes `cx`, sets `toggle_state` and the tooltip's title
  from `is_open_for_agent`, and the click hides when open, else opens.
- **File manifest:** those two, the guide, the scenario; docs.

### Visual check plan
| Criterion | What the scenario does | Proof |
|---|---|---|
| REQ-001 | starts the stand-in, clicks the pencil, holds the pointer on it | `677-01-open` |
| REQ-002 | types a draft, clicks the pencil, then clicks it again | `677-02-hidden`, `677-03-draft` |

### Checklist (no TaskCreate in this harness)
- [x] Pick, pre-flight, recall; discovery.
- [x] Mint the pair; the ticket in progress.
- [x] Prior-art sweep; spec; design; visual check plan.
- [x] Phase 1 PASS under Chad's request.

## Phase 2 — Code (2026-10-07)
### Built
- **`rich_input.rs`**: `is_open_for_agent` and `hide`.
- **`agent_bar.rs`**: `rich_input_button` takes `cx`; `toggle_state` and the tooltip's title from
  `is_open_for_agent`; the click reads it again and hides or opens.
- **The guide**: the rich input article.

### Review
- With #649's editor key on, `open` hands the key to the agent's own editor and the prompt never
  opens, so the button keeps opening, as the spec's scope leaves it.
- A tooltip already shown keeps its old title until the pointer leaves the button (gpui keeps the
  built tooltip view); the pressed state shows at once. The scenario moves the pointer off and back.

### Gate
`just gate-diff` on the tree with the scenario: GATE GREEN [diff], 17 of 17.

## Phase 3 — Test (2026-10-07)
`script/e2e/677-rich-input-button-hides-it.sh` (`compositor sway`), on the debug build, #481's
stand-in Claude Code.

| Criterion | Shot | What it shows |
|---|---|---|
| REQ-001 | `677-01-open` | the pencil clicked, the pointer off and back: "A prompt for Claude Code" above the bar, the pencil pressed (blue, filled), the tooltip "Hide Rich Input  Ctrl-G" |
| REQ-002 | `677-02-hidden` | "a draft for later" typed, the pencil clicked: the editor gone, the pencil unpressed, the terminal's cursor solid |
| REQ-002 | `677-03-draft` | the pencil again: the editor back holding "a draft for later", the pencil pressed |

The exploratory run before the gate showed the old title in the first shot, the tooltip having
been built on the hover before the click; the scenario now moves the pointer off and back. No code
changed. Focus: headless sway; Hyprland's one Marley window (Chad's) before and after, no rule
added.

## Phase 4 — Complete (2026-10-07)
- **Documented:** `CHANGELOG.md` (Changed: the Rich Input button hides it too); the in-app guide's
  rich input article (before the gate); `marley_workbench.md` (the rich input section).
- **Knowledge:** `L-claude-677-a-shown-tooltip-keeps-its-title-until-the-pointer-leaves-001`.
- **Brain:** no decision of weight; none recorded.
- **Ticket:** closed; the pair archived.
