# warp_ripgrep

> Per-crate reference (Marley round 2) for `crates/warp_ripgrep`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[permissive: standard deps]` — thin ripgrep wrapper. Gap. See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| | |
|---|---|
| **Subsystem** | [06 — Platform, Settings, Persistence & Infra](../subsystems/06-platform-settings-infra.md) |
| **License** | AGPL v3 (workspace `license`; no per-crate LICENSE marker) |
| **Internal deps** | 3 |
| **Used by** | 1 |

## Purpose

A **thin wrapper around ripgrep** (the `grep`/`ignore` crates) for full-text file search. It
provides two execution halves: an **in-process search engine** that walks a directory tree
in parallel, matches regex patterns, and prints JSON results to stdout; and an **async
client** that spawns the Warp CLI as a child process (`ripgrep-search` subcommand), feeds it
patterns/paths, and parses the JSON back into typed `Match` values — as a one-shot
collection or a live `Stream`. Running search out-of-process keeps a heavy/parallel walk
from blocking or crashing the main app, and the child self-terminates if the parent dies.

## Key types, modules & public API

- **`src/lib.rs`**
  - `monitor_parent_and_exit_on_change(parent_pid: Option<u32>)` (Unix only) — spawns a
    watchdog thread that calls `std::process::exit(0)` once the worker is reparented (parent
    died). Cited at `search.rs` start-of-subprocess.
- **`src/search.rs`**
  - `Match { file_path: PathBuf, line_number: u32, line_text: String, submatches: Vec<Submatch> }`
    and `Submatch { byte_start: ByteOffset, byte_end: ByteOffset }` — the result types.
  - `run_search_subprocess(patterns, paths, ignore_case, multiline, parent_pid) -> anyhow::Result<()>`
    — the **worker entry point**: builds a `RegexMatcherBuilder` (multi-pattern via
    `build_many`), a parallel `ignore::WalkBuilder`, a `SearcherBuilder` with
    `BinaryDetection::quit` and a 64 KiB per-line heap limit (`SEARCHER_LINE_HEAP_LIMIT`,
    single-line mode only), and writes JSON to a mutex-guarded stdout.
  - `mod process_impl` (native only) — the **client API**: `async fn search(...) -> Result<Vec<Match>>`
    and `fn search_streaming(...) -> Result<impl Stream<Item = Match>>` (the preferred entry
    point for responsive UI); internal `spawn_search_process` / `match_stream_from_child`.
- **`src/types.rs`** (native only, `pub(crate)`) — serde deserialization types for ripgrep's
  JSON message stream: `RipgrepMessage` (`begin`/`match`/`end`), `RipgrepMatchData`,
  `RipgrepSubmatch`, etc.

## Depends on (internal)

- [./warp_cli.md](./warp_cli.md) — provides `ripgrep_search_subcommand()` and `parent_flag()`, the CLI surface the client spawns the worker through.
- [./command.md](./command.md) — `command::r#async::Command`, the process-spawning abstraction (native only).
- [./string-offset.md](./string-offset.md) — `ByteOffset`, the typed byte offset used in `Submatch` and JSON parsing.

## Used by (internal dependents)

- [./warp.md](./warp.md) — the main binary; drives the global file-content search UI.

Total: **1** dependent.

## Related crates

- [./warp_search_core.md](./warp_search_core.md) — higher-level search orchestration; ripgrep is one backend.
- [./repo_metadata.md](./repo_metadata.md) — provides the repo/file tree the search runs over (path indexing vs content search).
- [./warp_cli.md](./warp_cli.md) — defines the subcommand this crate re-enters the binary through.

## Marley relevance

**KEEP.** Local, offline, no-auth, no-cloud file search — exactly the kind of capability a
Marley custom panel (goal 1) wants (a "search in project" panel calls `search_streaming`
directly). It has **no login or telemetry coupling**, so it is not a de-auth concern (goal
3). The only rebrand (goal 4) touchpoints are: the package name `warp_ripgrep` and its
dependency on `warp_cli`'s `ripgrep_search_subcommand()` / `parent_flag()` — because the
client **re-execs the host binary** (`std::env::current_exe()`), if Marley renames the main
binary the subcommand wiring in `warp_cli` must stay consistent (the current-exe approach is
name-agnostic, which helps). Rename `warp_ripgrep` → `marley_ripgrep` during the broad
`warp_*` sweep; it has only 1 dependent so the rename is cheap, but there's no urgency.

## Notes / gotchas

- **Re-exec architecture**: the async client spawns `std::env::current_exe()` with `warp_cli::ripgrep_search_subcommand()` — i.e. the app re-runs *itself* in search-worker mode. The main binary must dispatch that subcommand to `run_search_subprocess`, or search silently does nothing.
- **Parent-death watchdog is Unix-only** (`nix::unistd::Pid`); on non-Unix the `parent_pid` arg is accepted but unused (`#[cfg_attr(not(unix), allow(unused_variables))]`).
- **Line heap limit (64 KiB) only applies in single-line mode** — multiline search needs the whole file in memory, so a giant minified file can blow memory in `--multiline`.
- **WASM target** pulls `getrandom` with the `js` feature but the real search (`process_impl`, `types`) is `cfg(not(wasm))`; WASM only gets the in-process `run_search_subprocess` shape, not the spawning client.
- Binary content is skipped via `BinaryDetection::quit(b'\x00')`.
