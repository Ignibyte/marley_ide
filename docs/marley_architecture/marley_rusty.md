# `marley_rusty`

> Per-crate architecture note, written 2026-10-04 at #643, extended at #644. Provenance:
> **`[Marley-original]`** (`serde` and `serde_json` only), with one port from Ely GPUI Components
> (MIT) in `vault`, its notice on the file. The design record is
> [rusty-in-marley.md](../marley/rusty-in-marley.md) (R-D1, R-D8), D8 and D11 in
> [three-prong-plan.md](../marley/three-prong-plan.md).

The pure core of Rusty in Marley: typed views of what `rusty-mcp`, Rusty's MCP server, answers,
with no gpui, no clock and no IO. The adapter, `marley_workbench::rusty`, connects to the server
through Zed's MCP client and draws. Rusty stays the store and the only writer of its data; Marley
reads and writes through Rusty's tools. Plan D8's two adapters for Rusty (`marley_rusty` for
the agent socket, `marley_brain` for the brain's tools) are this one crate, since both talk to one
program (R-D1); Rusty's agent sessions are not rebuilt (R-D6).

## Modules
- `settings` (#643): `ServerSettings`, `settings_list`'s answer (a JSON array of `{key, value}`,
  only the keys Rusty stores, a credential-like value masked by Rusty), with `get` and
  `embedding_provider`; `EmbeddingProvider { Auto, Ollama, OpenAi, Off }`, read from a stored
  value as Rusty's `semantic.rs` reads it (trimmed and lower-cased; `off`, `none` and `false` off;
  anything else, and no value, auto), written back by `as_setting`, with each provider's label and
  Rusty's words for it; the tool names `SETTINGS_LIST` and `SETTING_SET`, and
  `setting_set_arguments`.
- `vault` (#644): `VaultNode`, `brain_tree`'s nested answer (`name`, `path`, `kind`, `pages` at any
  depth, `children` folders first), with `holds` and `find`; `NodeKind { Folder, Page, File }`,
  any other kind read as `File`; `SearchHit` from `brain_search`; the slugs `brain_new_page` and
  `brain_daily_note` answer; `RenameReport` from `brain_rename`. The rail's Brain view's model:
  `rows` (the visible rows through the open folders, each with its depth and parent; the root and
  files that are not pages left out), `step` over `Key { Next, Previous, First, Last, Child,
  Parent }` giving a `Move { To, Open, Close }`, `folder_of`, `name_of`, `folders_above`,
  `child_path`, `rename_target` (one part, `/` made `-`, `None` when empty or unchanged),
  `move_target` (`"<folder>/"` or `"/"`, `None` onto itself, under itself or into its own folder)
  and `reopen` (open folders following a moved folder). `rows`, `step`, `holds` and the drop guard
  are ported from Ely's `src/lists/tree/model.rs` at `2f8b2f6`, on vault paths in place of Ely's
  keys; Ely's rendering, lazy children, checkboxes and before/after drop places are not taken,
  since Zed's `ui` draws the tree. The tool names `BRAIN_*`, `VAULT_PATH_KEY` (`brain_vault_path`)
  and `SEARCH_LIMIT`.

## Fixtures and the stand-in
- `fixtures/settings_list.json`: an answer in `rusty-mcp`'s shape, neutral values.
- `stand_in/rusty-mcp`: a Python program on the standard library that answers as `rusty-mcp` does
  for the tools Marley reads, over a scratch folder (`$RUSTY_STAND_IN_STATE`): stdio with no
  arguments, or `--http ADDR` (port 0 picks one, written to `http-addr`). It logs every request with
  its pid in `calls`, seeds `settings.json` from the fixture, and on stdio sends
  `notifications/resources/list_changed` when that file changes. Since #644 it also serves the
  vault tools (`brain_tree`, `brain_search`, `brain_daily_note`, `brain_new_page`,
  `brain_new_folder`, `brain_rename`, `brain_delete_page`, `brain_delete_folder`) over a scratch
  vault at `$RUSTY_STAND_IN_STATE/vault`, made with `archive/` and stored as `brain_vault_path`,
  with Rusty's answers and refusals as of Rusty's TICKET-040 (the root's `archive/` left out of the
  tree and the search, writes into it refused); it announces a change to any file in that vault
  too, as Rusty's watcher does. It rewrites no links and matches search words, not embeddings. The e2e
  scenarios name it with `MARLEY_RUSTY_MCP` and never reach the user's own Rusty (R-D8).

## Later slices
#645 adds the page render's fields and the markdown pass, #646 the link and search views, #647
the graph and its layout.
