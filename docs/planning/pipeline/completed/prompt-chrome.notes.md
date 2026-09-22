# prompt + input row chrome — Notes

- **Forge ticket:** #37 `4121bade-c650-49c1-a0af-9e38d9102d5d`
- **AAR:** `f6d7851d-8f2f-4956-ab34-acfb52bb4895`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-037-prompt-chrome.md
- **Pipeline spec:** prompt-chrome.spec.md

## Phase 1 — Plan
- **Request:** forge #37 (M1.E "The Warp Look" seq-4, auto-approved) — the input-row chrome + a
  cwd/git context prompt.
- **Classification / tier:** work pipeline, `feature`, a PURE surface (current_prompt accessor in
  terminal_blocks + prompt_segments/pwd_label in marley_app — cov/MSI 100) + a SHIM input-row render.
  Two crates (terminal_blocks + marley_app).
- **Discovery (§18):**
  - `PromptInfo { pwd, git_branch, virtual_env, node_version }` (all `Option<String>`) — the pure
    prompt_segments decision surface.
  - `SessionModel` (apply.rs) holds `staged_prompt: Option<PromptInfo>` (private), set by a `Precmd`
    DCS (apply.rs:77) and `.take()`n when a command starts (apply.rs:64). So staged_prompt = the
    LIVE prompt's context. Expose via a `pub(crate)` accessor mirroring `blocks()`.
  - `TerminalSession` (session.rs) wraps `model: SessionModel` + delegates (`blocks`, `is_alt_screen`,
    `grid_styled_rows`) — add `current_prompt()` the same way. gpui-free.
  - marley_app modules: app/block_status/color/history/input/keymap/layout/palette/settings/
    shell_integration/themes/viewport/workspace → add `prompt`.
  - Deps #34 (mono) + #35 (accent) DONE.
- **Decisions:** D1–D5 in the spec (staged not last-block; pwd_label basename; pwd+git first cut;
  ❯ marker in accent; pure decision + shim row).
- **Open questions for Design:** pwd_label impl (`rsplit('/').find(!is_empty).unwrap_or(pwd)`) +
  its edge cases (root "/", trailing slash, empty, no-slash); the styled-caret block form (a thin
  accent bar vs a background span); whether current_prompt returns `Option<&PromptInfo>` (borrow) —
  yes, mirrors `blocks() -> &BlockList`.
- **AAR id:** `f6d7851d-8f2f-4956-ab34-acfb52bb4895`.

## Phase 2 — Design

### PURE — `crates/marley_app/src/prompt.rs` (NEW)
```rust
use marley_terminal::PromptInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentKind { Cwd, Git }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptSegment { pub text: String, pub kind: SegmentKind }

/// The context segments for a live prompt (R43): a `Cwd` segment iff `pwd` is set, then a `Git`
/// segment iff `git_branch` is set — order `[Cwd, Git]`, each absent field contributing nothing.
pub fn prompt_segments(info: &PromptInfo) -> Vec<PromptSegment> {
    let mut segs = Vec::new();
    if let Some(pwd) = &info.pwd {
        segs.push(PromptSegment { text: pwd_label(pwd).to_string(), kind: SegmentKind::Cwd });
    }
    if let Some(branch) = &info.git_branch {
        segs.push(PromptSegment { text: branch.clone(), kind: SegmentKind::Git });
    }
    segs
}

/// The last non-empty `/`-separated component of `pwd` (`/Users/c/marley → marley`, `/ → /`,
/// trailing-slash tolerant), so the cwd segment is compact. Env-free (`~`-relative deferred).
pub fn pwd_label(pwd: &str) -> &str {
    pwd.rsplit('/').find(|s| !s.is_empty()).unwrap_or(pwd)
}
```

### PURE — the `current_prompt` accessors (gpui-free terminal_blocks)
- `apply.rs` — `SessionModel::current_prompt(&self) -> Option<&PromptInfo>` (`pub(crate)`) =
  `self.staged_prompt.as_ref()` (mirrors `blocks()`).
- `session.rs` — `TerminalSession::current_prompt(&self) -> Option<&PromptInfo>` (pub) =
  `self.model.current_prompt()` (mirrors `is_alt_screen`/`blocks`).

