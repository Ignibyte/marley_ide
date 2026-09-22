# Subsystem 08 — Terminal, Tasks & the Fusion Wedge

Part of the Marley **Zed** architecture reference (`../README.md` — GPL provenance posture applies). Zed is the
**editor** reference; this doc covers the one subsystem where Marley is *already stronger* than Zed and where the
product wedge lives: **terminal ↔ editor ↔ block fusion**. Zed has an editor, a plain terminal, and a tasks/runnables
system — but they are **loosely coupled** (a task is a whole terminal *tab* with a tab-level status icon, its output is
plain scrollback). Marley already has a Warp-parity **block-terminal**; running a task/runnable *as a first-class command
Block* is where Marley wins. This doc maps Zed's design so Marley can reimplement the good parts (the tree-sitter runnable
discovery, the template/variable resolution model, the reveal/hide policy) **onto the Block model** instead of onto tabs.

> **TL;DR — the wedge in five lines:**
> - Zed's task pipeline is excellent up to the point of spawn: `runnables.scm` (`@run` + `#set! tag`) → tree-sitter
>   `runnable_ranges` → editor **gutter play button** → resolve a `TaskTemplate` with `$ZED_*` variables → `SpawnInTerminal`.
> - At spawn it degrades: the task becomes a **terminal tab** carrying `Option<TaskState>`; status is a **tab icon**
>   (`PlayFilled`/`Check`/`XCircle`), and the result is a **plain-text summary line** appended to scrollback
>   (`⏵ Task \`x\` finished with exit code: 1`). Nothing is structured; jump-to-failure relies on the *generic* terminal
>   path-hyperlink, not on any task-scoped parse.
> - Zed has **no command blocks at all** — it doesn't even know where one command's output ends. It infers the "running
>   command" for the tab *title* by **polling the OS process table** (`pty_info.rs`, sysinfo), not from shell hooks.
> - Marley already frames every command as a **Block** (exit code, output grid, cwd, duration, rerun). So a Marley
>   runnable is a Block with an inline status pill, framed output, a per-block rerun, and a jump-to-failure scoped to
>   *that block's* output — for free, on infra that already exists.
> - **Reimplement:** keep Zed's discovery+resolution model (`TaskTemplate`/`VariableName`/tags/reveal); replace its
>   *sink* (a tab + summary text) with a **Block spawn** that carries the `SpawnInTerminal` spec as block metadata.

---

## 1. Where the subsystem lives (crate map + provenance)

| Crate / file | Role | License / provenance |
|---|---|---|
| `alacritty_terminal` (external) | PTY + VTE grid/parser. The byte engine under Zed's terminal. | `[permissive/public: Apache-2.0/MIT — Marley ALREADY uses this exact crate]` |
| `terminal` (`src/terminal.rs`, `alacritty.rs`, `pty_info.rs`, `terminal_settings.rs`, `mappings/`) | Zed's model wrapper over alacritty: `Terminal` entity, `TaskState`, hyperlink/path detection, process-table title inference. | `[Zed-derived: GPL-3.0-or-later]` (wrapper only; grid is permissive) |
| `terminal_view` (`terminal_view.rs`, `terminal_element.rs`, `terminal_panel.rs`, `terminal_path_like_target.rs`, `terminal_scrollbar.rs`, `persistence.rs`) | The gpui render + the terminal **dock panel** that owns task tabs, reuse/affinity, reveal, and the tab status icon + rerun button. | `[Zed-derived: GPL-3.0-or-later]` (on `gpui`, Apache-2.0) |
| `task` (`task.rs`, `task_template.rs`, `static_source.rs`, `vscode_format.rs`, `debug_format.rs`) | UI-agnostic task **model**: `TaskTemplate` → `ResolvedTask` → `SpawnInTerminal`; `VariableName`, `TaskContext`, `RevealStrategy`/`HideStrategy`/`SaveStrategy`, tags. | `[Zed-derived: GPL-3.0-or-later]`; the `.zed/tasks.json` + VS Code `tasks.json` **schema** is `[public: VS Code task format]` |
| `tasks_ui` (`tasks_ui.rs`, `modal.rs`) | The `Spawn`/`Rerun` actions + the fuzzy task-picker modal (on the `picker` crate). | `[Zed-derived: GPL-3.0-or-later]` |
| `project` (`task_inventory.rs`, `task_store.rs`, `terminals.rs`) | Task **discovery/inventory** (`TaskSourceKind`), context resolution, and `create_terminal_task` (the terminal that carries a `TaskState`). | `[Zed-derived: GPL-3.0-or-later]` |
| `editor` (`src/runnables.rs`) | Turns tree-sitter runnable ranges into **gutter run buttons**; caches per buffer/version; merges LSP runnables. | `[Zed-derived: GPL-3.0-or-later]` |
| `language` (`src/runnable.rs`, `task_context.rs`) | Runs the runnables query over a buffer → `RunnableRange { run_range, full_range, tags, extra_captures }`; the `RunnableResolver` seam. | `[Zed-derived: GPL-3.0-or-later]`; tree-sitter itself is `[public: MIT]` |
| `grammars/src/<lang>/runnables.scm` | Per-language tree-sitter queries: `@run` capture + `(#set! tag <name>)`. Present for rust/python/go/ts/tsx/js/bash/c/cpp/json. | `[Zed-derived query files: GPL bundle]`; the *mechanism* (tree-sitter capture queries) is `[public: MIT]` |

