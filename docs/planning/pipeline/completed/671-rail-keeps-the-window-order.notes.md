# The rail keeps the window's order — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-671-rail-keeps-the-window-order.md
- **Pipeline spec:** 671-rail-keeps-the-window-order.spec.md

## Phase 1 — Plan (2026-10-07)
- **Request:** Chad, 2026-10-06 (quoted in the spec); queued the same night, built 2026-10-07 on
  "i think we move to work on this now".
- **Classification / tier:** bug; a default in `default.json`, `settings_content` (both Zed
  crates with touchpoint rows) and `marley_rail`.
- **Pre-flight:** green; no active pipeline; cargo idle; README marker present.
- **Recall (§18.3):**
  - #542's spec: the attention classes, the hold while the pointer is over the rail, and
    `rail_order: "window"` as the way out (its REQ-008).
  - #602: the drag order lives in the saved blob and applies in both orders.
  - The brain (consultation `0fd4b6022fe34360909ed818f89642d7`): nothing on this seam.
- **Discovery:** `assets/settings/default.json:1758-1762`; `settings_content/src/marley.rs:131-136`
  and `:583-590`; `marley_rail.rs:868-876`; `marley_workbench.rs:698-705` (`unwrap_or_default` on
  the content enum, so the content default matters only when no value reaches it).

### Design
- **`default.json`** (Zed crate, row exists): `"rail_order": "window"`, the comment naming
  `"attention"` as the other value.
- **`settings_content/src/marley.rs`** (Zed crate, row exists): `#[default]` on `Window`; the doc's
  `Default: "window"`.
- **`marley_rail.rs`** (Marley crate): `#[default]` on `Window`, and its doc no longer says "as
  before".
- **`script/e2e/671-…`**: sources #542's scenario for its stand-ins, bus and helpers, and
  replaces `setup` (all three projects start their stand-in) and `steps`.
- **Ledger rows:** both touchpoint rows gain "`rail_order` defaults to `"window"` (#671)".

### Visual check plan
| Criterion | What the scenario does | Proof |
|---|---|---|
| REQ-001 | c, the bottom project, works | `671-01-working` |
| REQ-001, 002 | b waits as well | `671-02-waiting` |
| REQ-003 | sets `rail_order` to `"attention"` | `671-03-attention` |

### Risks
- A user who set nothing and liked the attention order loses it silently; the CHANGELOG names the
  setting.

### Checklist (no TaskCreate in this harness)
- [x] Pick, pre-flight, recall, the brain; discovery.
- [x] Mint the pair; the ticket in progress; the backlog row removed.
- [x] Prior-art sweep; spec; design; visual check plan; risks.
- [x] Phase 1 PASS under Chad's request.

## Phase 2 — Code (2026-10-07)
### Built
- **`assets/settings/default.json`**: `"rail_order": "window"`, the comment naming both values
  with their tickets.
- **`settings_content/src/marley.rs`**: `#[default]` on `MarleyRailOrder::Window`; the doc's
  `Default: "window"`.
- **`marley_rail.rs`**: `#[default]` on `RailOrder::Window`.
- **`docs/marley/zed-touchpoints.md`**: both rows name the new default.
- **`script/e2e/671-…`**: sources #542's scenario and replaces `steps`.

### Review
- `rail_order()` in `marley_workbench.rs` maps the content enum with `unwrap_or_default`, so the
  content default and `default.json` agree and either path gives `Window`.
- `run()` (`marley_rail.rs:1040`) gives a drag no class bound in window order: a row can be dragged
  anywhere in its run, as #542 already allowed under `"window"`.
- The tests in `marley_rail` name their order explicitly (`RailOrder::Window` in their snapshot
  builder), so none leans on the default.

### Gate
`just gate-diff`: GATE GREEN [diff], 17 of 17.

## Phase 3 — Test (2026-10-07)
`script/e2e/671-rail-keeps-the-window-order.sh` (`compositor sway`), on the debug build, the copy's
settings with no `rail_order`.

| Criterion | Shot | What it shows |
|---|---|---|
| REQ-001 | `671-01-working` | a, b, c; c's agent `working · Ru… Bash: make`, still last |
| REQ-001, 002 | `671-02-waiting` | Needs you 1 at the top; a, b, c below it; under b, `b — bash` above its agent, which reads `waiting · Wri…` with the blue dot |
| REQ-003 | `671-03-attention` | after `rail_order: "attention"`: b (its waiting agent now above the shell), c, then a |

The one banner (b's permission) went to the run's private bus; the user's bus saw none. Focus:
headless sway; Hyprland's one Marley window (Chad's) before and after, no rule added. The shots
also show the filter row ending below the tab bar's line, which is #672.

## Phase 4 — Complete (2026-10-07)
- **Documented:** `CHANGELOG.md` (Changed: the rail keeps the window's order); the in-app guide's
  rail-order article and settings table (edited before the final gate, since the guide is under
  `crates/marley_workbench`); the walkthrough's 5.21; `marley_rail.md` and `marley_workbench.md`;
  both touchpoint rows.
- **Knowledge:** `AD-claude-671-the-rail-keeps-the-window-order-by-default-001`,
  `L-claude-671-the-quiet-timer-makes-typing-into-an-agent-read-as-working-001`.
- **Brain:** `decisions/marleys-rail-keeps-the-windows-order-by-default`; the plan's consultation
  `0fd4b6022fe34360909ed818f89642d7` closed with no decision the night before (deferred).
- **Gate:** `just gate-diff` again on the final tree (the guide changed): GATE GREEN [diff], 17 of 17.
- **Ticket:** closed; the pair archived.
