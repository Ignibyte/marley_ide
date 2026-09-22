# Marley Cross-Spec Seam Contracts — Binding (Round 2)

> The single, canonical source of truth for the shared types and crate boundaries that
> the round-1 specs forked, double-defined, or left unspecified. Every fix agent MUST
> conform to the names, ownership, and field shapes pinned here. Where a spec disagrees
> with this doc, **this doc wins** and the spec is edited to match. Resolves review-r1
> blockers 1–8 and the cross-spec HIGHs.
>
> **Clean-room Posture A (binding):** every name below is a **Marley-original** identifier.
> No Warp-internal type/module/static name (e.g. `FLAG_STATES`, `USER_PREFERENCE_MAP`,
> `LiteCommand`, `ParsedExpression`, `ClassifiedCommand`, the `Component`/`Params`/`Options`
> triad, `CONTRACTION_REGEX`, `enum_iterator::cardinality` array-sizing) may appear in any
> spec's **Public surface** or be the spec's organizing taxonomy. Public surfaces describe
> **observable behavior** (I/O), not a transcription of Warp's private module layout.

---

## 0. Crate map (who owns what)

| Crate | Milestone | Owns (canonical) |
|---|---|---|
| `marley_text_offsets` | M0 | `CharOffset`, `ByteOffset` (private field), `CharCounter` — §1 |
| `marley_core` | M0 | `SessionId` (process counter), paths, `Config`, feature flags — §2 |
| `marley_util` | M0 (**NEW**) | `FileId`, `ContentVersion`, `HostId`, `StandardizedPath`, `LocalOrRemotePath`, path helpers — §8 |
| `marley_command` | M0 | non-PTY child-process spawn seam — §5 |
| `marley_editor` | M1 | buffer-core: `BufferDelta`, `BufferVersion`, `Point`, `Rope` alias, `BufferEvent` + subscription, selection ownership, char↔byte — §3 |
| `marley_terminal` | M1 | Block model, `ShellSessionId`, `decode_hook`/`apply_hook`, PTY spawn (via `alacritty_terminal::tty`) — §4 |
| `marley_languages` | M1 | `IndentUnit`, grammar+query bundles — §3.6 |
| `marley_syntax` | M1 | highlight/indent runtime (consumes §3 + §3.6) |
| `marley_ui_components` | M1 | **theming value vocabulary** `Appearance`, `ThemeColors` + the five widgets — §6 |
| `marley_app` | M1 | `Theme`/`ThemeRegistry` (wraps §6), pane group, static M1 palette — §6, §7 |
| `marley_search_core` | M1 | `SearchMixer`, `QueryResult<T>`, `fuzzy_rank` — §7 |
| `marley_completer` | M1 | completions; `ByteOffset` spans; in-crate cache — §3.5 |
| `marley_input_classifier` | M1 | shell-vs-AI heuristic; in-crate `Tokens` — §3.5, §9 |
| `marley_asset_core` | M1 (**NEW**) | `Asset`, `AssetCache`, `AssetSource`, `AssetState`, `AsyncAssetType` — §10 |
| `marley_assets` (+ `marley_asset_macro`) | M1 | async URL/`data:` sources + embed macro, on top of `marley_asset_core` — §10 |

---

## 1. Shared offset vocabulary — owner `marley_text_offsets` (Blocker 1)

**There is exactly one `CharOffset` and one `ByteOffset` in the workspace.** They live in
`marley_text_offsets`. **No other crate may declare its own offset newtype.** In particular
`marley_editor` MUST delete its local `pub struct CharOffset(pub usize)` /
`pub struct ByteOffset(pub usize)` and import these.

```rust
// marley_text_offsets
pub struct CharOffset(usize);   // PRIVATE field
pub struct ByteOffset(usize);   // PRIVATE field
// construct only via: From<usize>, zero(), range(Range<usize>); read via as_usize()
```

- The `usize` field is **private**. Construction is via the documented API
  (`From<usize>`, `zero()`, `range()`, arithmetic operators) — never via a public tuple field.
- `CharOffset` and `ByteOffset` are **non-interchangeable**; mixing them must fail to compile.

