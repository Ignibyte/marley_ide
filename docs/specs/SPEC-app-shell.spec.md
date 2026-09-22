---
spec_id: app-shell
component: marley_app
bucket: REIMPLEMENT
milestone: M1
status: draft
title: App entry, RootView, pane-group layout, command palette, themes — workspace shell
goal: Boot a single offline gpui window whose RootView lays out a left dock, a center splittable pane group, and a right dock, with a fuzzy command palette and a selectable light/dark theme defaulting to Dark.
reuses: [gpui, nucleo]
spec_source: "behavior-only — observable I/O of the app shell: launch to a window hosting a RootView with a pane-group layout (splits/tabs), a command palette, and theme application — Warp-like layout, no auth/cloud/onboarding. No fork module/type/static names, no fork file paths. Seams per standards/seam-contracts.md."
clean_room: "behavior-derived from a fork-reference doc; IP-counsel sign-off pending"
browser_testable: no
visual_acceptance: One window titled "Marley" with the native macOS titlebar; a three-region body (left dock | center pane group | right dock); the center region splits into a 2x1 grid when split; a centered command-palette overlay listing commands; and all chrome painted in the active (Dark by default) theme — the layout reads as a Warp-style workspace shell.
mutation_testing: Enabled
references:
  - ./standards/quality-bar.spec.md
  - ./standards/seam-contracts.md
  - ../pipeline/visual-testing.spec.md
---

## Purpose
`marley_app` is the application shell: the top-of-graph crate that owns the gpui boot sequence, the per-window `RootView`, the three-region workspace layout (left dock, center pane group, right dock), the command palette, the keybinding-to-action registry, and the theme system. It is the cockpit chrome into which Marley's project/agent/Forge panels (M2) are later slotted. It boots fully offline — no auth, cloud, onboarding, or network — and provides the pure, headless-testable layout/palette/theme model behind the GPU-bound render surface. It reuses the permissive `gpui` crate (Apache-2.0/MIT, upstream Zed) for windowing and the element tree.

Two boundary postures pinned by `seam-contracts.md` shape this spec:
- **Theming vocabulary is not redeclared here.** The `Appearance { Light, Dark }` discriminant and the `ThemeColors` color bundle are owned by `marley_ui_components` (seam-contracts §6, the lowest crate that depends on `gpui`). `marley_app` *wraps* them in `Theme`/`ThemeRegistry`; it declares no local `Appearance` struct and no local `ThemeColors` shape.
- **The M1 command palette is an intentionally-temporary local static-list wrapper.** It ships a local `filter_commands` over a static in-memory command list, ranked by the same `nucleo` subsequence primitive that `marley_search_core::fuzzy_rank` uses (seam-contracts §7). This is a declared, temporary divergence: in **M2** the palette migrates onto `marley_search_core::SearchMixer` (registering commands as a `SyncDataSource`, unifying on `QueryResult`/`f32`). The local `ScoredCommand { score: u32 }` is therefore local-only and must not be read as a second permanent ranking engine. This note mirrors `SPEC-command-palette`'s isolation note.

## Public surface (the contract)
```rust
// entry.rs — boot
pub fn run() -> std::process::ExitCode;   // boots one gpui Application + one window, returns the process exit code

// root_view.rs — the per-window container (a gpui Render view)
pub struct RootView { /* workspace state, incl. the active Theme */ }
impl RootView {
    pub fn new(window: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> Self;
    pub fn dock(&self, side: DockSide) -> DockState;        // Open | Closed
    pub fn toggle_dock(&mut self, side: DockSide);          // flips open/closed + notifies
    pub fn pane_group(&self) -> &PaneGroup;                 // the center layout tree
    pub fn set_theme(&mut self, theme: &Theme);             // selects the active theme; marks the view dirty (R23)
    pub fn active_theme(&self) -> &Theme;                   // the in-effect theme; the Dark default until set_theme (R24)
}
pub enum DockSide { Left, Right }
pub enum DockState { Open, Closed }

// pane_group.rs — the pure, headless layout tree (the testable core)
pub struct PaneId(u64);
pub enum PaneAxis { Horizontal, Vertical }
pub enum SplitDirection { Before, After }
pub enum Direction { Up, Down, Left, Right }
pub enum PaneError { PaneNotFound, LastPane }
pub enum PaneGroup {
    Leaf(PaneId),
    Split { axis: PaneAxis, children: Vec<PaneGroup>, ratios: Vec<f32> },
}
impl PaneGroup {
    pub fn single(pane: PaneId) -> Self;
    pub fn split(&mut self, target: PaneId, new: PaneId, axis: PaneAxis, dir: SplitDirection) -> Result<(), PaneError>;
    pub fn close(&mut self, pane: PaneId) -> Result<(), PaneError>;
    pub fn panes(&self) -> Vec<PaneId>;                     // depth-first, stable order
    pub fn neighbor(&self, from: PaneId, dir: Direction) -> Option<PaneId>;  // adjacency rule defined at R12
}

// command_palette.rs
// NOTE (seam-contracts §7): this is the INTENTIONALLY-TEMPORARY M1 static-list palette.
// `filter_commands` ranks a static `&[Command]` via the same `nucleo` subsequence primitive
// `marley_search_core::fuzzy_rank` uses; `ScoredCommand { score: u32 }` is local-only.
// M2 replaces this surface by registering commands as a `SyncDataSource` on
// `marley_search_core::SearchMixer`, unifying on `QueryResult`/`f32`. It is NOT a second
// permanent ranking engine.
pub struct Command { pub id: CommandId, pub title: String, pub keywords: Vec<String>, pub binding: Option<KeyBinding> }
pub struct CommandId(u32);
pub struct ScoredCommand<'a> { pub command: &'a Command, pub score: u32 }   // score = nucleo subsequence score (M1-local; see note)
pub fn filter_commands<'a>(commands: &'a [Command], query: &str) -> Vec<ScoredCommand<'a>>;

// keymap.rs
pub struct KeyBinding { /* mods + key */ }
pub enum KeyContext { Terminal, Editor }                    // M16 #265 — the focused-surface identity
pub struct Keymap { /* (binding, action name, Option<KeyContext>) — None = global */ }
impl Keymap {
    pub fn default_bindings() -> Self;                      // Marley's built-in map
    // #265: resolves against the active tab's context stack — a scoped binding is eligible
    // only when its context is in the stack (ranked by deepest position); a global binding
    // ranks below any scoped match. Ties are impossible (chords_unique_scoped).
    pub fn action_for(&self, binding: &KeyBinding, stack: &[KeyContext]) -> Option<&str>;
}

// themes.rs
// `Appearance` and `ThemeColors` are OWNED by `marley_ui_components` (seam-contracts §6) —
// re-used here, NOT redeclared. `marley_app` declares no local `Appearance` and no local
// `ThemeColors` shape.
use marley_ui_components::{Appearance, ThemeColors};
// Appearance := { Light, Dark } (the discriminant only)
// ThemeColors := { background, foreground, accent, on_accent, surface, border, danger, /* + metrics */ }

pub struct Theme { pub name: String, pub appearance: Appearance, pub colors: ThemeColors }
pub struct ThemeRegistry { /* built-in themes, including a Dark default */ }
impl ThemeRegistry {
    pub fn builtin() -> Self;
    pub fn by_name(&self, name: &str) -> Option<&Theme>;
    pub fn default_for(&self, appearance: Appearance) -> &Theme;
    pub fn dark_default(&self) -> &Theme;                   // the Dark theme a fresh RootView renders until set_theme (R24)
}
```

