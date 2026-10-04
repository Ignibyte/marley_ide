# `marley_rusty`

> Per-crate architecture note, written 2026-10-04 at #643. Provenance: **`[Marley-original]`**
> (`serde` and `serde_json` only). The design record is
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

## Fixtures and the stand-in
- `fixtures/settings_list.json`: an answer in `rusty-mcp`'s shape, neutral values.
- `stand_in/rusty-mcp`: a Python program on the standard library that answers as `rusty-mcp` does
  for the tools Marley reads, over a scratch folder (`$RUSTY_STAND_IN_STATE`): stdio with no
  arguments, or `--http ADDR` (port 0 picks one, written to `http-addr`). It logs every request with
  its pid in `calls`, seeds `settings.json` from the fixture, and on stdio sends
  `notifications/resources/list_changed` when that file changes. The e2e scenarios name it with
  `MARLEY_RUSTY_MCP` and never reach the user's own Rusty (R-D8).

## Later slices
#644 adds the vault tree's view and a fixture vault, #645 the page render's fields and the
markdown pass, #646 the link and search views, #647 the graph and its layout.
