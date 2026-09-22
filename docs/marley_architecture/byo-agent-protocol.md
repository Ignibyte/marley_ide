# BYO-Agent Protocol — vendoring the agent contract (step 3 groundwork)

> **Status (2026-07-12) — SUPERSEDED; historical.** Marley did **not** vendor Warp's proprietary Agent Mode protobuf contract. The BYO-agent goal was met by a different design that made this plan moot: instead of owning `warp_multi_agent_api`, Marley runs agents as **CLI subprocesses in terminal panes** (`marley_agent`, an INVENT crate — `agent_kind_of` / `AgentRun`; ⌘⇧A launches Claude/Codex, ⌘⇧S sends the prompt, ⌘⇧G broadcasts, and the pump observes each session's status) and reaches models by **direct provider streaming**, replacing `warp_multi_agent_client` / `warp_graphql` outright (see the crate-map INVENT row). No `warp-proto-apis` was pulled into the workspace — there is no `marley_agent_proto` / `vendor/` crate, and no `warp_multi_agent*` dep in `Cargo.toml`. Nothing in the plan below was wired in. Current state: [marley_agent.md](marley_agent.md), [crate-map.md](crate-map.md).

> Marley plan doc. Status: **reference cloned; not yet wired into the build.** Wiring happens AFTER a clean baseline build (so the change is verifiable).

## Why this exists
Marley's whole agentic surface (Agent Mode) talks to Warp's proprietary cloud ("Oz") over a protobuf contract that lives in **out-of-repo git dependencies**. To host our *own* agent backend (the bring-your-own-agent goal), we must **own** that contract — vendor or re-spec it — rather than depend on Warp's pinned git revs. The round-2 survey flagged this as the long pole for the BYO-agent goal.

## The two external git deps (what Warp pulls at build time)
From `Cargo.lock`:

| Package | Source | Rev | Used by |
|---|---|---|---|
| `warp_multi_agent_api` | `github.com/warpdotdev/warp-proto-apis.git` | `ac1af730` | `ai`, `app` (`warp`), `integration`, `persistence`, `warp_multi_agent_client` |
| (session sharing) | `github.com/warpdotdev/session-sharing-protocol.git` | `b30fdd06` | `cloud_objects` (`Role`/`ProfileData` for `drive::sharing`) |

## Reference clones (on disk, outside the Marley build)
- `~/Projects/warp-refs/warp-proto-apis` @ `ac1af73` — **15 `.proto` files** under `apis/multi_agent/v1/`: `request`, `response`, `task`, `todo`, `orchestration`, `skill`, `conversation_data`, `input_context`, `suggestions`, `citations`, `attachment`, `lsp`, `file_content`, `document_content`, `options`. This is the complete client↔server Agent Mode contract.
- `~/Projects/warp-refs/session-sharing-protocol` @ `b30fdd0`.

## Plan (after baseline build is green)
1. Copy `warp-proto-apis` into the Marley workspace (e.g. `crates/marley_agent_proto/` or `vendor/`), switch the workspace dep from the git source to a **path** dep, rebuild to confirm parity.
2. Same for `session-sharing-protocol` if we keep Drive sharing.
3. Once owned: the `request.proto` / `response.proto` pair is the seam where Marley points Agent Mode at **our** backend instead of `app.warp.dev` — pairs with the de-auth `AuthProvider` seam (see [de-auth-and-login-stub.md](./de-auth-and-login-stub.md)) and `warp_multi_agent_client` [STUB] (see [../crates/warp_multi_agent_client.md](../crates/warp_multi_agent_client.md)).

## Why not wired in yet
Re-pointing 5 crates' deps from git→path is a build-affecting change. We do it **after** confirming vanilla Marley builds clean, so any breakage is unambiguously ours.
