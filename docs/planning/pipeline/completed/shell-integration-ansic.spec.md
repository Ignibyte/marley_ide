---
pipeline_id: 38ff9e79-08a3-45c3-b09a-ccae6789697e
ticket: forge#22 (02632a5b-cf30-48c7-a008-2cc1c0b9c5d7) · local docs/planning/tickets/open/TICKET-022-ansic-emit.md
aar_id: c88f9a9c-44b8-4020-8c02-b6a49c013244
status: Phase 5 — Complete PASS
title: shell-integration AnsiCQuoted emit — `;`/`=` in a command corrupts Block segmentation
type: bug
milestone: M1.C
references:
  - docs/specs/SPEC-terminal-blocks.spec.md (R9/R20 — the shipped decode contract)
  - docs/specs/SPEC-app-shell.spec.md (gains the emit-side clause this pipeline)
  - docs/marley_architecture/app_shell.md (documents the Plain limitation being fixed)
---

## Title
Fix the M1.A shell-integration emit encoding: `marley_zsh_init()` emits **Plain** DCS frames, so a
command (or `$PWD`) containing `;` is truncated at the first `;` and grows phantom fields
(`ls; pwd` → Block command "ls"), and a raw ESC in the payload can corrupt the DCS scan. The decode
side already ships `AnsiCQuoted` (`q` selector; `c_unescape` handles `\n \t \r \\ \; \xHH` —
SPEC-terminal-blocks R9/R20). Switch the emit to AnsiCQuoted with an in-rc zsh escaping chain so
every command round-trips exactly. Emit-side only; unblocks demo credibility for M1.C.

## Scope
### In
- `crates/marley_app/src/shell_integration.rs` — the rc text: `q`-selector frames + a zsh
  parameter-substitution escaping chain for `$1` (preexec command) and `$PWD` (precmd), covering
  the `c_unescape` grammar's dangerous bytes (`\`, `;`, ESC, newline, tab, CR).
- Extend `crates/marley_app/tests/integration.rs` (the `#[serial]` real-zsh test) with an exact
  round-trip: a torture command containing `;` `=` quotes and spaces.
- SPEC-app-shell: add the emit-side EARS clause(s) (the emit contract has no spec home today —
  SPEC-terminal-blocks owns only decode).
- **[AMENDED at Inspect]** `crates/terminal_blocks/src/dcs.rs` — the R24 decode-order fix:
  `decode_hook`'s AnsiCQuoted path splits the STILL-ESCAPED payload on unescaped `;`/`=`
  (`split_unescaped`/`find_unescaped`, backslash-consumes-next) then un-escapes per piece. The
  inspect critics proved (against the real decoder) that the original unescape-then-split order
  nullifies ANY emit-side escaping — the ticket's "unless design finds a real gap" clause covers
  this. Hex/Plain paths byte-identical.

### Out (explicitly deferred)
- Any decode-side behavior change in `marley_terminal` (R9/R20 already ship).
- The richer M2 shell integration (git branch / venv / node fields, bash/fish rcs, subshell
  tracking beyond the existing static `bootstrapped;subshell=0`).
- Multi-line command rendering polish; per-pane sessions (seq-2); any UI change
  (`visual_acceptance: N/A` — gate-15 rides the existing `shell_dark` baseline).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — **[AMENDED at Inspect]** The emit switches to AnsiCQuoted in `marley_app` AND
  `marley_terminal::decode_hook` gains the R24 unescaped-separator split order for AnsiCQuoted
  payloads (Hex/Plain byte-identical; every pre-existing test passes with assertions unmodified).
  Original premise ("decode untouched") was empirically refuted at inspect: the decoder
  un-escaped before splitting, so no emit-side encoding could protect `;`. No `ansi_c_quote`
  encode helper (D-2.3 stands).
- D2 — Escaping runs **inside the rc as pure zsh parameter substitution** (no external commands in
  the preexec/precmd hot path). The Rust surface stays pure rc-text generation → cov 100/MSI 100.
- D3 — The emit contract gets EARS clauses **added to SPEC-app-shell** this pipeline (design
  drafts the exact text; kept behavior-only, clean-room).
- D4 — Proof = the extended real-zsh `#[serial]` integration test with EXACT `Block.command` (and
  staged prompt `pwd`) round-trip assertions, plus rc-text unit tests.
- D5 — zsh `preexec` continues to read `$1` (the line as typed) — the value the Block header shows.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `marley_zsh_init()` renders the rc, the preexec and precmd hook frames shall use the AnsiCQuoted DCS selector (`q`) and shall pass the command / pwd values through an escaping chain covering `\`, `;`, ESC, newline, tab, and CR before interpolation. | rc-text unit tests (selector byte + each escape substitution present) |
| REQ-002 | WHEN a command line containing `;`, `=`, double quotes, single quotes, and spaces is executed in the integrated shell, the finished Block shall carry the exact typed command text with no truncation and no phantom fields, and the staged prompt shall carry the exact working directory. | extended `#[serial]` real-zsh integration test — exact equality assertions |
| REQ-003 | WHEN the integrated shell starts, the init and bootstrapped frames shall still decode to `InitShell{id=1}` and `Bootstrapped{is_subshell:false}` under the encoding the rc declares for them. | rc-text unit tests + the existing integration test still segmenting |
| REQ-004 | WHILE this change ships, `marley_terminal`'s Hex and Plain decode semantics shall remain byte-identical, the AnsiCQuoted change shall be limited to the R24 unescaped-separator split order, and every pre-existing test shall pass with assertions unmodified (comment-only edits + additive tests allowed). | full workspace suite green; review of the diff |
| REQ-005 | WHEN `scripts/gates.sh --diff` runs over the staged change, every gate shall be GREEN with coverage 100% and MSI 100% on touched files, writing a commit-valid receipt. | gate exit code 0 + receipt |

## Phase Plan
- **P2 Design** — the exact zsh escaping chain (substitution order: `\` first), uniform-`q` vs
  mixed selectors for the static frames, the `ansi_c_quote` helper call, the SPEC-app-shell clause
  text, the torture-string test vector, the rc-text unit-test mutation targets.
- **P3 Implement** — rc text + tests per design; SPEC-app-shell amendment; CHANGELOG entry (§21).
- **P3.5 Inspect** — independent critics vs the diff (zsh quoting edge cases, mutation-survivability
  of the rc-text fns, decode-contract drift); fix real findings.
- **P4 Validate** — write/RUN the planned tests; `scripts/gates.sh --diff` GREEN; negative smokes.
- **P5 Complete** — architecture doc + CHANGELOG staged, AAR capture, archive, close forge #22.