**Provenance boundary that matters for Marley:** the *discovery + resolution model* (§3–§5) is a set of ideas — a
JSON template with `$VAR` substitution, tags joined to tree-sitter captures, a reveal/hide policy enum — that Marley
should **re-express in its own types**, not copy. The one hard dependency Zed and Marley already **share cleanly** is
`alacritty_terminal` (permissive), so the byte-engine layer carries no copyleft risk. The GPL surface is the *glue*.

---

## 2. The terminal core — and why Zed has no blocks

Zed's `Terminal` (`terminal/src/terminal.rs`, the `terminal.rs` file is ~4900 lines) wraps `alacritty_terminal`'s
`Term` behind a `FairMutex`, pumps PTY events (`Wakeup`, `Title`, `Bell`, `Exit`, `ChildExit`) into gpui events, and
holds a `last_content` snapshot (`TerminalContent`: cells, cursor, `terminal_bounds`, `last_hovered_word`). This is the
**same substrate Marley already runs** — Marley's terminal is at least at parity here and exceeds it with Blocks.

Two architectural facts drive the entire wedge:

1. **Zed has no command-block model.** The grid is one flat scrollback. There is no `Block`, no per-command output
   framing, no per-command exit code. Alacritty gives cells; Zed renders cells. (Contrast: Marley's
   `terminal_blocks` frames each command with header/output/exit — see `../../marley_architecture/terminal_blocks.md`.)
2. **Zed infers the "current command" by polling the OS, not by shell hooks.** `pty_info.rs` (`PtyProcessInfo`, on
   `sysinfo`) periodically resolves the PTY's **foreground process group** to a `ProcessInfo { name, cwd, argv }` and
   uses it *only* for the tab title (`Terminal::title` → `"<cwd-basename> — <proc name+args>"`). There is **no** DCS/OSC
   shell-integration handshake like Warp's `Precmd`/`Preexec` (see warp `../../warp_architecture/subsystems/03-terminal-session-core.md` §6).
   So Zed cannot attach an exit code or cwd to a *command*; it only knows the *process currently in the foreground*.

The consequence: Zed's only notion of "a command finished with a status" exists **exclusively for tasks**, via
`TaskState` (§6) — and even that is a tab-level flag plus a text line, not a structured record. Marley's Block model is
strictly more expressive; the wedge is to route Zed's (very good) task *discovery* into it.

