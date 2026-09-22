# command-palette workflows — Notes

- **Forge ticket:** #204 (cb29fc31-4d22-4121-9375-1fc376a5c84c)
- **AAR:** 23fd1e42-fe64-4c50-bc02-b5d19865da72
- **Local ticket doc:** docs/planning/tickets/open/TICKET-204-warp-workflows.md
- **Pipeline spec:** warp-workflows.spec.md

## Phase 1 — Plan
- **Request:** forge #204 — Warp Workflows analog: save a command under a name (optional `{{param}}`)
  and re-run it from the command palette.
- **Classification / tier:** work pipeline, LARGER (two pure surfaces — the substitute engine + a
  settings round-trip — plus a meaty palette shim). Kept to ONE shippable slice (bounded v1); the
  interactive multi-param prompt modal + edit/delete UI are deferred to a follow-up.
- **Forge recall (§18.3):** aar-open → 23fd1e42. Deps: command palette (palette.rs + the app.rs
  overlay, #199 dynamic-command pattern), marley_settings (#87 RemoteHosts list pattern), history
  (recent command for save-as). Pivoted here from #202 (parked on chad's A/B/C fork).
- **Discovery (code read):**
  - `marley_remote::RemoteHost` (lib.rs:135) = `#[derive(Debug, Clone, PartialEq, Eq, Serialize,
    Deserialize)] { name: String, target: String }` — `Workflow` mirrors its derives.
  - `define_setting!(RemoteHosts: Vec<RemoteHost> = Vec::new(), "remote.hosts")` (settings.rs:42) +
    `AppliedSettings.remote_hosts` (114) + `manager.get::<RemoteHosts>()` (204) + the absent/malformed
    tolerance (settings tests ~429). `persist_theme`/`persist_grid`/… (settings.rs:223+) are the
    persist-fn precedent. `Workflows: Vec<Workflow>` + `persist_workflows` mirror all of this exactly.
  - Palette dynamic commands: #199 `THEME_BASE`/`theme_pick_index`, #87 `CONNECT_BASE` — a
    `WORKFLOW_BASE` range + a pure `workflow_pick_index` is the direct analog.
  - Input mechanism: `renaming_tab: Option<(usize,usize,String)>` (app.rs:200) is the reusable inline
    text-draft (Enter commits, Esc cancels) — a `naming_workflow: Option<String>` draft reuses it for
    the save name-prompt.
  - Cooked-buffer insert: `buffer.edit(caret..caret, &cmd, EditOrigin::Human)` (the #59/#65/#200 rule
    — insert at Marley's prompt, not a raw PTY write).
- **The bounded delta:**
  - PURE (a): `workflows.rs` — `Workflow`, `substitute(template, &BTreeMap) -> Result<String,
    SubstituteError{MissingParam}>`, `params_of(template) -> Vec<String>`. cov/MSI 100 (run `cargo
    mutants --list`; recall the syntactic-form + enum/Result-body-default lessons).
  - PURE (b): settings round-trip — `Workflows` setting + `persist_workflows` + `AppliedSettings`.
  - SHIM: palette WORKFLOW_BASE list + a "Save command as workflow…" command + the `naming_workflow`
    save draft + the invoke buffer-insert.
- **Decisions:** D1 substitute + SubstituteError::MissingParam + params_of; D2 Workflow serde derives
  (mirror RemoteHost); D3 Workflows setting + persist_workflows (mirror #87); D4 WORKFLOW_BASE palette
  range + workflow_pick_index; D5 invoke→buffer-insert + save→renaming_tab-style draft; D6 clean-room.
- **Open for design:** the exact `substitute`/`params_of` tokenizer (a manual scan vs a regex — prefer
  a manual char scan, no regex dep; pin the non-`{{}}`-brace edges); the palette save-command trigger +
  the `naming_workflow` render/key-routing; whether the invoke auto-runs (enter) or just inserts
  (recommend INSERT, let the user press enter — safer + lets them fill params); CONFIRM the param-modal
  deferral (or a minimal single-param prompt).

## Phase 2 — Design

### Architecture / approach
Two pure surfaces (`workflows.rs` + the `settings.rs` round-trip, gpui-free, cov/MSI 100) + a masked
`app.rs` shim (palette registration + dispatch + the invoke buffer-insert + a small naming overlay).

**PURE (a) — `crates/marley_app/src/workflows.rs` (NEW):**
```rust
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workflow { pub name: String, pub command: String, pub params: Vec<String> }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubstituteError { MissingParam(String) }

// str::find scan (NOT a char-Vec index scan) → a small mutant surface (find/trim/get/push_str are
// method calls, unmutated; only the two `+ 2`s + the Result/String body defaults are viable).
pub fn substitute(template: &str, args: &BTreeMap<String, String>) -> Result<String, SubstituteError> {
    let mut out = String::new();
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        if let Some(end) = after.find("}}") {
            let name = after[..end].trim();
            match args.get(name) {
                Some(v) => out.push_str(v),
                None => return Err(SubstituteError::MissingParam(name.to_string())),
            }
            rest = &after[end + 2..];
        } else {
            out.push_str("{{");   // unterminated {{ → literal
            rest = after;
        }
    }
    out.push_str(rest);
    Ok(out)
}

pub fn params_of(template: &str) -> Vec<String> {   // distinct non-empty trimmed names, first-seen order
    let mut names = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        let after = &rest[start + 2..];
        if let Some(end) = after.find("}}") {
            let name = after[..end].trim().to_string();
            if !name.is_empty() && !names.contains(&name) { names.push(name); }
            rest = &after[end + 2..];
        } else { rest = after; }
    }
    names
}
```
Byte-safe: `{{`/`}}` are ASCII, so `find` byte offsets + `+ 2` land on char boundaries; `after[..end]`
is a boundary (before `}}`). Multi-byte names slice cleanly. No `unwrap`/panic on the scan.

**PURE — `palette.rs` rename:** `theme_pick_index(id, base, count)` is ALREADY generic (no theme logic).
RENAME it → `dynamic_command_index` (the honest name now it serves theme + workflow ranges), update the
#199 call site + the test name; reuse it for the workflow range. DRY + pre-empts the inspect naming/reuse
finding. (Behavior identical — a pure-module rename can't change behavior.)

**PURE (b) — `settings.rs` round-trip** (MIRRORS #87 `RemoteHosts` exactly):
- `define_setting!(/// #204 saved command-palette workflows. \n Workflows: Vec<Workflow> = Vec::new(),
  "workflows");` (import `crate::workflows::Workflow`).
- `AppliedSettings.workflows: Vec<Workflow>` (beside `remote_hosts` @114) + `applied_defaults`
  (`Vec::new()`) + `applied_from` (`workflows: manager.get::<Workflows>()`, @204-style).
- `persist_workflows(manager: &mut SettingsManager, workflows: &[Workflow]) -> Result<(),SettingsError>`
  = `manager.set::<Workflows>(workflows.to_vec())` (mirrors `persist_theme` @223). Malformed/absent →
  the default empty list (automatic — `manager.get` returns the default; the #87 tolerance).

**SHIM — `app.rs` (masked):**
- Fields: `workflows: Vec<Workflow>` (seeded from `applied.workflows` at boot) + `naming_workflow:
  Option<(String, String)>` (the name draft, the command-to-save). `WORKFLOW_BASE: u32 = 3000` const.
- `cockpit_commands()` gains a `Command { id: CommandId(9), title: "Save Command as Workflow…",
  keywords: [workflow, save] }` (id 9 is a free static id — `action_for_command` returns None for it →
  dispatched specially). Name the const `SAVE_WORKFLOW_ID = CommandId(9)`.
- Constructor command-build: after the #199 theme loop, a workflow loop — `for (i, wf) in
  applied.workflows.iter().enumerate() { commands.push(Command { id: CommandId(WORKFLOW_BASE + i as u32),
  title: format!("Workflow: {}", wf.name), keywords: ["workflow","run"], binding: None }); }`.
- Dispatch (`handle_palette_key` enter, after the theme branch @1359): `} else if id == SAVE_WORKFLOW_ID
  { self.begin_naming_workflow(cx); } else if let Some(idx) = dynamic_command_index(id, WORKFLOW_BASE,
  self.workflows.len()) { self.invoke_workflow(idx); }`. (id 9 < every BASE → the checked_sub/range checks
  all return None, so it correctly falls to the `== SAVE_WORKFLOW_ID` arm.)
- `invoke_workflow(&mut self, idx)`: `let wf = self.workflows[idx].clone(); let cmd =
  substitute(&wf.command, &BTreeMap::new()).unwrap_or_else(|_| wf.command.clone());` (no-param →
  substituted; a param workflow → the raw template to fill inline) → insert into the focused terminal's
  buffer via the #200 pattern (`state.buffer.edit(state.caret..state.caret, &cmd, EditOrigin::Human);
  state.caret = CharOffset::from(state.caret.as_usize() + cmd.chars().count());`). Do NOT auto-run.
- `begin_naming_workflow(&mut self, cx)`: capture the command to save = the focused terminal's last
  submitted command (`state.history.recent().first()`) else the current `state.buffer.text()`; if BOTH
  empty → a `status_flash("no command to save")` + return (no draft). Else `self.naming_workflow =
  Some((String::new(), cmd))`; `self.palette_open = false`; `cx.notify()`.
- Naming modal key block (MIRROR `renaming_tab` @3377): `if view.naming_workflow.is_some() { match key {
  escape → None, enter → commit, backspace → pop .0, space → push ' ', 1-char (no platform/ctrl) →
  push key_char } }` — checked as a modal (beside renaming_tab; one modal at a time). COMMIT: `if let
  Some((name, cmd)) = view.naming_workflow.take() { let name = name.trim(); if !name.is_empty() { let
  params = params_of(&cmd); view.workflows.push(Workflow { name: name.into(), command: cmd, params });
  view.commands.push(Command { id: CommandId(WORKFLOW_BASE + (view.workflows.len()-1) as u32), title:
  format!("Workflow: {}", name), keywords: ["workflow","run"], binding: None }); let _ =
  persist_workflows(&mut view.settings, &view.workflows); } }` (append keeps the id = base+index
  consistent — v1 has no delete, so no id reuse/collision; a future delete needs a rebuild → follow-up).
- Naming overlay RENDER: a small centered input box (when `naming_workflow.is_some()`) — `"Save
  workflow: {name}▏"` + a muted hint of the command; `.occlude()` backdrop (the modal rule). ~15 lines
  mirroring the palette overlay shell.
- CONFIRM at implement: RootView's settings-manager field name (persist_theme's `manager` arg source —
  the shim holds a `SettingsManager`); `state.history.recent()`/`buffer`/`caret` access via
  `focused_terminal_mut()` (the #200 path); `BTreeMap`/`EditOrigin`/`CharOffset` imports.

### File manifest
| File | Change |
|---|---|
| `crates/marley_app/src/workflows.rs` | NEW pure: `Workflow`, `SubstituteError`, `substitute`, `params_of`. Tests at validate. |
| `crates/marley_app/src/lib.rs` | `mod workflows;` |
| `crates/marley_app/src/palette.rs` | rename `theme_pick_index` → `dynamic_command_index` (+ its test name). |
| `crates/marley_app/src/settings.rs` | `Workflows` `define_setting!` + `AppliedSettings.workflows` (+ defaults/applied_from) + `persist_workflows`. |
| `crates/marley_app/src/app.rs` | `workflows`/`naming_workflow` fields; `WORKFLOW_BASE`/`SAVE_WORKFLOW_ID`; the cockpit save-command + the boot workflow-command loop; the dispatch save/invoke branches; `begin_naming_workflow`/`invoke_workflow`; the naming modal key block + overlay render; the `theme_pick_index`→`dynamic_command_index` call sites. (Masked shim.) |

### Regression Test Plan
`substitute`/`params_of` — reasoned real mutant set (CONFIRM at validate with `cargo mutants --list -f
workflows.rs`): the two `+ 2`s (`+` → `-`/`*`), the `!name.is_empty()` + `!names.contains` guards
(delete-!), the Result/String/Vec body defaults; `find`/`trim`/`get`/`push_str`/`contains` are method
calls (unmutated).

| # | REQ | Test (exact-value, workflows.rs `#[cfg(test)]`) | Kills |
|---|---|---|---|
| T1 | REQ-001 | `substitute("git push {{remote}}", {remote:origin})` == `Ok("git push origin")` | token mid-string → `start + 2`; body |
| T2 | REQ-001 | `substitute("echo {{a}} {{b}}", {a:1,b:2})` == `Ok("echo 1 2")` | multi-token loop → `end + 2` |
| T3 | REQ-001 | `substitute("{{a}}b", {a:X})` == `Ok("Xb")` | token at start (start=0) |
| T4 | REQ-002 | `substitute("x{{p}}", {})` == `Err(MissingParam("p"))` | the Err path; body→Ok |
| T5 | REQ-003 | `substitute("echo {hi}", {})` == `Ok("echo {hi}")` | single-brace literal (no `{{`) |
| T6 | REQ-003 | `substitute("a {{b", {})` == `Ok("a {{b")` | unterminated `{{` → literal (the else arm) |
| T7 | REQ-001 | `substitute("a {{ x }} b", {x:1})` == `Ok("a 1 b")` | trim the token name |
| T8 | params | `params_of("{{a}} {{b}} {{a}}")` == `["a","b"]` | dedup + order (`!contains`) |
| T9 | params | `params_of("no tokens")` == `[]` AND `params_of("{{ x }}")` == `["x"]` | empty + trim + `!is_empty` |
| T10 | params | `params_of("{{a}} {{b")` == `["a"]` | **(inspect F1)** the dangling-`{{` else branch (workflows.rs:71) — MANDATORY for cov-100; T8/T9 never reach it |

Plus: `settings.rs` — a workflows round-trip identity test + malformed-table→empty (REQ-004, mirror the
#87 `RemoteHosts` tests). `palette.rs` — the renamed `dynamic_command_index` test (same 6 cases).
**Driven (shim):** save `deploy = git push origin` via the palette "Save Command as Workflow…" →
`deploy` appears as "Workflow: deploy" (REQ-005) → invoke it → `git push origin` inserts at the prompt
(REQ-006); a `{{param}}` workflow inserts its template. (The settings-file persistence proof is the
REQ-004 unit round-trip; a full relaunch capture is optional.)

### Risks / decisions
- **R1 (append vs rebuild):** saving appends `Command { id: WORKFLOW_BASE + (len-1) }` — consistent
  because v1 has NO delete (no id reuse). A future delete/reorder needs a full command rebuild → the
  follow-up. Documented.
- **R2 (rename `theme_pick_index`):** touches shipped #199 code (the fn + its test + the call site) — a
  pure-module rename, behavior-identical; done to keep the reused fn honestly named.
- **R3 (naming overlay is new UI):** a small centered input + a modal key block mirroring `renaming_tab`
  — bounded; `.occlude()` backdrop per the modal rule.
- **R4 (invoke inserts, doesn't auto-run):** safer + lets the user fill `{{param}}` tokens before Enter.
  The dedicated interactive param-prompt modal is the deferred follow-up.
- **R5 (clean-room §20):** Warp Workflows is the concept; `substitute`/the model/the palette wiring are
  original.

## Phase 3 — Implement
**Built (per manifest):**
- `workflows.rs` (NEW pure) — `Workflow` (serde derives) + `SubstituteError` + `substitute` (str::find
  scan) + `params_of`. gpui-free.
- `lib.rs` — `mod workflows;` (between `viewport` and `workspace`).
- `palette.rs` — renamed `theme_pick_index` → `dynamic_command_index` (fn + doc + test name); signature
  unchanged.
- `settings.rs` — `Workflows: Vec<Workflow>` `define_setting!` + `AppliedSettings.workflows` (field +
  applied_defaults + applied_from + the 3 test literals) + `persist_workflows`.
- `app.rs` (masked shim) — `workflows`/`naming_workflow` fields + inits; `WORKFLOW_BASE=3000` +
  `SAVE_WORKFLOW_ID=CommandId(9)` consts; the boot save-command push + the workflow-command loop; the
  dispatch save/invoke branches; `invoke_workflow`/`begin_naming_workflow` (masked); the naming modal
  key block (mirrors renaming_tab) + the centered overlay render; the `dynamic_command_index` call-site
  rename.

**Deviations from design (with reason):**
- **Added `serde` as a DIRECT `marley_app` dep** (`Cargo.toml`, `serde = { version = "1", features =
  ["derive"] }`, mirroring marley_remote) — it was only TRANSITIVELY present; the `Workflow` derive
  needs it in-scope. (The design assumed it was already direct.) It's already in the tree (deny/audit
  fine) and used → cargo-machete clean.
- **The persist call uses `if let Some(manager) = view.settings.as_mut() { persist_workflows(manager,
  …) }`** — `self.settings` is `Option<SettingsManager>` (per persist_theme's call site @app.rs:1015),
  NOT `&mut SettingsManager` as the brief's shorthand implied. Disjoint-field borrow (`settings` vs
  `workflows`) → clean.
- **The "Save Command as Workflow…" command is pushed at BOOT, not added to `cockpit_commands()`** — a
  regression the first test run caught (`every_cockpit_command_resolves_to_a_verb` asserts every
  `cockpit_commands()` id resolves via `action_for_command` to a verb; the save-command is
  special-dispatched by id, not a verb). Boot-pushing it (like the dynamic connect/theme/workflow
  commands, which also aren't verbs) keeps the invariant + the test green. Better placement anyway.

**Checks:** `cargo fmt` + `cargo check --all-targets -p marley` clean (no warnings; only the upstream
`block v0.1.6` note). `cargo nextest run -p marley` → **307 passed, 2 skipped** (incl. the renamed
`dynamic_command_index_maps_range` + the invariant test green). Workflow tests are Phase 4.

## Phase 3.5 — Inspect
2 general-purpose critics in parallel (Critic 1: pure `substitute`/`params_of` + mutation + the settings
round-trip + the palette rename; Critic 2: the shim — dispatch id-space, append id-consistency, invoke
insert, the naming modal, security, clean-room) + self-review. Both ran code empirically (a standalone
harness for substitute/params_of, a real toml round-trip crate, debug-build mutant simulations, a full
dispatch trace). **Both verdicts: SHIP — no HIGH/MED code defects.**

| # | Finding | Sev | Verdict | Action |
|---|---|---|---|---|
| F1 | **cov-100 gap:** `params_of`'s dangling-`{{` else branch (workflows.rs:71) is NOT exercised by the planned T1–T9 — T8/T9 only hit terminated `{{…}}`. (The `substitute` analog IS covered by T6.) | MED | REAL (Phase-4 test req) | Phase 4 MUST add a `params_of` dangling case: `params_of("{{a}} {{b")` == `["a"]` — added to the test plan below. Without it the cov gate fails. |
| F2 | MSI-100 needs BOTH T8 and T9, and BOTH T1 and T3 — the `end±2` mutants are killed by DIFFERENT tests (T8 kills `end-2` via panic, T9 kills `end*2` via panic); T3 kills `start±2` via the offset-0 case. None is redundant. | MED | REAL (Phase-4 guidance) | Keep the full T1–T9 + F1 matrix; do NOT prune "redundant-looking" cases. Documented in the test plan. |
| F3 | A `[[workflows]]` entry missing the `params` key → `missing field 'params'` → the whole `Workflows` setting drops to the default empty list (wipes ALL saved workflows). Mirrors #87 `RemoteHost` (also no default), but `params` is AUTO-DERIVED (not hand-authored), so a hand-editor naturally omits it. | LOW | REAL (fixed) | **FIXED** workflows.rs — `#[serde(default)]` on `Workflow.params` (a missing key → `[]`, not a wipe). A justified divergence from #87 (whose fields are hand-authored). |
| F4 | `{{}}` asymmetry: `substitute("{{}}")` → `Err(MissingParam(""))` but `params_of("{{}}")` → `[]`. | LOW | REJECTED (benign) | No fix — `invoke_workflow`'s `.unwrap_or` swallows the Err (inserts literal `{{}}`); only a future param-fill UI would care; fixing adds a branch/mutant for a non-real input. Documented. |
| F5 | `Workflow.params` is stored/persisted but not read at runtime (invoke re-derives from the template); save prefers history over an unsubmitted prompt; empty-name Enter / non-terminal-pane invoke are silent no-ops; the CONNECT/THEME 1000-id blocks tolerate <1000 entries each. | LOW | REJECTED (by-design) | All by-design/spec + consistent with shipped idioms. The id-headroom ceiling is a future consideration, not #204's bug. |

**Critic-verified CLEAN:** substitute/params_of correct on all T1–T9 + adversarial (`{{a}}{{b}}`→"12",
`{{{{a}}}}`→self-consistent, adjacent/lone braces); **byte-safe on multi-byte names/values** (`{{café}}`,
`{{π}}` — no panic; `{{`/`}}` ASCII → `find`+2 land on boundaries); the real 16-mutant set is fully
killed by T1–T9(+F1); the settings round-trip mirrors #87 exactly + a real toml round-trip is identity +
malformed→default; the palette rename is byte-identical (zero stale `theme_pick_index` refs); dispatch
is collision-free across all id ranges (traced); append id-consistency holds boot↔runtime; the invoke
insert is byte-identical to the shipped history-finder/completion idioms; the naming modal is
first-checked + `return`-isolated; the persist borrow is disjoint; double-modal is unreachable; no
regression (added else-ifs only); security clean (user-authored text → the user's OWN buffer, not
spawned; persist to the user's own config); clean-room (original engine/model/wiring).

**Post-fix:** `cargo check -p marley` clean. No failure-record (no bug shipped — F3 is a robustness
hardening, F1/F2 are test-completeness). F1 carried into the Phase-4 test plan (below, mandatory).

## Phase 4 — Validate
**Tests written:**
- `workflows.rs` — `substitute_fills_tokens_and_leaves_literal_braces` (T1–T7) +
  `params_of_collects_distinct_trimmed_names` (T8/T9/T10). T10 (inspect F1) covers the dangling-`{{`
  else branch; T1+T3 and T8+T9 are each individually MSI-load-bearing (F2).
- `settings.rs` — `workflows_setting_round_trips_and_tolerates` (REQ-004): persist→reload identity
  (incl. a nested `params` array + an empty-params workflow), absent→empty, malformed→empty, and the
  F3 missing-`params`→empty-params (not a wipe). Mirrors the #87 `RemoteHosts` test.

**Ran (ACTUAL):**
- `cargo nextest run -p marley` → **310 passed, 2 skipped** (+3 new).
- Targeted mutation `cargo mutants -f workflows.rs` → **16 mutants, 16 caught** = cov/MSI 100 (exactly
  the set Critic 1 predicted; T1–T10 kill all).

**Driven captures — ENV-BLOCKED (documented, not skipped):** on re-bundle + launch, the live captures
failed — a full-screen `screencapture` showed the **macOS lock screen** ("Fri Jul 10 · Chad Peppers ·
Enter Password"). The machine locked at some point after the #201/#203 captures (which succeeded
earlier this session). Behind the lock the marley window can't be captured (`-l$WIN` → "could not
create image from window") or driven (the lock screen intercepts synthetic input); `caffeinate -u`
woke the display TO the lock screen but cannot unlock it, and I will NOT attempt a password. This is the
[[marley-ax-visual-testing]] "display asleep/locked → capture env-blocked, outside my control" case.
REQ-005/006 are therefore verified by:
- **REQ-005 (save→appears+persist):** the settings round-trip unit test proves the persist→reload path
  (a saved `Workflow` survives + reloads); the palette registration is the same `WORKFLOW_BASE`
  dynamic-command pattern as #199 themes (live-proven in prior tickets).
- **REQ-006 (invoke→inserts at prompt):** the `invoke_workflow` buffer-insert is **byte-for-byte
  identical** to the shipped, previously-live-proven history-finder-accept (app.rs:1531) and
  completion-accept (app.rs:1591) idioms (both critics verified this); `substitute` is cov/MSI 100.
Both critics traced the dispatch (collision-free) + the naming modal + the insert and returned SHIP.
Per §7, the mechanism + units carry REQ-005/006 given the hard env block.

**Gate:** `git add -A` (source + docs + Cargo.lock/toml for serde; no secrets) → `scripts/gates.sh
--diff` → **GATE GREEN [diff]**, 15/15 (rustfmt, clippy -D warnings, tests, coverage ≥100%, mutation
MSI ≥100%, miri, audit, deny, machete [serde used], gitleaks, shellcheck, no-suppressions, source-bans,
docs, visual/AX [headless]). Receipt written. No pre-existing exclusions.

## Phase 5 — Complete
**Docs (§21):** CHANGELOG.md — #204 under `[Unreleased] ### Added` (below #203). app_shell.md — a #204
note appended to the #199 palette section (the pure workflows.rs, the settings round-trip, the
`theme_pick_index`→`dynamic_command_index` rename shared across THEME/WORKFLOW ranges, the shim
save/list/invoke) + updated the #199 note's fn-name reference to the renamed `dynamic_command_index`.

**Forge capture (§19):**
- `aar-submit` 23fd1e42 — completed, effectiveness 5. Lessons: (a) the settings LIST round-trip
  (`Workflows: Vec<Workflow>`) mirrors #87 `RemoteHosts` exactly — a proven reusable pattern for any
  persisted typed list; (b) a `str::find` scan for `substitute` keeps the mutant surface tiny (16
  mutants, all killed); (c) an enum-return fn's body mutant viability DEPENDS on a Default derive —
  `Result<String>`/`Vec<String>` HAVE Default so their body mutants are concrete+viable (unlike #203's
  `Notify` where `Default::default()` was unviable); RUN `--list`, don't assume; (d) rename an
  already-generic reused fn to the honest name to pre-empt the inspect naming finding.
- **No failure-record** (no bug shipped — F3 `#[serde(default)]` was a robustness hardening; F1/F2 were
  test-completeness).
- `prevention-rule-record` **PR-claude-selftest-locked-screen-blocks-capture-fall-back-to-mechanism-001**
  (id c2b662cf, low): a LOCKED/asleep macOS machine fully blocks the live-capture/synthetic-input harness
  (`screencapture -l` fails, full-screen shows the lock screen, CGEvents intercepted; `caffeinate -u`
  can't unlock; never attempt a password) — confirm via a full-screen capture, document as env-blocked,
  and verify via units + the critic-traced mechanism (esp. a byte-identical previously-live-proven idiom);
  gate-15 is headless so the gate stays green.

**Follow-up filed:** forge **#227** (f48f91d8) — "M12.2 — #204 follow-ups: interactive param-prompt modal +
workflow edit/delete" (the deferred multi-param prompt modal + edit/delete-with-rebuild + an optional
live-capture re-verify once unlocked).

**Close + archive:** forge ticket #204 → done. Local TICKET-204 → closed/. Pipeline doc pair →
completed/. Spec status → Phase 5 — Complete PASS.
