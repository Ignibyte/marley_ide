# Subsystem 09 — Vim, Keymap & Input Dispatch (the context-scoped keybinding model)

> Part of the Zed architecture reference (round 1). Zed is the EDITOR reference for Marley's editing
> surface. This subsystem is the **clean fix for editor-vs-terminal keybinding collisions** — Marley
> today has ONE flat keymap shared by the terminal and the new editor, and the collisions are already
> live (`⌘D` = new-terminal in Marley vs. select-next-match in Zed; `⌘F`, `⌘/`, `Enter`, `Esc`, and
> every single letter loom next).

## Provenance posture (READ THIS FIRST)

The three layers here have **different licenses**, and the boundary is the whole point:

| Layer | Zed crate(s) | License | Marley provenance tag |
|---|---|---|---|
| **KeyContext dispatch mechanism** (context stack, predicate language, keymap matcher, actions) | `gpui` (`keymap/`, `key_dispatch.rs`, `action.rs`) | **Apache-2.0** | `[gpui Apache-2.0]` — freely usable, Marley adopts the *design* directly |
| **Declarative keymap config** (JSON sections scoped by context, source layering) | `settings` (`keymap_file.rs`) | GPL-3.0 (the loader), but the *format* is a config schema | `[gpui Apache-2.0]` for the model / `[Marley-original]` for Marley's own loader |
| **Vim implementation** (the modal engine, default vim keymap) | `vim`, `vim_mode_setting` | **GPL-3.0-or-later** | `[Zed-derived]` — do not copy code |
| **Vim command grammar** (modes, operators, motions, text-objects, registers, dot-repeat) | — | public de-facto standard | `[permissive/public: vim's grammar is a public standard]` |
| **Marley's current flat keymap** | `marley_app` (`keymap.rs`) | Marley's | `[Marley-original]` |

The load-bearing fact: **the context-dispatch mechanism Marley needs is entirely in `gpui`
(Apache-2.0)**, not in the GPL `vim` crate. Marley can adopt the KeyContext model as its own pure code
with no copyleft exposure. Vim itself is a *later, optional* GPL-flavoured layer that rides on top of
the same mechanism — and even then only the *implementation* is GPL; the modal *model* is public.

Paths below the `gpui`/`vim` prefixes are relative to the session Zed clone
(`…/scratchpad/zed-src/crates/`); Marley paths are absolute from the repo root.

---

## Purpose (what this subsystem does)

It answers one question on every keystroke: **given the key that was pressed and *what currently has
focus*, which action fires — or does the key fall through as text input?** Zed decomposes that into
three cleanly separated pieces:

1. **A per-element `KeyContext`** — each focusable surface (editor, terminal, workspace, panel,
   modal) stamps a small set of identifiers + `key=value` pairs onto its node in the element tree
   (`Editor mode=full`, `Terminal screen=alt`, `vim_mode=normal`).
2. **A predicate language** (`KeyBindingContextPredicate`) — each keybinding carries a boolean
   expression over contexts (`Editor && vim_mode == normal && !menu`). A binding is *eligible* only
   where its predicate matches the focused surface's context stack.
3. **A precedence-resolving matcher** (`Keymap`) — when several eligible bindings match the same keys,
   it picks the winner by **depth in the focus tree** (the innermost/focused surface wins), then by
   **load order** (user bindings loaded last win ties).

The result: **one keymap, many surfaces, zero collisions** — `⌘D` can mean new-terminal under a
`Terminal` context and select-next-match under an `Editor` context, both declared in the same file,
because the predicate scopes each to its surface.

---

## A. The gpui KeyContext predicate model `[gpui Apache-2.0]` — the clean fix

This is the piece Marley should adopt first. It has no vim, no editor, and no GPL in it.

### A.1 `KeyContext` — the per-surface state stamp (`gpui/src/keymap/context.rs`)

A `KeyContext` is just an ordered set of `ContextEntry { key, value: Option<value> }`:

- **Bare identifiers** — `Editor`, `Terminal`, `Workspace`, `menu`, `VimControl`. Presence = truth.
- **Key/value pairs** — `mode = full`, `vim_mode = normal`, `screen = alt`, `os = macos`.

Each focusable element builds one at paint time and hangs it on its dispatch node. Two real examples
from the reference:

