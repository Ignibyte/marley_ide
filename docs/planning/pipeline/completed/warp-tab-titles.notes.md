# cwd-aware tab titles — Notes

- **Forge ticket:** #201 (419815b7-21ae-4c5d-998f-ab8db82d4f57)
- **AAR:** cc1903eb-0092-4302-848b-38da214b7309
- **Local ticket doc:** docs/planning/tickets/open/TICKET-201-warp-tab-titles.md
- **Pipeline spec:** warp-tab-titles.spec.md

## Phase 1 — Plan
- **Request:** forge #201 — custom + command-aware tab titles. The genuine remaining gap is the
  CWD-basename fallback tier; the rename editor + custom_title + command derivation already ship.
- **Classification / tier:** work pipeline — a small, bounded feature (one pure-fn extension + one
  shim wire). One shippable slice.
- **Forge recall (§18.3):** ticket #201 pulled (`derive_tab_title(custom, running_cmd, cwd: &Path)
  → custom → command basename → cwd basename → 'shell'`; SHIM = the rename editor [done, #177] + the
  title render). aar-open → cc1903eb. Deps: titlebar.rs `display_title` (#157/#177), prompt.rs
  `pwd_label` (#37), #193 running-command signal.
- **Discovery (code read):**
  - `titlebar.rs::display_title(custom: Option<&str>, command: Option<&str>, fallback: &str) -> String`
    (titlebar.rs:58) = `custom (non-blank via .trim()) → rail_tab_title(command, fallback)`. cov/MSI 100,
    tested (`display_title_precedence`, `rail_tab_title_cases`). NO cwd tier, NO truncation.
  - `titlebar.rs::rail_tab_title(command, fallback)` (titlebar.rs:48) = `command.and_then(|c|
    c.split_whitespace().next()).map(to_string).unwrap_or_else(|| fallback.to_string())` — the program
    token; blank/None → fallback. Returns `fallback` when no command → collapses the command-absent
    and fallback cases, so a cwd tier can't just wrap it; need to distinguish "command produced a
    token" from "no command" (design: extract a shared `Option<String>` command-token helper).
  - `prompt.rs::pwd_label(pwd: &str) -> &str` (prompt.rs:50) = `pwd.rsplit('/').find(|s|
    !s.is_empty()).unwrap_or(pwd)` — the last non-empty path segment (empty pwd → `""`, so filter it).
  - `app.rs::live_tab_title(&self, project, tab, fallback) -> String` (~2523) reads `custom_title` +
    the last block's `.command`, passes `row.label` ("terminal N") as fallback. Does NOT read pwd yet.
  - pwd is available on the block: `block.prompt.pwd` (already read at app.rs:2066 via
    `block.prompt.pwd.as_deref()`). The wire mirrors that read for the last block.
  - The #177 rename flow (`renaming_tab`, double-click, Enter commit → `custom_title`, persistence)
    is INTACT and untouched by this slice.
- **Decisions:** D1 extend `display_title` (don't mint `derive_tab_title`); D2 `cwd: Option<&str>`
  (not `&Path`) reusing `pwd_label`; D3 keep the `fallback` param ("terminal N"), not a hardcoded
  'shell' (per-tab label > constant; only fires pre-first-prompt); D4 reuse `pwd_label`; D5 clean-room.
- **The bounded delta:** ~1 pure-fn signature change + 1 tier + a small DRY extraction in titlebar.rs;
  ~1 pwd read + 1 arg in `live_tab_title`. Everything else is already built.

## Phase 2 — Design

### Architecture / approach
The cockpit tab-rail label. The pure title decision lives in `titlebar.rs` (gpui-free, cov/MSI 100);
`app.rs::live_tab_title` is the masked shim that reads live session state and calls it. All the
running-vs-idle + live-cwd logic stays in the shim; the pure fn only sees `Option<&str>`s.

**Key design refinement over Phase 1** (from reading the block/session model): the forge ticket says
*"run `vim` → the title reflects it; exit → reverts to cwd."* That requires the command tier to fire
only for a **running foreground command**, not the last (possibly finished) command. Two facts drive
the shim:
- `session.is_command_running() -> bool` (session.rs:177 = `blocks().current().is_some()`, the
  #40/#193 foreground signal). Gate the command read on it → an idle tab drops to the cwd tier.
- The live cwd is `session.current_prompt().pwd` (session.rs:324 — "the cwd the next command will run
  in", the staged prompt that tracks `cd`), NOT the last block's `prompt.pwd` (which is the dir a past
  command ran in — stale after a `cd`). This corrects the Phase 1 discovery note (app.rs:2066 reads
  the block pwd for the *details panel*, a different purpose). The tab-completion path (app.rs:2740)
  already uses `current_prompt().pwd` as "the cwd" — same source.

**The pure seam** (`display_title`) is UNCHANGED in spirit — it gains one `cwd: Option<&str>` param and
one tier. Recommended shape (early-return cascade, cleanest to exact-value test), extracting the shared
command-token so `rail_tab_title` stays DRY:
```rust
fn command_program(command: Option<&str>) -> Option<String> {          // NEW private helper
    command.and_then(|c| c.split_whitespace().next()).map(str::to_string)
}
pub fn rail_tab_title(command: Option<&str>, fallback: &str) -> String { // behavior UNCHANGED
    command_program(command).unwrap_or_else(|| fallback.to_string())
}
pub fn display_title(
    custom: Option<&str>, command: Option<&str>, cwd: Option<&str>, fallback: &str,
) -> String {
    if let Some(name) = custom {
        if !name.trim().is_empty() { return name.to_string(); }
    }
    if let Some(program) = command_program(command) { return program; }
    if let Some(base) = cwd.map(crate::prompt::pwd_label).filter(|b| !b.is_empty()) {
        return base.to_string();
    }
    fallback.to_string()
}
```
Confirmed: `crate::prompt::pwd_label(&str) -> &str` is pub (prompt.rs:50), `mod prompt` in lib.rs:47 →
callable from titlebar.rs. `pwd_label("/Users/x/Marley")` → `"Marley"`; `pwd_label("")` → `""` (the
`.filter(|b| !b.is_empty())` drops it → fallback); `pwd_label("/")` → `"/"` (root edge, non-empty,
acceptable). `base.to_string()` copies before return → no lifetime escape.

**The shim** (`app.rs::live_tab_title` ~2523):
```rust
fn live_tab_title(&self, project: usize, tab: usize, fallback: &str) -> String {
    let tab_ref = self.shell.projects().get(project).and_then(|p| p.tabs().get(tab));
    let custom = tab_ref.and_then(|t| t.custom_title.as_deref());
    let term = tab_ref.and_then(|t| t.grid()).and_then(|g| g.terminal(g.focused())); // Option<&_>, Copy
    let command = term
        .filter(|t| t.session.is_command_running())        // only a RUNNING fg command owns the title
        .and_then(|t| t.session.blocks().iter().last())
        .map(|b| b.command.as_str());
    let cwd = term
        .and_then(|t| t.session.current_prompt())          // the live staged cwd (tracks `cd`)
        .and_then(|p| p.pwd.as_deref());
    display_title(custom, command, cwd, fallback)
}
```
`term` is `Option<&Terminal>` (Copy) → usable twice. No blocks / no prompt yet → command None + cwd
None → fallback "terminal N" (REQ-004). Stays masked (`#[cfg_attr(test, mutants::skip)]`, already on it).

