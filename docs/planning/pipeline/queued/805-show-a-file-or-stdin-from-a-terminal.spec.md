---
pipeline_id: d027cc3f-0c9e-4c15-abc5-262f3ca81092
ticket: docs/planning/tickets/open/TICKET-805-show-a-file-or-stdin-from-a-terminal.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Show a file or stdin from a terminal
type: feature
slice: T (the block terminal)
references:
  - docs/planning/design-notes/terminal-output-as-components-2026-10-17.md
  - docs/planning/pipeline/queued/799-marley-components-and-the-component-library.spec.md
---

## Title
A command in a Marley terminal hands Marley a file or its stdin to show in a tab, typed by its
extension or by `--as`: a table, a component tree (#799), markdown or an image.

## Scope
### In
- **The request:** `marley-agent show [--as table|csv|tsv|jsonl|component|markdown|image]
  [--title TEXT] <file|->`, a new request on `marley-agent`'s socket beside `Report`, `Release`
  and `Approve` (`marley_agent/src/report.rs`). The socket checks the caller's credentials as it
  does for those, and the terminal it came from is named by the terminal's id, so the tab opens
  beside that terminal.
- **The name typed:** Marley's shell integration defines `show` as a function that runs
  `"$MARLEY_BIN" show "$@"` when `MARLEY_BIN` is set, unless the user's shell already has a
  `show` command, function or alias; the Plan phase settles the name with Chad.
- **What is sent:** a path, or stdin up to 16 MB (read by `marley-agent`, sent in pieces).
  Typing: `--as` first, then the extension (`.csv`, `.tsv`, `.jsonl` table; `.md` markdown;
  `.png`, `.jpg`, `.gif`, `.svg` image; `.json` a component tree when it parses as one, else a
  table when it is an array of objects).
- **What shows:** a table in #803's tab (`tabular_data_preview`), a component in #800's component
  tab, markdown in Zed's markdown preview, an image in Zed's image viewer; a tab per title, reused.
- **Outside Marley** (`MARLEY_BIN` unset, or the socket refuses): `show` prints the file or
  stdin unchanged and exits 0, so a script that uses it works anywhere.
- **Errors** (a missing file, a tree that does not parse) print on stderr and exit 1.

### Out (explicitly deferred)
- An escape sequence a program prints (iTerm2's `File=` or DomTerm's `OSC 72`): any printed text
  can be faked by a `cat`ed file, and Marley's own frames are signed with a nonce programs never
  see.
- Inline images in the terminal.

## Reference (§20)
N/A — Marley-specific in its mechanism (the `marley-agent` socket). Published material: Wave's
`wsh view` (<https://github.com/wavetermdev/waveterm/blob/main/docs/docs/wsh-reference.mdx>) and
Extraterm's `show` with its secret cookie (<https://extraterm.org/guide.html>) are the same idea:
an explicit command, typed by MIME type or extension, shown in a view of its own.

### Prior art
- The code we ship: `marley-agent` and its socket (`crates/marley_mcp/src/agent_socket.rs`, the
  credential check; `crates/marley_agent/src/report.rs`, the requests); the terminal's
  `MARLEY_TERMINAL_ID` and `MARLEY_BIN` (#520, #652); #803's table tab; #800's component tab;
  Zed's `markdown_preview` and `image_viewer`.
- Behavior maps: none.
- Published material: Wave, Extraterm, iTerm2's `File=`, DomTerm, in the design note.

## UI proof
Scenario `script/e2e/805-show-a-file-or-stdin-from-a-terminal.sh`, with #803's fixtures and a
`card.json` component tree:
- `805-01-table`: `show data.csv`: a table tab beside the terminal (REQ-001).
- `805-02-stdin`: `python3 -c` printing CSV `| show --as csv --title People`: a tab titled People
  (REQ-001).
- `805-03-component`: `show card.json`: the component drawn (REQ-001).
- `805-04-markdown` and `805-05-image`: `show notes.md`, `show logo.png` (REQ-001).
- `805-06-outside`: `env -u MARLEY_BIN bash -c 'show notes.md'`: the file printed in the
  terminal (REQ-002).
- `805-07-error`: `show missing.csv`: the error on stderr, exit 1 in the block's status (REQ-003).

## Locked-In Decisions
- **D1 — Through the socket, not the byte stream**, so only a program the user ran can ask.
- **D2 — Never guess:** the type comes from `--as` or the extension.
- **D3 — Plain output outside Marley**, so `show` is safe in scripts.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `show` runs in a Marley terminal with a file or stdin, Marley shall open a tab beside that terminal showing it as the type `--as` or the extension names | shots `805-01` to `805-05` |
| REQ-002 | WHEN `show` runs where Marley cannot be reached, it shall print its input unchanged and exit 0 | shot `805-06-outside` |
| REQ-003 | IF the file is missing or its content does not parse as its type, THEN `show` shall print the error on stderr and exit 1 | shot `805-07-error` |

## Phase Plan
- **P1 Plan:** this spec; at promotion, the command's name with Chad, and the request's wire
  shape.
- **P2 Code:** the request, the program's subcommand, the shell function, the tabs; a review of
  the diff; `script/gates.sh --diff` green.
- **P3 Test:** the visual check above.
- **P4 Complete:** CHANGELOG, guide, ledger, close, archive, commit.
