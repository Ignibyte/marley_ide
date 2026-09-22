# Zed architecture reference (Marley)

Per-crate + per-subsystem reference for **Zed** (github.com/zed-industries/zed), the EDITOR reference for
Marley's editing surface — the counterpart to [`../warp_architecture/`](../warp_architecture/) (the terminal /
cockpit reference). Built the same way the Warp map was: a crate-by-crate deconstruction of the reference
codebase's architecture + responsibilities, so Marley can **reimplement each capability in its own code**.

## Provenance + licensing posture (READ THIS)

- **Zed is GPL-3.0.** This reference is *architecture analysis* — what each crate is responsible for, its public
  design, and how Marley rebuilds the capability. The Zed source itself is **cloned only into session scratch,
  never committed into this repo.** These docs are Marley's own descriptions.
- **Intended license outcome (chad, 2026-07-11):** Marley's editor/terminal layer is expected to be **GPL-licensed**
  (an open-core model — the GPL editor is free; the proprietary **"brain" / agent layer** is the sold product,
  the same shape as Warp). Expert + legal review is the gate before release.
- **The boundary that protects the model:** GPL-derived code stays in the editor/terminal layer; the **brain must
  stay clean of any Zed/Warp-derived code** or it inherits copyleft and can't be sold. Every doc marks
  **provenance** — `[Zed-derived]`, `[Warp-derived]`, `[Marley-original]`, `[public/permissive: gpui Apache-2.0 /
  tree-sitter MIT / LSP spec / CS literature]` — so the legal review is possible and the boundary is auditable.
- **gpui is Apache-2.0** (Zed's UI framework, and Marley's own dependency) — freely usable; not part of the GPL
  copyleft surface.

## Layout
- `subsystems/` — the subsystem overviews (mirrors `warp_architecture/subsystems/`).
- `crates/` — per-crate references (mirrors `warp_architecture/crates/`), editing-relevant crates first.

## Status
Round 1 (2026-07-11) — comprehensive subsystem map + the editing-critical crates. Deeper per-crate coverage
fans out from here.
