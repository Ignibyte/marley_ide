# Warp Architecture Map — v1 (Ignibyte fork research)

**Date:** 2026-06-27
**Goal:** Fork Warp, strip login, and expand the UI surface to (a) visualize the agentic workflow and (b) spawn/write/read terminal sessions from a custom UI component. Own it via agent-maintained understanding + upstream rebasing.

## Repos on disk
| Path | Repo | Role |
|---|---|---|
| `~/Projects/warp` | `warpdotdev/warp` (blobless, full history) | **Base** — our fork tracks this |
| `~/Projects/warp-refs/openwarp` | `zerx-lab/warp` ("Zap", formerly OpenWarp) | **Primary patch-donor / reference** |
| `~/Projects/warp-refs/warp-offline` | `the1812/warp-offline` | Secondary de-auth reference |

## Licensing
- **MIT** → `crates/warpui`, `warpui_core`, `warpui_extras` (the GPU UI framework — permissive, reusable in a closed product)
- **AGPL v3** → everything else (the client). Our fork inherits AGPL: if we distribute or host it, we must publish source.
- **Proprietary (NOT in repo):** "Oz" cloud agent orchestration, server backend, AI routing (all behind `app.warp.dev` GraphQL). We replace this with our own agent layer.

## Scope
~1.46M LOC of Rust across `crates/` + `app/`, **76 workspace crates**. But the crates relevant to our goals are small and tractable.

## Subsystem map (the crates that matter)
- **UI framework (MIT):** `warpui` (33k), `warpui_core` (77k), `warpui_extras`, `ui_components` (2.5k), `editor` (63k, SumTree-based), `sum_tree`
- **Terminal / session core:** `warp_terminal` (8.9k), `warp_core` (8.8k), `local_control` (2.6k), `command`, `ipc`  ← spawn/write/read lives here, ~20k LOC total
- **Agent / AI:** `warp_multi_agent_client` (**247 LOC**), `ai` (27k), `mcp` (2.2k), `computer_use` (4.3k), `jsonrpc`, `input_classifier`
- **Auth / cloud seam (de-auth targets):** `warp_server_auth` (**1.5k**), `firebase` (**145**), `warp_server_client`, `graphql` (9.3k), `cloud_object_*`, `onboarding` (12k), `managed_secrets`, `websocket`, `warp_web_event_bus`
- **Other:** `vim`, `voice_input`, `lsp`/`languages`/`syntax_tree`, `ipynb_parser` (notebooks), `settings`, `persistence`, `virtual_fs`

## The three hook points (our actual work surface)
1. **Visualize the agentic workflow → a new UI panel.** Template = Warp's existing **Agent Mode** panel (a non-terminal view beside the terminal). Build a sibling on `warpui` (Draw trait + entity tree). Tap `warp_multi_agent_client` (247 LOC) for agent state.
2. **Trigger / write / read terminal sessions.** Lives in `warp_terminal` + `warp_core` + `local_control`. Blocks = per-command PTY-backed grids. We expose the session spawn + PTY write + output-stream APIs to our panel.
3. **Bring-your-own agent.** `mcp` (tools) + `ai` (provider routing) + `warp_multi_agent_client` (orchestration). Zap already wired Claude Code/Codex/etc. into Blocks here.

## De-auth = small + already solved
- Surface: `warp_server_auth` (~1.5k) + `firebase` (145) + `onboarding` + `graphql`.
- **Zap** already removed mandatory cloud/login, added BYOP AI providers (any OpenAI-compatible endpoint, keys local), and wired external CLI agents incl. **Claude Code** into **Blocks** + the notification center. → harvest, don't reinvent.

## Open questions for the agent survey (next phase)
- [ ] Exact file(s) enforcing the login/account gate, and the cleanest neutralization.
- [ ] `warpui` coupling — is it usable/extractable standalone or deeply tied to the app?
- [ ] The session API in `warp_terminal`: precise spawn/write/read entry points.
- [ ] How an Agent Mode panel is registered + rendered (the template for our panel).
- [ ] How Zap wired Claude Code into Blocks (diff Zap vs upstream on `warp_terminal`/`ui_components`/`ai`).
- [ ] The `app.warp.dev` GraphQL contract — what breaks offline, what Zap stubbed.

## Build/run
`./script/bootstrap` (platform setup) → `./script/run` (build+run) → `./script/presubmit` (fmt/clippy/test). Full engineering guide in repo `AGENTS.md`.
