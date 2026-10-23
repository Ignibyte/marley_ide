# Boards vs Marley today: side panels, drawer, editor, search, keymap, settings, states, theme

Shots are in `review shots/`, and the tour ran with default settings. Code paths are relative to `/srv/stacks/marley_ide`. Sizes: S (an hour or two), M (a day), L (more).

## SidePanels

- **The Inspector is off by default.** The board shows Changes and Outline in a ~340 px column at the right of the editor. Marley opens Git and Outline as full-width tabs titled "Git Panel" and "Outline Panel", each taking half the window (tour-07, tour-08). The Inspector only appears with `marley.side_inspector` on, and it is off in the Settings shot (tour-20). Kind: deferred (#837 open decision 1: a choice beside #747's tabs, default off). Size: S to flip the default.
- **Panel tabs use Zed's names.** The board says "Changes" and "Outline". With the Inspector off, the tabs read "Git Panel" and "Outline Panel" (tour-07/08), even though the status bar buttons say "Changes" and "Outline". Kind: look. Size: S.
- **No Files in the Inspector.** The board's switcher has Files | Changes | Outline. Marley's has only Changes and Outline (`crates/marley_workbench/src/inspector.rs:24`); Files pops out of the left dock instead. Kind: deferred (#868, Chad's request that Files be a left-hand pop-out). Size: none unless Chad changes his mind.
- **No count on Changes.** The board's switcher says "Changes 1". Marley's says "Changes" (`inspector.rs:253`). Kind: deferred (#837 Out). Size: S.
- **Switcher drawn as plain buttons.** The board draws a segmented control: three equal tabs in a rounded track. Marley draws a row of Subtle buttons over a bottom border (`inspector.rs:245-258`). Not seen in a run. Kind: look. Size: S.
- **No Inspector button in the tab bar.** The board has an "Inspector" toggle at the right of the tab strip. Marley has the `marley: toggle inspector` action and the settings row, but no button; the tab bar ends with + / split / maximize (tour-05). Kind: missing. Size: S.
- **Inspector not restored.** It is not saved with the workspace and is gone after a restart. Kind: deferred (#837 Out). Size: M.
- **Files has no filter field.** The board puts "Find a file" at the top of the tree. Zed's project panel has none (tour-06). Kind: missing. Size: M.
- **No "M" badge on changed files.** The board shows a mono "M" at the right of a changed file's row. Marley only tints the name (tour-06, `main.rs`). Kind: look. Size: S.
- **Files column is too wide.** The board's column is 290–340 px. Marley's left dock opens about 570 px wide (tour-06, x 260–830). Kind: look. Size: S.
- **Changes panel layout differs.** The board shows a "Changes on main" heading with Stage All, a checkbox at the left of each file row, and a full-width coat-orange "Commit" button. Marley shows Zed's Changes (1) | History tabs, "View Diff", a "Tracked" group, checkboxes at the right, and a small "Commit Tracked" split button (tour-07). Kind: look. Size: M.
- **No "Ask an agent" button by Commit.** The board puts "Ask an agent" next to Commit and Publish. Grepping "Ask an agent" in `crates/git_ui` finds nothing; Zed's git panel has Publish and Commit Tracked only (tour-07). Kind: missing. Size: S–M.
- **Outline lacks its header and kind labels.** The board heads the list "Outline · main.rs" and labels every symbol with its kind (fn, struct, field, impl). Marley shows "Search buffer symbols…" and no kind on fields: "title" has no "field" (tour-08). Kind: look. Size: S.
- **No "+2 since main" in the breadcrumb.** The board ends the breadcrumb with a teal "+2 since main" chip. Not found: grepped "since" in `crates/marley_workbench` and `crates/breadcrumbs`; only the rail shows "+2 −0" (tour-05). Kind: missing. Size: S.
- **Breadcrumb format differs.** The board reads "repo › src › main.rs › fn main". Marley reads "src/main.rs › fn main", with no project name and the path as one segment (tour-05). Kind: look. Size: S.

## BottomPanel

- **No Output or Tasks tabs.** The board's drawer has four tabs: Problems, Debug, Output, Tasks. Marley's has Problems and Debug (`crates/marley_workbench/src/drawer.rs:25`; tour-10). Kind: deferred (#838 Out, open decision 4). Size: M–L.
- **Tab labels lack count pills.** The board gives each tab a pill: Problems "1" (red when there are errors), Debug "paused", Output "rust-analyzer", Tasks "0". Marley's tabs are plain Subtle buttons: "Problems N" as text, and "Debug" with no state (`drawer.rs:90-98`; tour-10). Kind: look. Size: S.
- **Drawer shape differs.** The board draws the drawer as a rounded card inset 10 px, with a grab handle at its top edge. Marley uses Zed's flush bottom dock (tour-10). Kind: look. Size: S–M.
- **The drawer's own buttons are missing.** The board has Move to the side, Maximize and Close at the strip's right. Marley shows the debugger's toolbar instead (+, debug.json, docs, trash, ×; tour-10), and Problems shows Zed's. Kind: missing. Size: S.
- **Debug view keeps Zed's layout.** The board puts a row of Continue / Step over / Step in / Step out / Stop over the call stack on the left, Variables (name, type, value) in the middle, and Breakpoints with checkboxes and descriptions on the right. Marley shows Zed's debugger panes as they are (tour-10 shows Breakpoints and the New Session menu; no session was running). Kind: deferred (#838 Out: restyling the debugger's panes). Size: M.
- **No "Ask an agent about this stop".** Grepping it in `crates` finds nothing. Kind: deferred (#838 Out). Size: S–M.
- **Problems is not a list.** The board shows a "Filter problems" field, then one row per problem: severity dot, message, place, and a "Fix with an agent" pill. Marley's Problems is Zed's diagnostics multibuffer of excerpts (`drawer.rs:146`). "Fix with an agent" is deferred (#838 Out), but the filter field and the row list were not deferred. The editor's code actions do have "Ask an agent to fix this" (#899). Kind: missing (list, filter); deferred (#838, Fix with an agent). Size: M.
- **No "paused" chip on the stopped line.** The board shows "paused · name = "thread"" on the paused line. Marley has only Zed's stop arrow and highlight, plus Zed's value hints (`show_value_hints`, `assets/settings/default.json:855`); no "paused" label was found. Not seen in a run. Kind: look. Size: S.

## EditorPopups

- **Completion kinds are single letters.** The board labels each completion with a short word ("fn", "fld"). Marley shows Zed's colored single letter ("f", "m"; #856 Phase 3 shot `856-01-completions`). Kind: look. Size: S.
- **No note beside the grey prediction.** The board shows the next edit as grey text with "Tab takes the prediction" beside it. Not found: grepped "Tab takes" and "prediction" in `crates`. Zed draws its own accept hint, and predictions need a provider (`edit_predictions.provider: "zed"`, which needs sign-in). Not seen in a run. Kind: missing. Size: S.
- **No error at the line's end.** The board shows "⚠ unused variable: rows" at the end of the line. Zed's inline diagnostics are off by default (`assets/settings/default.json:2113`, `"enabled": false`), so the error appears only on hover. Kind: missing (setting default). Size: S.
- Code actions (heading, key, "Ask an agent to fix this…"), the rename note and inline assist's line match the board (#856, #899, #900).

## SearchMore

- **Search results are excerpts, not rows.** The board lists each hit as a `file:line` row with the hit's line under it. Zed shows a multibuffer of excerpts with file headers; Marley changed only the count's wording. Kind: deferred (#891 Out). Size: M.
- **Replace field is hidden.** The board shows "Replace with" under the query at all times. Zed hides it until replace is toggled. Not seen in a run. Kind: look. Size: S.
- **Open Recent wording differs.** The board labels a project open in another window "open" and uses the placeholder "Open a recent project". Marley says "this window" only for its own window and keeps Zed's "Search projects…" (`crates/recent_projects/src/recent_projects.rs:977`). The "open" label is deferred (#854 Out: the picker does not know other windows); the placeholder is a look gap. Size: S.
- **Branch picker placeholder differs.** The board says "Switch to a branch, or name a new one". Marley says Zed's "Switch or type to create a branch…" (`crates/git_ui/src/branch_picker.rs:1285`). Kind: look. Size: S.
- **History is Zed's tab.** The board's History is a list in the Changes inspector: a dot, the subject, then "sha · when". Marley uses Zed's git panel History tab, which shows subject, author, when and sha (tour-07 shows the tab). Kind: deferred (#897 Out: dropped as a duplicate). Size: S–M if wanted.
- **"Upgrade" instead of "Update".** The board's extension button says "Update". Marley says Zed's "Upgrade" (`crates/extensions_ui/src/components/extension_card.rs:361`). Kind: look. Size: S.
- The branch dots and notes, the extension tiles with Installed in green, the match count, Open Recent's marks and the guide with Try it match the board (#854, #884, #891, #897).

## Keymap

- **Column names differ.** The board has Action | Keys | Where | From. Marley has Zed's Action | Arguments | Keystrokes | Context | Source (`crates/keymap_editor/src/keymap_editor.rs:2149`). Kind: look. Size: S.
- **Marley's bindings are not marked as Marley's.** The board's From column says "Marley" for Marley's bindings and "Yours" for the user's. Marley tags its own keymap as Zed's `Default` (`crates/marley_workbench/src/marley_workbench.rs:1289`), so those rows read "Default", and the user's read "User". Kind: missing. Size: M (a new source in `KeybindSource`, which is a Zed crate).
- **"Find by keys" is an icon.** The board has a "Find by keys" text button beside "Find an action". Zed has an icon toggle for keystroke search (`keymap_editor.rs:203`). Kind: look. Size: S.
- **Record-keys dialog wording differs.** The board titles it "New keys for marley: filter block", notes the clash as "Also Find in Block, in Terminal && MarleyBlockSelected", and offers Cancel and "Save Anyway". Zed says "Your keybind would conflict with the "X" action" and offers Cancel and "Save" (`keymap_editor.rs:2842, 3206-3209`). Kind: look. Size: S.
- **Zed's settings pages keep Zed's look.** The board's lower half draws General, Appearance, Editor, Languages & Tools, Search & Files, Window & Layout, Panels, Terminal, Version Control, Debugger, AI, Collaboration, Network and Developer as cards, like the Marley page. Only the Marley page is carded (`crates/settings_ui/src/settings_ui.rs:3734`). Kind: deferred (#844 Out: "Other pages keep Zed's look"). Size: M.
- The keymap editor's colors are the theme's; #844 says the board asks nothing more.

## Settings

- **No glyph tiles in the section list.** The board puts a small tile before each section (M, G, Ap, Km, Ed…). Marley shows a disclosure chevron (tour-20). Kind: look. Size: S.
- **Zed's scope tabs stay.** The board has "You | This project" scope tabs. Marley keeps Zed's "User | repo" (tour-20). Kind: deferred (#844 Out). Size: S.
- **Segmented choices are in Zed's order.** The board puts the Marley option first: "Marley | Zed". Marley shows "Zed | Marley" (tour-20). Kind: look. Size: S.
- **No "At the prompt" group.** On the board, Prompt editor, English at the prompt and Follow Omarchy's theme form their own card. Marley lists them under Layout, before the Agents section (`crates/settings_ui/src/marley_page.rs:61-238`). Kind: look. Size: S.
- **"Agent text full width" is not on the Marley page.** The board puts `agent.limit_content_width` in Layout. Not found in `marley_page.rs`. Kind: missing. Size: S.
- **Row titles use Title Case.** The board uses sentence case ("Rail order", "Prompt editor"). Marley uses Title Case ("Rail Order", "Prompt Editor"; tour-20, `marley_page.rs:83`, 169). Kind: look. Size: S.
- **Lists and maps have no controls.** The 17 list and map keys still open settings.json. Kind: deferred (#844 Out). Size: L.
- Cards, the key line, "changed · reset", switches and segmented choices match the board (tour-20).

## States

- **No card for a missing agent CLI.** The board shows "Codex is not installed / How to install". Kind: deferred (#894 Out: Troubleshooting says which CLI is missing, and launchers offer only the CLIs found). Size: S–M.
- **Home's empty card differs.** The board's card reads "No project open" and shows in a window with no project. Marley's reads "No recent projects" and shows only when there are no recent projects (`crates/marley_workbench/src/home_page.rs:617`). Kind: look. Size: S.
- **Files keeps Zed's empty state.** In a window with no folder, the Files panel shows Zed's "Choose one of the options below to use the Project Panel / Open Project / Clone Repository", not the board's card (tour-20, left pane). Kind: look. Size: S.
- **Rusty dropped shows no retry countdown.** The board says "rusty-mcp exited. Trying again in 4 s." Marley shows the reason (or "It does not answer.") and Try Now, with no countdown (`crates/marley_workbench/src/rusty.rs:458-470`). Kind: missing. Size: S.
- **Harness card title differs.** The board says "The harness is stopped". Marley says "The harness is not running" (`crates/marley_workbench/src/rail.rs:5269`). Kind: look. Size: S.
- **No focus ring on rail rows or pickers.** The board's example is a rail row with the keyboard on it. Rail rows and pickers show a selected row, not the ring. Kind: deferred (#895 Out, Chad's answer). Size: M.
- **No lighter ring on orange buttons.** The board uses a lighter orange ring on an orange button so the ring still shows. `focus_ring` always uses `border_focused`, #ff8c42 in Night (`crates/ui/src/styles/elevation.rs:122-125`). Kind: missing. Size: S.
- The rail's 180 ms fold with fading labels, the needs-you pulse, reduced motion, and the no-threads, no-match, Chromium, Forge-offline and harness cards match the board (#857, #858, #894, #895).

## Theme

- **Geist fonts are not shipped.** The board sets words in Geist and code and paths in Geist Mono. Marley ships only `assets/fonts/ibm-plex-sans` and `lilex`, and defaults to `.ZedSans` and `.ZedMono` (`assets/settings/default.json:33, 61`). Kind: missing. Size: M (bundle the fonts, set the defaults, check every screen).
- **About has half the line.** The board's line goes on: "Her coat is the color you act with, her eyes the color of things done." About shows only the first sentence (`crates/zed/src/zed.rs:1638`). Kind: look. Size: S.
- **Day accent is darker.** The board's Day accent is #d9661a. Marley Day uses #b4500e, because white on #d9661a is 3.6:1 and fails contrast. Kind: deferred (deliberate, #828 notes). Size: none.
- **No mark on Home or a first-run screen.** Kind: deferred (#829 Out). Size: S.
- **Primary buttons use the lighter orange.** The board fills "Start agent" with the coat, #ff8c42, under dark ink. Marley's primary buttons use `text_accent`, Marmalade #ffb27a (`crates/marley_workbench/src/prompts.rs:144-148`). Kind: look. Size: S.
- The theme family, both modes' grounds, text and state colors, the cat mark at its sizes and the app icon match the board (#828, #829).