### 1.1 char↔byte conversion seam (editor / completer / syntax)

The only place that can convert between the two flavors is a component that **holds the
text**. Two converters exist, with non-overlapping roles:

- **Stateful, whole-buffer:** `marley_editor::Buffer` owns
  `char_to_byte(CharOffset) -> ByteOffset` and `byte_to_char(ByteOffset) -> CharOffset`
  (random access over its rope). This is the editor↔completer seam.
- **Streaming, forward-only:** `marley_text_offsets::CharCounter` converts a monotonically
  non-decreasing stream of `ByteOffset`s to `CharOffset`s over a borrowed `&str`
  (for decode/scan paths that do not hold a `Buffer`).

**Editor↔completer rule:** `marley_completer` works entirely in **`ByteOffset`** spans
(`Spanned<T>{ value, span: Range<ByteOffset> }`, `SuggestionResults::replaced_span:
Range<ByteOffset>`). The prompt-input integration converts a completer span to a
`Range<CharOffset>` via `Buffer::byte_to_char` **before** calling `Buffer::edit`.
`marley_editor` never imports any `marley_completer` type; `marley_completer` never imports
any `marley_editor` type. The shared currency is `marley_text_offsets`.

---

## 2. `SessionId` ownership (Blocker 2)

**`marley_core` owns the one process-unique session counter.** `marley_terminal` does NOT
redefine it.

```rust
// marley_core — UNCHANGED owner
pub struct SessionId(u64);          // process-unique, monotonic
impl SessionId { pub fn next() -> SessionId; pub fn as_u64(self) -> u64; }
// Copy + Clone + Eq + Hash + Debug
```

The id decoded from a shell bootstrap `InitShell` DCS hook is a **different thing** and gets
a **distinct Marley-original name**:

```rust
// marley_terminal
pub struct ShellSessionId(u64);     // value SELF-REPORTED by the shell hook; correlation only
// Copy + Clone + Eq + Hash + Debug — has NO next()/allocation API
```

**Mapping (binding):**
- `TerminalSession::spawn` allocates one `marley_core::SessionId` via `SessionId::next()`.
- `Block.session_id: Option<marley_core::SessionId>` — this is what `marley_app` reads to
  wire a pane to a session. It is the **core** SessionId, never the shell id.
- An `InitShell` hook carries a `ShellSessionId`; `apply_hook` records the
  `ShellSessionId -> SessionId` correlation in the session's registry (used for subshell
  detection), and confirms subsequently opened blocks carry the session's `SessionId`.
- The two specs' dependency-edge statements MUST agree: `marley_terminal` **depends on
  `marley_core`** (`SessionId`); `marley_core` lists `marley_terminal` as a downstream
  consumer.

---

## 3. Editor buffer-core contract — owner `marley_editor` (Blocker 4 + editor HIGHs)

`marley_editor` is the single owner of the buffer-core seam types that `marley_syntax` (and
the prompt input) bind to. All names below are Marley-original and replace round-1 forks.

### 3.1 Delta — `BufferDelta` (renames `EditDelta`)

```rust
pub struct BufferDelta {
    pub char_range: Range<CharOffset>,  // pre-edit replaced span, char-indexed
    pub byte_range: Range<ByteOffset>,  // pre-edit replaced span, byte-indexed (for tree-sitter)
    pub new_char_len: usize,            // chars inserted by the replacement
    pub new_byte_len: usize,            // bytes inserted by the replacement
}
```

`marley_syntax` derives a tree-sitter `InputEdit` from this with:
`start_byte = byte_range.start`, `old_end_byte = byte_range.end`,
`new_end_byte = byte_range.start + new_byte_len` (and the row/col points via `Point`, §3.3).
No `EditDelta` name survives.

### 3.2 Version — `BufferVersion` (replaces bare `u64`)

```rust
pub struct BufferVersion(u64);          // PRIVATE field, monotonic
impl BufferVersion { pub fn initial() -> Self; pub fn next(self) -> Self; pub fn as_u64(self) -> u64; }
```

