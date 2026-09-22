# command-signatures-v2

> Per-crate reference (Marley round 2). Crate dir: `crates/command-signatures-v2`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance:** `[Warp-derived: AGPL-3.0]` shim embedding a Warp JS completions asset (also Warp-owned). **Not** on the spawn/read/write path; Marley drops it (its own completions) — not carried. See subsystem [Provenance & licensing](../subsystems/03-terminal-session-core.md#provenance--licensing).

| | |
|---|---|
| Subsystem | [terminal-session-core](../subsystems/03-terminal-session-core.md) |
| License | AGPL v3 (inherits workspace `license = "AGPL-3.0-only"`; no own LICENSE marker) |
| Internal-deps count | 1 |
| Used-by count | 1 |

## Purpose

`command-signatures-v2` is a thin Rust shim that **embeds a compiled TypeScript/JavaScript bundle** into the Warp binary. The JS bundle (`js/`) is the built-in "Completions" plugin: it ships *command signatures* — declarative descriptions of CLI tools (subcommands, options, arguments, value suggestions) that drive Warp's autocomplete. The Rust crate exists purely so the rest of the app can pull those bytes out of the binary at runtime via `rust-embed` rather than reading files off disk.

The "v2" name reflects that completions moved from a native Rust engine to a JS-plugin model (the bundle is loaded into the plugin host as a built-in plugin).

## Key types, modules & public API

Everything lives in `src/lib.rs` (six lines of real code):

- `pub struct CommandSignaturesJs` — a `#[derive(RustEmbed)]` type with `#[folder = "js/build"]`. The derive generates `CommandSignaturesJs::get(path: &str) -> Option<EmbeddedFile>` and an iterator over embedded files.
- `pub static COMMAND_SIGNATURES_JS: CommandSignaturesJs` — a convenience singleton.

The only consumer call in the codebase is `command_signatures_v2::CommandSignaturesJs::get("main.js")` (see `app/src/plugin/host/native/plugin_ref.rs`, in `BuiltInPluginType::Completions::plugin_bytes`).

### Build pipeline (`build.rs`)

`build.rs` shells out to `yarn build` (which runs `tsc -p tsconfig.json`) inside `js/`, producing `js/build/main.js`, which `rust-embed` then bakes in. Source of truth:

- `js/src/main.ts` — the plugin entrypoint; exports `activate(warp: Warp)` and calls `warp.completions.registerCommandSignature({...})`.
- `js/src/types/warp.d.ts`, `js/src/types/command-signature.d.ts` — the ambient TS interfaces (`Warp`, `Completions.registerCommandSignature`, the `CommandSignature` shape).

If `yarn build` fails but a stale `js/build` exists, the build emits a `cargo:warning` and proceeds with the stale bundle; if no build dir exists at all it `panic!`s with Node/yarn/corepack remediation instructions.

## Depends on (internal)

- [`command`](./command.md) — **build-dependency only.** `build.rs` uses `command::blocking::Command` to invoke `yarn build`. Not a runtime dependency.

## Used by (internal dependents)

- [`warp`](./warp.md) (the `app` crate) — declared `optional = true` and gated behind the `completions_v2` feature (`dep:command-signatures-v2`). Consumed in `app/src/plugin/host/native/plugin_ref.rs`.

## Related crates

- `warp-command-signatures` *(external / not a workspace crate)* — the *other* signatures crate `app` depends on (`features = ["embed-signatures"]`); read alongside to disentangle v1/native signatures vs. this JS-plugin v2 path.
- The plugin host machinery under `app/src/plugin/host/native/` and the [`ipc`](./ipc.md) crate that wires the host to the app.

## Marley relevance

**Classify: KEEP (low priority) — likely STUB/REMOVE if completions are cut.**

This crate touches none of the four Marley goals directly. It is autocomplete data, not session spawn/read/write, not auth, not UI surface. Relevance:

- It is **already optional** (gated behind `completions_v2`); for an early offline Marley boot we can simply leave the feature off — `BuiltInPluginType::Completions::plugin_bytes` returns `None` under `#[cfg(not(feature = "completions_v2"))]`, so nothing breaks. That is the cheapest "stub".
- The embedded JS contains the literal string "Warp" and a sample `jack` command; for the **de-Warp rebrand** goal the user-facing bits are trivial, but the bundle is generated TS, so rebrand happens in `js/src/`, not in Rust.
- **Build-quirk risk:** it requires Node 18.x + corepack + yarn 4 (PnP) at compile time. For Marley's CI/dev ergonomics this is the single most annoying thing in the crate. If we don't ship JS completions on day one, prefer **REMOVE the build.rs yarn step** (commit a prebuilt or empty `js/build`) so the workspace builds without a Node toolchain.

No rename urgency: package name is generic (`command-signatures-v2`), not "warp_*".

## Notes / gotchas

- **Out-of-repo toolchain:** compiling from clean requires Node/yarn; `js/.pnp.cjs` (~414 KB) and `js/.yarn/` are Yarn Plug'n'Play artifacts checked into the tree.
- The embedded folder is `js/build` (the *output*), which may not exist until `build.rs` runs — clean checkouts depend on the build step succeeding or a stale build being present.
- `rust-embed` in release mode bakes bytes into the binary; in debug it may read from disk depending on features — confirm behavior before relying on hot-reload.
- Despite the `command` build-dep, there is **no runtime coupling** to the `command` crate — don't infer a terminal-execution relationship.
