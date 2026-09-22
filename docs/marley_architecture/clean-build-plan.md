# Marley — Clean-Build Plan (the owned product)

> **Status (2026-07-12) — LARGELY REALIZED; historical as a milestone plan.** The North Star and the four-bucket method held, and the product shipped through **M15** ([../../CHANGELOG.md](../../CHANGELOG.md)) — far past the M0–M5 sequence below. **DONE:** the clean-room terminal MVP (Blocks / input editor / palette / themes / completions), the project model, the net-new panel system (terminal + Forge detail panes + splits), agent orchestration (`marley_agent` launches / observes / controls agent CLIs — ⌘⇧A launch, ⌘⇧S send, ⌘⇧G broadcast), a remote **ssh** pane (`marley_remote`), and a full **editable editor** (M15 — see [editor.md](editor.md)). The "Open decisions" are settled: the owned product kept the name **Marley** in **this** repo on **GPUI**. **STILL AHEAD (unbuilt):** the embedded **Chromium / CEF** browser pane, brain-as-MCP as a local/remote deployment flag, and the **ops** surfaces (k8s / Acquia) + agents-on-web runners. Current state: [app_shell.md](app_shell.md), [crate-map.md](crate-map.md), [editor.md](editor.md). **Amended 2026-08-09 (the scrap-forge pivot, #409/#410/#411):** the **Forge** halves of this plan are **retired** — the shipped Forge panes/overlay/client were ripped out (`marley_forge_client` deleted at #411) and the CONSTITUTION preamble's product thesis now reads **project + agents + ops**; every "+ Forge" mention below is the 2026-06 plan as written, kept as history.

> **Status:** planning / vision captured 2026-06-27. This is the plan for the **clean-room, owned** product — **this repository** (`~/Projects/ignibyte/Marley`). The AGPL Warp fork that guides the build (prototype + reference/spec) lives **separately** at `~/Projects/warp-refs/marley-fork`; its source is **never** the source the clean code is copied from. See [licensing-ownership-strategy.md](licensing-ownership-strategy.md) for why.

## Two Marleys (don't confuse them)
- **Marley (fork)** — `~/Projects/warp-refs/marley-fork`, the AGPL Warp fork. Working prototype, de-authed. Its purpose: prove the UX, and serve (with the `docs/` we wrote) as the **clean-room spec/reference**. Internal/dogfood only; AGPL — never the source the clean code is copied from.
- **Marley (clean build)** — **this repository** (`~/Projects/ignibyte/Marley`), the owned product, **clean-room** so Ignibyte owns it outright (no AGPL). This doc plans *that*.

## North Star
A **clean, crisp, Rust, UI-first agentic dev cockpit** with the power and feel of Warp — *nearly identical outcome* — but where the **terminal is one flexible pane among many** and the product is the **project + agents + brain + Forge + ops** layer on top *(the Forge half retired at #409–411; the preamble now reads "project + agents + ops")*. Own it; escape copyleft via clean-room.

## The product (what we're building)
- **Project model** — open a project = a **git repo or local folder**; the project owns its **agents, sessions, and Forge state** *(the Forge-state half retired #411)*. (Warp has no project concept — this is ours.)
- **Panes with Forge detail pages** *(the Forge-page half retired #411)* — a **net-new panel system**: a pane can be a terminal, an embedded **Chromium browser** (CEF OSR — feasible; DRM out of scope, see [chromium-embed-feasibility.md](chromium-embed-feasibility.md)), an **agentic-workflow visualizer**, or a **Forge detail page** (tickets/sprints/RLM/knowledge from the forge system).
- **Agent orchestration** — the **brain delegates tasks down to agents** scoped to the project. Two first-class isolation modes: **git worktree** and **1-agent = 1-folder + full setup** (the preferred pattern). Agents **eventually run on the web** (remote runners).
- **Brain as an MCP server** — local (in-app) **or** remote, as a deployment flag, not a rewrite. Lineage: the forge RLM / Rusty brain *(a lineage note; Forge itself was scrapped — #409/#411)*.
- **Ops** — manage **Kubernetes clusters** and **Acquia (via Acquia CLI)** from panes/agents. (Ties to [[Lagoon-in-Rust]] + the Forge Delivery Platform *(tie retired #411)*.)
- **The terminal substrate** — Warp-like **Blocks**, command model, input editor, palette, themes, completions — reimplemented clean.

## The method: four-bucket triage (not 77 rewrites)
Every Warp crate triages into one of four buckets. The detailed per-crate verdict lives in [crate-triage.md](crate-triage.md).

| Bucket | What | Examples | Effort |
|---|---|---|---|
| **REUSE** (permissive foundation — don't rewrite) | The hardest crates, already MIT/Apache | GPU UI: **GPUI** (Apache) or `warpui` (MIT); terminal core: **`alacritty_terminal`** (Apache); rope/editor | integrate only |
| **REIMPLEMENT** (Warp *behavior*, from our spec) | The Warp-specific UX, written fresh in clean Rust | Blocks, command model, input editor, palette, completions, themes | the real build |
| **SKIP** (Warp cloud — delete, don't need) | ~half of Warp | auth/Firebase, cloud-objects/Drive, the proprietary agent protocol, telemetry, onboarding, the channel matrix | none |
| **INVENT** (the Ignibyte layer — the moat) | No Warp equivalent → zero clean-room concern | project model, panel system, agent orchestration, brain-MCP, ops (k8s/Acquia), Chromium pane | where value lives |

**Reuse the bottom, reimplement the middle from the spec, delete Warp's cloud, pour effort into the top.**

## Clean-room discipline (the one rule that fixes the license)
- Implement bucket REIMPLEMENT from the **spec** — our [architecture](../architecture/) + [crates](../crates/) docs and observed behavior — **not** by reading/translating the AGPL source line-by-line (a reworded translation is still a derivative work).
- **REUSE** crates (GPUI/Apache, `alacritty_terminal`/Apache, `warpui`/MIT) are unrestricted — use freely with attribution.
- The fork + Zap are **guides to understand behavior**, never the code you ship.
- **Forced per-spec (`enforce-warp-reference.sh`, TICKET-248)** — every pipeline spec carries a `## Reference (§20)` section naming the reference app (Warp for the terminal/cockpit/UX; Zed for the editor — a same-`gpui`-stack reference) + how Marley matches its BEHAVIOR, or `N/A — Marley-specific + why`. Plan fills it, design confirms it, and the commit hook blocks an empty one. Observed captures live in [../warp_architecture/observed/](../warp_architecture/observed/) (committed, not scratchpad). Codified in CONSTITUTION §20.
- **The source itself is brand-clean (TICKET-262, M16)** — the audited brand-scrub reworded every whole-word Warp/Zed mention in `crates/**/*.rs` (56 hits, all comments) into Marley's own terms, and gate:14 now fails on any new whole-word brand mention in source (`grep -rniwE 'warp|zed' crates --include='*.rs'`; the `docs/*_architecture/` reference transcriptions are the deliberate, scope-exempt exception). The `color.rs` palette attestation asserts clean-room originality without naming a brand ("independently authored, not lifted from any AGPL-licensed source"). Audit + catalogs: `docs/planning/design-notes/brand-scrub/`.
- Get IP-counsel sign-off before commercializing (see [licensing-ownership-strategy.md](licensing-ownership-strategy.md)).

## Foundation picks (decide before M0)
- **UI framework:** **GPUI** (Apache, battle-tested standalone, same Sobo lineage) vs extracting `warpui` (MIT but coupled to the Warp workspace). **Lean GPUI.**
- **Terminal core:** `alacritty_terminal` (Apache) — Warp adapted Alacritty too.
- **Agent engine:** omp / Claude Code via **MCP** ([[omp-capabilities]]).
- **Brain:** MCP server (forge RLM lineage *(retired lineage — scrap-forge #409/#411)*), local-or-remote from day one.

## Sequencing (milestones)
> The M0–M5 below is the ORIGINAL plan (historical). **The live forward sequencing — M0–M15 done + the expanded
> phases (cleanup → the Zed-level editor frontier → fusion → brain → browser → ops) — now lives in
> [roadmap.md](roadmap.md).**

- **M0 — Foundation spike:** GPUI window + `alacritty_terminal` PTY + render one Block. Proves the stack.
- **M1 — Terminal MVP:** Blocks, input editor, run commands, panes/splits, palette, themes. "Clean crisp Warp-like terminal."
- **M2 — Project + first Forge pane:** project model (repo/folder), the panel system, one Forge detail page. *(Shipped, then retired at #411 — the scrap-forge rip.)*
- **M3 — Agent orchestration:** brain delegates; folder-per-agent **and** worktree isolation.
- **M4 — Brain-MCP + Chromium pane:** brain as in-app MCP; embedded browser pane (live preview / agent browsing).
- **M5 — Ops + remote:** k8s + Acquia panes/agents; agents-on-web (remote runners); brain optionally remote.

## Open decisions
- **Name & repo** of the clean build (keep "Marley" for the owned product? new repo location?).
- GPUI vs `warpui` foundation (pending a coupling check).
- Default brain deployment (in-app vs remote).
- How much of the fork's de-auth/Forge experiments to port as throwaway spikes vs build clean from M0. *(Settled long since; Forge itself later scrapped — #409/#411.)*

## Related
[crate-triage.md](crate-triage.md) · [../architecture/00-overview.md](../architecture/00-overview.md) · [../crates/README.md](../crates/README.md) · [chromium-embed-feasibility.md](chromium-embed-feasibility.md) · [licensing-ownership-strategy.md](licensing-ownership-strategy.md) · [byo-agent-protocol.md](byo-agent-protocol.md)