## EARS Requirements

R1. WHEN `run()` is invoked, the system shall boot exactly one gpui `Application` and open exactly one window whose root view is a `RootView`, returning `std::process::ExitCode::SUCCESS` on clean application quit.

R2. WHEN the application window is opened, the system shall title it "Marley" and present it with the platform-native macOS titlebar (the traffic-light window controls).

R3. The system shall complete its entire boot sequence without performing any network request, authentication, cloud, or onboarding step.

R4. The system shall render `RootView` as three horizontally-ordered regions — a left dock, then the center pane-group region, then the right dock — in that left-to-right order.

R5. WHILE a dock's state is `Closed`, the system shall allocate that dock zero width and shall give the freed width to the center pane-group region.

R6. WHEN the `toggle_dock(side)` action is dispatched, the system shall invert that side's `DockState` between `Open` and `Closed` and mark the view dirty so it re-renders.

R7. The system shall construct `PaneGroup::single(p)` as a `Leaf(p)` whose `panes()` returns exactly `[p]`.

R8. WHEN `split(target, new, axis, dir)` is called on a leaf holding `target`, the system shall replace that leaf with a `Split` node of the given `axis` whose children are the original and the new pane, ordered with `new` before the original when `dir == Before` and after it when `dir == After`.

R9. WHEN `close(p)` removes a pane that is one of exactly two children of a `Split` node, the system shall replace that `Split` node with its surviving child so that no `Split` node with fewer than two children remains in the tree.

R10. IF `close(p)` is called when `p` is the only pane in the tree, THEN the system shall return `Err(PaneError::LastPane)` and leave the tree unchanged.

R11. WHEN a `split` adds a child to a `Split` node or a `close` removes one, the system shall reset that node's `ratios` to equal fractions that sum to `1.0` (within `f32` rounding) and whose length equals the node's child count.

