---
pipeline_id: 13d72b7b-3240-4532-a697-ca1e8f6508b0
ticket: docs/planning/tickets/open/TICKET-619-block-path-links-at-the-blocks-folder.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "A block's path links resolve against the block's own folder"
type: feature
slice: prong 1 T2
references: [docs/marley/three-prong-plan.md, docs/zed_architecture/subsystems/08-terminal-tasks-fusion.md]
---

## Title
A relative `path:line:col` in a block's output opens against the folder that block's command ran
in, not the shell's folder at the time of the click (plan T2, first half).

## Scope
### In
- `crates/terminal/src/terminal.rs` (Zed crate, an additive `// Marley:` hunk): where
  `process_hyperlink` builds a `PathLikeTarget`, a match line that lies inside a local block
  (`AnchoredBlocks`'s `block_lines`) takes that block's `prompt.pwd` as `terminal_dir`; any other
  line keeps `cwd_at_line`. `process_hyperlink` cannot lock the terminal, so the absolute line is
  computed from what the caller passes, as `history_size` is passed today.
- The hover tooltip and the click both follow, since both go through `PathLikeTarget`.

### Out (explicitly deferred)
- OSC 8 links (they carry their own URI) and URLs.
- Blocks on a remote host (`block_host` set): their folders are on the other machine; they keep
  upstream's resolution.
- The prompt line itself and lines outside any block.

## Reference (§20)
- **Upstream Zed:** `terminal::Terminal::process_hyperlink` and `cwd_at_line` (a per-line folder
  history fed by process polling, which falls back to the current folder once the scrollback is
  full and for remote terminals), opened by `terminal_view::terminal_path_like_target` through
  `workspace::path_link::resolve_open_target`. Kept: detection, the regexes, the opening.
- **Warp (behavior):** a failed build's `file:line:col` resolves against the block's own captured
  folder (`docs/warp_architecture/subsystems/03-terminal-session-core.md`, the fusion paragraph at
  424-431; `docs/zed_architecture/subsystems/08-terminal-tasks-fusion.md` §7).

### Prior art
- **Behavior maps:** the two notes above, and plan D6 (`docs/marley/three-prong-plan.md`).
- **Published material:** none needed.
- **Code we already ship:** `AnchoredBlock { prompt: PromptInfo { pwd }, prompt_line,
  output_start, output_end }` and `block_lines` (`crates/marley_terminal/src/anchored.rs`);
  `Terminal::marley_anchored()`; `process_hyperlink` and `cwd_at_line` (`terminal.rs`);
  `hyperlinks::find_from_grid_point`. No crate owns a block's folder but `marley_terminal`.

## UI proof
`script/e2e/619-block-path-links-at-the-blocks-folder.sh` (sway): `repo/a/src/main.rs` and
`repo/b/src/main.rs`, each with its own first lines; `terminal.max_scroll_history_lines` small so
Zed's per-line folder history is full; in `a`, `printf 'src/main.rs:2:5\n'`; then `cd ../b`.
- `hover.png`: the pointer on the printed link with Ctrl held, its tooltip naming `a/src/main.rs`;
- `opened.png`: after the click, the editor on `a/src/main.rs` at line 2 (its own text).

## Locked-In Decisions
- D1 — A line inside a local block resolves against that block's `prompt.pwd`; Zed's resolution
  stays for everything else.
- D2 — The change is one hunk in Zed's `terminal` crate, with its ledger row; the block lookup
  is a pure function in `marley_terminal`.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user opens a relative path link inside a finished local block, Marley shall resolve it against the folder that block's command ran in. | Shot `opened.png` |
| REQ-002 | WHILE the user points at such a link, its tooltip shall name the file in the block's folder. | Shot `hover.png` |
| REQ-003 | WHEN the link lies outside any block or in a remote block, Marley shall resolve it as Zed does. | Review |

## Phase Plan
- **P1 Plan** — promote, re-verify the seams, the design.
- **P2 Code** — the lookup in `marley_terminal`, the hunk in `terminal.rs` and its row; a review;
  the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
