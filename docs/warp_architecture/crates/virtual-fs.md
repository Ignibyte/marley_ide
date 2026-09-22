# virtual-fs

> Per-crate reference (Marley round 2) — crate dir `crates/virtual_fs` (package name `virtual-fs`). Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[permissive: standard deps]` — test-only in-memory VFS over `tempfile`; Marley tests use `tempfile` directly. Gap / mostly N/A. See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| | |
|---|---|
| **Subsystem** | [platform-settings-infra](../subsystems/06-platform-settings-infra.md) |
| **License** | AGPL v3 (`license.workspace = true` → `AGPL-3.0-only`; no per-crate marker) |
| **Internal deps** | 0 |
| **Used by** | 4 |

## Purpose

`virtual-fs` is a small **test-fixture filesystem builder**. It creates a throwaway
temp directory and gives tests a fluent API to scaffold a controlled on-disk layout
(directories, files with content, symlinks, mock executables) so that filesystem-
touching code — path resolution, repo metadata detection, file watching — can be
exercised against real files without polluting the developer's working tree.

Despite the generic name it is **not** a runtime VFS abstraction; it is a testing
utility (the crate doc literally says "A virtual filesystem for testing purposes").

## Key types, modules & public API

Everything is in `src/lib.rs`:

- **`struct VirtualFS`** (`#[derive(Getters)]`, getters via the `getset` crate) —
  fields `root: TempDir`, `cwd: PathBuf`, `tests: String`. Entry point:
  - `VirtualFS::test(tag: &str, test_callback: impl FnOnce(Dirs, VirtualFS))` —
    creates a `tempdir()`, makes a `tag`-named subdir, canonicalizes it (via
    `dunce::canonicalize` for clean Windows paths), and invokes the callback.
  - Fluent builders (all return `&mut Self`): `mkdir(dir)`, `with_files(Vec<Stub>)`,
    `touch(Vec<Stub>)`, `back_to_root()`, and `ln(target, link)` (Unix-only symlink).
- **`enum Stub<'a>`** — fixture file specs: `FileWithContent(name, content)`,
  `FileWithContentToBeTrimmed(name, content)` (strips/trims lines),
  `EmptyFile(name)`, and `#[cfg(unix)] MockExecutable(name)` (written with mode
  `0o755`).
- **`struct Dirs`** (`#[derive(Default, Getters, Clone)]`) — `root` and `tests`
  paths handed to the callback; `git_repository_fixture()` resolves a bundled git
  fixture.
- **`struct Warp`** — path helpers for the harness: `Warp::root()` (walks up from
  `CARGO_MANIFEST_DIR` looking for `Cargo.lock`), `Warp::executable()` (locates the
  built `warp` binary under `target/{debug,release}`), `Warp::fixtures()`
  (`tests/fixtures`).

External deps: `tempfile`, `getset`, `typed-path`, `dunce`.

## Depends on (internal)

- *(none)* — leaf crate; only third-party deps.

## Used by (internal dependents)

- [warp](./warp.md) — filesystem-dependent tests.
- [ai](./ai.md) — tests over file/workspace fixtures.
- [repo_metadata](./repo_metadata.md) — git/repo detection tests (uses
  `Dirs::git_repository_fixture()`).
- [warpui](./warpui.md) — UI-layer tests needing a scratch FS.

Total: 4 dependents.

## Related crates

- [persistence](./persistence.md) — the *real* durable state layer (this is only its
  test-side cousin in the same subsystem doc).
- [repo_metadata](./repo_metadata.md) — the most natural consumer (repo fixtures).

## Marley relevance

**Classification: KEEP, RENAME the embedded `Warp` helper (cosmetic, low-risk).**

A dev-only test utility — no auth, no network, no product surface. It is irrelevant
to goals (1)–(3) directly but matters for goal (4) **de-Warp rebrand** in one spot:
the public `struct Warp` and its methods (`Warp::executable()` pushes the literal
binary name `"warp"`). After Marley renames the final binary, update
`Warp::executable()` (and ideally rename the helper to `Marley`) so fixture-based
tests still find the built binary. That binary-name string is the only hard "warp"
coupling.

Do not rename the crate's package or restructure it — 4 dependents reference
`VirtualFS`/`Stub`/`Dirs` by path, and it carries no brand in those names.

## Notes / gotchas

- Package name is `virtual-fs` (hyphen) but the directory is `crates/virtual_fs`
  (underscore) — link/import as the crate name `virtual_fs`. The doc file is named
  by the **package** (`virtual-fs.md`).
- API is panic-on-error by design (`.expect(...)` everywhere) — appropriate for
  tests, **not** for production use. Don't repurpose it as a runtime FS.
- `Warp::executable()` hard-codes the binary name `"warp"` and respects
  `CARGO_TARGET_DIR` — a Marley binary rename must be reflected here.
- Several items are `#[allow(dead_code)]` (e.g. `MockExecutable`,
  `git_repository_fixture`, `Warp::executable`) since usage is conditional across
  the dependent test suites.
- `edition = "2021"`, pinned `version = "0.1.0"`, and the only crate in this batch
  whose Cargo.toml still carries an explicit `authors = ["Warp Team <dev@warp.dev>"]`
  line (rebrand candidate).