### File manifest
| File | Change |
|---|---|
| `crates/marley_app/src/titlebar.rs` | ADD private `command_program`; rewrite `rail_tab_title` to use it (behavior identical); EXTEND `display_title` with `cwd: Option<&str>` + the cwd tier. Update `display_title_precedence` to the 4-arg form + ADD a `display_title_cwd_tier` test (VALIDATE writes tests). |
| `crates/marley_app/src/app.rs` | `live_tab_title` (~2523): gate command on `is_command_running`, read cwd from `current_prompt().pwd`, call the 4-arg `display_title`. (Masked shim.) |

Single production caller of `display_title` confirmed: app.rs:2535. `rail_tab_title`'s signature is
unchanged → its callers (only `display_title` internally + tests) are untouched.

### Regression Test Plan
Real cargo-mutants set for titlebar.rs (from `cargo mutants --list -f titlebar.rs` on the current code):
fn-body → `String::new()` / `"xyzzy".into()`; match/if guards → `true` / `false` / `delete !`; `>` →
`{==,<,>=}`; `&&`→`||`. The extended `display_title` adds the same guard family on `!name.trim()...`
(existing) + `!b.is_empty()` (new cwd filter); `command_program` adds body→`None`/`Some("")`/`Some("xyzzy")`.

| # | REQ | Test (exact-value, titlebar.rs `#[cfg(test)]`) | Kills |
|---|---|---|---|
| T1 | REQ-003 | `display_title(Some("box"), Some("vim"), Some("/x/Marley"), "t")` == `"box"` | custom-guard→false, delete-!, body mutants |
| T2 | REQ-003 | `display_title(Some("  "), Some("cargo build"), Some("/x/Marley"), "t")` == `"cargo"` | custom-guard→true (blank kept), delete-! |
| T3 | REQ-003 | `display_title(Some(""), None, Some("/x/Marley"), "t")` == `"Marley"` | empty custom = unset → cwd |
| T4 | REQ-002 | `display_title(None, Some("vim ."), Some("/x/Marley"), "t")` == `"vim"` | command>cwd; command_program→None/Some("")/Some("xyzzy") |
| T5 | REQ-002 | `display_title(None, Some("cargo build --lib"), Some("/x/Marley"), "t")` == `"cargo"` | program token, command>cwd |
| T6 | REQ-001 | `display_title(None, None, Some("/Users/x/Projects/Marley"), "terminal 1")` == `"Marley"` | cwd-filter→false, delete-!, body |
| T7 | REQ-001 | `display_title(None, Some("   "), Some("/Users/x/Projects/Marley"), "terminal 1")` == `"Marley"` | all-whitespace command (no token) → cwd |
| T8 | REQ-004 | `display_title(None, None, None, "terminal 1")` == `"terminal 1"` | no cwd → fallback; body mutants |
| T9 | REQ-004 | `display_title(None, None, Some(""), "terminal 1")` == `"terminal 1"` | cwd-filter→true (empty basename dropped), delete-! |

