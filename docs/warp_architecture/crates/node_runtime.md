# node_runtime

> Per-crate reference (Marley round 2) — crate dir `crates/node_runtime`. Marley is Ignibyte's fork of Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[permissive: standard deps]` — pinned Node/npm installer; standard concept. Gap. See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| Field | Value |
|---|---|
| Subsystem | [platform-infra](../subsystems/06-platform-settings-infra.md) |
| License | AGPL v3 (no in-crate LICENSE marker; inherits workspace `license`) |
| Internal deps | 3 (`command`, `http_client`, `warp_core`) |
| Used by | 1 (`lsp`) |

## Purpose

Bootstraps a usable **Node.js / npm runtime** for Warp. It downloads a pinned Node release, extracts it into Warp's data dir, validates it, and otherwise falls back to a recent-enough system Node — plus a couple of npm-registry helpers. It exists so features that need Node (notably Language Servers via the [`lsp`](./lsp.md) crate, which often ship as npm packages) have a guaranteed interpreter without making the user install one.

## Key types, modules & public API

A single-file crate (`src/lib.rs`). The heavy filesystem/download logic is gated behind the **`local_fs`** cargo feature and `#[cfg(not(target_arch = "wasm32"))]`; only the npm-registry query is always available.

Always available:
- `pub struct NpmInfo` / `pub struct NpmInfoDistTags` with `NpmInfo::latest_version(&self) -> Option<&str>` — deserialized npm registry metadata.
- `pub async fn fetch_npm_package_version(client: &http_client::Client, package_name: &str) -> Result<String>` — hits `https://registry.npmjs.org/<pkg>`.

Behind `feature = "local_fs"`:
- `pub async fn install_npm(client: &http_client::Client) -> Result<PathBuf>` — checks for a valid existing install, else downloads `NODE_VERSION` (`v22.12.0`) from `https://nodejs.org/dist/...`, wipes any corrupt prior dir, and extracts under `warp_core::paths::data_dir().join("node")`.
- `pub async fn find_working_node_binary(path_env_var: Option<&str>) -> Option<PathBuf>` — prefers the custom install, falls back to system `node`.
- `pub async fn detect_system_node(path_env_var: impl AsRef<OsStr>) -> Result<()>` — runs `node --version` (via `cmd.exe /c` on Windows for correct PATH resolution) and enforces `MIN_NODE_VERSION` (`20.0.0`).
- Path helpers: `node_installation_dir()`, `node_binary_path()`, `npm_binary_path()`.
- Archive helpers: `extract_tar_gz`, `extract_gz`, `extract_zip<F>` (filtered), plus `pub enum ArchiveType { TarGz, Zip }` and the private `NodeDistribution` (maps `std::env::consts::OS`/`ARCH` to Node's `darwin`/`linux`/`win` × `x64`/`arm64` naming).

## Depends on (internal)

- [`command`](./command.md) — `command::r#async::Command` to run `node --version` for validation (non-wasm only).
- [`http_client`](./http_client.md) — `http_client::Client` for downloading the Node tarball and querying the npm registry.
- [`warp_core`](./warp_core.md) — `warp_core::paths::data_dir()` for the install location.

## Used by (internal dependents)

- [`lsp`](./lsp.md) — language-server manager; uses `node_runtime` to guarantee a Node/npm interpreter for npm-distributed language servers.

## Related crates

- [`http_client`](./http_client.md) — the download transport.
- [`command`](./command.md) — subprocess execution.
- [`asset_cache`](./asset_cache.md) — sibling "download + persist to data dir" pattern in this subsystem.

## Marley relevance

**Classification: KEEP (defer rename).** This is functional, brand-light infrastructure (the only Warp coupling is `warp_core::paths::data_dir()` and the literal string `"node"` subdir). It does not touch auth, the UI surface, or session I/O, so goals (1)–(4) leave it alone. We keep it because the `lsp` crate depends on it; ripping it out would cascade. If we want a Marley-owned data dir for goal (4) de-Warp rebrand, that change belongs in `warp_core::paths`, not here — this crate just inherits it. The endpoints it talks to (`nodejs.org`, `registry.npmjs.org`) are neutral and need no de-Warp work. Only real decision: whether Marley ships LSP at all; if not, this can ride along unused (it's already feature-gated) or be dropped with `lsp`.

## Notes / gotchas

- **Pinned version:** `NODE_VERSION = "v22.12.0"`; `MIN_NODE_VERSION = 20.0.0`. Bumping Node = editing the const.
- **Feature gate:** without `local_fs`, only the npm-registry query compiles — the install/extract path silently disappears. `extract_zip` uses sync `zip` reading then async `async_fs` writes.
- **Network at runtime:** `install_npm` downloads ~tens of MB from `nodejs.org` on first run; pair with [`prevent_sleep`](./prevent_sleep.md) for long fetches.
- Windows version detection deliberately uses `cmd.exe /c node` so the *captured* PATH (not Warp's inherited env) resolves `node.exe` — see the inline comment in `detect_system_node`.
- `edition = "2021"` here (most siblings use 2024); has a no-op `build.rs` (only `anyhow` build-dep).