`Buffer::version() -> BufferVersion`; `EditResult { delta: BufferDelta, version: BufferVersion }`;
every `BufferEvent` and `DecorationEvent` version field is `BufferVersion` (not `u64`).

### 3.3 `Point`

```rust
pub struct Point { pub row: u32, pub column: u32 }   // row = 0-based line; column = UTF-8 BYTE offset within the row (tree-sitter convention)
```

Owned by `marley_editor`, consumed by `marley_syntax::indentation_at(&Rope, Point)`.

### 3.4 `Rope` alias

```rust
pub type Rope = ropey::Rope;            // exported from marley_editor
```

`marley_syntax` takes `&marley_editor::Rope` in `apply_edit`/`highlights_in_range`/
`indentation_at`. It does not re-alias or re-export its own `Rope`.

### 3.5 `BufferEvent` subscription seam + selection ownership (editor HIGHs)

```rust
pub enum BufferEvent {
    Edited { delta: BufferDelta, origin: EditOrigin, version: BufferVersion },
    SelectionChanged { version: BufferVersion },
}
impl Buffer {
    pub fn subscribe(&self) -> BufferSubscription;   // a Receiver<BufferEvent>; Edited emitted before edit() returns
    // selection lives on the buffer (undo restores it; coalescing keys on it):
    pub fn selection(&self) -> &SelectionSet;
    pub fn set_selection(&mut self, sel: SelectionSet);   // clamps, merges, emits SelectionChanged, breaks undo coalescing
}
```

- `BufferEvent::SelectionChanged` is **emitted by `set_selection`** — it is no longer an
  orphan variant.
- Undo coalescing (R9) keys on "no intervening `set_selection`"; undo (R11) restores the
  selection snapshot recorded with each undo group. `Selection`'s endpoints are clamped at
  the `SelectionSet`/`Buffer` boundary (fields not publicly mutable in a way that bypasses
  clamping).
- The completer's `ParsedTokensSnapshot` and any cross-crate cache are **not** part of this
  seam (see §3.5a).

### 3.5a Completer↔classifier snapshot (cross-spec HIGH)

The shared `ParsedTokensSnapshot` hand-off is **deferred past M1**. At M1:
- `marley_completer` may keep an **in-crate** parse cache (behind a `pub struct Completer`
  with `async fn suggestions(&mut self, …)` + a private snapshot) — but it does **not**
  expose that snapshot as a cross-crate contract and ships **no** classifier-consumer seam
  test.
- `marley_input_classifier` parses its **own** in-crate `Tokens` (`Tokens::parse`) and lists
  **no** dependency on `marley_completer`. It does not accept `ParsedTokensSnapshot` at M1.
- The shared-snapshot unification is a named M2 item in both specs and the build order.

### 3.6 `IndentUnit` source

`IndentUnit` is owned by `marley_languages`, not `marley_editor`. `marley_syntax` sources
`IndentUnit` from `marley_languages` and `CharOffset` from `marley_text_offsets`.

---

## 4. Terminal hook error surface + PTY spawn (Blockers 3 + 5)

### 4.1 Stateless decode vs stateful apply (Blocker 3)

Two functions, two **distinct** error enums (Marley-original names; the single `HookError`
is removed):

```rust
// STATELESS free fn — codec only, no registry access:
pub fn decode_hook(encoding: DcsEncoding, payload: &[u8]) -> Result<DcsHook, DecodeError>;
pub enum DecodeError { UndecodablePayload, UnknownHook }

// encoding SELECTION is observable (terminal-blocks R9 HIGH):
pub fn encoding_for_dcs_terminator(terminator: u8) -> Option<DcsEncoding>;

// STATEFUL — owns the session registry + BlockList:
impl TerminalSession {
    pub fn apply_hook(&mut self, hook: DcsHook) -> Result<(), ApplyHookError>;
}
pub enum ApplyHookError { MissingSession }
```

- `decode_hook` returns **only** `DecodeError` (codec). R9 (decode-under-encoding) and R10
  (`UndecodablePayload`) bind to `decode_hook`; the terminator→encoding selection binds to
  `encoding_for_dcs_terminator`.