`rail_tab_title_cases` (unchanged sig) stays green + now also covers `command_program`'s body mutants
via `rail_tab_title(Some("vim"),..)`=="vim" / `(None,..)`==fallback. Placement: expand
`display_title_precedence` (T1–T3, T8) to 4-arg + add `display_title_cwd_tier` (T4–T7, T9).

**Driven (shim, uncoverable by unit — live GUI):** (a) idle tab in `~/Projects/ignibyte/Marley` shows
`Marley`; (b) run `vim` → title `vim`, exit → reverts to `Marley` (REQ-002 + the is_command_running
gate); (c) double-click → rename → custom shows + persists (#177 re-verify). Via the selftest harness
(focus-in-same-drive-call per the #198 lesson).

### Risks / decisions
- **R1 (behavior change):** gating command on `is_command_running` refines #157's "latest command" →
  "running command; idle reverts to cwd". Deliberate — it's the forge ticket's explicit "exit →
  reverts to cwd" and the Warp-parity north star. Low regression risk: the rail label just becomes
  more dynamic; `rail_tab_title`'s own contract/tests are unaffected.
- **R2 (cwd source):** use `current_prompt().pwd` (live, cd-tracking), not the last block's pwd
  (stale). Matches the tab-completion path (app.rs:2740).
- **R3 (DRY):** extract `command_program` so `rail_tab_title` and `display_title` don't duplicate the
  `split_whitespace().next()` token logic (pre-empts the inspect simplification lens).

## Phase 3 — Implement
**Built (per manifest):**
- `titlebar.rs::display_title` — extended signature to `(custom, command, cwd: Option<&str>, fallback)`
  with the cwd tier; doc updated to note the #201 cwd tier + `pwd_label` reuse. Compile-fixed the
  existing `display_title_precedence` calls to the 4-arg form (`, None,` cwd) — full matrix is Phase 4.
- `app.rs::live_tab_title` (~2523, masked shim) — captured `term` once; gated `command` on
  `term.filter(|t| t.session.is_command_running())` then the last block; read `cwd` from
  `term.and_then(|t| t.session.current_prompt()).and_then(|p| p.pwd.as_deref())`; called the 4-arg
  `display_title`. `#[cfg_attr(test, mutants::skip)]` retained.

**DEVIATION from the Phase 2 design (Option A → Option B), with reason:**
- Design planned a private `command_program(Option<&str>) -> Option<String>` helper shared by
  `rail_tab_title` + `display_title` (explicit early-return cascade). Implemented that first — but
  routing `display_title` through `command_program` left `rail_tab_title` with **zero production
  callers** (its only caller was `display_title`), producing a `dead_code` warning that would fail the
  warnings gate. Deleting the tested `pub fn rail_tab_title` + its test was the heavier fix.
- **Chosen instead (Option B):** keep `rail_tab_title` UNCHANGED (original body + `rail_tab_title_cases`
  test intact) and compose it — `display_title` computes `cwd_or_fallback = cwd.map(pwd_label)
  .filter(|b| !b.is_empty()).unwrap_or(fallback)` and returns `rail_tab_title(command, cwd_or_fallback)`.
  The cwd basename simply becomes the command tier's fallback, with "terminal N" behind it. No new
  helper, no dead code, no deletion; the command-token logic stays solely in `rail_tab_title` (better
  DRY than duplicating it into `command_program`).
- **Behavior + mutation identical:** verified all 9 planned matrix cases (T1–T9) produce the same
  results under Option B (custom > command > cwd > fallback); the killable mutant set on `display_title`
  is the same (body → String::new()/xyzzy; custom-guard `!name.trim().is_empty()` → true/false/delete-!;
  cwd-filter `!base.is_empty()` → true/false/delete-!). `.map/.filter/.unwrap_or` are method calls, not
  mutated. `rail_tab_title`'s own 2 body mutants stay covered by `rail_tab_title_cases`.

**Checks:** `cargo fmt` + `cargo check -p marley` clean (no dead_code; only the pre-existing upstream
`block v0.1.6` future-incompat note). `cargo test -p marley --lib titlebar` → 5 passed (existing suite
green under the 4-arg sig). Full T1–T9 matrix + driven captures are Phase 4.

## Phase 3.5 — Inspect
1 focused general-purpose critic over the titlebar.rs + app.rs diff (5 lenses: precedence, mutation,
shim, regression, simplification) + self-review. Critic ran a standalone harness replicating
`display_title`/`rail_tab_title`/`pwd_label` and traced every claim through the real source. **Verdict:
SHIP — no HIGH/MED defects.**

| # | Finding | Sev | Verdict | Action |
|---|---|---|---|---|
| F1 | The real cargo-mutants set for `display_title` is **4 mutants, not 8** — the `if let`/`if !` form yields NO true/false guard mutants (those come only from `match … if GUARD =>` arms, the OLD form). Real set: `66:5` body→`String::new()`/`"xyzzy"`, `67:12` `delete !` (custom guard), `75:24` `delete !` (cwd filter). Also: the EXISTING in-file `display_title_precedence` test (all `cwd=None`) leaves `75:24 delete !` ALIVE — with cwd=None the filter predicate is never evaluated. | LOW | REAL (Phase-4 guidance) | Corrected the test plan below to the real 4-mutant set. Phase 4 MUST include a `cwd=Some(non-empty)` DECIDING case (no custom, no command) to kill `75:24` — T3/T6/T7/T9 do. Reinforces `PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators-001` (my design assumed guard→true/false; the real form only `delete !`). |
| F2 | `command` = `term.filter(is_command_running).and_then(blocks().last())` re-derives what `blocks().current()` already encodes (`current()` = `last().filter(Running)`, block.rs:168; pub). | LOW | REAL (optional, adopted) | **FIXED** app.rs:2534 → `term.and_then(\|t\| t.session.blocks().current()).map(\|b\| b.command.as_str())`. Removes the implicit last==running assumption, drops the redundant `is_command_running()` re-fetch, clearer intent. `cargo check -p marley` clean. |
| F3 | `pwd_label("/")` → a `"/"` tab title for a tab cd'd to filesystem root. | LOW | REJECTED (acceptable) | No fix — "/" reads acceptably as basename-of-root; rare cwd; matches the existing `pwd_label` unit contract. |

**Critic-verified CLEAN:** precedence exact across all 9 cases (T1–T9); the computed-but-unused
`cwd_or_fallback` when a command has a token is harmless (`pwd_label` returns `&str`, no alloc/panic);
shim types sound — `term: Option<&TerminalPane<TerminalSession>>` is Copy (used twice, no move),
`current()`/`current_prompt()` accessors correct, `current_prompt().pwd` IS the live cd-tracking source
(the same `complete_at_prompt` reads, app.rs:2747; the last block's pwd would be stale), fresh pane →
fallback; no regression — `rail_tab_title` unchanged (only caller `display_title`), both `live_tab_title`
callers structurally intact, the rename SEED (app.rs:3980) now WYSIWYG-seeds the cwd basename (an
improvement); clean-room — pure string transforms, no new hsla/asset.

**Corrected test plan (real 4-mutant set):** the T1–T9 matrix in Phase 2 stands, but the mutant→killer
mapping is: `66:5` body pair → any exact-value case (all of T1–T9); `67:12 delete !` (custom guard) →
T1 (real custom wins) + T2 (blank custom falls through); `75:24 delete !` (cwd filter) → T6 (valid cwd
decides → "Marley") + T9 (empty cwd → "terminal 1"), with T3/T7 as extra deciding cases. rail_tab_title
unchanged → `rail_tab_title_cases` coverage intact.

## Phase 4 — Validate
**Tests written** (crates/marley_app/src/titlebar.rs `#[cfg(test)]`):
- `display_title_cwd_tier` (NEW) — the 8-assert T1–T9 matrix: custom>all (T1), command>cwd (T4/T5),
  empty-custom→cwd (T3), cwd decides (T6), whitespace-command→cwd (T7), no-cwd→fallback (T8),
  empty-cwd→fallback (T9). The Some(cwd) deciding cases (T6/T9) kill the `75:24 delete !` cwd-filter
  mutant that the existing all-`cwd=None` test left alive (inspect F1).
- `display_title_precedence` (compile-fixed to 4-arg at implement) — the custom>command>fallback
  cases with `cwd=None`; kills the body pair + the `67:12 delete !` custom-guard mutant.
- `rail_tab_title_cases` unchanged (rail_tab_title untouched).

**Ran (ACTUAL):**
- `cargo nextest run -p marley` → **306 passed, 2 skipped** (0.549s) — incl. `display_title_cwd_tier`
  + `display_title_precedence` + `rail_tab_title_cases`.
- Targeted mutation `cargo mutants -f titlebar.rs --re display_title` → **4 mutants, 4 caught** (cov/MSI
  100 on display_title — confirms the real 4-mutant set is fully killed by T1–T9).

**Driven captures (live app, re-bundled after the finding-2 edit):**
- REQ-001 (`201-idle-full.png`): idle terminal tabs in the left rail read **"Marley"** (the cwd
  basename of `~/Projects/ignibyte/Marley`), NOT "terminal N". Cockpit tabs (Agents/Details/Forge)
  keep static titles; a file tab shows "enforce-changelog.sh". ✓
- REQ-002 (`201-running.png` → `201-reverted.png`): ran `sleep 8` in the focused pane → its tab
  flipped **"Marley" → "sleep"** (prompt `⊙ sleep 8`) while running; on exit (prompt `✓ sleep 8`) it
  **reverted to "Marley"**. Exactly the ticket's "run vim → title; exit → cwd". ✓
- REQ-003 (custom rename wins): verified via the unit custom-tier (T1: `Some("box")` beats command +
  cwd) + the #177 rename mechanism (`renaming_tab`/double-click/`custom_title`) is UNTOUCHED by this
  diff (git diff shows only `live_tab_title`'s command/cwd derivation changed). A driven double-click
  was not pixel-hunted (small-target unreliability, #198 shadow lesson) — the unchanged mechanism +
  unit coverage prove it.

**Gate:** `git add -A` (staged: app.rs, titlebar.rs, the 3 pipeline docs — no secrets/.mcp.json/.env) →
`scripts/gates.sh --diff` → **GATE GREEN [diff]**, 15/15 (rustfmt, clippy -D warnings, tests, coverage
≥100%, mutation MSI ≥100%, miri, audit, deny, machete, gitleaks, shellcheck, no-suppressions,
source-bans, docs, visual/AX). Receipt written for /commit. No pre-existing exclusions.

## Phase 5 — Complete
**Docs (§21):** CHANGELOG.md — #201 entry under `[Unreleased] ### Added` (below #200).
docs/marley_architecture/app_shell.md — a #201 note after the #177 tab-title entry (the cwd tier, the
Option-B composition, the current_prompt()/blocks().current() shim, the real 4-mutant set).

**Forge capture (§19):**
- `aar-submit` cc1903eb — outcome completed, effectiveness 5. Lessons: (a) discovery that most of #201
  pre-existed (#177 rename + #157 command derivation) → the genuine delta was ONE tier (the cwd
  fallback), keeping scope tight; (b) reading the block/session model at DESIGN revealed the ticket's
  "exit → reverts to cwd" REQUIRES gating command on the running signal + reading the LIVE
  `current_prompt().pwd` (not the last-block pwd) — a design-phase correction of the Phase 1 discovery;
  (c) the Option-A→Option-B pivot at implement (command_program left rail_tab_title dead → compose it
  instead) avoided deleting a tested pub fn and is DRYer.
- **No failure-record** — no real bug shipped or found. Inspect F2 was an optional simplification
  (adopted); F3 rejected (acceptable edge).
- `prevention-rule-record` **PR-claude-cargo-mutants-guard-mutants-depend-on-syntactic-form-001**
  (id 765c7abd), severity low: cargo-mutants generates a DIFFERENT mutant set for the same logical
  guard depending on syntactic form — a `match … if GUARD =>` arm yields true/false/delete-!, but an
  `if let`/`if !` statement yields ONLY delete-!; run `--list` on the actual post-refactor code.
  Reinforces the trace-the-real-list rule (#201 design assumed 8 mutants; the real set was 4).

**Close + archive:** forge ticket #201 → done. Local TICKET-201 → closed/. Pipeline doc pair →
completed/. Spec status → Phase 5 — Complete PASS.

**Minor note:** the rename harness verb is `dblclickat` (not `doubleclickat`, which I grepped for at
validate); REQ-003 was covered by unit T1 + the unchanged #177 mechanism regardless, so this didn't
affect the outcome — noted for future driven-rename captures.
