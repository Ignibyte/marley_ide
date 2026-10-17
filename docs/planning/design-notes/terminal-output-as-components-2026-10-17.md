# Terminal output as components — 2026-10-17

Chad, 2026-10-17: "also while you are at it i'd like to spec out and research from alacritty how
we may be able to transform command line output into gpui components if possible. For example
if we read an output from a csv it could be formatted automatically in a table."

The specs: #803 (a block's output opens as a table in a tab), #804 (the table drawn in the
terminal in place of the block's rows), #805 (`show`: a program hands Marley a file or its stdin
to show). They follow the agent-components batch (#797 to #802) in the Queue.

## What the tree has

- **Blocks.** Each command is an `AnchoredBlock` (`marley_terminal/src/anchored.rs`) with its
  command, exit code, prompt info (the folder) and its output's absolute lines.
  `Terminal::block_output` reads a finished block's text from the main screen, wrapped rows
  joined back into lines; `block_output_kept` says whether its first line is still in the
  scrollback (10,000 lines by default). A full-screen program's output (`less`, `vim`) is on the
  alternate screen and never in a block, so only output printed to the main screen can become a
  table.
- **Zed's table view.** `crates/tabular_data_preview` reads CSV, TSV, PSV, SSV and JSONL by the
  file's extension and draws a sortable, filterable `TableView` that renders only the rows on
  screen. Its `TableView::new` and `set_contents` take cells built from strings
  (`TableCell::Generated`), so a block's text can fill it without its file parser, and
  `TabularDataPreviewPane::open_preview_to_the_side_of_pane` opens a file directly.
- **Block headers and their controls.** `marley_block` in `terminal_view/src/terminal_element.rs`
  draws each block's header; Marley's chips come through `MarleyBlockChip` (set in
  `marley_workbench/src/bookmarks.rs`: the agent mark, the bookmark, Jump to failure, Ask the
  agent) and its hover buttons through `MarleyBlockExtras` (Bookmark, Find, Save as Workflow).
  The block menu is `marley_workbench/src/blocks.rs`.
- **Drawing over a block's rows.** The headers Marley draws (#628) hide the cells of their rows
  and lay an element over the same rows, so it scrolls line by line with the text. The same
  mechanism over a block's output rows can draw a table in their place without moving a row of
  the grid. A pixel gap under a block cannot: the display-row map (`marley_terminal/src/
  display_rows.rs`) only adds uniform space above rows, and a tall gap pushes grid rows out of
  view.
- **A program talking to Marley.** Marley's hooks are DCS frames signed with a nonce the shell
  keeps from the programs it runs, so a program cannot forge one; unknown OSCs are dropped by
  vte. The `marley-agent` program (`$MARLEY_BIN`) talks to Marley over a Unix socket that checks
  the caller's credentials; it knows `Report`, `Release` and `Approve` today. A `show` request
  goes there, not into the byte stream: a `cat`ed file cannot fake it.
- **No inline images** of any protocol (kitty, iTerm2, sixel).

## What other terminals do (researched 2026-10-17)

| Tool | Trigger | Drawn as | Raw text |
|---|---|---|---|
| Wave | `wsh view <file>`, by MIME type | an HTML table in its own block, files up to 1 MB | the file |
| Warp (docs only) | `cat x.md` shows a banner | a viewer pane | the block is unchanged |
| DomTerm | `OSC 72` HTML | sanitized HTML inline | copied with the HTML |
| Extraterm | `show` with a secret cookie | a viewer frame | the frame's text |
| iTerm2 | `OSC 1337 File=` with `type=` | a native view | not documented |
| Nushell | the value's type | a text table | `open --raw` |

Sources: <https://github.com/wavetermdev/waveterm/blob/main/docs/docs/wsh-reference.mdx>,
<https://docs.warp.dev/terminal/more-features/markdown-viewer>,
<https://github.com/warpdotdev/Warp/issues/9138> (tables with a raw toggle, an open request),
<https://domterm.org/Wire-byte-protocol.html>, <https://extraterm.org/guide.html>,
<https://iterm2.com/documentation-escape-codes.html>, <https://www.nushell.sh/book/loading_data.html>.
No terminal turns plain command output into a table on its own; the shells that draw tables know
the data's type. Warp's behavior is from its docs and issue tracker only (§20).

Detection in practice: Python's `csv.Sniffer` calls `has_header` "a rough heuristic"
(<https://docs.python.org/3/library/csv.html>); VisiData and Wave choose by the file's extension;
qsv's `sniff` samples 1,000 rows and "can sometimes fail". Quoted fields hold commas and newlines
(RFC 4180), and ragged rows are common.

## What Marley does

1. **#803 — an offer, then a tab.** A finished block whose output reads as a table gets a Table
   chip; it and the block menu's Open Output as Table open Zed's `TableView` in a tab beside the
   terminal. Detection: a quote-aware parse with at least 2 columns and 3 rows and the same field
   count in nearly every row, or JSON that is an array of objects, or a markdown table; a `cat`,
   `head`, `tail` or `bat` of a `.csv`, `.tsv` or `.jsonl` file is taken from the file itself.
   Never on the alternate screen, never on output with colors or cursor moves, and sniffed on a
   background task over a capped sample.
2. **#804 — in place.** The same table drawn over the block's own rows, toggled per block from
   the header, with a setting to open detected tables in place by themselves. The terminal's
   text stays the source: copy, search and links read the rows.
3. **#805 — the explicit way.** `show <file|->` through `marley-agent` (the socket, so nothing
   can fake it): a file or stdin, typed by extension or `--as`, shown in a tab: a table, a #799
   component tree, markdown or an image. Outside Marley it prints the text.
