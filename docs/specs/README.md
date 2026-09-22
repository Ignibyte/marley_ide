# Marley Specs — Index & First-Round Ticket Manifest

`docs/specs/` is the behavioral source of truth for **Marley** — Ignibyte's clean-room,
Rust + gpui, UI-first agentic dev cockpit with the power and feel of Warp. Each file here
is a **clean-room EARS specification**: it describes WHAT a component does (its public
surface + behavior) for a fresh Rust implementation that REUSES named permissive
(MIT/Apache) crates and is written **from our reference docs, never from Warp's AGPL
source**. Every REIMPLEMENT spec carries the provenance line *spec'd from
`docs/warp_architecture/…`, behavior-only, no AGPL source read*.

## House style — EARS

Every requirement is **one testable clause** in an EARS form:

- **Ubiquitous** — `The system shall …`
- **Event** — `WHEN <trigger>, the system shall …`
- **State** — `WHILE <state>, the system shall …`
- **Optional** — `WHERE <feature>, the system shall …`
- **Unwanted** — `IF <condition>, THEN the system shall …`

No "gracefully"/"appropriately"/vague verbs. Each clause maps 1:1 to an
acceptance-criteria row **and** to at least one named planned test. This is the
AIC/ucsosv2 house style.

**Each SPEC becomes a forge ticket.** This manifest is the **first round** of tickets:
one ticket per spec below, built in milestone/build order.

## Standards

- Binding quality bar — every spec's test/mutation/coverage plan must satisfy
  [standards/quality-bar.spec.md](standards/quality-bar.spec.md): `clippy -D warnings`,
  100%-on-touched coverage, mutation **MSI 100%**, `miri` for any `unsafe`-bearing crate,
  and the AXUIElement + screenshot visual harness for every `visual_acceptance` clause.
- Spec structure — every spec follows [_TEMPLATE.spec.md](_TEMPLATE.spec.md) exactly.

## Specs by milestone

### M0 — Foundation (build first)

| Spec | Summary | EARS | Bucket |
|------|---------|------|--------|
| [SPEC-marley_core](SPEC-marley_core.spec.md) | App/runtime core — `SessionId` newtype, `~/.marley` path layout, single hardcoded offline `Config`, feature-flag registry | 18 | REIMPLEMENT |
| [SPEC-marley-util](SPEC-marley-util.spec.md) | Workspace value-type vocabulary — `FileId`, `ContentVersion`, `HostId`, `StandardizedPath`, `LocalOrRemotePath`, `standardize_path` (async-git/worktree-name deferred) | 16 | REIMPLEMENT |
| [SPEC-text-offsets](SPEC-text-offsets.spec.md) | Type-safe `CharOffset`/`ByteOffset` newtypes + incremental `CharCounter` byte→char converter | 20 | REIMPLEMENT |
| [SPEC-process-command](SPEC-process-command.spec.md) | The workspace's single process-spawn seam (blocking + async `Command`) with Windows `CREATE_NO_WINDOW` + JobObject parity | 15 | REIMPLEMENT |
| [SPEC-foundation-spike](SPEC-foundation-spike.spec.md) | Throwaway viability spike — gpui window + alacritty PTY + render one Block (validates the REUSE stack) | 15 | INVENT |

### M1 — Terminal MVP

| Spec | Summary | EARS | Bucket |
|------|---------|------|--------|
| [SPEC-editor](SPEC-editor.spec.md) | Headless rope-backed editing core — buffer, anchors, selection, undo/redo, find, layout pipeline | 23 | REIMPLEMENT |
| [SPEC-terminal-blocks](SPEC-terminal-blocks.spec.md) | UI-agnostic terminal session + Block model — PTY spawn/write, shell-hook DCS/OSC metadata, output read-back | 19 | REIMPLEMENT |
| [SPEC-syntax-highlight](SPEC-syntax-highlight.spec.md) | Incremental tree-sitter highlight + auto-indent runtime + language registry | 18 | REIMPLEMENT |
| [SPEC-markdown-render](SPEC-markdown-render.spec.md) | Markdown/GFM → `FormattedText` line model (tables, task lists, links) for blocks | 20 | REIMPLEMENT |
| [SPEC-settings](SPEC-settings.spec.md) | Typed declarative settings framework — `Setting` trait, macros, TOML-backed `SettingsManager` | 18 | REIMPLEMENT |
| [SPEC-ui-components](SPEC-ui-components.spec.md) | Reusable widget set (button/switch/dialog/tooltip/shortcut chip) on gpui | 16 | REIMPLEMENT |
| [SPEC-command-palette](SPEC-command-palette.spec.md) | Search mixer / command palette — fan a query to data sources, return ranked deduped results | 22 | REIMPLEMENT |
| [SPEC-completions](SPEC-completions.spec.md) | Command-line completion engine — parse line+cursor → nucleo-ranked suggestions | 18 | REIMPLEMENT |
| [SPEC-input-classifier](SPEC-input-classifier.spec.md) | Shell-vs-AI decision engine for the unified prompt (heuristic-first, ML path stubbed) | 21 | REIMPLEMENT |
| [SPEC-app-shell](SPEC-app-shell.spec.md) | Warp-like app shell — offline gpui boot, `RootView`, pane-group layout, palette, keymap, themes | 24 | REIMPLEMENT |
| [SPEC-assets](SPEC-assets.spec.md) | Asset cache (URL/data-URI sources) + compile-time embed proc-macro | 23 | REIMPLEMENT |

**Total: 16 specs — 5 M0 (84 EARS) + 11 M1 (222 EARS) = 306 EARS clauses.**

## Build order

Per [../marley_architecture/crate-triage.md](../marley_architecture/crate-triage.md)
(§ Build order, M0–M5):

1. **M0 — Foundation** lands first: `marley_core` (SessionId/paths/flags), `marley_util`
   (the value-type vocabulary — `FileId`/`ContentVersion`/`HostId`/`StandardizedPath`/
   `LocalOrRemotePath`/`standardize_path`), `text-offsets` (`string-offset`),
   `process-command` (the spawn seam), and the throwaway `foundation-spike` that validates
   the REUSE stack (gpui + alacritty_terminal). Downstream crates block on these value
   types, so they ship before anything in M1. (5 REIMPLEMENT/INVENT specs — the round-1
   "4 vs 5" build-order discrepancy is closed: `marley_util` is the missing fifth.)
2. **M1 — Terminal MVP** builds on M0: Blocks, the input editor, panes, the command
   palette, and themes — the first user-runnable cockpit.

## Next

These 15 specs are the **first round of forge tickets** (one ticket per spec). The
**INVENT cockpit layer** — the project / agents / brain / Forge / ops panels that make
Marley more than a Warp clone — gets its own spec round at **M2**.