- `apply_hook` is where `MissingSession` lives (a `Preexec`/`Precmd` decoded before any
  `InitShell` registered a `ShellSessionId`), and where "leave the `BlockList` unchanged on
  error" is asserted. R11 re-points here. `hook_without_session_returns_missing_session`
  tests `apply_hook`, not `decode_hook`.

### 4.2 Single PTY spawn seam (Blocker 5)

**`marley_terminal` owns PTY spawn, end to end, via `alacritty_terminal::tty`** (its own
openpty + fork + exec). `marley_command` is **not** in the PTY path at M1.

- `marley_command` is scoped to **non-PTY** child-process spawns for the rest of the
  workspace (e.g. `git`, helper subprocesses, the `is_wsl` probe). Its Purpose must drop the
  claim "the shell process that feeds the terminal PTY is built through
  `marley_command::blocking::Command`", and it must delete the `pty_shell_spawn_seam`
  integration test.
- `marley_terminal` lists **no** dependency on `marley_command` for PTY spawn; the raw
  openpty/fork/exec lines remain the existing ACCEPTED-UNTESTABLE reuse-crate path covered
  by the real-PTY integration tests.

---

## 5. `marley_command` scope (Blocker 5 corollary)

`marley_command` = the workspace's **single non-PTY** process-spawn seam (`blocking` +
`async` `Command`, Windows `CREATE_NO_WINDOW` + `JobObject` parity, `wsl::is_wsl()`). It is
foundational substrate for git/helper spawns; it does **not** spawn the terminal PTY (§4.2).

---

## 6. Theming value vocabulary (Appearance collision HIGH)

There is **one** theming value vocabulary, owned by **`marley_ui_components`** (the lowest
crate that depends on `gpui` — `marley_core` must not depend on `gpui`, so the
`gpui::Hsla`-bearing color bundle cannot live there):

```rust
// marley_ui_components
pub enum Appearance { Light, Dark }     // the discriminant ONLY
pub struct ThemeColors {                // the shared color bundle every widget renders from
    pub background: gpui::Hsla, pub foreground: gpui::Hsla, pub accent: gpui::Hsla,
    pub on_accent: gpui::Hsla, pub surface: gpui::Hsla, pub border: gpui::Hsla,
    pub danger: gpui::Hsla, /* + metrics */
}
```

- `marley_ui_components` widgets render from **`&ThemeColors`** (+ `Appearance` where a
  default must branch Light/Dark). The round-1 ui-components `struct Appearance { accent,
  surface, … }` is **renamed to `ThemeColors`**; the bare `Appearance` becomes the
  Light/Dark enum.
- `marley_app::themes` reuses these: `Theme { name: String, appearance: Appearance, colors:
  ThemeColors }`, `ThemeRegistry`. **`marley_app` depends on `marley_ui_components`** for
  `Appearance`/`ThemeColors` (natural direction; no cycle). `marley_app` does **not** declare
  its own `Appearance` struct or its own `ThemeColors` shape.

### 6.1 Widget contract de-Warp (clean-room HIGH)

The `Component`/`Params`/`Options` trait triad is **struck** from the public surface. Each
widget is a Marley-original struct with a render entry plus **pure decision accessors** that
make its logic headless-testable (so the unit tests in the spec can bind to the contract):

```rust
// shape, per widget (names illustrative, Marley-original):
impl Button {
    pub fn render(&mut self, colors: &ThemeColors, props: ButtonProps<'_>) -> gpui::AnyElement;
    pub fn visual_state(&self, props: &ButtonProps) -> VisualState;          // Rest/Hovered/Pressed/Disabled
}
pub fn button_colors(theme: &ButtonTheme, colors: &ThemeColors, state: VisualState) -> ButtonColors;
impl Switch  { pub fn knob_position(&self, on: bool) -> KnobPosition; /* Leading|Trailing */ }
impl Tooltip { pub fn is_visible(&self) -> bool; }
// content order is exposed as a descriptor, e.g. ContentOrder { IconThenLabel } for R4.
```

