---
pipeline_id: 9c847336-2bd7-4201-8975-7c4ce2a34a94
aar_id: 1090af94-6457-4151-80a3-1accaa2f2916
---

# marley_app (app-shell, minimal) — pipeline notes

## Phase 1 — Plan (2026-06-29)

**Intent:** THE FINALE (M1.A seq 5/5, the last ticket of chad's /goal "/work 12 to 16") — the first
runnable Marley: `run()` boots one "Marley" gpui window mounting a live marley_terminal pane + the
marley_editor input; type → Enter → the command runs → a Block appears. `cargo run -p marley` opens a
usable bare terminal. The minimal cut of SPEC-app-shell (the full workspace shell grows in M1.B).

**The cut call:** SPEC-app-shell is the FULL workspace shell (R1-R24: docks, pane algebra, palette,
keymap, multi-theme). The forge ticket scopes M1.A to "minimal: one terminal pane + the input, Enter→run
→block." So KEEP R1/R2/R3 (boot) + R21-R24 (theme) + ADD the NEW wiring ACs W1-W5 (mount terminal, mount
input, key→edit, Enter→run→block, live pump) — the spec OUT-OF-SCOPES terminal content (line 239 "lays
out the panes but does not drive sessions"), so #16 adds the drive/wire on top. DEFER R4-R20 + R7
(PaneGroup entirely — RootView owns terminal+input directly).

**CRITICAL — the crate layout (3 constraints, resolved zero-spec-edit):**
- gate-15's `visual_g` reads `component: marley_app` from the spec → checks `[ -d crates/marley_app ]`.
  A different dir → the UI component is SILENTLY SKIPPED (the blank-shipped-green trap the gate prevents).
  So **dir MUST be `crates/marley_app`** (NOT crates/app-shell).
- `cargo run -p marley` (the /goal) → `-p` is by PACKAGE name → `[package] name = "marley"`.
- The frozen surface `marley_app::{run,RootView,ThemeRegistry}` (M2 binds) → `[lib] name = "marley_app"`.
- → dir `crates/marley_app`, package `marley`, lib `marley_app`, `[[bin]] name="marley"`. Run: `cargo run
  -p marley`. (Valid Cargo; do NOT let cargo new auto-align the names.)

**Facts confirmed:** gpui = "0.2.2" (the spike/ui_components/harness pin); the spike has the WORKING gpui
boot (app.rs + read.rs + bin/marley_spike.rs) + a headed precedent (tests/headed_one_block.rs); the
harness has HeadedSession (launch.rs) + the #12 text-tolerant masked baseline (headed_text.rs). The mount
crates (marley_terminal/marley_editor/marley_ui_components) are all present + committed on this branch.

## Carry to Design (Phase 2)
1. **READ** crates/marley_spike/src/{app.rs,lib.rs,read.rs,shutdown.rs} + bin/marley_spike.rs (the gpui
   Application::new/run, the window-open + title/titlebar options, the Render impl, the pump) +
   crates/marley_visual_harness (HeadedSession::launch/snapshot/capture + assert_matches_baseline_masked +
   the #12 headed_text.rs precedent) + the 3 mount crates' lib.rs (TerminalSession::spawn/write_command/
   pump/blocks; Buffer::new/edit/text + movement + EditOrigin; Appearance/ThemeColors/default_for).
   DELEGATE this read.
2. **Module manifest** (crates/marley_app/src/):
   - lib.rs — #![deny(missing_docs)] + mod decls + re-exports (run/RootView/Theme/ThemeRegistry).
   - themes.rs (PURE) — Theme{name,appearance,colors} + ThemeRegistry{builtin/by_name/default_for/
     dark_default} wrapping marley_ui_components::{Appearance,ThemeColors}. cov 100/MSI 100 (AC5).
   - input.rs (PURE) — `enum Key{Char(char),Backspace,Enter,...}` (gpui-FREE), `enum KeyOutcome{Edited,
     Submit(String),Ignored}`, `apply_key(&mut Buffer,Key)->KeyOutcome` (insert at caret EditOrigin::
     Human / backspace / submit-on-Enter), `submit_line(&Buffer)->Option<String>` (trim; None if empty).
     cov 100/MSI 100 (AC3/AC4). NOTE: apply_key needs the caret — decide how (track a caret offset in the
     RootView + pass it, OR use the Buffer's selection; the editor's set_selection clamps).
   - terminal_view.rs (PURE) — `struct BlockRow{command:String,output:Vec<String>}` + `rows_from_blocks(
     &BlockList)->Vec<BlockRow>` (each Block → command + output_text().lines()). cov 100/MSI 100 (AC2).
   - app.rs (SHIM, ACCEPTED-UNTESTABLE: #[cfg_attr(test,mutants::skip)] per fn) — `RootView` (gpui Render
     holding TerminalSession + Buffer + caret + active Theme), new/set_theme/active_theme/render/the
     on_key_down handler (gpui KeyDownEvent→Key→apply_key→on Submit write_command+clear+pump), the
     pump-on-timer, `pub fn run()->ExitCode`.
   - bin/marley.rs (SHIM) — `fn main()->ExitCode{ marley_app::run() }`.
3. **THE PUMP-ON-TIMER** (the NEW surface — prototype FIRST): the spike pumped to completion BEFORE
   run(); M1.A needs a LIVE pump on the gpui event loop. Likely `cx.spawn(async … loop { background_
   executor().timer(~16ms).await; this.update(cx,|v,cx| if !v.session.pump()?.is_empty(){cx.notify()}) })`.
   CONFIRM the cx.spawn/timer signatures at gpui 0.2.2. ACCEPTED-UNTESTABLE; marley_app only decides
   "events non-empty → notify" (trivial; pump()'s logic is already tested in marley_terminal).
4. **rust_cov exclude** (explicit, documented — §0): append `|marley_app/src/app\.rs|marley_app/src/bin/`
   to scripts/gates.sh rust_cov + the prose justification (like the spike/pty_os entries).
5. **Gate-15 headed plan**: tests/headed_shell.rs (#[ignore]) modeled on the spike's headed_one_block.rs +
   #12's headed_text.rs — launch CARGO_BIN_EXE_marley, assert AX window title "Marley", ONE baseline
   `shell_dark` via assert_matches_baseline_masked + Tolerance::gate15_text() + a COMPOSITE mask
   (titlebar band + the non-deterministic terminal-output region) so only the Dark bg + input chrome is
   pixel-asserted; + the spike's blank-detector backstop. gate-15 in CI = dir exists + harness green (it
   does NOT run the #[ignore] headed test — that's the headed lane).
6. **Mutation map** — apply_key (each Key arm: Char inserts EXACT char at caret, Backspace deletes,
   Enter→Submit, others→Ignored — assert the Buffer text + the outcome), submit_line (trim + None-if-empty
   — non-trivial whitespace fixture), rows_from_blocks (command + multi-line output, a 2-block list —
   non-origin), ThemeRegistry (default_for appearance match, by_name exact, dark_default==Dark). Reuse the
   marley_editor/marley_terminal value types in fixtures.

**Phase 1 status:** PASS (autonomous-through-commit per chad's /goal). → Phase 2 Design.

## Phase 2 — Design (2026-07-01)

**SCOPE DECISION (chad, AskUserQuestion):** on FINDING 2 (below), chad chose **"Ship a shell-integration
script"** — the full Warp-style Blocks. So #16 ADDS a Marley-original zsh rc emitting the DCS hooks (via
SessionOptions.env) so real commands segment into VISIBLE Blocks — the richer, headless-PROVABLE finale
(a real command → a real Block in an integration test, not just the headed visual). Design of that piece
is in the "Shell integration" section below.

### Two findings the plan didn't anticipate (subagent, verified against source)
- **FINDING 1 — Block/BlockList NOT constructible with content outside marley_terminal** (open_running/
  current_mut/set_output are pub(crate); Block.output private; no public BlockId ctor). So a
  `rows_from_blocks(&BlockList)` PURE fn can NEVER hit gate-4 100% (empty-only BlockList → dead .map()
  closure). RESOLUTION: the AC2 pure substance is `block_row(command:&str, output_text:&str)->BlockRow`
  (constructible &str, 100/MSI); the `&BlockList` iteration lives in app.rs (the coverage-excluded shim).
- **FINDING 2 — a bare shell emits no DCS hooks → blocks() empty → hollow pane.** → chad chose shell-
  integration (above), which ALSO makes W4 headless-provable.
- **DEP CORRECTION:** must depend directly on `marley_text_offsets` (Buffer::edit takes Range<CharOffset>;
  marley_editor does NOT re-export CharOffset). Machete-clean (named in input.rs).

### Crate layout (Cargo.toml — resolves the gate-15 dir trap + `cargo run -p marley`)
dir `crates/marley_app`; `[package] name="marley"` · `[lib] name="marley_app" path="src/lib.rs"` ·
`[[bin]] name="marley" path="src/bin/marley.rs"`. Deps gpui="0.2.2" + marley_ui_components +
marley_terminal + marley_editor + **marley_text_offsets**; dev marley_visual_harness + mutants. NO
nucleo/marley_core/serial_test-in-main (the integration test adds serial_test — see shell-integration).
Workspace `members=["crates/*"]` auto-registers; no workspace edit.

### Module manifest (crates/marley_app/src/) — the pure/shim seam
- **lib.rs** — #![deny(missing_docs)] + mod decls + `pub use app::{run,RootView}; themes::{Theme,
  ThemeRegistry}; input::{apply_key,submit_line,Key,KeyOutcome}; terminal_view::{block_row,BlockRow};` +
  the shell_integration re-exports.
- **themes.rs (PURE, cov/MSI 100 — AC5)** — Theme{name,appearance,colors} wrapping marley_ui_components::
  {Appearance,ThemeColors} (NO redeclare); ThemeRegistry{themes:Vec<Theme>} + builtin()[Light+Dark via
  ThemeColors::default_for] + from_themes(pub ctor — lets tests build the fallback) + themes/by_name
  (EXACT `t.name==name`)/default_for(find appearance==a, `unwrap_or_else(||&themes[0])` fallback REACHABLE
  via from_themes(vec![light_only]).default_for(Dark))/dark_default(delegates to default_for(Dark) — no
  dead panic line).
- **input.rs (PURE, cov/MSI 100 — AC3/AC4), gpui-FREE** — `enum Key{Char(char),Backspace,Enter,Other}`,
  `enum KeyOutcome{Edited,Submit(String),Ignored}`, `apply_key(&mut Buffer, &mut CharOffset, Key)->
  KeyOutcome`, `submit_line(&Buffer)->Option<String>`. **CARET = a `CharOffset` owned by RootView, threaded
  `&mut` into apply_key** (NOT the Buffer selection — Buffer::edit doesn't advance selection, so tracking
  a caret is strictly leaner + apply_key stays pure/testable). Semantics: Char(c)→edit(at..at,&c.to_string,
  Human)+caret=from(as_usize+1)→Edited; Backspace→if as_usize>0 edit(from(end-1)..caret,"",Human)+caret=
  from(end-1)→Edited else Ignored; Enter→submit_line?Some→Submit(line):None→Ignored (does NOT mutate —
  shim clears on Submit); Other→Ignored. submit_line=text().trim(); None if empty. **CharOffset has only
  Sub<Self> not Sub<usize>** → use `CharOffset::from(caret.as_usize()±1)` (mirrors editor movement.rs); NO
  clamp inside (in-range caret precondition — a redundant clamp = equivalent mutant).
- **terminal_view.rs (PURE, cov/MSI 100 — AC2)** — `struct BlockRow{command:String,output:Vec<String>}`
  + `block_row(command:&str, output_text:&str)->BlockRow` (output = output_text.lines().map(String)).
- **shell_integration.rs (PURE + a tested *_in(dir) IO)** — SEE the "Shell integration" section (pending).
- **app.rs (SHIM — ACCEPTED-UNTESTABLE, #[cfg_attr(test,mutants::skip)] per fn)** — RootView{session:
  TerminalSession, buffer:Buffer, caret:CharOffset, theme:Theme, focus_handle} + new(window,cx)[write the
  shell rc to a tempdir, spawn zsh w/ ZDOTDIR env, empty Buffer, caret zero, Dark theme=ThemeRegistry::
  builtin().dark_default().clone(), focus, start pump-timer] + active_theme/set_theme + rows_from_blocks
  (session.blocks().iter().map(|b| block_row(&b.command,&b.output_text()))) [the un-constructible-type
  touch] + on_submit(write_command+clear+caret 0+pump+notify) + key_from_keystroke(gpui Keystroke→Key) +
  impl Render (div().track_focus.flex_col.bg(theme.colors.background).text_color(foreground) + the rows +
  the input line; STATIC caret — no blink, for headed determinism) + the on_key_down listener + run().
- **bin/marley.rs (SHIM)** — `fn main()->ExitCode{ marley_app::run() }` (mutants::skip).

### Resolved: pump-timer (gpui 0.2.2 API CONFIRMED against source)
The spike pumped to completion BEFORE run(); M1.A needs a LIVE pump. In RootView::new (shim):
`cx.spawn(async move |this:WeakEntity<Self>, cx:&mut AsyncApp| loop { cx.background_executor().timer(
Duration::from_millis(16)).await; let alive = this.update(cx,|v,cx| match v.session.pump(){ Ok(e) if
!e.is_empty()=>cx.notify(), _=>{} }).is_ok(); if !alive {break} }).detach();`. Signatures confirmed:
Context::spawn (context.rs:237, AsyncFnOnce(WeakEntity,&mut AsyncApp)), background_executor().timer
(executor.rs:357 →Task, awaitable), WeakEntity::update (entity_map.rs:430, Err⇒dropped), notify
(context.rs:229). The ONLY decision marley_app owns = "events non-empty → notify" (trivial; pump()'s logic
is tested in marley_terminal). ACCEPTED-UNTESTABLE; prototype first.

### Resolved: run() + window (promote spike app.rs:45-91)
`Application::new().run(move |cx| { let bounds=Bounds::centered(None, size(px(1024.),px(768.)), cx);
cx.open_window(WindowOptions{ window_bounds:Some(WindowBounds::Windowed(bounds)), titlebar:Some(
TitlebarOptions{ title:Some("Marley".into()), ..default }), ..default }, |window,cx| cx.new(|cx|
RootView::new(window,cx)))?; cx.activate(true); })` → ExitCode::SUCCESS (AC1/AC2). render promotes spike
app.rs:23-34 (div/flex/bg/text_color/String children).

### Resolved: headed mask (composite via RegionMask public fields — no new harness ctor)
RegionMask{pub w,h,inside:Vec<bool>} composes by OR-ing membership inline in the #[ignore] headed test:
mask everything above the input band (`y<titlebar_px(80) || y<h-input_band_px(64)`) → shell_dark asserts
ONLY the deterministic Dark bg + input chrome. + assert_matches_baseline_masked(CARGO_MANIFEST_DIR,target,
"shell_dark",Tolerance::gate15_text(),&mask) + the spike's blank-detector backstop + the hard AX
title/size assert (window w==1024, h∈768..=820).

### gates.sh edits
rust_cov: append `|marley_app/src/app\.rs|marley_app/src/bin/` + the prose justification (like the spike/
pty_os entries). gate-15: `crates/marley_app` dir makes visual_g assert the component (NOT skip); CI runs
`cargo nextest -p marley_visual_harness` (harness headless, green); the shell_dark baseline rides the
headed lane (`cargo test -p marley --test headed_shell -- --ignored`). Whole-workspace mutation WILL try
app.rs/bin → every fn there MUST carry mutants::skip; the pure files carry none → MSI 100.

### Test/mutation plan (pure surface)
AC2 block_row (multi-line "On branch main\nnothing" → 2 rows; "" → []; kills command-drop + output
mutants). AC3 apply_key Char at NON-ZERO caret (from_text("ac"),caret 1,Char('b')→"abc",caret 2 — kills
at..at→0..0, +1→+0/-1) + Backspace mid-string (caret 2→"ac",caret 1) + Backspace at 0→Ignored. AC4
submit_line("  ls -la  ")→Some("ls -la") / ""→None / "   "→None; Enter on "ls"→Submit("ls") buffer
unchanged. AC5 ThemeRegistry (Light+Dark present, default_for matches BOTH, fallback via from_themes(
[light]).default_for(Dark)→themes[0], by_name exact + case-sensitive, dark_default==Dark). AC6 (Dark-
default substance = the pure dark_default; RootView::new/active_theme are shim → headed shell_dark).
Headed: AC7 headed_shell.rs #[ignore] (AX "Marley" + shell_dark masked + blank-detector).

### Risks
gpui pump-timer API (confirmed vs source; prototype first). Headed determinism (composite mask + blank-
detector + STATIC caret — no blink). Headless TestAppContext for AC6 = NO (needs a real PTY in a unit
test — shim + headed covers it). Spawn-failure = .expect (boot precondition). FINDING 2 shell-integration
(chad chose it — the next section). Compile-only: clippy --all-targets compiles headed_shell.rs; gate-14
needs /// on every pub item.

### Shell integration (chad's choice) — designed from dcs.rs's EXACT wire format
(Designed in-context after verifying dcs.rs directly + zsh 5.9 on this machine — the backgrounded
subagent over-ran; if it lands, cross-check, else its output is redundant.)

**The DCS wire format decode_hook accepts (dcs.rs:76-150, EXACT):** framing `ESC P p <payload> ESC\`
(selector `p`=Plain — simplest for the rc); payload `name;key=value;...` (split_once(';') → name +
fields; each field `key=value` split on ';'). Hooks + keys:
- `init;id=<u64>` → InitShell (REQUIRED first — apply_hook returns MissingSession for Preexec/Precmd until
  the session is registered).
- `preexec;command=<cmd>` → opens a Running block with the command.
- `precmd;exit=<i32>;pwd=<p>;git=<b>;venv=<v>;node=<v>` → finishes the Running block w/ the exit code +
  stages the next prompt (exit absent → ExitCode(None); pwd/git/venv/node all optional).
- `bootstrapped;subshell=<0|1>`.

**The emit ORDER (matches apply.rs):** at .zshrc source (before the first prompt): `init;id=1` then
`bootstrapped;subshell=0`. Per command: zsh `preexec` hook → `preexec;command=$1` (opens the block); the
command's output flows as passthrough into that block; zsh `precmd` hook → `precmd;exit=$?;pwd=$PWD`
(finishes it). The first precmd (startup, no Running block) just stages the prompt — fine.

**marley_zsh_init() -> &'static str (PURE const):** the exact rc (verified `printf '\ePp…\e\\'` emits
`1b 50 70 … 1b 5c` on zsh 5.9):
```zsh
autoload -Uz add-zsh-hook
printf '\ePpinit;id=1\e\\'
printf '\ePpbootstrapped;subshell=0\e\\'
__marley_preexec() { printf '\ePppreexec;command=%s\e\\' "$1" }
__marley_precmd()  { local ec=$?; printf '\ePpprecmd;exit=%d;pwd=%s\e\\' "$ec" "$PWD" }
add-zsh-hook preexec __marley_preexec
add-zsh-hook precmd  __marley_precmd
```
(`local ec=$?` FIRST in precmd — before $? is clobbered. `printf` interprets `\e`/`\\` — verified.)

**ENV wiring (module shell_integration.rs, PURE + a tested *_in(dir) IO):**
- `pub fn marley_zsh_init() -> &'static str` — the const above.
- `pub fn write_shell_integration_in(dir: &Path) -> io::Result<PathBuf>` — writes `dir/.zshrc` =
  marley_zsh_init(); returns `dir` (the ZDOTDIR). Testable via a tempdir (§14).
- `pub fn session_env(zdotdir: &Path) -> Vec<(String,String)>` — `[("ZDOTDIR", zdotdir.display())]` (zsh
  sources `$ZDOTDIR/.zshrc` for an interactive shell; a PTY slave IS a tty → zsh is interactive with no
  args, matching SessionOptions[no args field]). PURE.
- The spawn glue (app.rs SHIM): a tempdir + write_shell_integration_in + SessionOptions{shell:"/bin/zsh",
  cwd:current_dir, env:session_env(zdotdir), cols:80, rows:24} → TerminalSession::spawn.

**Headless integration test (tests/integration.rs, #[serial] — THE payoff, proves W4 real):** write the
rc to a tempdir, spawn TerminalSession(/bin/zsh, ZDOTDIR=tempdir), write_command("echo hi"), pump in a
BOUNDED loop (retries + timeout) until a Finished Block whose command=="echo hi" + output_text contains
"hi". This is the REAL shell-integration proof (vs marley_terminal's printf stub). Deps add serial_test +
tempfile (dev). Flakiness mitigations: bounded-pump-with-retries; `echo` is a zsh BUILTIN so no PATH
needed (session_env=[ZDOTDIR] suffices); the ZDOTDIR isolates a MINIMAL .zshrc (no user config). RISK: if
alacritty REPLACES (not merges) the child env, PATH is absent — but `echo hi` (builtin) still works; note
+ verify. RISK: zsh must be interactive in the PTY (verify — if hooks don't fire, the block never opens).

**Pure test/mutation plan (shell_integration.rs, cov/MSI 100):** marley_zsh_init() — assert it CONTAINS
the exact frame substrings (`\x1bPpinit;id=1\x1b\\`, `preexec;command=%s`, `precmd;exit=%d;pwd=%s`, both
`add-zsh-hook` lines) so a blank/mutated const is killed. write_shell_integration_in — tempdir → `.zshrc`
exists + reads back == marley_zsh_init() + returns the dir. session_env — the vec contains ("ZDOTDIR", the
dir) exactly (kills key/value swaps + empties).

**Command-escaping limitation (M1.A, documented):** Plain payload → a command containing `;` or `=`
mis-parses (`echo a;b` → field("command")="echo a"). Acceptable M1.A (the AnsiCQuoted `q` selector is the
M1.B fix — already in decode_hook). Note in the rc doc + editor.md-style arch note.

**Clean-room:** the rc is Marley-ORIGINAL — it emits Marley's OWN DCS wire format (the EMIT counterpart to
marley_terminal::decode_hook), no fork shell-integration script reproduced. No Warp identifiers.

**Phase 2 status:** PASS. → Phase 3 Implement.

## Phase 3 — Implement (2026-07-01)

Built `crates/marley_app` (package `marley`, lib `marley_app`, bin `marley`): Cargo.toml + src/{lib,
themes,input,terminal_view,shell_integration,app}.rs + src/bin/marley.rs. check + clippy(-D warnings) +
fmt + rustdoc(-D warnings) + shellcheck all clean; **no unsafe**; deny ok; Cargo.lock adds ONLY `marley`
(no new external crate — gpui already in the tree). scripts/gates.sh rust_cov exclude extended with
`marley_app/src/app\.rs|marley_app/src/bin/` + the prose justification.

**PROCESS NOTE (lesson):** THREE subagents stalled on this crate — each spent the whole run *researching*
the gpui 0.2.2 API (compiling gpui to check signatures) and never wrote a file. I stopped them and wrote
the crate MYSELF: confirmed the mount signatures (ThemeColors/SessionOptions/Buffer/CharOffset) with
targeted greps, then wrote all 8 files against the spike's PROVEN gpui calls + `cargo check` — **0 errors
first try**. The gpui surfaces the subagents feared (`cx.spawn` async-closure pump-timer, `on_key_down` +
`cx.listener`, `Keystroke.key_char`, `track_focus`, `open_window`) all compiled as written. LESSON:
`BF-claude-subagent-gpui-research-paralysis` — for a new crate over a known-API dep with a proven
in-repo exemplar (the spike), WRITE-then-cargo-check-iterate beats research-first; a subagent told to
"confirm the API" can spin indefinitely on a slow-to-compile dep. Capture at complete.

**Deviations (justified):**
1. **Wrote it in-context** (not delegated) — see the process note.
2. **ZDOTDIR persistence:** `std::env::temp_dir().join("marley-zsh-<pid>")` created + written in
   RootView::new, stored as `_zdotdir: PathBuf` on RootView (outlives the session). NO tempfile in
   production (tempfile is dev-only, for the validate tests).
3. **rustdoc:** the lib crate doc linked `[`themes`]`/etc. (PRIVATE modules) → -D warnings error; changed
   to code spans (the `[`run`]`/`[`RootView`]`/`[`Key`]` links to the pub re-exports stay).
4. **Caret marker:** the input line renders `\u{258f}` (▏) + buffer.text() (a STATIC caret, no blink —
   for headed determinism).
5. app.rs is the ONLY shim (mutants::skip on EVERY fn + rust_cov-excluded); the 4 pure modules + bin
   carry the pure surface. bin/marley.rs is `fn main()->ExitCode{ marley_app::run() }`.

**The shell-integration rc** (shell_integration.rs) emits the EXACT frames dcs.rs parses (verified: init;
id=1 / preexec;command=%s / precmd;exit=%d;pwd=%s / bootstrapped;subshell=0, all `\ePp…\e\\` Plain). The
Rust const uses `concat!` with `\\e`→`\e` / `\\\\`→`\\` so the written .zshrc has the literal
`printf '\ePp…\e\\'` zsh interprets at runtime.

**Phase 3 status:** PASS. → Phase 3.5 Inspect.

## Inspect (Phase 3.5) — clean (no code bugs); shell-rc double-confirmed

**Verdict:** NO code findings. My self-inspection + the two critics (killed after over-running the slow
`cargo mutants --list`, but both reported their partials) CONVERGED on "correct." Lenses covered:

- **Shell-rc byte-correctness (the #1 risk) — CONFIRMED TWICE (empirically):** I ran the exact rc through
  a live zsh: `init;id=1`→`1b 50 70 … 1b 5c` = `ESC P p init;id=1 ESC\`; the preexec fn→`ESC P p
  preexec;command=echo hi ESC\`; precmd fn→`ESC P p precmd;exit=0;pwd=/…/Marley ESC\`. All match dcs.rs's
  parser EXACTLY (name + field keys id/command/exit/pwd, Plain selector 'p'). Critic 2 independently:
  "sourcing the exact rc emits all four frames byte-exact, and `false` before `__marley_precmd` correctly
  yields `exit=1` (proving `local ec=$?` captures the prior command's code)." (Note: preexec/precmd fire
  only under interactive zle / a real PTY — not `zsh -i -c` — so validate's real-PTY integration test is
  the end-to-end proof; the frame FORMAT is proven here.)
- **Seam complete:** all 8 app.rs fns carry `#[cfg_attr(test, mutants::skip)]` (new/active_theme/set_theme/
  rows/on_submit/key_from_keystroke/render/run); the 4 pure files (themes/input/terminal_view/
  shell_integration) are gpui-FREE + carry NO mutants::skip → they stay in the cov/MSI denominator. The
  rust_cov regex excludes `marley_app/src/app\.rs|marley_app/src/bin/` ONLY.
- **§14:** the 4 pure files are unwrap/expect/panic-FREE (app.rs's `.expect("spawn zsh")` + `themes[0]`
  indexing are shim/precondition — OK). **Clean-room:** grep warp empty; the rc is Marley-original (emits
  Marley's own DCS format); all public names Marley-original. **machete:** clean. **deny:** licenses ok.
- **Process lesson (Phase 3):** the 3-subagent gpui research-paralysis → forge BF-…-subagent-research-
  paralysis-on-slow-compile-dep-001 + PR-claude-write-first-not-research-first-known-api-exemplar-001.

**No fixes applied** (nothing to fix). 

### CARRY TO VALIDATE — mutation kill map (the pure surface; the design notes' fixtures)
- **input.rs apply_key (the disjoint-index center):** NON-ZERO caret fixture — `Buffer::from_text("ac")`,
  caret `CharOffset::from(1)`, `Char('b')` → text `"abc"`, caret `2`, `Edited` (kills `at..at`→`0..0`,
  `+1`→`+0`/`-1`). Backspace mid-string: `"abc"` caret 2 → `"ac"` caret 1 (kills `end-1`→`end`,
  `start..*caret`→`0..`). Backspace at 0→`Ignored` (kills `end>0`→`>=`). Enter on `"ls"` caret 2 →
  `Submit("ls")` + buffer/caret UNCHANGED. Other→Ignored. submit_line: `"  ls  "`→`Some("ls")`, `""`→None,
  `"   "`→None.
- **themes.rs:** builtin has Light+Dark; default_for(Light)==Light AND default_for(Dark)==Dark; the
  `unwrap_or(&themes[0])` FALLBACK reachable+killable via `from_themes(vec![light_only]).default_for(Dark)`
  →themes[0] (the Light); by_name EXACT (`"Marley Dark"`→Some, `"Marley"`/`"marley dark"`→None);
  dark_default==Dark.
- **terminal_view.rs block_row:** `("git status","On branch main\nclean")`→`{command, output:["On branch
  main","clean"]}` (multi-line); `("ls","")`→`{output:[]}`.
- **shell_integration.rs:** assert marley_zsh_init() CONTAINS the exact substrings (`\u{1b}Ppinit;id=1\u{1b}\\`,
  `preexec;command=%s`, `precmd;exit=%d;pwd=%s`, both `add-zsh-hook` lines); write_shell_integration_in
  (tempdir → `.zshrc` == marley_zsh_init()); session_env → `[("ZDOTDIR", dir)]`.
- **Integration (real zsh, #[serial]):** write the rc to a tempdir, TerminalSession::spawn(/bin/zsh,
  ZDOTDIR=dir), write_command("echo hi"), pump until a Finished Block command=="echo hi" + output "hi"
  (uses serial_test + tempfile — machete). **Headed:** tests/headed_shell.rs #[ignore] (AX "Marley" +
  masked shell_dark baseline; uses marley_visual_harness). AC6 (RootView active_theme==Dark) is shim →
  covered by the headed test (or a gpui TestAppContext if feasible — else note).

**Phase 3.5 status:** PASS. → Phase 4 Validate.

## Phase 4 — Validate (2026-07-01)

**Tests (19)** — I WROTE THEM MYSELF (a 4th subagent stalled on this crate, still reading; same
research-paralysis → same fix). in-crate `#[cfg(test)]`: input.rs (7: char@nonzero-caret, backspace-del,
backspace@0→Ignored, enter→Submit no-mutate, enter@empty→Ignored, other→Ignored, submit_line trim/None),
themes.rs (5: builtin Light+Dark, default_for both, the from_themes(vec![light]) fallback, by_name exact
case-sensitive, dark_default), terminal_view.rs (2: multi-line split, empty), shell_integration.rs (3:
init_contains the exact frame substrings, write installs the .zshrc == marley_zsh_init(), session_env
ZDOTDIR) + `tests/integration.rs` (1 real-zsh `#[serial]`) + `tests/headed_shell.rs` (1 `#[ignore]`).

**Results:** `cargo nextest -p marley` = **18 passed** (17 pure + 1 integration) + 1 skipped (headed);
`cargo llvm-cov` = **100% lines** on input/themes/terminal_view/shell_integration (app.rs 0% — rust_cov-
EXCLUDED; shell_integration's 1 missed *region* is the untaken `?` error branch, lines 100%); `cargo
mutants` (the 4 pure files) = **30 mutants → 22 caught + 8 unviable + 0 MISSED → MSI 100%**. clippy(-D)
+ fmt clean.

**THE PAYOFF — W4 empirically PROVEN:** `tests/integration.rs` spawns a REAL `/bin/zsh` with the ZDOTDIR
shell-integration, `write_command("echo hi")`, pumps, and **a Finished Block whose output contains "hi"
appears** (passed in 0.06s). So the finale is a genuinely usable Warp-style terminal — a real command →
a visible Block (chad's shell-integration choice delivers). (Earlier I also verified the emitted frames
are byte-exact vs dcs.rs at inspect.) The headed test (AX "Marley" + blank-detector, spike-modeled) is
#[ignore]; the masked shell_dark baseline is a headed-lane follow-up (can't capture/approve a PNG without
a display).

**FULL gate (016):** First run GATE RED on **gate:8 (cargo-deny) ONLY** — everything else green (cov
100, MSI 100 [22 caught/0 missed], miri, gate-15 visual PASS, machete, SAST). The failure: a
NEWLY-PUBLISHED advisory `RUSTSEC-2026-0192` — `ttf-parser` is UNMAINTAINED (not a vuln), a transitive
**gpui** font-parsing dep (unprunable, pre-existing in the lockfile since #12; the advisory was just
published, so it would break EVERY commit now). FIX (§0-compliant, precedented): added the per-id ignore
to deny.toml's [advisories] — the file's own header sanctions exactly this (gpui-transitive unmaintained
notices, "visible + reviewed, never a silent baseline"; it already ignores 5 such from TICKET-007). NOT
my code; no fixed version. Re-ran (deny.toml is gate-defining).

**FULL gate (016b, re-run):** `GATE GREEN [diff]` (09:59:13) — **15 passed, 0 failed**. Coverage 100%
(the 4 pure files; app.rs excluded); mutation **22 caught / 0 missed → MSI 100.0%**; **gate-15 PASS**
(the crates/marley_app dir → visual_g asserts the component; the harness headless tests green); miri +
deny(now-ok) + machete + SAST green. Commit receipt written (41 b, MATCH).

**Phase 4 status:** PASS.
