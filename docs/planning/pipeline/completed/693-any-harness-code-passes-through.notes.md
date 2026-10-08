# Any harness code passes through — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-693-any-harness-code-passes-through.md
- **Pipeline spec:** 693-any-harness-code-passes-through.spec.md

## Phase 1 — Plan
- **Request:** rustal-harness's message, 2026-10-07.
  - "please don't pass through a fixed list of twelve. Take the token between 'rh: ' and the next
    ':' as the code when it matches [a-z_]+, and show the rest as given. Treat exit 2 as a form or
    usage bug on your side."
  - Its set: thirteen seat codes, with `seat_surface` the one #692 lacked, plus the runtime's own
    names through `seat start`.
- **Recall:** #692's `HARNESS_CODES`, a fixed list forced by `Refusal.code: &'static str`.
- **Design:**
  - `marley_mcp.rs`: `code: Cow<'static, str>`, and `new` takes `impl Into<Cow<'static, str>>`.
    `dispatch.rs`'s struct update needs nothing.
  - `harness_seat.rs`: `refusal_of` parses the code, `run_seat` handles exit 2, and
    `HARNESS_CODES` goes.
  - **Manifest:** those two files, plus the docs.
- **Visual check plan:** `just shot` (no UI change), then #692's scenario, whose
  `seat_role_reserved` check now goes through the parser.

## Phase 2 — Code
- **Built:**
  - `Refusal.code: Cow<'static, str>`, with `Refusal::new(impl Into<Cow<'static, str>>, …)`.
    Every literal call compiles unchanged (`cargo check --tests` on both crates).
  - `refusal_of` takes any `[a-z_]+` code, and `HARNESS_CODES` is gone.
  - `run_seat` turns exit 2 into "Marley called the harness wrongly (a usage error): <first line>".
- **Review:**
  - A runtime line such as "failed to connect to …: …" has spaces before its first `: `, so it
    stays `refused` with all of it.
  - `tools::tool_refusal` serializes a `Cow` as it did the `&str`.
- **Gate:** `scratchpad/693-gate-1.log`: **GATE GREEN [diff]**.

## Phase 3 — Test
- **#692's scenario rerun** (`scratchpad/693-e2e-692.log`): 3 of 3. Its refusal now goes through
  the new parser and still reads `"code":"seat_role_reserved"`.
- **`just shot`** (`COMPOSITOR=sway`, the Hyprland runner refuses beside an open Marley, L-681):
  `693-no-ui-delta`, Marley started on its scratch repo with Zed's trust question. Nothing on
  screen changed.
- **Not reached by a scenario:** a code outside the old list, `seat_surface` or a runtime
  `harness_*`. The parser takes any `[a-z_]+` token, which the review covers. Exit 2 needs a
  harness argument error Marley does not make.

## Phase 4 — Complete
- **Docs:**
  - `CHANGELOG.md` (Fixed, #693).
  - `docs/marley_architecture/marley_mcp.md` (`Refusal`'s code).
  - `marley_workbench.md` (`refusal_of`).
- **Knowledge:** `F-claude-693-a-fixed-list-of-another-programs-codes-went-stale-001`, with
  `PR-claude-693-pass-another-programs-codes-by-their-form-not-a-list-001`.
- **Ticket:** TICKET-693 closed.
