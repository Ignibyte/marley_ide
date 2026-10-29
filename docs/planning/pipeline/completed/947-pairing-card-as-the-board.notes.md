# The pairing card as the AgentAccess board draws it — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-947-pairing-card-as-the-board.md
- **Pipeline spec:** 947-pairing-card-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-29: "lets do 795, 794, 947 then clean up all your builds etc and
  reinstall".
- **Classification / tier:** feature, M. Marley crates and the plugin's bridge; no Zed hunk.
- **Recall (§18.3):**
  - #811: pairing through `client_pair` and `client_pair_status`; the code is "ddd ddd"; the wait
    is `PAIR_WAIT`, 120 s; the bridge prints "Check that Marley shows the code …".
  - #520: the bridge's caller headers are labels, never an authority; `Marley-Via` joins them.
  - #938 D6 split this card off; #871 made `Ask`.
  - The brain: consultation 242beb88ba1c4bc7921639851f6ebdb3; nothing on this seam.

### Design
- **Bridge** (`claude_plugin/marley/bin/marley-mcp-bridge`, `caller_headers`): `Marley-Via: ssh` when
  `SSH_CONNECTION` is set and not empty.
- **`marley_mcp`** (`marley_mcp.rs`): `pub enum Reach { #[default] Local, Ssh }`, `Caller.reach`.
  `transport.rs`: `header_value(line, "marley-via")` → `Reach::Ssh` for `ssh`, else `Local`.
- **`asks.rs`:** `Ask::code(code)` draws each digit in a bordered box (a space keeps a gap);
  `Ask::choices(Choices { items: Vec<Choice { label, line }>, chosen: Rc<Cell<usize>> })` draws a
  radio row per item, a filled accent dot for the chosen one; a click sets the cell and
  `window.refresh()`.
- **`clients.rs`:** `PairQuestion.reach`; `show_pairing` builds the card: title, the from-line
  (`compact` folder, " over SSH" for `Reach::Ssh`; no folder: the check sentence alone), the
  code, the three grants (`GRANTS`: power, label, line), Refuse (Quiet) and Pair (Primary), which
  reads the cell; footnote "No answer in two minutes pairs nothing."
- **File manifest:** `crates/marley_mcp/src/marley_mcp.rs`, `crates/marley_mcp/src/transport.rs`,
  `crates/marley_workbench/src/asks.rs`, `crates/marley_workbench/src/clients.rs`,
  `crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge` (all Marley-owned),
  `script/e2e/947-pairing-card-as-the-board.sh`.

### Visual check plan
- REQ-001, REQ-002, REQ-004: setup as #811's; `pair laptop` from the harness; `947-01-local`.
- REQ-003: click Read and Write's row; `947-02-picked`; click Pair; the command's output holds
  "Paired: laptop may read and write."
- REQ-002: `SSH_CONNECTION='127.0.0.1 50000 127.0.0.1 22'` on `pair laptop2`; `947-03-ssh`; Refuse.
- Day: `profile_setting theme '"Marley Day"'`, `pair laptop3`; `947-04-day`; Refuse.

### Risks
- A local process can set `SSH_CONNECTION` and be called "over SSH"; that is the label's limit,
  said in `Caller`'s doc.
- `window.refresh()` redraws the whole window on a pick; a pick is a click, not a stream.

## Phase 2 — Code
- **Built:**
  - `marley-mcp-bridge`: `caller_headers` adds `Marley-Via: ssh` when `SSH_CONNECTION` is set;
    `pair` now ends its first line "then pick what it may do and press Pair."
  - `marley_mcp`: `Reach { Local, Ssh }` and `Caller.reach`; `transport` reads `marley-via`.
  - `asks.rs`: `Choice`, `Choices`, `Ask::code` (`render_code`, a 28×34 box per digit, a gap for the
    space) and `Ask::choices` (`render_choices`, a radio row per item; a click sets the cell and
    `window.refresh()`).
  - `clients.rs`: `PairQuestion.reach`, `Grant` and `GRANTS`, and `show_pairing` drawn as the board
    with Refuse (Quiet) and Pair (Primary), which reads the pick.
  - `docs/marley/guide.md`: the pairing steps say what the card shows and that Pair confirms.
- **Deviations:** none from the design. The guide's wording was not in the manifest; it described
  the old notification.
- **Review:** the pick starts at 0, Read, and Pair falls back to Read if the index were ever out of
  range (`GRANTS.get`), so no input panics. Each window builds its own card and cell; the slot gives
  the first answer only, as before. `Reach` is an enum with its doc saying it grants nothing.
  Style: one module per `use`, `PathExt` imported as a trait, a named struct for each grant.
- **Gate:** the first run failed clippy's `doc_markdown` on "AgentAccess" in two doc comments; put
  in backticks; the second run was green, 17 passed.

## Phase 3 — Test
- **Scenario:** `script/e2e/947-pairing-card-as-the-board.sh` under sway, after `just build`; the
  first run (`STOP_AT=1`) measured the card's rows and buttons.
- **Shots:**
  - `947-01-local` (REQ-001, REQ-002, REQ-004): "“laptop” asks to pair with Marley"; "From <the
    repository's folder>. Check the code matches what the command printed."; the code the command
    printed (`393 751`) in six boxes with a gap; Read picked (accent dot, filled row), Read and
    Write and Read, Write and Run each with its line; "No answer in two minutes pairs nothing.";
    Refuse and Pair.
  - `947-02-picked` (REQ-003): Read and Write's dot filled and its row tinted, Read's empty. After
    Pair the command printed "Paired: laptop may read and write." (`expect` passed).
  - `947-03-ssh` (REQ-002): "“laptop2” … From <folder> over SSH. Check the code …", with Read picked
    again on a new card.
  - `947-04-day` (REQ-001): the card in Marley Day: boxes, dots and the brown Pair legible.
- The folder is the scenario's scratch path, which `compact` leaves whole; a folder under the home
  shows as `~/…`.
- Nothing pre-existing failed.

## Phase 4 — Complete
- Documented: `CHANGELOG.md` (Added), `docs/marley/look-plan.md` (Wave 6), the architecture notes
  for `marley_workbench` and `marley_mcp`, `docs/marley/guide.md`. No Zed path.
- Knowledge: none new; the card follows #871's and #938's.
- Brain: consultation 242beb88ba1c4bc7921639851f6ebdb3 closed with no decision (D1–D3 in the spec).
- Ticket closed; pipeline archived.