- **Terminal** (`terminal_view.rs::dispatch_context`) starts from `KeyContext::new_with_defaults()`
  (which stamps `os = macos|linux|windows`), then `add("Terminal")`, `add("vi_mode")` when the
  terminal's own vi-mode is on, `set("screen", "alt"|"normal")`, `add("DECCKM")` for app-cursor mode.
- **Editor** (`editor.rs::key_context_internal`) does `add("Editor")`, `set("mode", "full"|
  "single_line"|"auto_height"|"minimap")`, `add("menu")` + `add("showing_completions")` when a
  completion popup is open, `add("renaming")`, `add("in_snippet")`, etc.

The context is **dynamic** — it reflects live sub-state, so the same editor publishes a *different*
context when a completion menu is open. That is how one surface routes `Enter` to "accept completion"
vs. "insert newline" without any imperative branching in the handler.

### A.2 The context **stack** (`gpui/src/key_dispatch.rs::DispatchTree`)

Contexts nest. During paint, `DispatchTree` accumulates a `context_stack: Vec<KeyContext>` by walking
from the window root down to the focused element: `[Workspace, Pane, Editor]`. Actions **bubble up**
from the focused node toward the root — so an `Editor` binding shadows a `Pane` binding shadows a
`Workspace` binding for the same keys. `dispatch_path()` produces the root→focused ordering the
matcher consumes.

### A.3 `KeyBindingContextPredicate` — the predicate grammar (`context.rs`)

A small boolean expression language, parsed from the `"context"` string on each binding. The AST:

```
Identifier(name)          e.g.  Editor
Equal(k, v)               e.g.  vim_mode == normal
NotEqual(k, v)            e.g.  mode != full
Not(p)                    e.g.  !menu
And(a, b) / Or(a, b)      e.g.  Editor && vim_mode == normal
Descendant(parent, child) e.g.  Workspace > Editor   (parent above child in the tree)
```

Operators, by precedence: `>` (descendant) < `||` < `&&` < `==`/`!=` < `!`, with `( )` grouping.
`eval` walks the context stack; `depth_of(contexts)` returns **the deepest stack depth at which the
predicate matches** — this integer *is* the precedence key. A binding with **no** context is treated
as matching at the deepest depth (global, lowest-priority tie). `is_superset()` powers conflict/unbind
reasoning (does predicate A cover everything B covers?).

Two subtleties Marley must preserve if it reimplements the grammar:
- **`!` negation is evaluated over the *whole* stack** (`!Workspace` is false if `Workspace` appears
  anywhere below, not just at the top) — this prevents a child from "escaping" an ancestor identifier.
- **`>` is genuinely hierarchical**: `Picker > Editor` matches an `Editor` nested under a `Picker`,
  but *not* a bare `Editor`, and `(Pane > Pane) > Editor` is not the same as `Pane > (Pane > Editor)`.

### A.4 The matcher & precedence (`gpui/src/keymap.rs::Keymap::bindings_for_input`)

Given the typed keystroke(s) and the context stack, `bindings_for_input` returns the eligible bindings
**in precedence order** plus a `pending` flag (are we mid-chord?). Precedence is:

```
sort by (depth DESC, then insertion-index DESC)
```

i.e. **deeper context wins**; at equal depth, **later-loaded wins**. Disabling is first-class:

- A binding to the **`NoAction`** built-in (written `"key": null` in JSON) *removes* the key in that
  context. A **targeted `Unbind("some::Action")`** removes one specific action's binding while leaving
  others on the same key. Both are evaluated with the same depth/order rules, so you can disable a key
  *only* in a deeper context (`test_disable_deeper`) — exactly Marley's editor-vs-terminal need.
- Multi-key chords: `match_keystrokes` returns `Some(pending=true)` when the typed prefix is a strict
  prefix of a longer binding. `dispatch_key` (in `key_dispatch.rs`) tracks pending input, and on a
  timeout `flush_dispatch` *replays* the buffered keys as literal input — so `ctrl-w` alone still
  types if `ctrl-w left` was the only binding and the second key never came.

### A.5 Config-layer precedence — `KeybindSource` (`settings/src/keymap_file.rs`)

When the same key/context collides across *keymap files*, the tie-break is the source, loaded so the
higher-priority source lands later in insertion order and wins the equal-depth tie:

```
User  >  Vim  >  Base(vscode/jetbrains/etc.)  >  Default
```

`KeybindSource` is recorded per binding (via `KeyBindingMetaIndex`) for display and round-tripping.

### A.6 Actions — the dispatch target (`gpui/src/action.rs`)

