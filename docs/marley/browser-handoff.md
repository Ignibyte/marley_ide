# Handoff: the embedded browser (Prong 3)

Written 2026-09-24 from a Rusty session, for the agent working on Marley. It gathers what
the record already holds about the browser, a Chromium that both Chad and an agent can drive
("cursor-like"), so nothing has to be rediscovered. It decides nothing new. Where this note
and a source doc disagree, the source doc wins.

## Read these, in this order

1. [three-prong-plan.md](three-prong-plan.md), section "Prong 3: the browser service". This
   is the current plan for the fork: D12 to D15, slices B0 to B5, risks, open decisions 4
   and 5.
2. `docs/planning/intake/embedded-agent-browser-chromium-cdp.md`: Chad's original ask
   (2026-07-09) and the **2026-08-11 amendment**, which is the product spec: one Chromium,
   and the three pillars with their mechanisms.
3. `docs/marley_architecture/embedded-browser-model.md`: the history of the macOS
   WKWebView pane (#402 to #406, emptied by #410), and the Q2 "trigger has fired" note.
   Read it for the lessons, not the substrate.
4. `docs/decisions/chromium-embed-feasibility.md`: the 2026-06-27 CEF off-screen-render
   study. It is the named fallback if the screencast route fails B0.
5. `docs/marley_architecture/orchestration-shell.md` §8–9: the `browser.*` tool family on
   a hosted agent seat, and the rule "don't scrape what you can query".

## What Chad asked for

In his words (2026-07-09): "a web browser that is cursor-like where it understands what its
looking at… i dont mind the 150mb part." The 2026-08-11 amendment makes it two-fold:

1. The agent drives the browser to verify its own work, debug and test.
2. The agent sees what Chad is looking at and describes it, with a layer to draw on the
   page, a "record this" that captures what happened, and an element picker ("click the
   dead accordion, the agent gets that exact element, durably").

Goal 2 is why there is **one Chromium** shared by human and agent: same session, cookies,
scroll and DOM. Two engines would let the agent describe a page Chad is not on.

## The settled shape

- Chromium runs as a user service with its own profile and `--remote-debugging-port` on
  loopback. Marley attaches over CDP to render it; Marley's MCP tools, the Playwright MCP
  and Claude Code attach to the same endpoint. Chad's real Chrome is never touched.
- Rendering is `Page.startScreencast` frames decoded to a gpui image; input goes back
  through `Input.dispatchMouseEvent`, `dispatchKeyEvent` and `insertText`. Page pixels are
  part of gpui's scene, so gpui overlays sit above the page with no hide-shim.
- `marley_browser` is a pure core (CDP JSON-RPC, target and session handling, coordinate
  math from frame metadata, locator ranking, the annotation model, the trace ring buffer)
  with thin adapters for the socket, the decoder and the gpui element.
- The pillars, in order:
  - **A, the element picker**: `Overlay.setInspectMode` plus `inspectNodeRequested`. Each
    pick captures a durable bundle: ranked locators, the AX node, listeners with source
    locations, and click-blocking styles. The signature move is picked element → listener
    source → source map → open the file in the editor at that line. Walk up to the nearest
    interactive ancestor, and pierce shadow DOM.
  - **B, annotations**: drawn by gpui, stored in page coordinates, placed per frame from
    screencast metadata. Nothing is injected into the page for drawing. The only injection
    allowed is ResizeObserver/MutationObserver telemetry in an isolated world.
  - **C, the flight recorder**: a rolling 30–60 s trace of input, frames, console, network
    and AX snapshots; "record this" saves what already happened. Marley already forwards
    every input event, so the action log is almost free.
- Security from the first slice (D15): loopback only, a per-boot token where Chromium
  allows it, a navigation allowlist per project, no `Runtime.evaluate` to arbitrary origins
  without a grant, and headers and tokens redacted from traces before they are written.

## Where it stands

- **Nothing of Prong 3 is built in the fork.** There is no `marley_browser` crate; the
  `marley_*` crates are agent, dcs, fleet, mcp, rail, remote, terminal and workbench.
- The WKWebView pane from the old `/srv/stacks/marley` repo was macOS-only and did not come
  over. Don't port it.
- The plan's order ("Cross-cutting") runs: workbench shell, then T0/T1, then C0, then **B0
  in parallel with C1**, then B1 to B2 alongside the other prongs, then B3 to B5. Check
  where the other prongs stand before starting B0 early.

## B0, the spike

Deliver: Chromium as a user unit, `marley_browser` connecting and screencasting, a pane
that shows a page and takes clicks and keys. It must answer the five questions from the
amendment:

1. IME and composition over `Input.dispatch*`: dead keys, CJK, the return trip. This is
   the highest risk; WKWebView gave it for free and this route rebuilds it.
2. Screencast latency at interactive speed for a human, not only an agent.
3. Whether the `Overlay` inspect highlight shows up in screencast frames.
4. Cross-origin iframes (OOPIF): the main frame's `getFullAXTree` omits them, so `Target`
   handling is needed.
5. Coordinate composition (device pixel ratio × page zoom × screencast scale), taken from
   frame metadata, never hand-rolled.

If screencast fails on latency or fidelity, the fallback is CEF off-screen rendering (the
feasibility study). Record that call with `brain_decide`.

## Lessons from the WKWebView train that still apply

- A callback stored inside a resource its owner holds must capture weakly, or the
  resource's `Drop` never runs (`PR-claude-callback-stored-inside-owned-resource-captures-weak-001`).
- Persist a bare marker and re-derive; never put a URL or other arbitrary payload into a
  layout codec (#403; the grid codec reserves `:` and `=`).
- Transport failures can arrive as silence. #406 needed an out-of-band probe and a
  deadline because the webview reported nothing. Expect CDP to need its own liveness
  handling when Chromium restarts or the target crashes.
- Check subframe navigation as well as the main frame (#405).

## Open decisions (Chad's, not the agent's)

1. Chromium packaging: a Marley-owned user unit, or one shared with the Playwright MCP
   (plan open decision 4).
2. Chad mentioned a written three-prong browser plan that nobody has found in Rusty's repo,
   the ops handbook or the brain (plan open decision 5). Ask him before designing past B0.
3. Picks sent to the agent at once, or staged for Chad to confirm and caption first. The
   intake leans toward staged.
4. Whether annotations persist across sessions, and at what layer.

## Useful context outside this repo

- Browser tooling measured on the dev box, 2026-09-18 (brain `daily/2026-09-18`):
  agent-browser 0.38.1 `-i` snapshots cost about 3,400 tokens, Playwright MCP about 11,900
  plus a 4,600-token schema. Keep this in mind when shaping the `browser.*` tools (B2): an
  interactive-only AX snapshot is the cheap default.
- Brain decision `decisions/marley-on-zed-the-three-prong-plan-block-terminal-control-plane-browser-service`
  records the plan; its follow-up is due 2026-10-16.
- Brain research pages `research/mcp-and-skills/playwright-mcp` and
  `research/mcp-and-skills/browserskill` cover neighbouring tools.