R12. WHEN `neighbor(from, dir)` is called, the system shall resolve adjacency by this rule: map `dir` to an axis (`Left`/`Right` → `Horizontal`, `Up`/`Down` → `Vertical`) and a movement side (`Left`/`Up` → preceding sibling, `Right`/`Down` → following sibling); ascend from the `from` leaf toward the root and, at the first ancestor `Split` whose `axis` equals `dir`'s axis and in which `from`'s subtree has a sibling on the movement side, step into the immediately-adjacent sibling subtree on that side and descend it, at each `Split` taking the child nearest the shared boundary (the last child for `Left`/`Up` moves, the first child for `Right`/`Down` moves) until a leaf is reached, and return `Some(that_leaf)`; and the system shall return `None` when no such ancestor exists (`from` is at the layout's edge in `dir`).

R13. IF `split` or `close` is given a `PaneId` that is absent from the tree, THEN the system shall return `Err(PaneError::PaneNotFound)` and leave the tree unchanged.

R14. WHEN the `open-command-palette` action is dispatched, the system shall present the command-palette overlay horizontally centered over the window with an empty query string.

R15. WHEN `filter_commands` is called with an empty query, the system shall return one `ScoredCommand` for every input command, in the commands' original registration order.

R16. WHEN `filter_commands` is called with a non-empty query, the system shall rank each command's `title` and each of its `keywords` against the query using the `nucleo` subsequence matcher (the same primitive `marley_search_core::fuzzy_rank` uses), include a command only if at least one of those fields yields a `nucleo` match (the query characters occur in order, case-insensitively), set that command's `ScoredCommand.score` (a `u32`, higher = stronger subsequence match) to the maximum `nucleo` score across its matching fields, order the results by descending `score`, and break score ties by ascending registration order.

R17. WHEN a command is activated from the palette, the system shall dispatch that command's bound action and then dismiss the overlay.

R18. WHEN the Escape key is received while the command palette overlay is open, the system shall dismiss the overlay without dispatching any command.

R19. The system shall provide `Keymap::default_bindings()` mapping the chord `cmd-shift-p` to `"open-command-palette"`, `cmd-d` to `"new-terminal"` (M12.2 #197 — renamed from `"split-pane"`; ⌘D makes a new terminal, not a tile-split), `cmd-shift-l` to `"split-right"` and `cmd-shift-j` to `"split-down"` (M12.2 #197 — the tile-split chords, dispatching to `split_focused_pane`), `cmd-w` to `"close-pane"`, `cmd-b` to `"toggle-left-dock"`, and `cmd-shift-b` to `"toggle-right-dock"`, such that `action_for(binding, stack)` returns the named action for each on every stack (they are GLOBAL rows); and no two bindings shall share both a chord AND a context (verified by `chords_unique_scoped` — M16 #265; the same chord under disjoint contexts is the KeyContext model's point).

R19a (M16 #265; ⌘D rebound + ⌘⇧L added M19 #298). Bindings may carry a `KeyContext` (`Terminal` | `Editor`) scoping them to a focused surface: the active tab publishes its context stack (`Tab::key_context()` — terminal tab → `[Terminal]`, editor tab → `[Editor]`, cockpit tab → `[]`), and `action_for` shall resolve a chord to the eligible binding whose context matches DEEPEST in the stack, with a context-less (global) binding applying only when no scoped binding for that chord matches. The default map scopes `cmd-d` → `"add-next-occurrence"` under `Editor` (shadowing the global `"new-terminal"` there), `cmd-shift-l` → `"select-all-occurrences"` under `Editor` (shadowing the global `"split-right"` there — while an editor pane is focused that chord no longer splits the pane; split-right remains reachable from the command palette), and `cmd-f` → `"open-find"` under `Terminal`.

R20. IF `action_for` is queried with a binding that is not present in the keymap, or present only under contexts absent from the caller's stack, THEN the system shall return `None`.

R21. The system shall make `ThemeRegistry::builtin()` contain at least one theme with `appearance == Light` and at least one with `appearance == Dark`, and `default_for(a)` shall return a theme whose `appearance` equals `a`.

R22. WHEN `by_name(name)` is called, the system shall return `Some(theme)` for an exact (case-sensitive) registered theme name and `None` for any unregistered name.

R23. WHEN `RootView::set_theme(theme)` is called, the system shall make `active_theme()` return that theme and shall mark the view dirty so that every region of `RootView` re-renders using the new theme's `ThemeColors`, leaving no region painting a previous theme's color.

R24. WHILE `set_theme` has never been called on a `RootView`, the system shall make `active_theme()` return the `Dark` default theme (`ThemeRegistry::dark_default()`, `appearance == Dark`) and shall render the shell with it.

R25. WHEN the bundled zsh shell integration emits its `preexec` or `precmd` hook frame, the system shall pass the frame's dynamic values (the command line; the working directory) through an escaping step covering backslash, semicolon, ESC (0x1b), newline, tab, and carriage return per the `AnsiCQuoted` payload grammar — applying the backslash escape before every other step — and shall frame the payload with the DCS selector naming the `AnsiCQuoted` encoding, such that the decoded hook fields equal the original shell values exactly.

R26. The system shall emit every shell-integration hook frame (`init`, `bootstrapped`, `preexec`, `precmd`) with a DCS selector that `marley_terminal` maps to a supported payload encoding, and each frame's payload shall decode without error under that encoding for any UTF-8 command line / working directory (invalid-UTF-8 raw bytes are rejected by the decoder — a documented M1 limitation, not covered by this clause).

R27. WHEN the pane rect layout is computed for a `PaneGroup` and a bounds rectangle, the system shall tile the bounds depth-first — a `Leaf` owning its full rect; a `Split`'s children partitioning the rect along the split axis with child `i`'s extent equal to the parent extent × `ratios[i]`, origins accumulating in child order — such that AREAS are conserved exactly and sibling boundaries are exact for the binary equal-ratio tree, and the result order equals `PaneGroup::panes()`. (On non-dyadic fractional bounds the accumulated origins may drift ≤1 f32 ULP against the parent's far edge — sub-visible; revisit with M2 drag-resize.)

R28. WHEN the split-pane action is dispatched, the system shall obtain a NEW terminal session from the spawn seam BEFORE mutating any workspace state (a spawn failure shall leave the workspace unchanged) and shall register independent per-pane state — session, input buffer, caret — for the new pane; WHEN the close-pane action is dispatched, the system shall remove the closed pane's state after the tree collapse succeeds and RETURN it to the caller, which shall end the session off the main thread (a session drop blocks in the child reap — ~600 ms for a running foreground child, unbounded for a HUP-immune one; the shell must never freeze on close).

R29. WHILE a pane is focused, the system shall route prompt input and command submission ONLY to that pane's state, and split-pane/close-pane shall target the focused pane; WHEN a split completes, the new pane shall become focused; WHEN the focused pane closes, focus shall move to its depth-first predecessor in the pre-close pane order (the successor when it was first); focusing an absent pane shall be rejected with the workspace unchanged; WHEN a pane is clicked, it shall become focused; and the focused pane shall render a border in the theme's accent color while unfocused panes render the theme border color.

R30. WHEN any workspace operation completes, the per-pane state registry's keys shall equal `PaneGroup::panes()` exactly.

R31. WHEN the shell body's region widths are computed for a window width, the two dock states, and a dock width, the system shall give each `Open` dock exactly the dock width, each `Closed` dock exactly zero width, and the center exactly the remainder CLAMPED AT ZERO (a window narrower than its open docks collapses the center; the right dock never overlaps the left) — for all four dock-state combinations — and the render shall size the dock panels AND inset the center pane tiling from that ONE computation.

R32. WHILE the command palette is open, ArrowDown/ArrowUp shall move a selected index over the CURRENT filtered list clamping at both ends (an empty list keeps the selection at zero); WHEN the query is edited (a character or a backspace), the selection shall reset to the top; WHEN Enter is pressed, the system shall dispatch the SELECTED command's verb through the same action router the keymap chords use and dismiss the overlay — or, on an empty filtered list, dismiss with NO dispatch (R17's activation is the SELECTED command, not the top result); the selected row shall render highlighted and each bound command's row shall show its chord.

R33. WHEN `RootView` boots, the system shall load the persisted settings from the config directory (creating the directory if absent; a missing file yields all-defaults and is not created), and shall apply the resolved theme (the saved name validated against the registry — an unknown or absent name falls back to the Dark default), the saved dock states, and the saved terminal column/row counts — such that a missing or invalid settings file boots with the schema defaults and never fails the boot.

R34. WHEN the active theme or a dock's open state changes at runtime, the system shall persist the new value to the settings file so a subsequent boot restores it; a persist failure shall not interrupt the running session (the in-memory state has already changed).

R35. WHEN a caret-motion key (left/right, word-left/right, home, end) is applied to the input prompt, the system shall move the caret to the corresponding `marley_editor` movement result (leaving the buffer text unchanged); WHEN forward-delete is applied, it shall remove the char at the caret if one exists (else no-op); WHEN a character or backspace is applied at an interior caret, the edit shall occur at that position; and the prompt shall render the caret glyph at its offset (the text split before/after the caret's char position, multibyte-safe).

R36. WHEN a command is submitted, the system shall record it in the focused pane's command history unless it equals the most-recent entry, retaining a bounded number of commands (evicting the oldest past capacity); WHILE the command palette is closed, ArrowUp shall recall previous commands into the prompt (stashing the in-progress line on the first step, clamping at the oldest) and ArrowDown shall walk toward newer commands, restoring the stashed line when stepping past the newest; an edit (character / backspace / forward-delete, not a caret motion) shall leave history navigation. Each pane has its own history.

R37. WHEN a pane's rect is laid out, the system shall size that pane's PTY to `(cols, rows) = (floor(rect.w / cell.w), floor(rect.h / cell.h))` where `cell` is the monospace advance width × line height, clamping each axis to at least 1 (a non-positive or non-finite extent/cell yields 1), and shall issue the resize only when the grid differs from the last size applied to that pane. The settings `terminal.cols/rows` remain the initial spawn size.

R38. WHEN a block's styled output is rendered, the system shall map each run's alacritty `Color` to a gpui `Hsla` against an `AnsiPalette` — named colors index the 16-color table (foreground/background/cursor → the theme's default fg/bg); indexed colors resolve 0–15 via the table, 16–231 via the 6×6×6 cube over the `[0,95,135,175,215,255]` ramp, and 232–255 via the grayscale ramp `8+(i-232)·10`; a spec color passes its truecolor through — and shall paint each run as its own span in that color (the `BOLD` flag → bold weight; `INVERSE` → the run painted with its background color). The 16-color ANSI table is an original Marley palette (clean-room).

R39. WHILE a pane is following the latest output, the system shall render the bottom `capacity` content rows so new output stays in view; WHEN the user scrolls up (wheel, PageUp, or Shift+ArrowUp), the pane shall hold a scrolled position — the same window as content grows — until scrolled back to the bottom, when it shall re-anchor to following; scrolling shall clamp at the first row and at the bottom. Running a command re-anchors to the latest output. Each pane has its own viewport. The pane content column is BOTTOM-anchored (`justify_end`) so the prompt sits at the bottom with output above it and empty space (when content is short) at the TOP — a normal-terminal feel (M1.H #49); this also keeps the prompt from being clipped by `overflow_hidden` (the oldest top row clips instead).

R40. WHILE a pane's session holds the alternate screen (a full-screen program such as vim/top/less), the system shall render the live grid (`grid_styled_rows`, colored per R38) instead of the Block list, and shall stream every keystroke to the PTY (`encode_key` → `write_bytes`); WHILE in the cooked Block view, a control key (Ctrl-C/D/Z) shall likewise stream to the PTY while other keys feed the local prompt. Tab-completion at the bare prompt is out of scope (the local-editing-vs-shell-ZLE model decision — intake `prompt-shell-line-editing-model.md`). WHILE a foreground command is running (`is_command_running`, M1.F #40) — even on the PRIMARY screen (an inline interactive program: an arrow-key menu, `read`) — the system shall stream every keystroke to the PTY so the running program receives it; local editing + history apply ONLY at the bare prompt (no running command, not alt-screen).

R41. The system shall render terminal content (the Block command + output, the prompt, and the alt-screen grid) in a MONOSPACE font, and shall measure the pane cell size from that font — the cell width from its em-advance (falling back to `fallback_cell`'s width if unavailable) and the cell height from `fallback_cell`'s font-size ratio, which the terminal rows also render as their line height so the cell metric and the drawn glyphs agree. The cockpit chrome keeps the UI font.

R45. The cockpit shall render structural text through a single type scale — `type_scale(role) -> TextStyle{size, weight}` (`Command → (14, Medium)`, `Output → (14, Normal)`, `Caption → (12, Normal)`) with `weight_value(weight)` giving the CSS numeric weight (`Normal 400 / Medium 500 / Bold 700`) — applying `Command` to the Block command header (with a hover tint) and `Caption` (in the `muted` color) to the dock header; the pure `type_scale`/`weight_value` are gpui-free (the shim wraps the weight in `FontWeight`).

R52. WHEN ⌘↑ / ⌘↓ is pressed, the system shall scroll the focused pane's viewport to the PREVIOUS / NEXT command-block boundary — `block_boundary_rows` (each block's header content-row, from `1 + output_lines` accumulated) + `jump_target(boundaries, viewport_top, forward)` (the first boundary above/below, `None` at the ends — no wrap). WHEN cmd-K is pressed, the system shall clear the focused shell (a `\x0c` Ctrl-L) and snap the viewport back to following. (M1.G #48.)

R51. WHEN cmd-F is pressed while a TERMINAL tab is active (the chord is a `Terminal`-context keymap row since M16 #265 — an editor or cockpit tab shall NOT open this bar), the system shall open a find bar over the focused pane's scrollback; typing a query shall highlight the rows containing a match (`find_matches` per row — all NON-overlapping, case-insensitive matches; an empty query matches nothing); Enter / Shift-Enter shall cycle the match cursor (`match_navigation`, wrapping) and scroll the viewport to the current match; Esc closes. (M1.G #47.)

R50. WHEN a block's re-run affordance is clicked (or cmd-R for the most recent finished block) AND the session is idle (`!is_command_running()`, R26), the system shall resend `Block::rerun_command()`'s command to the focused pane's shell via `write_command` (running it as a new block); WHEN a command is already running it shall not inject. The block is looked up by `(pane, index)` at click-time. (M1.G #46.)

R49. WHEN a command block is hovered, the system shall reveal copy-command and copy-output affordances at the block header's right edge; clicking one shall write `Block::copy_text(that)` (terminal-blocks R29) to the clipboard — the block looked up by `(pane, index)` at click-time. Warp's signature block interaction. (M1.G #45.)

R48. WHEN cmd-C is pressed with a non-empty text selection, the system shall write the selected text to the clipboard — `copy_payload(rows, selection)` (`None` for an absent OR empty selection, so cmd-C with nothing selected is a no-op, never an empty clipboard write; else `Some(selected_text)`); `selected_text` reuses `row_selection` (R47) so the copied text matches the drag-highlight exactly. Completes copy/paste with R46. (M1.G #44.)

R47. The system shall model a terminal text SELECTION — `Selection { anchor, head }` over content `GridPos { row, col }` (col in chars), `normalized()` so a backward drag reads the same as forward; `row_selection(sel, row, row_len)` shall return the selected char span `[from, to)` for a row inside the selection (the first row from `anchor.col`, the last to `head.col`, middle rows whole, clamped to `row_len`) or `None` outside it. A mouse-down starts the selection at the cell under the pointer and a drag extends `head`; the render highlights the selected rows; a new command clears it. (Copy uses this in #44.)

R46. WHEN cmd-V is pressed, the system shall read the clipboard text and, WHILE a foreground command is running, send it to the PTY as `paste_bytes(text, is_bracketed_paste())` (bracketed-wrapped iff the program enabled DECSET 2004, so a multi-line paste is literal data); at the bare prompt it shall insert the text into the local buffer at the caret. (M1.F #42.)

R44. WHEN an open dock is rendered, the system shall paint it as a PANEL — the `surface` elevation, a header row showing `dock_title(side)` (`Left → "Files"`, `Right → "Details"`) with a divider beneath it, a divider border on the dock's INNER edge toward the center (right border for `Left`, left border for `Right`), and a padded content area — over the root `background` so the docks/center/native-titlebar read as one cohesive surface.

R43. WHEN the live prompt is rendered, the system shall paint an input ROW — a `surface` strip with a `❯` (U+276F) marker in `accent`, the cwd/git context segments from `prompt_segments(TerminalSession::current_prompt())` (a `Cwd` segment iff `pwd` is set, a `Git` segment iff `git_branch` is set, order `[Cwd, Git]`, the cwd shortened by `pwd_label` to its last path component), then the input buffer split at the caret (`split_at_caret`) with a styled `accent` caret bar. WHEN no precmd has reported context, `current_prompt()` is `None` and the row shows just the marker + buffer.

R42. WHEN a command Block is rendered, the system shall paint a command HEADER row — an exit-status indicator (a distinct glyph AND color per `status_indicator`: `Running →("○", border)`, `Success →("✓", success)`, `Failure →("✗", danger)`) followed by the command — with a separator above the block; the status is classified by the pure `exit_status_kind(state, exit)` (`Pending`/`Running` → `Running`; `Finished`+`Some(0)` → `Success`; `Finished`+`Some(nonzero)`|`None` → `Failure`, state checked before exit). The output rows render below unchanged (R38).

## Acceptance Criteria
| # | Criterion (maps to R#) | Status |
|---|---|---|
| 1 | `run()` opens exactly one window and returns `ExitCode::SUCCESS` on clean quit (R1) | planned |
| 2 | Window AXTitle == "Marley" with native titlebar / traffic-light controls present (R2) | planned |
| 3 | Boot issues zero network calls and no auth/onboarding step runs (R3) | planned |
| 4 | RootView renders left, center, right regions in that order (R4) | planned |
| 5 | A closed dock has zero width; center absorbs the freed width (R5) | planned |
| 6 | `toggle_dock(Left)` flips Open↔Closed and triggers a re-render (R6) | planned |
| 7 | `single(p).panes() == [p]` and the node is a `Leaf` (R7) | planned |
| 8 | `split` replaces the leaf with a 2-child split, ordering honoring `dir` (R8) | planned |
| 9 | Closing one of two split children collapses to the survivor; no 1-child splits (R9) | planned |
| 10 | `close` on the last pane returns `LastPane`, tree unchanged (R10) | planned |
| 11 | After split/close, a node's ratios are equal and sum to ~1.0 with len == child count (R11) | planned |
| 12 | `neighbor` returns the boundary-rule adjacent pane (incl. nested 2x2), `None` at the edge (R12) | planned |
| 13 | `split`/`close` on an absent id returns `PaneNotFound`, tree unchanged (R13) | planned |
| 14 | open-command-palette shows a centered overlay with empty query (R14) | planned |
| 15 | `filter_commands("")` returns all commands in registration order (R15) | planned |
| 16 | Non-empty query: nucleo-ranked, non-matches excluded, score = max nucleo field score, sorted desc, ties by reg order (R16) | planned |
| 17 | Activating a command dispatches its action and dismisses the overlay (R17) | planned |
| 18 | Escape dismisses the palette and dispatches nothing (R18) | planned |
| 19 | Default keymap maps the five named chords to their action strings (R19) | planned |
| 20 | `action_for` on an unbound chord returns `None` (R20) | planned |
| 21 | builtin has ≥1 Light and ≥1 Dark; `default_for(a).appearance == a` (R21) | planned |
| 22 | `by_name` is `Some` for an exact name, `None` otherwise (R22) | planned |
| 23 | `set_theme` updates `active_theme()`, marks dirty, repaints all regions with the new `ThemeColors` (R23) | planned |
| 24 | A fresh RootView (no `set_theme`) reports a Dark `active_theme()` and paints Dark (R24) | planned |
| 25 | preexec/precmd payloads are AnsiCQuoted-escaped (`\` `;` ESC `\n` `\t` `\r`, backslash first) under the `q` selector; a `;`/`=`/quotes command round-trips into its Block exactly (R25) | planned |
| 26 | Every rc hook frame uses a selector `marley_terminal` supports and decodes clean — zero Plain (`p`) frames remain in the rc (R26) | planned |
| 27 | `pane_rects` tiles exactly — axis partition, ratio extents, origin accumulation, depth-first order == `panes()` — for flat 2x1 + nested 2x2 real-API trees (R27) | planned |
| 28 | split spawns via the seam FIRST (Err → workspace unchanged) and registers independent state; close drops the state, ending the session (R28) | planned |
| 29 | input routes to the focused pane only; actions target focused; split focuses the new pane; close focuses the depth-first predecessor (successor when first); absent focus rejected; click focuses; accent border marks the focused pane (R29) | planned |
| 30 | registry keys == `group.panes()` after every workspace op (R30) | planned |
| 31 | `region_widths`: Open → dock width, Closed → zero, center = remainder clamped at zero (incl. a shrunken-window case), all four combos; the render sizes docks + insets the center from the ONE computation (R31) | planned |
| 32 | Palette selection: arrows clamp at both ends; edits reset to top; Enter dispatches the SELECTED verb once then dismisses (empty list → dismiss, no dispatch); every CommandId maps to its exact verb; highlight + chord chips render (R32/R17) | planned |
| 33 | Boot loads + applies the persisted theme (validated, unknown → dark), dock states, and terminal size from `<config>/settings.toml`; a missing/invalid file boots defaults, never fails (R33) | planned |
| 34 | A theme or dock change persists so it survives relaunch; a persist failure does not interrupt the session (R34) | planned |
| 35 | Prompt caret motion (←/→/word/home/end) via `marley_editor` movement; forward-delete + interior insert/backspace; the caret renders at its char offset (R35) | planned |
| 36 | Per-pane command history: record dedup-last + capacity eviction; ↑ recalls (stash draft, clamp at oldest); ↓ walks newer + restores draft; an edit detaches (R36) | planned |
| 37 | PTY sizes to the pane rect: `plan_resize` floor-divides rect÷cell per axis (clamp ≥1, guard bad cell), resizes only on grid change (R37) | planned |
| 38 | ANSI color: `ansi_color_to_hsla` maps Named/Indexed(cube+grayscale)/Spec against an original theme palette; `run_paint` (inverse-swap + bold); per-run colored spans (R38) | planned |
| 39 | Scrollback: `Viewport` shows the bottom `capacity` rows while following, holds a scrolled window as content grows, re-anchors at the bottom, clamps both ends (R39) | planned |
| 40 | Interactive/raw mode: alt-screen → live-grid render + all keys stream to the PTY (vim/top/less); cooked → Ctrl-C/D/Z stream; the pure `encode_key`/`input_route` (R40) | planned |
| 41 | Terminal content is monospace; the cell metric is measured from that font (`fallback_cell` when the advance is unavailable), height shared with the rendered line-height (R41) | planned |
| 42 | Block header: exit-status indicator (✓ success / ✗ failure / ○ running, colored) + command + separator; pure `exit_status_kind`/`status_indicator` (R42) | planned |
| 43 | Prompt input row: ❯ marker + cwd/git context segments (`prompt_segments`/`pwd_label` from `current_prompt`) + styled caret bar (R43) | planned |
| 44 | Dock panels: header (`dock_title`) + inner-edge divider + padding over `surface`; cohesive root `background` (R44) | planned |
| 47 | Text selection: `Selection`/`normalized` + `row_selection` (per-row span) drive the drag-highlight; copy consumes it (R47) | planned |
| 48 | Copy: cmd-C → `copy_payload`/`selected_text` (reuse `row_selection`) → clipboard; empty selection is a no-op (R48) | planned |
| 49 | Block actions: hover the block header → copy-command / copy-output affordances → `Block::copy_text` → clipboard (R49) | planned |
| 50 | Re-run: click ↻ (or cmd-R) → `Block::rerun_command` → `write_command` to the focused shell, guarded by `!is_command_running` (R50) | planned |
| 51 | Find (cmd-F): `find_matches` (non-overlapping, ASCII-fold) highlights + `match_navigation` (wrap) cycles/scrolls; Esc closes (R51) | planned |
| 52 | Block nav: ⌘↑/⌘↓ → `block_boundary_rows` + `jump_target` (no wrap) scroll to prev/next block; cmd-K → `\x0c` clear (R52) | planned |
| 45 | Type scale: `type_scale(Role)`/`weight_value(TextWeight)` (Command/Output/Caption) + a `muted` caption color; applied to the header (hover) + dock caption (R45) | planned |

## Visual / Behavioral Acceptance
Asserted by the headed-launch macOS accessibility (AXUIElement) + screenshot harness whose normative API, screenshot tolerance, baseline/approval policy, and headed-launch driver are defined in [`../pipeline/visual-testing.spec.md`](../pipeline/visual-testing.spec.md) (quality-bar gate 15). The clauses this spec routes there:
- Exactly one window in the AX tree; `AXTitle == "Marley"`; native titlebar with traffic-light controls present (R2).
- The window body has three horizontally-adjacent regions in left→right order: left dock, center pane group, right dock (R4); toggling a dock removes/restores its region and reflows the center width (R5, R6).
- After a split action, the center region shows two adjacent panes (a 2x1 grid) of equal extent; after closing one, it shows a single full-width pane (R8, R9, R11 visual half).
- The command palette appears as a centered overlay listing commands and disappears on Escape / on activation (R14, R17, R18 visual half).
- Default launch is painted in the Dark theme; calling `set_theme` with a Light theme recolors background/foreground/border of every region (R23, R24 paint halves).
- Screenshot regression baselines (captured/compared per the harness spec): default dark shell, both-docks-open, center split 2x1, palette-open.

## Test Plan
- **Unit (100% on touched logic lines):**
  - R7 → `single_is_leaf_with_one_pane`
  - R8 → `split_orders_children_by_direction`
  - R9 → `close_collapses_two_child_split_to_survivor`
  - R10 → `close_last_pane_is_err_unchanged`
  - R11 → `ratios_renormalize_equal_after_split_and_close`
  - R12 → `neighbor_returns_adjacent_and_none_at_edge` (flat 2x1) **and** `neighbor_resolves_nested_2x2_by_boundary_rule` (a nested `Split{Horizontal,[Split{Vertical,[A,B]}, Split{Vertical,[C,D]}]}` fixture: assert `neighbor(A, Right) == Some(C)`, `neighbor(A, Down) == Some(B)`, `neighbor(D, Left) == Some(B)`, `neighbor(A, Up) == None`, `neighbor(A, Left) == None`)
  - R13 → `pane_ops_on_absent_id_are_pane_not_found`
  - R15 → `filter_empty_query_returns_all_in_order`
  - R16 → `filter_ranks_nucleo_matches_and_breaks_ties_by_reg_order` (assert non-matches excluded, `score` == max nucleo score across `title`/`keywords`, descending sort, ascending reg-order tie-break)
  - R19 → `default_keymap_maps_named_chords`
  - R20 → `action_for_unbound_is_none`
  - R21 → `builtin_has_light_and_dark_and_default_for_matches`
  - R22 → `by_name_exact_match_only`
  - R3 → `boot_makes_no_network_call` (boot the headless path behind a no-network guard / fake clock; assert zero socket use and no auth/onboarding hook fires)
  - R6 → `toggle_dock_inverts_state` (drive `RootView::toggle_dock` against a headless `TestAppContext`, assert `dock(side)` flips and the view is marked dirty)
  - R24 → `fresh_root_view_active_theme_is_dark` (build `RootView` headless without calling `set_theme`; assert `active_theme().appearance == Appearance::Dark` and it is `ThemeRegistry::dark_default()`)
  - R25 → `rc_escape_chain_is_complete_and_backslash_first` (rc text: all six `__marley_quote` substitutions present; the backslash substitution precedes every other; both hooks route their value through `__marley_quote`)
  - R26 → `rc_uses_ansic_quoted_selector_uniformly` (rc text: the four `\ePq` frames present; zero `\ePp` frames remain)
  - R27 → `pane_rects_single_fills_bounds`, `pane_rects_2x1_partitions_along_axis` (BOTH axes), `pane_rects_nested_2x2_tiles_exact_quadrants`, `pane_rects_children_sum_exactly_to_parent` (all fixtures via the real split API)
  - R28 → `split_spawns_via_seam_and_inserts_state`, `split_spawn_failure_leaves_workspace_unchanged`, `close_drops_state` (a drop-tracking mock session)
  - R29 → `focused_state_mut_routes_to_focused_only`, `split_focuses_new_pane`, `close_targets_focused`, `close_moves_focus_to_depth_first_predecessor`, `close_first_pane_focuses_successor`, `focus_absent_pane_rejected`
  - R30 → `registry_matches_tree_after_every_op` (assert after EACH op of a scripted sequence)
  - R31 → `region_widths_all_four_combinations` (ASYMMETRIC dock-state fixtures kill the left↔right swap; the center asserted as the exact remainder for a distinct window width — never a constant) + `region_widths_shrunken_window_clamps_center` (window < open docks → center exactly 0.0; kills the clamp-deletion and `max`→`min` mutants — the clamp is a REACHABLE, tested arm, not dead defense)
  - R32 → `palette_selection_clamps_at_both_ends`, `palette_selection_resets_on_edit` (a push AND a backspace both reset a moved selection — the shrink-rebound mechanism), `activate_returns_selected_command` (a non-zero selection → that exact id), `activate_on_empty_is_none`, `move_down_on_empty_stays_zero`, `action_for_command_maps_every_row` (all five verbs exact + unknown → None + the REMOVED CommandId(3) → None), `toggled_appearance_flips_both_ways`, `keybinding_display_formats_chords` (each modifier arm + ordering)
  - R33 → `applied_from_reads_saved_values` (ASYMMETRIC docks + DISTINCT rows/cols kill the swaps), `applied_from_defaults_on_empty` (each default), `applied_from_unknown_theme_falls_back_to_dark`, `load_manager_in_missing_file_yields_defaults_and_creates_no_file`, `load_manager_in_invalid_toml_errors`, `settings_file_in_joins_settings_toml`, `applied_defaults_are_the_schema_defaults`
  - R34 → `settings_round_trip_survives_reload` (over a NESTED missing config dir — exercising `create_dir_all`: set the theme + BOTH docks → drop → `load_manager_in` again → `applied_from` reflects all three — the persistence proof; without `create_dir_all` the `set` write would fail)
  - R35 → `apply_key_motions_delegate_to_the_right_movement` (an ALL-DISTINCT fixture where the six motion results are pairwise-different — e.g. `"xx aaaaa yy"` caret 5 → Left 4 / Right 6 / WordLeft 3 / WordRight 8 / Home 0 / End 11 — so any delegation swap changes the caret; each returns Edited + text-stable), `delete_forward_deletes_at_caret_and_guards_end`, `char_and_backspace_edit_at_interior_caret`, `split_at_caret_ascii_multibyte_and_ends` (char-not-byte split, both ends)
  - R36 → `record_appends_dedups_last_and_evicts` (dedup-last guard; `with_capacity(2)` eviction; cursor reset), `prev_stashes_draft_walks_older_and_clamps` (first prev stashes + returns newest; walk; clamp at oldest; empty→None), `next_walks_newer_restores_draft_and_none_at_live`, `detach_lets_next_prev_restash_current_line`
  - R37 → `plan_resize_floor_divides_each_axis` (non-square 800×240 rect / 8×16 cell → (100,15) — kills axis-swap + divide→multiply; floor via 805→100), `grid_axis_clamps_below_one_and_guards_bad_cell` (sub-cell→1; zero cell→1 kills `>0`→`>=`; NaN→1), `plan_resize_none_when_unchanged` (equal→None, differ→Some)
- **Integration (cross-crate / headless gpui seams):**
  - R6 + R14 + R17 + R18 → `palette_open_activate_escape` using gpui's `TestAppContext`: dispatch `open-command-palette`, assert overlay state present; activate a command → action dispatched + overlay gone; reopen + Escape → overlay gone, no dispatch.
  - R23 + R24 → `theme_set_repaints_regions`: build `RootView` headless, assert default `active_theme()` is Dark; call `set_theme(light)`; assert `active_theme()` is the new theme, the view was marked dirty, and each region reads the new `ThemeColors` (the `marley_ui_components::ThemeColors` bundle, not a local shape).
  - `marley_ui_components` seam → `theme_wraps_ui_components_vocabulary`: assert `Theme.appearance` is `marley_ui_components::Appearance` and `Theme.colors` is `marley_ui_components::ThemeColors` (no local redeclaration compiles in this crate).
  - R25 end-to-end → `torture_command_round_trips_exactly` (`#[serial]` real zsh over a PTY: a command containing `;`, `=`, double/single quotes, spaces, and a backslash round-trips into `Block.command` exactly; after `cd` into a `;`-named directory, the FOLLOWING block's `prompt.pwd` carries that directory — precmd stages the prompt for the next block).
  - R28 real half → `workspace_two_real_sessions_are_independent` (`#[serial]`: a real second zsh via `split_focused`; a command written to the focused pane produces a Block ONLY there; after `close_focused` the survivor still executes commands).
  - R27/R29 headed → `headed_split_shows_two_live_panes` (`#[ignore]`, headed lane: drive cmd-d through the harness keystroke driver; the content region shows two non-blank halves along the split axis and accent-border pixels mark the focused pane).
- **Visual (headed launch):** the AXUIElement + screenshot assertions in the Visual/Behavioral Acceptance section, executed by the harness defined in `../pipeline/visual-testing.spec.md` (R2, R4, R5 visual half, the split grid and palette overlay, dark/light repaint).
- **Regression:** the full `cargo nextest` suite is green before the M1 gate; the four screenshot baselines must match per the harness spec; the public `PaneGroup`/`filter_commands`/`Keymap`/`ThemeRegistry`/`RootView::{set_theme,active_theme}` signatures are frozen contracts M2 panels bind to.

## Mutation Targets
`cargo mutants` must kill every viable mutant on the testable surface:
- `PaneGroup::split` — swap the `Before`/`After` ordering, drop the leaf-replacement, mutate the chosen axis (caught by R8).
- `PaneGroup::close` — skip the collapse, return `Ok` instead of `LastPane`/`PaneNotFound`, remove the wrong child (caught by R9, R10, R13).
- ratio renormalization — drop the re-equalize, mutate the sum/length (caught by R11).
- `neighbor` — flip the dir→axis map, flip the movement side (preceding↔following sibling), flip the descend boundary choice (first↔last child), drop the edge `None` (caught by R12 flat + nested 2x2 fixtures).
- `filter_commands` — drop the empty-query short-circuit, invert the nucleo match predicate (include a non-match), take the min instead of the max field score, flip the score sort order, drop the registration-order tie-break (caught by R15, R16).
- `Keymap::action_for` / `default_bindings` — perturb a chord→action mapping, return `Some` for an unbound chord (caught by R19, R20).
- `ThemeRegistry` — break the `default_for` appearance match, loosen `by_name` to non-exact, make `dark_default` return a non-Dark theme (caught by R21, R22, R24).
- `RootView::toggle_dock` — drop the invert or the notify (caught by R6).
- `RootView::set_theme` / `active_theme` — drop the active-theme assignment, drop the dirty-mark, return a stale theme (caught by R23, R24).
- `marley_zsh_init` — the whole-fn `""` / `"xyzzy"` replacement mutants (caught by the R25/R26 rc-text containment tests: selector frames, the six-step `__marley_quote` chain, the hook registrations).
- `pane_rects` — flip the axis arms (x/w ↔ y/h), drop the origin accumulation, perturb the ratio extent multiply, reorder the depth-first walk (caught by R27's exact-rect 2x1 + nested-2x2 fixtures).
- `Workspace::split_focused` — reorder spawn-vs-mutate, skip the registry insert, drop the focus-follows-split reassignment, drop the `next_id` increment (caught by R28/R29/R30).
- `Workspace::close_focused` — drop the state removal, flip the predecessor arithmetic (`idx-1` ↔ `idx+1`), invert the `idx == 0` arm, capture the pane order AFTER the collapse instead of before (caught by R28/R29/R30).
- `Workspace::focus` / `focused_state_mut` / `pane_ids` — accept an absent pane, route to a non-focused key, unsort the ids (caught by R29/R30).
- `region_widths` — swap the Open/Closed arms, swap left↔right, replace the center remainder with a constant or flip its arithmetic, delete or invert the zero-clamp (caught by R31's asymmetric four-combination test + the shrunken-window case).
- `PaletteState` — swap up↔down, break the clamp arithmetic at either end, drop a reset-on-edit, activate a fixed index instead of the selected one, return `Some` on an empty list (caught by R32's unit tests).
- `action_for_command` — wrong-verb any row, `Some` for an unknown/removed id (caught by the per-row assertions). `toggled_appearance` — arm swap (caught by the both-ways test). `KeyBinding::display` — drop any modifier arm or reorder (caught by the per-arm formatting test).
- `settings::applied_from` — swap `left`/`right` (asymmetric-dock test), swap `cols`/`rows` (distinct-value test), drop the unknown-theme fallback, mis-read a default. `dock_state` — flip the `true`→Open arm. `applied_defaults` — mutate a default. `settings_file_in` — mutate the `"settings.toml"` join. (All caught by the R33/R34 unit tests; the boot/persist call sites are the app.rs shim.)
- `apply_key` motion arms — call the WRONG movement fn per key (Left↔Right, Word↔char, Home↔End), return `Ignored` instead of `Edited`; `DeleteForward` — flip the `<`-len guard, mutate the `caret..caret+1` range; `split_at_caret` — take/skip off-by-one, char↔byte split. (Caught by R35's asymmetric-fixture per-key + multibyte tests; the movement internals are already `marley_editor`'s tested surface.)
- `CommandHistory` — drop the dedup-last guard (consecutive dup pushed), drop/flip the capacity eviction, mis-index prev/next (the `len-1` newest, the `Some(0)` clamp, the `i-1`/`i+1` steps, the past-newest→draft arm), stash the draft on every prev instead of only the first. (Caught by R36's record/prev/next/detach tests, the eviction via `with_capacity(2)`.)
- `plan_resize`/`grid_axis` — divide→multiply, `.floor()`→ceil, `.max(1.0)`→`.min`/`.max(0.0)`, the `cell > 0.0` guard→`>=` (divide-by-zero → 65535), the axis pairing (cols from w / rows from h), the `!= current`→`==`/drop. (Caught by R37's non-square + sub-cell + zero-cell + unchanged tests; the high-end saturating `as u16` cast + `f32::max`-ignores-NaN are language semantics — no inert clamp/guard beyond `cell > 0.0`.)
- `fallback_cell` — the `* 0.6` advance ratio, the `* 1.2` height ratio, the `> 0.0` guard + the `14.0` non-positive default. (Caught by R41's `fallback_cell` scale + guard tests.)
- `exit_status_kind`/`status_indicator` — the `Pending|Running` vs `Finished` arm, the `Some(0)`→`Success` vs `_`→`Failure` split (the `0` literal + `Some` vs `_`), and the 3 indicator arms (each glyph + color role). (Caught by R42's every-state/exit-combo test incl. `Finished`+`None`→`Failure` + `Running`+`Some(0)`→`Running` [state-first], and the per-kind exact glyph+color asserts.)
- `dock_title` — the `Left`/`Right` arms (+ the whole-fn `""`/`"xyzzy"` replace). (Caught by R44's `dock_title(Left)=="Files"` + `dock_title(Right)=="Details"` asserts.)
- `block_boundary_rows`/`jump_target` — `block_boundary_rows`'s `row += 1 + count` accumulation (the header `+1` + the `+count`) + the push-before-increment; `jump_target`'s forward `> current_top` vs backward `< current_top`, the backward `.rev()` (nearest previous, not first), the `None`-at-ends (no wrap). (Caught by R52's `[2,0,3]→[0,3,4]` + fwd/back-mid + both-ends-None fixtures.)
- `find_matches`/`match_navigation` — `find_matches`'s `query.is_empty()` guard (empty → `[]`), the `start = end` non-overlapping advance (a `+1` would double-count `"aa"` in `"aaa"`), the `to_lowercase` fold; `match_navigation`'s `len == 0` guard (→ 0), the `(current+1)%len` forward wrap, the `(current+len-1)%len` backward wrap. (Caught by R51's non-overlap/case/empty + the fwd/back-wrap/len-0 fixtures.)
- `row_slice`/`selected_text`/`copy_payload` — `row_slice`'s `from.min`/`to.min` clamps + `to<=from` empty; `selected_text`'s `start.row..=end.row` range + the `row>start.row` `\n` guard + the `row>=rows.len()` break; `copy_payload`'s `selection?` (None→None) + `text.is_empty()` guard. (Caught by R48's slice/clamp/multibyte + single/multi-row join + None/empty/Some fixtures.)
- `Selection::normalized` / `row_selection` — the `anchor <= head` compare; the `row < start.row || row > end.row` out-of-range guard; the `row == start.row` (from = anchor.col else 0) + `row == end.row` (to = head.col else row_len) branches; the `.min(row_len)` clamps. (Caught by R47's backward==forward + the multi-row per-row-span asserts + the out-of-range→None + clamp fixtures.)
- `type_scale`/`weight_value` — `type_scale`'s 3 role arms (struct returns; whole-fn→`Default` unviable — no `Default` — so coverage-carried like the palette); `weight_value`'s 3 weight arms + the whole-fn→`0.0`/`1.0` replaces. (Caught by R45's per-role `TextStyle` asserts + per-weight `400/500/700` asserts.)
- `prompt_segments`/`pwd_label` — the generated mutants are `prompt_segments → {vec![], vec![Default]}` (the `Default` one unviable — `PromptSegment` has no `Default`) and `pwd_label → {"", "xyzzy", delete the `!` in `!s.is_empty()`}`. (Caught by R43's both/only-pwd/only-git segment asserts — `both` pins the `[Cwd, Git]` order + that `pwd_label` is applied — and the `pwd_label` edge cases: `/a/b/c→c` [kills `!`-delete, which would return the leading `""`], `foo→foo`, `/a/b/→b`, `/→/`, `""→""` [the fallback-value regression guards].)
- `ansi_color_to_hsla` — the cube index arithmetic (`/36`, `%36`, `/6`, `%6`, the `CUBE_STEPS` lookups), the grayscale `8 + (i-232)*10`, the `Indexed` range bounds (`<16`, `<232`), the `named_index` table (each color→slot), the fg/bg/cursor→default resolution, the Spec passthrough; `run_paint` — the `INVERSE` swap (fg↔bg source) and the `BOLD` flag. (Caught by R38's named/spec + cube-corner/mid + grayscale + inverse/bold tests.)
- `Viewport` — the `max_scroll` subtraction + saturation, the `following` branch in `visible` (bottom vs `top`), the `top.min(max_scroll)` clamp + the `(start+cap).min(content)` end, `capacity.max(1)`, `scroll_up`'s materialise-base selection + `saturating_sub`, `scroll_down`'s `following` early-return + the `next >= max_scroll` re-anchor threshold + the `top=next` hold. (Caught by R39's following/scroll_up/scroll_down/held-grows/capacity-zero tests.)

**ACCEPTED-UNTESTABLE (closed decisions — GPU/display/event-loop-bound, no headless unit harness; asserted only by the visual harness at `../pipeline/visual-testing.spec.md`, excluded from `cargo mutants` via `mutants::skip` with `// justification: headed gpui render path, covered by the AXUIElement + screenshot harness`):**
- The gpui window-open + titlebar configuration (R1 window-open half, R2) and the per-region `Render` element-tree emission (R4, R5 paint half, R23 paint half, R24 paint half) — including the multi-pane absolute-positioned render, the per-pane mouse listeners, and the pump-all timer (R27 paint half, R29 click/paint halves; the DECISIONS live in the pure `workspace` module).
- The `marley_visual_harness` `send_keystroke` HID shell-out the headed lane drives splits with (the harness's existing os_shim/launch exclude class).
- The OS quit callback that yields `ExitCode::SUCCESS` (R1 quit half) — exercised by the headed launch, not unit-mutable.

## Dependencies
- REUSE (permissive, MIT/Apache): `gpui` (Apache-2.0/MIT, upstream Zed) — `Application`, `App`/`Context`, `Window`, `Render`, `Entity`, the element builder, actions, and key bindings. `nucleo` (MIT) — the subsequence fuzzy matcher the M1 `filter_commands` ranks with (the same primitive `marley_search_core::fuzzy_rank` uses); see the §7 M2-merge note.
- Marley components:
  - `marley_core` (M0) for `SessionId`, `~/.marley` paths, single-channel `Config`, and `FeatureFlag` gating.
  - `marley_ui_components` (M1) for the theming value vocabulary `Appearance { Light, Dark }` and the `ThemeColors` bundle (seam-contracts §6) that `Theme`/`ThemeRegistry` wrap. `marley_app` depends on `marley_ui_components` (natural direction, no cycle) and declares no local `Appearance`/`ThemeColors`.
- **M2 (declared, not an M1 dependency):** `marley_search_core::SearchMixer` — the M1 static-list palette migrates onto it (commands registered as a `SyncDataSource`, unifying on `QueryResult`/`f32`); `marley_search_core` is built/tested in isolation at M1 (seam-contracts §7).

## Out of scope / deferred
- The project/agent/Forge panel **contents** that fill the left/right docks and pane kinds — INVENTed in M2; this spec ships the empty dock/pane-group scaffold only.
- PTY internals, the grid, and Block segmentation — owned by `marley_terminal` (separate spec). Since R28 (M1.C) this shell DRIVES one session per pane through `marley_terminal`'s public spawn/write/pump API via the workspace registry, but never reaches into PTY/grid internals.
- **Migrating the command palette onto `marley_search_core::SearchMixer`** (registering commands as a `SyncDataSource`, unifying the command model on `QueryResult`/`f32`) — M2. The M1 `filter_commands`/`ScoredCommand{score:u32}` static-list wrapper is the temporary stand-in (seam-contracts §7).
- Quake/dropdown window mode, multi-window view transfer, and tabs/sessions chrome — deferred (M2+).
- Persisting dock/layout/theme selection to disk, user-defined keybindings/themes, and a keymap-bound theme action — deferred to the settings crate (M2+); this spec uses the built-in defaults and exposes theme selection only through `RootView::set_theme`.
- Auth, cloud, onboarding, telemetry, crash reporting, autoupdate — intentionally removed (offline single-channel shell), not revisited.

## Clean-room provenance
Spec'd from `docs/specs/behavior/app-shell.behavior.md` — a behavior-only reference (observable boot → three-region workspace layout → split/close pane-tree algebra → centered fuzzy command palette → selectable Dark-default theme I/O) authored walled-off from any AGPL/fork source, carrying no private module/type/static names and no fork file paths. Per `seam-contracts.md` §11 (Blockers 10 + 11), this spec's `clean_room` line is downgraded to `behavior-derived from a fork-reference doc; IP-counsel sign-off pending`: the behavioral wall plus IP-counsel sign-off are an open item in `clean-build-plan.md` and must land before any REIMPLEMENT code. The public surface carries no Warp-internal identifier — the theming vocabulary (`Appearance`/`ThemeColors`) is sourced from `marley_ui_components` (§6), and ranking reuses the permissive `nucleo` primitive (§7); the shell links none of Warp's AGPL crates and translates no AGPL source. REUSE crates (`gpui`, `nucleo`) are MIT/Apache.