Pixel-layout assertions move to the visual harness (§ visual-testing); the decision seams
above are unit-tested and mutation-covered.

---

## 7. Command palette / ranking (palette-twice + R17 HIGHs)

- **`marley_search_core`** is the general engine: `SearchMixer<T>`, `QueryResult<T>` with
  `score: f32`, tiers, dedup. It is **built and tested in isolation at M1**; it has no M1
  palette consumer. It exposes a public ranking entry so ranking is in the surface:
  `pub fn fuzzy_rank(query: &str, candidates: &[&str]) -> Vec<(usize, u32)>` (nucleo
  subsequence score; non-match → omitted). `marley_search_core` R17 binds to `fuzzy_rank`.
- **`marley_app`'s M1 command palette is intentionally a static in-memory list** with a
  local `filter_commands` that ranks via the same `nucleo` primitive (a thin wrapper). This
  is a **declared, temporary** divergence: both `marley_app` and `marley_search_core` (and
  the build order) state that in **M2** the palette migrates to registering commands as a
  `SyncDataSource` on `SearchMixer`, unifying on `QueryResult`/`f32`. Until then app-shell's
  `ScoredCommand { score: u32 }` is local-only and must carry the "M2-merge" note so it is
  not mistaken for a second permanent ranking engine.

---

## 8. `marley_util` value types (Blocker 8) — see NEW spec

`marley_util` (M0, REIMPLEMENT) owns the value-type vocabulary downstream crates bind to.
Pinned Marley-original surface (the M1-binding subset; async-git/worktree-name deferred with
a crate-triage note until a consumer needs them):

```rust
pub struct FileId(u64);            // process-unique file identity; new()/as_u64()
pub struct ContentVersion(u64);    // monotonic content revision; next()/as_u64()
pub struct HostId(String);         // local-or-remote host identity; HostId::local()
pub enum StandardizedPath { /* normalized, absolute, separator-canonical */ }
pub enum LocalOrRemotePath { Local(StandardizedPath), Remote { host: HostId, path: StandardizedPath } }
pub fn standardize_path(p: &Path) -> StandardizedPath;   // normalize . / .. / separators
```

`marley_assets`' dangling `make_absolute_url` dependency is resolved by **dropping** the
remote-CDN macro path (§10.2), so `marley_util` need not own a CDN base-URL helper at M1.

---

## 9. Input-classifier threshold regimes (Blocker 6)

The two scoring clauses cover **non-overlapping** token-count regimes:

```text
LOW_TOKEN_COUNT = 2
DETECT_AS_NATURAL_LANGUAGE_THRESHOLD = 0.6   // high-token regime
LOW_TOKEN_THRESHOLD                  = 0.8   // low-token regime
```

- **R12 (high-token):** `WHILE words.len() > LOW_TOKEN_COUNT`, classify `Ai` iff
  `ratio >= 0.6`, else `Shell`. R12's fixture uses **> 2** tokens.
- **R13 (low-token):** `WHILE words.len() <= LOW_TOKEN_COUNT`, classify `Ai` iff
  `ratio >= 0.8`, else `Shell`.

No `(words.len(), ratio)` pair satisfies both clauses. `ratio` numerator is
`natural_language_words_score(words, is_first_token_command)` (R10, which skips the leading
token when `is_first_token_command`); denominator is `words.len()` — the off-by-one basis is
defined: numerator skips the head, denominator does not (a leading command token therefore
lowers the ratio). `is_first_token_command` is derived from `util::is_likely_shell_command`
on the head token.

---

## 10. AssetCache foundation (Blocker 7) — see NEW spec

### 10.1 Foundation types — owner `marley_asset_core` (NEW)

```rust
pub trait Asset: 'static { type Output: Clone + 'static; /* async decode from bytes */ }
pub trait AsyncAssetType: 'static { /* distinct marker per async source kind */ }
pub enum AssetSource {
    Bundled { path: &'static str },             // resolved via rust-embed at runtime
    Async(AsyncSource),                          // carries an AsyncAssetType marker + locator
}
pub enum AssetState<T> { Loading, Loaded(T), Failed(AssetError) }
pub enum AssetError { Fetch, Decode, NotFound }
pub struct AssetCache { /* sync, keyed by AssetSource identity; loads off the render thread */ }
impl AssetCache {
    pub fn get<T: Asset>(&self, source: &AssetSource) -> AssetState<T>;   // returns Loading sync; never blocks a frame
}
```

