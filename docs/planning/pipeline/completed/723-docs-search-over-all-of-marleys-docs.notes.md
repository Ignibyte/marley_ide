# Docs search over all of Marley's docs — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-723-docs-search-over-all-of-marleys-docs.md
- **Pipeline spec:** 723-docs-search-over-all-of-marleys-docs.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-09: a document search on the Marley MCP over Zed's and Marley's docs.
- **Recall:**
  - #681 built `docs_search`/`docs_read` over `docs/src` and `docs/marley/guide.md`, with word
    scoring by held words, then hits, a heading hit counting five.
  - The tools are served to every client (`Family::Docs` in `is_served`) and are in the Marley
    agent's eight.
  - A repo-root embed walks about 7,000 files, which is cheap.
- **Design:**
  - **`docs_tools.rs`:**
    - The bundle's includes gain `marley/walkthrough.md`.
    - `RootBundle` (`crate_relative = "../../.."`, `root_relative = "."`, `include =
      ["CHANGELOG.md"]`) feeds `build_index` as `marley/CHANGELOG.md`.
    - `split_page` takes a `bullets` flag, set for the changelog.
    - `stem(word)` and `search` match a word or its stem.
  - **`registry.rs`:** the two descriptions name the walkthrough and the changelog.
  - **`assistant.rs`:** the instructions' docs line adds "the changelog says what changed and
    when".
  - **`guide.md`:** the docs tools' lines.
- **File manifest:** Marley crates only (`marley_workbench`, `marley_mcp`); docs.
- **Visual check plan:** the client's four calls; one shot.

## Phase 2 — Code
- **Built:**
  - **`docs_tools.rs`:**
    - The bundle includes `marley/walkthrough.md`.
    - `RootBundle` serves `CHANGELOG.md` as `marley/CHANGELOG.md`.
    - `split_page` takes `bullets` (true for the changelog), and `bullet_heading` heads each entry
      by its bold title, else its first eight words.
    - `stem` and `search` count the word, else its stem; the snippet looks for either.
  - **The text around the tools:** the registry's two descriptions, the Marley agent's
    instructions, guide.md's two tool rows (and a line rewrapped), and the HTML guide's
    `docs_search` row.
- **Deviations:**
  - `RootBundle`'s `crate_relative` is `../..`, not `../../..`: the docs bundle's `../../docs`
    puts the repository root two levels up.
  - The stem rules were refined during review: "-es" comes off only after s, x, z, ch or sh, so
    "files" stems to "file", not "fil".
- **Review:**
  - An exact word scores as before; a stem counts only where the word has no hit, so ranking for
    queries that already worked is unchanged.
  - In a dev build the root bundle walks the checkout (about 7,000 files) once, on first use, off
    the main thread like the rest of the index.
- **Gate:** `723-gate-1.log` GATE GREEN [diff].

## Phase 3 — Test
- **Scenario:** `script/e2e/723-docs-search-over-all-of-marleys-docs.sh`, under `compositor sway`,
  `nice -n 19` while the harness's gate ran. #704's client is rewritten for `docs_search` and
  `docs_read`.
- **First run (`shots-723a`): every check passes.**
  - **"kill switch agent activity"** (REQ-001): `marley/guide.md | Agent activity and the kill
    switch | words=4`, then `marley/CHANGELOG.md | Agent activity and a kill switch for Marley's
    tools | words=4`. The changelog entry is a section of its own.
  - **"practice projects"** (REQ-001): `marley/walkthrough.md | 1.2 Open the practice project`
    and `| 0.3 Make the practice projects` lead.
  - **"agent activities"** (REQ-002): "activities" is in no doc (grep: 0), yet the top hits hold
    words=2 through the stem.
  - **`docs_read marley/CHANGELOG.md`** with the entry's heading (REQ-003): the entry from its bold
    title through its bullets, "(#703, 2026-10-09)".
  - **723-01-marley:** the run's Marley, the scratch project open, the one that answered.
- Chad's Hyprland untouched.

## Phase 4 — Complete
- **Documented:**
  - `CHANGELOG.md` (Added);
  - `docs/marley_architecture/marley_workbench.md` (#723's lines under the docs tools);
  - the guide's two rows and the HTML guide's row (Phase 2).
- **Knowledge appended:** AD-claude-723-agents-search-marleys-user-docs-not-its-design-record-001.
  No bug was found.
- **Brain:** no `rusty` MCP server in this repository's sessions.
- **Ticket:** closed; it never had a BACKLOG row.
- **Gate:** `723-gate-2.log` RED (typos: the scenario spelled out the stem "activit"; reworded); `723-gate-3.log` GATE GREEN [diff], on the tree committed.