`terminal_settings.rs` exposes the fusion-relevant knobs Marley should mirror: `path_hyperlink_regexes` +
`path_hyperlink_timeout_ms` (custom file-path patterns for the terminal→editor jump, §7), `max_scroll_history_lines`,
`detect_venv`, `cursor_shape`, `alternate_scroll`.

---

## 3. The task model (`task` crate) — template → resolved → spawn spec

The heart Marley should reimplement. Three types form a resolution pipeline, all UI-agnostic:

- **`TaskTemplate`** (`task_template.rs:24`) — the *authored* task (from JSON or a language provider):
  `label`, `command: String`, `args: Vec<String>`, `env`, `cwd: Option<String>`, plus **policy fields**
  `use_new_terminal`, `allow_concurrent_runs`, `reveal: RevealStrategy`, `reveal_target: RevealTarget`,
  `hide: HideStrategy`, `save: SaveStrategy`, `show_summary`, `show_command`, `shell: Shell`, and crucially
  `tags: Vec<String>` (the join key to tree-sitter runnables, §4) and `hooks: HashSet<TaskHook>`.
- **`ResolvedTask`** (`task.rs:114`) — a template *bound to a context*: keeps `original_task` (for rerun/edit),
  `resolved_label`, `substituted_variables`, and `resolved: SpawnInTerminal`. Produced by
  `TaskTemplate::resolve_task(id_base, &TaskContext)` (`task_template.rs:162`), which substitutes `$ZED_*` variables in
  every field and computes a `TaskId` (`hash(id_base + label + substituted vars)` — this id is what drives **tab affinity
  and rerun**).
- **`SpawnInTerminal`** (`task.rs:42`) — the fully-resolved spawn spec handed to the terminal layer:
  `id`, `label`/`full_label`/`command_label`, `command: Option<String>`, `args`, `cwd`, `env`, and the same policy
  flags flattened out (`use_new_terminal`, `allow_concurrent_runs`, `reveal`, `reveal_target`, `hide`,
  `show_summary`, `show_command`, `show_rerun`, `save`).

**`VariableName`** (`task.rs:154`) is the substitution vocabulary — Marley wants this list nearly verbatim (renamed to a
`MARLEY_` prefix): `File`, `RelativeFile`, `Filename`, `Dirname`, `Stem`, `WorktreeRoot`, `Symbol`, `Row`, `Column`,
`SelectedText`, `Language`, **`RunnableSymbol`** (the text of the `@run` capture — this is how `cargo test $ZED_SYMBOL`
targets the exact test under the cursor), git vars (`GitSha`, `GitBranch`/`GitRef`, `GitRepositoryName`), and
`Custom(name)` for tree-sitter `extra_captures`. **`TaskContext`** (`task.rs:341`) = `{ cwd, task_variables:
TaskVariables, project_env }` — the resolved values for one invocation.