`marley_assets` **depends on** `marley_asset_core` and adds `AssetCacheExt`, the URL/`data:`
sources, and the distinct markers (`UrlAssetWithoutPersistence`, `UrlAssetWithPersistence`,
`DataUriAsset`) — all implementing `marley_asset_core::AsyncAssetType`. `marley_asset_core`
is built **before** `marley_assets` in the M1 order; M1 is no longer gated by a crate whose
base types are unspecified.

### 10.2 Scope trim (assets HIGH/low)

The remote-CDN macro path is **dropped** at M1: `remote_asset!`,
`bundled_or_fetched_asset!`, and `make_absolute_url` are removed (they reintroduce the
Warp-CDN/wasm machinery `crate-triage` marked SKIP and contradict the local-first charter).
M1 ships `bundled_asset!` + `data:`-URI/`url_source` sources only.

---

## 11. Clean-room provenance wall (Blockers 10 + 11)

- **Spec layer (Blocker 11):** the quality-bar provenance gate is extended to **spec
  artifacts** — see `quality-bar.spec.md` gate 16. No spec's "Public surface" may contain a
  Warp-internal-only identifier; `spec_source` must point at a behavior-level doc.
- **`spec_source` (Blocker 10):** every REIMPLEMENT spec's `spec_source` is repointed at a
  **behavior-only** description (observable I/O — no private module/type/static names, no
  fork file paths), and the `clean_room` frontmatter line is downgraded from
  "behavior-only; no AGPL source read" to the accurate
  `clean_room: "behavior-derived from a fork-reference doc; IP-counsel sign-off pending"`
  until the behavioral wall + sign-off land (open item in `clean-build-plan.md`).

### 11.1 Naming map (strip → use), applied across all specs

| Warp-internal (STRIP from public surface) | Marley-original (USE) | Spec |
|---|---|---|
| `EditDelta` | `BufferDelta` | editor |
| bare `u64` buffer version | `BufferVersion` | editor |
| editor-local `CharOffset`/`ByteOffset` (pub field) | `marley_text_offsets::{CharOffset, ByteOffset}` (private field) | editor |
| `LiteCommand`, `ParsedToken`, `ParsedExpression`, `ClassifiedCommand`, `classify_command`, the `parsers` module, the legacy/`v2` split, `matchers`/`MatchStrategy`/`MatchType` | observable `suggestions()`/`describe()`/`SuggestionResults`/`SuggestionType` (+ private `Match{score,indices}`) | completer |
| `Component`/`Params`/`Options` trait triad; "borrowed from Warp's ui_components" | per-widget struct + `render(&ThemeColors, props)` + decision accessors (§6.1) | ui-components |
| ui-components `struct Appearance { accent,… }` | `ThemeColors` (render input) | ui-components |
| terminal-local `SessionId` | `ShellSessionId` + `marley_core::SessionId` | terminal-blocks |
| single `HookError` enum | `DecodeError` (stateless) + `ApplyHookError` (stateful) | terminal-blocks |
| `FLAG_STATES`, `USER_PREFERENCE_MAP`, `AtomicTriState`; `enum_iterator::cardinality` array-sizing in the contract | private impl detail; restate R18 as observable "adding/removing a variant needs no manual length update" | marley_core |
| `CONTRACTION_REGEX`, `RESERVED_KEYWORDS` | private impl detail | input-classifier |
| `FormattedText` table/tag strings flagged "deferred de-Warp rebrand" | line/inline model names derived from the GFM features the R# clauses require | markdown-render |
| app-shell `ScoredCommand{score:u32}` presented as a 2nd engine | local static-list wrapper ranked by the shared `nucleo` primitive; M2 merges onto `SearchMixer`/`QueryResult` | app-shell / search-core |