### SHIM — `crates/marley_app/src/app.rs` (the prompt row, mutants::skip region)
Replace the bare `{before}▏{after}` with an input ROW: `.bg(colors.surface)` + padding + rounded, a
leading `❯` (U+276F) in `colors.accent`, the `prompt_segments(current_prompt())` (Cwd in `foreground`,
Git in `accent`), then the buffer split at the caret (#28 `split_at_caret`) with a styled caret block
(a thin `accent` bar) between the halves.

### File manifest
- M `crates/terminal_blocks/src/apply.rs` — `SessionModel::current_prompt` + model test.
- M `crates/terminal_blocks/src/session.rs` — `TerminalSession::current_prompt` + mock session test.
- A `crates/marley_app/src/prompt.rs` — the pure module + tests.
- M `crates/marley_app/src/lib.rs` — `mod prompt;`.
- M `crates/marley_app/src/app.rs` — the input-row render (imports `prompt::{prompt_segments,
  SegmentKind}` + `current_prompt`).
- M `docs/specs/SPEC-app-shell.spec.md` (R43) + `docs/specs/SPEC-terminal-blocks.spec.md`
  (current_prompt) + Mutation-Targets. CHANGELOG; arch docs.

### Mutation Targets
- `pwd_label` — the `'/'` separator, the `!s.is_empty()` predicate (→`is_empty()` returns the empty
  trailing → caught by `/a/b/c→c`), the `unwrap_or(pwd)` fallback (→`unwrap_or_default` → `/`→`""` →
  caught by `/→/`). Killed by the 5 edge-case fixtures.
- `prompt_segments` — the two `if let Some` pushes + the order. Killed by both/only-pwd/only-git/
  neither asserting the exact `Vec<PromptSegment>` (text+kind, order).
- `current_prompt` (both) — `as_ref()`/delegate → `None`. Killed by the Some-after-precmd tests.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `apply.rs::current_prompt_is_staged_context` — fresh→None; `precmd(PromptInfo{pwd:/home/c,git:main})`→Some with those values; `preexec`→None. `session.rs::current_prompt_delegates` — `dcs_plain("precmd;exit=0")`+pump→`current_prompt().is_some()`; fresh session→None. | unit (in-crate) |
| REQ-002 | `prompt_segments_by_field` — both→`[Cwd(label),Git(branch)]`; only pwd→`[Cwd]`; only git→`[Git]`; neither→`[]` | unit |
| REQ-003 | `pwd_label_last_component` — `/a/b/c→c`, `foo→foo`, `/a/b/→b`, `/→/`, `""→""` | unit |
| REQ-004 | the input-row render (❯ + segments + styled caret) | shim + masked prompt visual — chad-verified |
| REQ-005 | `scripts/gates.sh --diff` GREEN + receipt | gate |

Uncoverable: the app.rs input-row div layout (shim exclude; needs a live window).

### Risks / decisions
- D-2.1 `current_prompt` = `staged_prompt` (live context), NOT the last block's `.prompt` (stale after
  `cd`). D-2.2 Split the current_prompt tests: model-level asserts VALUES (direct PromptInfo, easy),
  session-level covers the DELEGATOR via `dcs_plain` (is_some/none — no exact DCS field syntax needed;
  the parser's pwd/git parsing is already covered elsewhere). D-2.3 `pwd_label` borrows (`&str`) —
  zero-alloc; `prompt_segments` owns via `.to_string()`. D-2.4 `❯` U+276F per the glyph-set rule.
- The prompt-context render depends on the shell-integration precmd actually reporting pwd/git — when
  it doesn't (no integration, or a bare shell), `current_prompt()` is `None` and the row shows just
  `❯` + the input; graceful.

## Phase 3 — Implement
- **Built (per manifest):** `prompt.rs` (NEW) — `SegmentKind{Cwd,Git}` + `PromptSegment{text,kind}` +
  `prompt_segments` (pwd→Cwd via `pwd_label`, git_branch→Git, order [Cwd,Git]) + `pwd_label`
  (`rsplit('/').find(!is_empty).unwrap_or(pwd)`); `apply.rs` `SessionModel::current_prompt` (pub(crate),
  `staged_prompt.as_ref()`) + `session.rs` `TerminalSession::current_prompt` (pub delegator) + the
  `PromptInfo` import; `lib.rs` `mod prompt;`; `app.rs` — the input ROW (surface + rounded + ❯ accent
  marker + the colored segments + the buffer split with a `w(2)×h(TERMINAL_FONT_SIZE)` accent caret
  bar) replacing the bare `{before}▏{after}`. SPEC-app-shell R43 + row 43 + Mutation-Targets;
  SPEC-terminal-blocks R27; CHANGELOG.
- **Deviations from design:** none. (Caret bar height uses `TERMINAL_FONT_SIZE` const — always in
  scope, no dependency on the `fallback` binding.)
- **Verification at this phase:** `cargo check --workspace` 0 errors; fmt; clippy `-D warnings` 0
  (marley + marley_terminal); docs gate 0. The prompt.rs + current_prompt unit suites are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (real scoped cargo-mutants on prompt.rs + a verbatim probe + apply.rs/session.rs/dcs.rs
  reads). Verdict: **SOUND — ship the code; the findings are all in the Phase-4 test plan.**
- **prompt.rs MSI CONFIRMED:** cargo-mutants = **4 caught / 1 unviable (`vec![Default]` — PromptSegment
  has no Default) / 0 missed → MSI 100**; NO equivalent mutant (the #34 concern clear). Edge outputs
  all verified incl. trailing-space component (`/a/b /`→`"b "`).
- **Findings table:**
  | # | Sev | Finding | Verdict | Carry-forward |
  |---|---|---|---|---|
  | F1 | MED | The `current_prompt` tests MUST run `InitShell` before `precmd` — `Precmd` returns `Err(MissingSession)` + stages NOTHING without a registered session (apply.rs:70). Without init: model test's `.unwrap()` panics; session test's `is_some()` is false → the delegator `→None` mutant SURVIVES (MSI<100). `dcs_plain("precmd;exit=0")` alone stages `Some(PromptInfo::default())` (fields None, Option Some) → `is_some()` true, but ONLY after init. | REAL (Phase-4 test bug, not a code bug) | Model test: `init(&mut m,1)` before `precmd`. Session test: `[dcs_plain("init;id=1"), dcs_plain("precmd;exit=0")].concat()` then pump then `is_some()`; keep fresh-session `is_none()`. |
  | F2 | LOW | My SPEC Mutation-Targets cited `'/'`-char + `unwrap_or→unwrap_or_default` mutants cargo-mutants never generates. | REAL (doc) | SPEC corrected to the real 5-mutant set. |
  | F3 | LOW | `prompt.rs` is UNTRACKED → a `--diff` gate before `git add` skips its mutants (gate:5 "0 viable"); gate:4 whole-workspace coverage still fails closed on 0% until tests exist. | REAL (process) | `git add -A` before the gate (my standing pattern) — prompt.rs's mutants get seen. |
- **Verified CLEAN:** accessors killable (both `→None` + `→Some(Box::leak(Default))` mutants, killed by
  Some-with-values + fresh/consumed→None); the shim (app.rs:808) uses `current_prompt().map(prompt_
  segments).unwrap_or_default()` + a `SegmentKind→color` match — NO inline segment logic; prompt.rs
  imports only `marley_terminal::PromptInfo` (gpui-free); terminal_blocks stays Hsla-free; no
  panic/unwrap-on-input; `marley_terminal` suite 80/80 green with the diff.
- **No code fix** — the diff is sound. F1 is the load-bearing Phase-4 instruction.
- **Lesson:** `PR-claude-state-machine-accessor-test-must-satisfy-preconditions` — a test for a
  staged/derived-state accessor must drive the state machine's FULL precondition sequence (here
  `InitShell` → `Precmd`), not just the final hook; skip a precondition and the stage silently no-ops,
  the assertion inverts, and the accessor's mutant survives (MSI<100) — a false-green trap.
- **Probe-hygiene note (from the critic):** running cargo-mutants `-j>1` against a SHARED target dir a
  prior `cargo test` populated can mis-report a stale test binary as a mutant "Build: Success"
  (a target-dir race) — the authoritative run is `-j1` / fresh isolated target dir. (The gate uses its
  own `mutants.out` copies, so unaffected; relevant only to hand probes.)

## Phase 4 — Validate
- **Tests added:** `prompt.rs` — `pwd_label_last_component` (5 edge cases incl. trailing-slash + root
  + empty) + `prompt_segments_by_field` (both/only-pwd/only-git/neither). `apply.rs` —
  `current_prompt_is_the_staged_context` (init→None; precmd(pwd,git)→Some values; preexec→None).
  `session.rs` — `current_prompt_delegates_the_staged_context` (fresh→None; `[init;precmd]` dcs+pump→
  Some). Both accessor tests INIT before precmd (the critic's MED — else the stage no-ops + the mutant
  survives).
- **Runs (actual):** `cargo nextest run -p marley -p marley_terminal` → 185 passed (all 4 new PASS);
  `cargo nextest run --workspace` → all green.
- **Gate:** `git add -A` (tracks the new prompt.rs so its mutants are seen — the critic's F3) then
  `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, mutation **8 caught / 0 missed →
  MSI 100.0%** (4 prompt.rs + 2 model + 2 session accessor mutants — the init-first tests kill both
  delegators). Receipt written. No PTY hang.
- **Pre-existing:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added` (at implement); `app_shell.md` (prompt-row bullet) +
  `terminal_blocks.md` (current_prompt bullet). SPEC-app-shell R43 + SPEC-terminal-blocks R27 at
  implement.
- **Knowledge captured:** `PR-…-state-machine-accessor-test-must-satisfy-preconditions` (the MED —
  test the current_prompt accessor by driving init→precmd, else the stage no-ops + the mutant
  survives). aar-submit `completed` (score 5). Win: the critic ran a real cargo-mutants (prompt.rs
  MSI 100, no equivalent) AND caught the test-precondition trap BEFORE Phase 4 — so the accessor
  tests were right first try (8/8 mutants). Exposing `current_prompt` on the gpui-free session gave
  the prompt a real cwd/git context (a signature Warp element) with a fully-tested pure surface.
- **Ticket:** forge #37 → done; local doc → closed/; pipeline pair archived. 4 of 6 in M1.E.
