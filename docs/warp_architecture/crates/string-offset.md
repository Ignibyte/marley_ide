# string-offset

> Per-crate reference (Marley round 2). Crate dir: `crates/string-offset`. Marley is forked from Warp (warpdotdev/warp).

| | |
|---|---|
| Subsystem | [editor-and-text](../subsystems/02-editor-and-text.md) |
| License | AGPL v3 (`AGPL-3.0-only`, inherited from workspace; no per-crate `LICENSE`) |
| Internal deps | 0 |
| Used by | 12 |
| Provenance | Trivial type-safety newtypes → concept **`[permissive]`**. Marley reimplemented as **`marley_text_offsets`** (**`[Marley-original]`**, MIT/Apache). See [subsystem — Provenance & licensing](../subsystems/02-editor-and-text.md). |

## Purpose

`string-offset` is the tiny, foundational newtype crate that gives the whole codebase **type-safe text offsets**: it distinguishes a position measured in Unicode `char`s from one measured in bytes, so the two can never be silently mixed. It solves the perennial editor bug class of byte-vs-char index confusion. Because it is dependency-free (internally) and `serde`/`get-size`-friendly, it is the lowest common vocabulary shared by the editor, terminal, search, completer, AI, and vim layers.

## Key types, modules & public API

All in `crates/string-offset/src/lib.rs`.

- **`struct CharOffset(usize)`** — an offset counted in Rust `char`s. Derives `Copy, Ord, Hash, Serialize, Deserialize, GetSize`.
- **`struct ByteOffset(usize)`** — an offset counted in bytes. Same derives.
- **`macro_rules! impl_offset!`** (exported) — generates the shared API for each newtype: `zero()`, `as_usize()`, `empty_range()`, `add_signed(isize)` (debug-panics on overflow), `range(Range<usize>) -> Range<Self>`, plus `From<usize>`, `AddAssign`, `AddAssign<usize>`, and the `Add`/`Sub`/`SubAssign` arithmetic impls.
- **`struct CharCounter<'a>`** — an incremental byte→char converter over a `&str`: `new(str)` then `char_offset(byte_offset)` walks forward, returning the `CharOffset` for a given `ByteOffset` (cheap when called with monotonically increasing byte offsets).

## Depends on (internal)

- None. External only: `get-size`, `num-traits`, `serde`.

## Used by (internal dependents)

Twelve crates — effectively the entire text/terminal stack:

- [warp_editor](./warp_editor.md), [syntax_tree](./syntax_tree.md), [vim](./vim.md), [warp_core](./warp_core.md), [warp_completer](./warp_completer.md), [warp_search_core](./warp_search_core.md), [warp_ripgrep](./warp_ripgrep.md), [warp_terminal](./warp_terminal.md), [warp_tui](./warp_tui.md), [warpui_core](./warpui_core.md), [ai](./ai.md), [warp](./warp.md).

(12 dependents total.)

## Related crates

- [sum_tree](./sum_tree.md) — these offsets are the natural `Dimension`s seeked inside the editor's `SumTree`-backed buffer.
- [warp_editor](./warp_editor.md) — defines `TextSummary` carrying both offset flavors and threads them everywhere.

## Marley relevance

**KEEP (verbatim).** Pure type-safety primitive, no branding, auth, network, or UI surface; 12 dependents make it a hub. None of the four Marley goals touch it. Do **not** rename the package — `string-offset` is generic and the churn across 12 manifests would be pure cost. When we add the custom Marley panel (goal 1) or session read/write (goal 2), we will *consume* `CharOffset`/`ByteOffset` to talk to the buffer, not modify them.

## Notes / gotchas

- `add_signed` only `debug_assert!`s on overflow — in release builds an out-of-range offset wraps silently. Keep that in mind for any Marley code that does signed offset math from untrusted input.
- The newtypes wrap a private `usize`; you must go through `as_usize()` / `From<usize>` rather than touching the field.
- `GetSize` derive participates in Warp's memory-accounting; harmless for Marley but a reason the `get-size` external dep is present.