**Policy enums** (the reveal/hide model — small, worth copying exactly):
- `RevealStrategy` (`task_template.rs:103`): `Always` (show pane + focus tab), `NoFocus` (show, don't focus), `Never`.
- `RevealTarget` (`zed_actions`): `Dock` (terminal dock) vs `Center` (main editor pane group).
- `HideStrategy` (`task_template.rs:116`): `Never`, `Always`, `OnSuccess` — what to do with the tab *after* the task
  finishes.
- `SaveStrategy`: `All` / `Current` / `None` — which dirty buffers to save *before* running.

> **Marley note:** `RevealTarget::Center` vs `Dock` and `HideStrategy` are *tab-oriented* — they answer "where does the
> tab go and does it auto-close." In Marley's block model these become **block placement + block lifecycle** questions
> ("inline block in the active session vs a dedicated session; collapse-on-success vs keep-open"). The *enums* survive;
> their *meaning* moves from tabs to blocks.

---

## 4. Runnables — tree-sitter `@run` → editor gutter button (the good part)

This is the discovery mechanism Marley most wants. It has three layers:

**(a) The query** `grammars/src/<lang>/runnables.scm` `[Zed-derived files; tree-sitter mechanism public]`. A runnable is
a tree-sitter pattern that captures a `@run` node and sets a **tag**. Rust example (paraphrased): a `function_item`
whose parent has a `#[test]` attribute captures `name: (_) @run` and `(#set! tag rust-test)`; `fn main` →
`(#set! tag rust-main)`; doc-tests and `#[cfg(test)] mod` have their own patterns. Python tags
`pytest`/`unittest`/`__main__`. The `@run` capture's *text* becomes `$ZED_RUNNABLE_SYMBOL`; other `@_name` captures
become `$ZED_CUSTOM_<name>` extras.

**(b) The extraction** `language/src/runnable.rs`. `runnable_ranges(buffer, offset_range)` runs the grammar's runnable
query and yields `RunnableRange { run_range, full_range, runnable: Runnable { tags, language, buffer }, extra_captures }`.
There's a `RunnableResolver` seam for *grouped* runnables (one query match → many run items, e.g. every call in a body),
but the common path is one `@run` per match. Tags come from `#set! tag` via `runnable_tags_from_pattern`.

**(c) The gutter binding** `editor/src/runnables.rs`. `Editor::refresh_runnables` (debounced ~`UPDATE_DEBOUNCE`, cached
per `(BufferId, buffer version)`; skipped for collab guests and unsaved buffers) turns runnable ranges into
`RunnableTasks { templates: Vec<(TaskSourceKind, TaskTemplate)>, offset, column, extra_variables, context_range }` per
row by **matching each `RunnableTag` against `TaskTemplate.tags`** in the inventory (`templates_with_tags`), and also
folds in **LSP-provided runnables** (rust-analyzer's `Runnables` request → `Cargo`/`Shell` args) with a `prefer_lsp`
setting. `render_run_indicator` (`runnables.rs:518`) draws the gutter **`IconName::PlayOutlined`** button; clicking it
opens a code-actions menu (`CodeActionSource::RunMenu`) listing the row's tasks. `spawn_nearest_task` (used by a
keybinding) walks the syntax tree from the cursor to the nearest enclosing tagged node (`find_enclosing_node_task`, else
`find_closest_task` by row distance), resolves it, and calls `workspace.schedule_resolved_task`.

```
runnables.scm  @run + (#set! tag rust-test)
      │  tree-sitter query
      ▼
language::runnable_ranges ─▶ RunnableRange { run_range, full_range, tags, extra_captures }
      │
      ▼  (editor, debounced + cached per buffer version; + LSP runnables)
RunnableTasks per row  ── tag ⋈ TaskTemplate.tags ──▶ (TaskSourceKind, TaskTemplate)
      │  render_run_indicator = gutter ▷ button ;  spawn_nearest_task = cursor→nearest
      ▼
resolve_task(context)  ─▶ ResolvedTask { resolved: SpawnInTerminal }
```

> **Marley note:** layers (a) and (b) are *language intelligence* that belong in Marley's editor/language layer and are
> **provenance-clean to re-derive** (tree-sitter is MIT; the query *concept* is public — Marley writes its own `.scm`).
> The `tag ⋈ template.tags` join is the elegant bit: it decouples "where in the code is runnable" from "what command
> runs it," so a user's `tasks.json` can rebind `rust-test` to `nextest` without touching the grammar. Marley should
> keep this indirection.

---

## 5. Discovery, inventory & the task modal

**`TaskSourceKind`** (`project/task_inventory.rs:147`) enumerates where templates come from, in priority order
(`task_source_kind_preference`, lower = stronger): `Lsp{server}` (0) → `Language{name}` from extensions (1) →
`UserInput` oneshot (2) → `Worktree{.zed/tasks.json}` (3) → `AbsPath{~/.config/zed/tasks.json}` (4). The `Inventory`
holds templates + a **history** of used tasks; `list_tasks` returns available templates for a buffer/language/worktree,
and `used_and_current_resolved_tasks` merges recent history with currently-available tasks (this is what the modal
shows). `task_scheduled` records a run into history for rerun.

**The modal** (`tasks_ui/src/modal.rs`, on the `picker` crate) is a fuzzy picker over
`used_and_current_resolved_tasks` — history first (with a separator), then available, filtered by fuzzy match. Actions
(`tasks_ui.rs`): `Spawn::ViaModal { reveal_target }` opens the picker; `Spawn::ByName { task_name }` and
`Spawn::ByTag { task_tag }` run without a picker (e.g. keybound "run all tests"); `Rerun { task_id, reevaluate_context,
… }` re-runs the last/identified task, either reusing its stored `SpawnInTerminal` or re-resolving context if
`reevaluate_context`. `TaskOverrides` lets a caller force a `reveal_target`.

> **Marley note:** the modal is just the command palette pattern Marley already ships (M1.B). A Marley "Tasks" source
> plugs its `used_and_current_resolved_tasks` equivalent into the existing fuzzy launcher — no new UI primitive needed.

---

## 6. The spawn pipeline & the loosely-coupled sink (where Zed stops short)

Once a `ResolvedTask`/`SpawnInTerminal` exists, the spawn path is:

1. **`Workspace::schedule_resolved_task`** (`workspace/tasks.rs:54`) — records history, runs `save_for_task` per
   `SaveStrategy`, then calls `terminal_provider.spawn(spawn_in_terminal)`. The result is a
   `Task<Option<Result<ExitStatus>>>` — the *only* structured signal that escapes, and it's used just for a toast on
   failure.
2. **`TerminalPanel::spawn_task`** (`terminal_view/terminal_panel.rs:534`) — the **tab-affinity + reuse** logic:
   - `prepare_task_for_spawn` wraps the command for the shell.
   - If `allow_concurrent_runs && use_new_terminal` → always a fresh tab (`spawn_in_new_terminal`).
   - Else it looks up existing tabs for this task by **`full_label`** (`terminals_for_task`); with `allow_concurrent_runs`
     it `replace_terminal`s the existing tab's contents; otherwise it queues a `deferred_task` that **waits for the prior
     run to finish** (`wait_for_terminals_tasks`) before reusing/replacing — this is how "don't run two of the same task
     at once" is enforced, keyed by `TaskId`.
   - `reveal_target` routes to `add_center_terminal` (main pane) or `add_terminal_task` (dock); `reveal` decides focus.
3. **`Project::create_terminal_task`** (`project/terminals.rs:64`) — builds the actual `Terminal`: resolves directory
   env, optional Python venv **activation script**, wraps `command`+`args` in a `Shell::WithArguments`, and constructs a
   `TerminalBuilder` carrying `Some(TaskState { spawned_task: SpawnInTerminal, status: Running, completion_rx })`.
4. **`Terminal::register_task_finished`** (`terminal/terminal.rs:2835`) — on `Exit`/`ChildExit`: pushes the exit status
   through `completion_tx`, flips `TaskStatus::Running → Completed { success: code==0 }` (or `Unknown` if the terminal
   died without a code), then — and this is the tell — **appends a plain-text summary line to the scrollback** via
   `append_text_to_term`: `⏵ Task \`label\` finished with exit code: N` (+ optional `⏵ Command: …`), gated by
   `show_summary`/`show_command`. Finally applies `HideStrategy` (emit `CloseTerminal` on `Always`, or on success for
   `OnSuccess`).

**`TaskState` / `TaskStatus`** (`terminal.rs:1467/1475`) is the entire structured footprint of a task: `Unknown`,
`Running`, `Completed { success }`, plus a `completion_rx` one-shot and the `SpawnInTerminal`. It is surfaced in exactly
two places in the UI (`terminal_view.rs`):
- the **tab icon** (`terminal_view.rs:1467`): `Running → PlayFilled (Disabled)`, `Unknown → Warning`,
  `Completed{true} → Check (Success)`, `Completed{false} → XCircle (Error)`;
- the **rerun button** (`rerun_button`, `terminal_view.rs:1082`): shown on hover if `show_rerun`, dispatches
  `zed_actions::Rerun { task_id }`.

```
schedule_resolved_task ──(save)──▶ terminal_provider.spawn(SpawnInTerminal)
        │                                   │
        │                                   ▼
        │              TerminalPanel::spawn_task  ── affinity by full_label / TaskId
        │                 ├─ new tab (use_new_terminal | concurrent)
        │                 ├─ replace_terminal (reuse)          reveal → Dock|Center, focus
        │                 └─ deferred: wait_for prior run to finish
        │                                   │
        ▼                                   ▼
   Task<ExitStatus>  ◀── completion_rx ── Project::create_terminal_task ─▶ Terminal + TaskState{Running}
   (only used for a                          │
    failure toast)                           ▼  child exits
                         register_task_finished:  status→Completed{success},
                             APPEND PLAIN-TEXT "⏵ Task `x` finished…" to scrollback,
                             HideStrategy → maybe CloseTerminal
                                             │
                       UI footprint = tab ICON (▷/✓/✗) + hover RERUN button
```

**The gap, precisely:** the task's structured result lives for a moment in `TaskStatus` and the `completion_rx`, then is
**flattened into scrollback text and a tab glyph**. The output is not framed, not addressable, not re-runnable per
command, and jump-to-failure is not task-aware (§7 is generic). Zed reaches the *edge* of fusion — it knows a command,
its status, its cwd, its rerun spec — and then throws the structure away because it has no block to hang it on.

---

## 7. The terminal ↔ editor link (the *other* fusion direction — Zed AND Marley have this)

Zed does fuse the terminal *into the editor* for path navigation, and this is the direction Marley **already shipped**
(clickable `file:line:col` → editor, tickets #196/#214). Mechanism:

- `terminal/src/terminal.rs` detects a hovered/clicked **`PathLikeTarget { maybe_path, terminal_dir }`** where
  `maybe_path` may carry `file.rs:1:23`. Detection uses (a) OSC-8 hyperlinks the program emitted, and (b)
  `RegexSearches` built from the user's `path_hyperlink_regexes` plus built-in heuristics, throttled by
  `FIND_HYPERLINK_THROTTLE_PX` and `path_hyperlink_timeout_ms`.
- `terminal_view/src/terminal_path_like_target.rs` resolves it: `possible_open_target(workspace, maybe_path,
  terminal_dir)` (in `workspace::path_link`) does the FS/worktree checks on a background thread, sets a hover tooltip,
  and on click opens the file in an `Editor` at the parsed row/column via `workspace.open_path` (`OpenTarget::Path` /
  `OpenTarget::Worktree`).

> **Marley note:** Marley's block-terminal makes this *better* than Zed even in the direction they share: because Marley
> knows a path came from a *specific command Block*, a failed build Block can resolve its links against **that command's
> cwd** (Zed uses the terminal's current foreground cwd, which may have `cd`'d away by the time you click). Marley already
> has the link-open half; the fusion upgrade is **scoping resolution to the Block's captured cwd**.

---

## 8. THE WEDGE — runnable-as-Block vs task-as-tab

Side-by-side, holding the discovery/resolution model constant (Marley keeps §3–§5) and changing only the **sink**:

| Concern | Zed today (loosely coupled) | Marley (block-fused) — the win |
|---|---|---|
| Where a run's output lands | A whole terminal **tab** (dock or center pane) | An **inline Block** in the active session (or a chosen session) — no tab sprawl |
| Status surface | Tab **icon** (`▷/✓/✗/⚠`), one per tab | A **status pill in the Block header** — visible in the command stream next to the command it ran |
| Result record | `TaskStatus` flattened to a **plain-text summary line** in scrollback | Structured Block fields Marley **already stores**: `exit_code`, `duration`, `cwd`, framed `output_grid` |
| Output framing | None — interleaved with prior scrollback; `⏵ Task…` text is the only boundary | Native Block framing — the run's output is a first-class, collapsible, addressable region |
| Rerun | `show_rerun` hover button on the **tab**, re-dispatches `Rerun{task_id}` | A **per-Block rerun** affordance; the `SpawnInTerminal` rides as **Block metadata**, so any historical run re-executes in place |
| Jump-to-failure | Generic terminal path-hyperlink; **not task-aware**; resolves against the *current* foreground cwd | Parse **this Block's** output for the first `file:line:col`, resolve against **the Block's captured cwd**, offer a scoped "jump to first failure" |
| Concurrency / affinity | `deferred_tasks` + `terminals_for_task` keyed by `full_label`/`TaskId` at the **tab** level | Same policy, keyed at the **Block/session** level — a re-run supersedes or stacks Blocks; no orphan tabs |
| Agent observability | A task is opaque scrollback text; the brain can't cleanly read "did the test pass" | A Block *is* the structured unit the Marley brain already observes (`session.read` delta, exit code) — a runnable's pass/fail is machine-readable **without instrumentation** |
| `reveal`/`hide` policy | Tab placement + auto-close-tab | Re-mapped to **Block placement + Block lifecycle** (collapse-on-success = `HideStrategy::OnSuccess`) |

**Why this is defensible, not cosmetic:** Zed cannot retrofit blocks cheaply — it has *no* command-boundary signal at
all (no shell hooks, only OS process polling, §2), so even its own tasks resort to appending marker text. Marley already
paid for the block infrastructure (shell-integration boundaries, exit codes, per-block cwd, rerun). Routing a runnable
into a Block is **reuse of shipped infra**; for Zed it would be a new subsystem.

### Concrete reimplementation: runnables → block-terminal

1. **Discovery/resolution (port the model, re-derive the code):** Marley re-writes `runnables.scm` per language
   (MIT-clean), a `runnable_ranges`-equivalent over its own buffer/tree-sitter layer, a `TaskTemplate`/`ResolvedTask`/
   `SpawnInTerminal`-equivalent with a `MARLEY_*` `VariableName` set, and the `tag ⋈ template.tags` join. Discovery
   sources mirror `TaskSourceKind` (worktree JSON, user oneshot, language, LSP). **Provenance: `[Marley-original]` types
   modeled on `[Zed-derived]` design; the `.scm` files are `[public: MIT tree-sitter]`.**
2. **Editor gutter:** a gutter run button (Marley's editor is post-M13/M14) that, on click, resolves the nearest tagged
   node's template with a `TaskContext` (cursor `Row`/`Column`/`Symbol`/`RunnableSymbol`, selection, git, project env).
3. **Swap the sink — this is the wedge:** instead of `terminal_provider.spawn → new tab + TaskState`, Marley spawns the
   resolved command **as a Block** in a target session, attaching the `SpawnInTerminal` (command/args/cwd/env/id) and the
   `reveal`/`hide` policy as **Block metadata**. The Block's normal lifecycle already yields the exit code, duration, and
   framed output; the header renders a **status pill** driven by the same `Running/Completed{success}/Unknown` states
   Zed tracks — but *inline*, not on a tab.
4. **Rerun = re-spawn the Block's stored spec.** Because the `SpawnInTerminal` is Block metadata, "rerun" is generic over
   *any* historical run Block, and tab-affinity/`deferred_tasks` policy becomes a session/Block-stacking rule.
5. **Jump-to-failure, block-scoped:** on a `Completed{success:false}` Block, scan *that Block's* `output_grid` for the
   first `file:line:col` (reusing Marley's shipped path-link parser, #196/#214) and resolve it against the **Block's
   captured cwd**, exposing a "jump to failure" action in the Block header. Zed cannot do this cleanly — it has no block
   to scope the scan or the cwd to.
6. **Brain integration (Marley-only upside):** a runnable Block is exactly the structured unit the proprietary brain
   layer observes. "Run the tests, jump me to the first failure, and tell the agent whether it passed" is one flow over
   one Block — no scrollback scraping. **Keep this in the `[Marley-original]` brain layer, clean of GPL glue.**

---

## 9. Marley relevance — consolidated

- **Keep (re-derive) from Zed:** the *task model* (`TaskTemplate`/`ResolvedTask`/`SpawnInTerminal`, `VariableName`,
  `TaskContext`), the *reveal/hide/save policy enums*, the *runnables tag-join* (tree-sitter `@run`+tag ⋈
  `template.tags`), the *discovery sources* (`TaskSourceKind`), and the *modal-over-history* pattern. These are the parts
  Zed does well and Marley currently lacks.
- **Replace with Marley's strength:** the *sink*. Zed's task = tab + status glyph + summary text. Marley's task/runnable
  = **first-class Block** with an inline status pill, framed output, per-block rerun, block-scoped jump-to-failure, and
  brain-observability. This is the concrete wedge — documented feature-by-feature in §8's table.
- **Already shipped (leverage):** the block-terminal (`terminal_blocks`), per-command exit/cwd/duration, and the
  clickable `file:line:col → editor` link (#196/#214). The runnable-Block work is **integration of existing infra**, not
  new subsystems.
- **Provenance discipline:** `alacritty_terminal` is shared and permissive (no risk). The runnable *queries* are MIT
  tree-sitter (re-write cleanly). The task *model* and *glue* are GPL `[Zed-derived]` — reimplement as `[Marley-original]`
  types from this architecture description, never copy the Zed source. The **brain-side** runnable observability must stay
  in the proprietary layer, clean of any GPL-derived glue (per `../README.md`).

---

## Key files (paths relative to the Zed clone)

- `crates/terminal/src/terminal.rs` — `Terminal`, `TaskState`/`TaskStatus` (`:1467`), `register_task_finished` (`:2835`),
  `task_summary` (`:2936`), `title` process-inference (`:2731`), `PathLikeTarget` (`:665`).
- `crates/terminal/src/pty_info.rs` — `PtyProcessInfo`/`ProcessInfo` (OS process-table polling; no shell hooks).
- `crates/terminal/src/terminal_settings.rs` — `path_hyperlink_regexes`, `detect_venv`, scroll/cursor knobs.
- `crates/terminal_view/src/terminal_panel.rs` — `spawn_task` (`:534`), tab affinity/reuse/`deferred_tasks`,
  `replace_terminal`, reveal routing.
- `crates/terminal_view/src/terminal_view.rs` — tab status icon (`:1467`), `rerun_button` (`:1082`),
  `rerun_task`/`RerunTask` (`:653`).
- `crates/terminal_view/src/terminal_path_like_target.rs` — terminal→editor open (`possible_open_target`, hover+click).
- `crates/task/src/task.rs` — `SpawnInTerminal` (`:42`), `ResolvedTask` (`:114`), `VariableName` (`:154`),
  `TaskContext` (`:341`), `RunnableTag`.
- `crates/task/src/task_template.rs` — `TaskTemplate` (`:24`), `resolve_task` (`:162`), `RevealStrategy`/`HideStrategy`/
  `SaveStrategy`, `tags`.
- `crates/tasks_ui/src/tasks_ui.rs` / `modal.rs` — `Spawn` (`ByName`/`ByTag`/`ViaModal`), `Rerun`, the picker modal.
- `crates/project/src/task_inventory.rs` — `TaskSourceKind` (`:147`), `Inventory`, `list_tasks`,
  `used_and_current_resolved_tasks`, priority ordering.
- `crates/project/src/terminals.rs` — `create_terminal_task` (`:64`) building a `Terminal` + `TaskState`.
- `crates/editor/src/runnables.rs` — `refresh_runnables` (`:107`), `render_run_indicator` (`:518`),
  `spawn_nearest_task`, tag→template matching, LSP-runnable merge.
- `crates/language/src/runnable.rs` — `runnable_ranges`, `RunnableRange`, `RunnableResolver` (grouped runnables).
- `crates/grammars/src/{rust,python,go,typescript,tsx,javascript,bash,c,cpp,json}/runnables.scm` — the `@run`+tag queries.