Bindings resolve to **`dyn Action`** (a `TypeId`-identified unit/struct, declared with the `actions!`
macro or `#[action]`, optionally carrying JSON params — e.g. `["vim::PushObject", {"around": false}]`).
Elements register handlers with `.on_action(...)`; the dispatcher matches by `TypeId` up the path.
This decouples *what key* from *what code* — the same action can have many bindings and none.

### A.7 Marley today vs. the reimplementation

**Marley today** (`crates/marley_app/src/keymap.rs`, `[Marley-original]`): a **single global flat
table** — `Vec<(KeyBinding, String)>` where `KeyBinding` is a pure `{cmd,ctrl,alt,shift,key}` chord and
the action is a name string. `action_for(&chord)` returns the **first** matching entry; a
`chords_unique` guard forbids duplicate chords precisely *because* there is no context to disambiguate
them. `⌘D → "new-terminal"` is bound once, for every surface, unconditionally. There is **no context,
no predicate, no depth** — the terminal and the editor are indistinguishable to the keymap.

**Reimplementation (adopt the `gpui` model as Marley's own pure code):**
1. Add an optional **context predicate** to each binding and a **`KeyContext` stack** the app builds
   from the focused surface. Marley already gates per-pane rendering on `is_focused` (the M12 pane
   loop) — the same focus signal names the context: `Terminal` vs `Editor`, plus live sub-state
   (`screen`, later `vim_mode`).
2. Change `action_for(&chord)` → `action_for(&chord, &context_stack)`: filter bindings by
   `predicate.depth_of(stack)`, then resolve by (depth desc, order desc). Keep it a **pure** function
   (no gpui types leak in) — mirrors the existing `Keymap` seam, so it stays cov/MSI-100 testable.
3. Each surface **publishes its context** at focus time. The `chords_unique` invariant *relaxes* to
   "unique per context" — two `⌘D` entries are now legal because their predicates are disjoint.

This is contained (the keymap is already an isolated pure module) and is the single highest-leverage
change in this subsystem — see §D.

---

## B. The declarative keymap config model `[gpui Apache-2.0 model]`

Zed's keymaps are **data, not code**: `assets/keymaps/*.json` plus the user's `keymap.json`, loaded by
`settings/src/keymap_file.rs`. A `KeymapFile` is a `Vec<KeymapSection>`; each section is
`{ "context": <predicate string>, "use_key_equivalents": bool, "bindings": { <keys>: <action> } }`:

```jsonc
{
  "context": "Editor && mode == full && !menu",
  "bindings": {
    "cmd-d": "editor::SelectNext",          // fires only in a focused full editor
    "escape": "editor::Cancel"
  }
},
{
  "context": "Terminal",
  "bindings": {
    "cmd-d": "workspace::NewTerminal"        // same chord, different surface — no collision
  }
}
```

Loading (`load_keymap`) parses each `context` into a `KeyBindingContextPredicate`, resolves each
action name (+ optional JSON args) to a `dyn Action` via the action registry, and produces `KeyBinding`
values tagged with their `KeybindSource`. `os = macos|linux|windows` from `new_with_defaults()` lets a
*single* file carry per-platform bindings (`"context": "Editor && os == macos"`).
`use_key_equivalents` remaps for non-US physical layouts. `null` disables a key; `["Unbind", "…"]`
disables a specific action.

**Marley today:** the default bindings are a **Rust literal** in `default_bindings()` — not
user-editable, not context-scoped. Marley *does* already own a typed declarative TOML settings
framework (`marley_settings`, the M1.B finale) and a settings round-trip pattern (RemoteHosts,
Workflows).

**Reimplementation:** express the keymap as a `marley_settings` group — a list of
`{ context, bindings }` sections deserialized to `(predicate, chord → action-name)` — reusing the exact
TOML round-trip Marley already ships. Provenance: the *format/model* is `[gpui Apache-2.0]`-shaped, the
loader is `[Marley-original]` (do not lift the GPL `keymap_file.rs`). Sequence this **after** §A — the
in-code predicate model must exist before a config file can target it.

---

## C. Vim mode — the modal layer `[Zed-derived]` impl / `[permissive/public]` model

Vim in Zed is **not** a special-cased input path — it is *ordinary keybindings scoped by extra context
keys*, plus a state machine that publishes those keys. That is the elegant part worth stealing
(conceptually): once §A exists, vim is "just another context."

### C.1 How vim projects into the keymap (`vim/src/vim.rs::extend_key_context`)

The editor's `key_context()` calls each registered **addon**'s `extend_key_context` (`editor.rs:2709`).
`VimAddon` delegates to `Vim::extend_key_context`, which *adds keys to the focused editor's existing
`KeyContext`*:

- `vim_mode = normal | visual | insert | replace | operator | waiting | literal | helix_normal | …`
- `vim_operator = none | d | c | y | f | …` (the pending operator, if any)
- `VimControl` identifier when in a command-accepting mode (normal/visual/operator) — this is the
  identifier every motion/operator binding predicates on (`"context": "VimControl && !menu"`).
- `VimCount` when a numeric count is pending.

So the *whole* vim keymap (`assets/keymaps/vim.json`, `[Zed-derived]`) is contextual data: `w`, `b`,
`d`, `ci"` etc. are bound under `VimControl` / `vim_mode == …`. In **insert** mode those bindings are
absent, so letters fall through as text input. **This is the mechanism that makes single letters mean
"motion" in normal mode and "type a character" in insert mode — no separate code path.**

### C.2 The modal state machine (`vim/src/state.rs`) `[permissive/public model]`

- **`Mode`**: `Normal, Insert, Replace, Visual, VisualLine, VisualBlock` (+ Helix variants). Zed
  defaults to Normal; note Warp's `vim` crate defaults to **Insert** (terminal-appropriate) — a choice
  Marley must make deliberately for a terminal audience.
- **`Operator`**: a large enum — `Change/Delete/Yank/Replace`, `Object{around}`, find/till
  (`f/t/F/T`), surrounds (`ys/cs/ds`), `Indent/Outdent`, case ops, `Mark/Jump`, `Register`,
  record/replay (`q`/`@`), comment toggles, plus Helix ops. `is_waiting()` (needs another key),
  `starts_dot_recording()`, and `id()`/`status()` (the `showcmd` string) drive the FSA.
- **Registers & clipboard**: `Register`, the named-register map, and `UseSystemClipboard` policy
  (`Always`/`OnYank`/`Never`) live in the global `VimGlobals`, with the special `"`/`0`/`-`/`1-9`/`+`/
  `*`/`_` registers modelled.
- **Dot-repeat & macros**: `VimGlobals` records `ReplayableAction`s (actions **and** raw insertions)
  for `.` repeat and `q{reg}`/`@{reg}` macros — replayed through the same action dispatcher.
- **Marks**: `MarksState` persists per-buffer/global marks to a small SQLite table (`VimDb`).
- **Count parsing**: `pre_count`/`post_count` (the `3` and `2` in `3d2w`) accumulate in `VimGlobals`.

The design keeps **semantics** (parse `d2aw`, track the register, stage dot-repeat) separate from
**mechanics** (mutate the buffer) — the FSA emits actions; the editor executes them.

### C.3 Marley today vs. the reimplementation

**Marley today:** no modal state at all — every key is either a `⌘`-chord (dispatched) or literal
input to the terminal/prompt/editor. There is no `vim_mode`, no operator-pending state.

**Reimplementation (a *later* layer, phased):**
1. Prerequisite: §A (the context model) must exist — vim needs `vim_mode`/`VimControl` context keys to
   scope its bindings.
2. A `Vim` state object (per editor, `[Marley-original]` code implementing the `[permissive/public]`
   model) that publishes `vim_mode`/`vim_operator`/`VimControl` into the focused surface's
   `KeyContext` — Marley's editor is a peer surface (M13/M15), so it can carry an addon-like hook.
3. A vim-scoped binding set (Marley's own, *not* copied from `vim.json`) under those contexts.
4. The FSA (operators/motions/objects/registers/dot-repeat) is the bulk of the work and can be
   incremental — start with normal/insert + basic motions + `d/c/y`, grow from there. Gate the whole
   thing behind a `vim_mode` setting (mirroring `vim_mode_setting`, a deliberately tiny separate
   crate so surfaces can toggle vim without depending on the engine).

Priority note: a modal layer is **plausibly high-value for a terminal audience** (Warp ships one too),
but it is strictly downstream of §A and much larger — treat it as a milestone, not a step.

---

## D. The `⌘D` collision — a worked example, and why KeyContext is step one

**The live bug.** Marley binds `⌘D → new-terminal` globally (`#197`). The moment the editor grows a
`⌘D → select-next-match` (the Zed idiom), Marley's flat table cannot hold both — `chords_unique`
forbids the duplicate, and even if allowed, `action_for` returns the *first* unconditionally. Whatever
has focus, one surface gets the wrong behaviour.

**The KeyContext fix (one keymap, two contexts):**

```jsonc
{ "context": "Editor",   "bindings": { "cmd-d": "editor::select-next-match" } },
{ "context": "Terminal", "bindings": { "cmd-d": "workspace::new-terminal"   } }
```

With the focused editor publishing `Editor` and the focused terminal publishing `Terminal`, the
matcher resolves `⌘D` to the surface that owns focus. Depth precedence then handles the *nested* case
for free: an `Editor` inside a `Workspace` still resolves the editor binding because it is deeper. The
same move fixes every looming collision at once — `⌘F` (editor find vs terminal search), `⌘/` (toggle
comment vs terminal literal), `Enter` (newline vs run-command vs accept-completion), `Esc` (clear
selection vs normal-mode vs dismiss-popup) — each becomes a context-scoped binding, not a special case.

**Why it is *first*.** The cost is bounded and paid once; the alternative grows without bound. Marley's
editor is about to add multi-cursor, LSP completions, rename, code actions — each ships **dozens** of
editor-only chords, and *every one* is a latent terminal collision under a flat keymap. Adopting the
context model now means those land as ordinary `"context": "Editor"` bindings with zero collision
triage. Deferring it means re-triaging the entire keymap later, by hand, under a `chords_unique` guard
that actively fights you. And because the mechanism is `[gpui Apache-2.0]` and Marley's keymap is
already an isolated pure module, the change is small and copyleft-free.

---

## E. Recommended sequencing

| Step | What | Size | Provenance | Unblocks |
|---|---|---|---|---|
| **1 (FIRST)** | **KeyContext + predicate + depth/order matcher** in Marley's pure keymap; each surface publishes a `KeyContext` at focus | **Small** | `[gpui Apache-2.0]` model, `[Marley-original]` code | The `⌘D` collision *and every future editor-only chord* |
| 2 | `NoAction`/`Unbind` disabling + multi-key chord (`pending`/replay) support | Small–Med | `[gpui Apache-2.0]` model | Per-context key disabling; `g`-prefix / `ctrl-w`-prefix chords |
| 3 | Declarative keymap as a `marley_settings` TOML group (context-scoped sections) | Med | `[gpui Apache-2.0]` format / `[Marley-original]` loader | User rebinding; per-platform bindings |
| 4 (LATER) | Vim modal layer: `Vim` state publishes `vim_mode`/`VimControl`; vim-scoped bindings; FSA (modes → operators → motions → objects → registers → dot-repeat), behind a `vim_mode` setting | **Large, phased** | `[permissive/public]` model, `[Marley-original]` code (NOT the GPL `vim` crate) | Modal editing for the terminal audience |

---

## Notes / gotchas

- **The mechanism is Apache-2.0, the vim *content* is GPL.** Marley may model its dispatch on
  `gpui`'s KeyContext freely; it must **not** copy the `vim` crate or `vim.json`. The vim *behaviour
  spec* (modes/operators/registers) is public and safe to reimplement from scratch.
- **Context is dynamic per frame.** A surface republishes its `KeyContext` every paint, reflecting live
  sub-state (`menu`, `showing_completions`, `screen=alt`, `vim_mode=insert`). Route behaviour by
  *adding a context key*, not by branching inside a handler — that is the whole design pattern.
- **"Fall through to input" is the default, not a special case.** A key with no matching binding in the
  current context is delivered to the focused element's `InputHandler` as text. Vim insert mode works
  *because* it removes the motion bindings, not because it adds an insert path.
- **Precedence has two axes** — context **depth** (focus tree) then **load order** (source layering,
  User > Vim > Base > Default). Marley's reimplementation must model both, or user overrides and
  editor-shadows-workspace will misbehave (see `keymap.rs` tests `test_depth_precedence`,
  `test_source_precedence_sorting`, `test_overlay_after_base_restores_shadowed_picker_binding`).
- **Default-mode choice is a real decision.** Vim-classic starts in Normal; a terminal-first product
  (Warp) starts in Insert. Pick deliberately for Marley's audience.
- **Marley already has the seams.** The pure `Keymap` module, per-pane `is_focused` gating, and the
  `marley_settings` TOML round-trip are exactly the three hooks steps 1–3 need — this is an
  *integration*, not a green-field build.
