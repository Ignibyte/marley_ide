---
pipeline_id: 62bbb8ee-367f-45e7-b489-b00584a2dfec
ticket: docs/planning/tickets/closed/TICKET-592-launch-approval-verbatim.md
status: Phase 4 — Complete PASS
title: Show the launch approval verbatim
type: bug
slice: W-series, launch configs (#527's follow-up)
references: [docs/planning/pipeline/completed/527-project-launch-configs.spec.md]
---

## Title
The launch approval's text shows exactly what runs. Zed's prompt on Linux renders its detail
as Markdown, so the approval #527 added loses its line breaks, rewrites `--` and quotes, and
can hide part of a repository's command as markup.

## Scope
### In
- `launch.rs`: the approval's detail given to the prompt as a Markdown code block, so Zed's
  prompt shows it verbatim.

### Out (explicitly deferred)
- The config's name in the question (`Run <name>?`) stays as it is: the name is the menu's
  label, not something that runs.
- The other five `window.prompt` calls in Marley carry Marley's own text, not a repository's.
- Zed's `ui_prompt` itself is not changed.

## Reference (§20)
Upstream Zed: `crates/ui_prompt/src/ui_prompt.rs` draws the prompt on Linux and renders both
the message and the detail with `markdown::Markdown`, whose parser enables smart punctuation
(`crates/markdown/src/parser.rs:17`). Marley keeps Zed's prompt and uses the Markdown it
already renders: a fenced code block, which CommonMark shows literally, line breaks included.

### Prior art
- The code we ship: `markdown` renders a fenced code block's text as-is, with no smart
  punctuation, no inline markup and no HTML. A fence closes only on a run of at least as many
  backticks as opened it (CommonMark), so a fence longer than any run inside the text cannot
  be closed early. `code_block_overflow_x_scroll` is false in the prompt's default style, so
  long lines wrap.
- `ui_prompt` offers no plain-text detail, and `cx.set_prompt_builder` is app-wide, so a
  plain-text renderer for one prompt would mean a Zed change for no gain.
- Behavior maps: nothing on this seam in `docs/zed_architecture/`.

## UI proof
The scenario `script/e2e/527-project-launch-configs.sh` (#527's visual check, under
`compositor sway`), re-run on the fixed build:
- `527-02-approve`: the approval lists the three items one per line, `--bind` with two hyphens
  and the title in straight quotes.
- `527-05-changed`: the changed-file question shows the new text the same way.

## Locked-In Decisions
- D1 — The hash that records an approval covers the plain text, as before, so a config
  approved before the fix is not asked about again.
- D2 — The fence is backticks, one longer than the longest run of backticks in the text and
  never shorter than three.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a launch config asks for approval, the prompt shall show each item on its own line. | Shot 527-02-approve |
| REQ-002 | WHEN an item's text holds Markdown punctuation (`--`, quotes, `*`, `<!-- -->`), the prompt shall show those characters as written. | Shot 527-02-approve (`--bind`, the quotes); review of the diff for the rest |
| REQ-003 | WHEN the text holds a run of backticks, the code block shall not end inside it. | Review of the diff |
| REQ-004 | WHEN a config was approved before this change and has not changed, Marley shall open it with no question. | Review of the diff (the hash input is unchanged) |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `verbatim()` in `launch.rs` and its one call; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — re-run 527's scenario on the new build, read 527-02 and 527-05.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_workbench.md` if it describes the
  approval, an `F-` block, close the ticket, archive, commit with 527's scenario.
