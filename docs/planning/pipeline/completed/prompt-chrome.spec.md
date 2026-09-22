---
pipeline_id: d3c894fe-9e13-466d-80cd-521016e84a1e
ticket: forge#37 (4121bade-c650-49c1-a0af-9e38d9102d5d) · local docs/planning/tickets/open/TICKET-037-prompt-chrome.md
aar_id: f6d7851d-8f2f-4956-ab34-acfb52bb4895
status: Phase 5 — Complete PASS
title: prompt + input row chrome
type: feature
milestone: M1.E
references:
  - crates/terminal_blocks/src/session.rs + apply.rs (expose current_prompt → the model's staged_prompt)
  - crates/marley_app/src/prompt.rs (NEW — prompt_segments + pwd_label, pure)
  - crates/marley_app/src/app.rs (the input-row render — shim)
  - crates/terminal_blocks/src/block.rs (PromptInfo{pwd, git_branch, …})
  - docs/specs/SPEC-app-shell.spec.md + SPEC-terminal-blocks (the input-row + current_prompt clauses)
---

## Title
The prompt renders as a bare `{before}▏{after}` row — no affordance that it's the live input, and
none of the cwd/git context a real (Warp) prompt shows. Give it Warp's input treatment: a `❯` marker,
a styled caret, an input-row surface, and a cwd/git context prompt from the live `PromptInfo`.

## Scope
### In
- `crates/terminal_blocks/src/apply.rs` — `SessionModel::current_prompt(&self) -> Option<&PromptInfo>`
  (`pub(crate)`, returns `self.staged_prompt.as_ref()` — the context precmd staged for the LIVE
  prompt, before the next command runs).
- `crates/terminal_blocks/src/session.rs` — `TerminalSession::current_prompt(&self) ->
  Option<&PromptInfo>` (pub, delegates). gpui-free.
- `crates/marley_app/src/prompt.rs` (NEW, PURE — cov/MSI 100):
  - `SegmentKind { Cwd, Git }`; `PromptSegment { text: String, kind: SegmentKind }`.
  - `prompt_segments(info: &PromptInfo) -> Vec<PromptSegment>` — `pwd` present → a `Cwd` segment
    (`pwd_label`); `git_branch` present → a `Git` segment; order `[Cwd, Git]`; each absent field
    contributes nothing.
  - `pwd_label(pwd: &str) -> &str` — the last non-empty path component (`/Users/c/marley → marley`,
    `/ → /`, trailing-slash tolerant), so the cwd segment is compact.
- `crates/marley_app/src/lib.rs` — `mod prompt;`.
- `crates/marley_app/src/app.rs` (SHIM) — the prompt row becomes an input ROW: a subtle
  `surface`/padding/rounded container, a leading `❯` marker in `accent`, the `prompt_segments`
  (Cwd/Git colored, from `current_prompt()`), then the buffer split at the caret (#28) with a styled
  caret block (a thin `accent` bar, not the bare `▏` char).
- SPEC-app-shell (R43 input-row) + SPEC-terminal-blocks (current_prompt) + Mutation Targets.
  CHANGELOG + arch docs.

### Out (explicitly deferred)
- `virtual_env` / `node_version` segments (the same Some/None pattern — a trivial follow-up; the
  first cut is pwd + git_branch). `~`-relative pwd (basename is pure/env-free; `~`-shortening needs
  `$HOME`). The full shell-ZLE prompt richness / tab-completion (the #33 `prompt-shell-line-editing-
  model` intake — separate). Prompt theming/customization, right-prompt, timing (M2).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `current_prompt()` exposes `staged_prompt` (the LIVE context) — NOT the last block's `.prompt`
  (which is stale after a `cd`). It's `None` until the first precmd; the shim shows just `❯` then.
- D2 — `pwd_label` = last non-empty path component (pure, env-free); `~`-relative deferred.
- D3 — First cut = pwd + git_branch segments; `SegmentKind{Cwd,Git}` (virtual_env/node deferred).
- D4 — `❯` (U+276F) marker per the common-matched-set glyph rule (#36). Marker in `accent`.
- D5 — PURE: `current_prompt` + `prompt_segments` + `pwd_label` (cov/MSI 100); the input-row div +
  caret block are SHIM (app.rs, masked visual).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a precmd has staged prompt context, `TerminalSession::current_prompt()` shall return `Some(&PromptInfo)`; before any precmd (or after a command starts and consumes it) it shall return `None`. | unit test (terminal_blocks: precmd→Some; fresh/consumed→None) |
| REQ-002 | WHEN `prompt_segments(info)` is called, it shall emit a `Cwd` segment iff `pwd` is `Some` and a `Git` segment iff `git_branch` is `Some`, in order `[Cwd, Git]`; the `Cwd` text shall be `pwd_label(pwd)`. | unit tests (both/only-pwd/only-git/neither; the label) |
| REQ-003 | WHEN `pwd_label(pwd)` is called, it shall return the last non-empty `/`-separated component (`"/a/b/c"→"c"`, `"foo"→"foo"`, trailing-slash tolerant, `"/"→"/"`). | unit tests (each case) |
| REQ-004 | WHEN the prompt is rendered, the system shall paint an input-row surface with a `❯` accent marker, the cwd/git segments, and the input buffer with a styled caret. | shim + the masked prompt visual — chad-verified |
| REQ-005 | WHEN `scripts/gates.sh` runs, every gate shall be GREEN with coverage 100%/MSI 100% on `prompt_segments`/`pwd_label`/`current_prompt`; the prompt visual rides the masked deferral. | gate exit 0 + receipt + gate:15 |

## Phase Plan
- **P2 Design** — the exact `prompt.rs` shapes (PromptSegment/SegmentKind, prompt_segments order,
  pwd_label impl + edge cases), the `current_prompt` delegators, the app.rs input-row shim, the SPEC
  clauses + mutation targets.
- **P3 Implement** — terminal_blocks accessors + prompt.rs + mod + app.rs input row + specs + CHANGELOG.
- **P3.5 Inspect** — critics: pwd_label edge cases (root/trailing/empty), prompt_segments branches +
  order, current_prompt None-after-consume, the ❯ glyph, the shim uses the pure decision.
- **P4 Validate** — the prompt.rs + current_prompt unit tests + gate GREEN + the masked visual.
- **P5 Complete** — docs, AAR, archive, close #37.
