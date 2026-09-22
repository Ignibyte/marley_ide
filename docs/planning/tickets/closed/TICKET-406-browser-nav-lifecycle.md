# TICKET-406 — Browser nav chrome + lifecycle: reload, loading state, error page, teardown (the #389 train slice-4)

- **Forge ticket:** #406 `f72268be-1018-4e0b-8843-10d70cd718a2` (feature, M29)
- **Owner:** claude (promoted by /work 2026-08-08)
- **AAR:** deferred — forge unreachable at promote (connection refused); open on first reachable capture
- **Pipeline doc:** ../../pipeline/active/406-browser-nav-lifecycle.spec.md
- **Source ticket:** sprint M29 (forge sprint #40 `6dff19c6-fb61-472b-89c3-98d3215a489b` — "The Embedded
  Browser: proof-of-embed → Forge-URL pane", the #389 train slices 1–4)
- **Status:** closed (2026-08-08 — shipped through the full pipeline; gate GREEN [diff]; forge
  `ticket-close` deferred, server unreachable)
- **Depends:** #403 (`Content::Browser` residency + `TabContent::Browser` + the `B` tag) and #405 (the
  wry child on the pane rect + the z-order hide shim) — both **HARD**; transitively #402's GO verdict
  and #404's `forge_web_base` seam (both consumed via #405). /work must not promote this while #403 or
  #405 is unshipped. Verified 2026-08-06: none of the train is in the tree yet (`TabContent` has no
  Browser variant, tabs.rs:25-36; `Content::Browser` is declared-uninhabited, content.rs:48-49; no
  `wry` in any Cargo.toml) — Phase 2 re-binds every dep cite to what actually shipped.
- **React-first:** UI-AFFECTING — Zone B pane chrome (the loading / error+retry states + the
  header-row question). Full section in the queued spec.

## Summary
The #389 train's slice-4 — the closer that makes the Forge pane livable and its lifecycle honest.
Three halves. **(a) Nav chrome:** a "Browser: Reload" palette verb (static `CommandId`, the
`cockpit_commands()` + `action_for_command` mechanism; whether a slim pane header row earns its pixels
is a Phase-2 D-OPEN answered by the React prototype — recommend no), a loading state while the page
loads, and an honest in-pane error page when the origin is unreachable — clamped reason + a retry that
re-derives the origin (the #384 misconfigured-not-silent house pattern; Forge being down ≠ a broken
pane, and ≠ the no-config empty state). **(b) Lifecycle/teardown:** closing the Browser tab — the last
view of the Browser content — takes the webview OFF the render path deterministically: every close
path routes `release_view` (`#[must_use]` consumed at every site), the last-view return hands the
webview to teardown, detach-from-the-view-hierarchy + drop — never a dangling native child painting
over gpui (the worst failure mode: a wry child composites ABOVE the whole Metal scene). The PTY-reaper
analog: #348's bounded-reap contract is the template if teardown goes off-thread; #397's inline editor
drop is the contrast; Phase 2 decides from measured WKWebView teardown cost. **(c)** The #405
hide-on-overlay / hide-on-tab-switch interactions HARDEN here: one pure state machine owns
loading × error × visibility, and the driven matrix covers overlay-over-pane, tab away/back,
close-while-loading, close-while-hidden, overlay-then-close.

## Out (other tickets / later)
Address bar, arbitrary navigation, user-typed URLs (pinned origin stands); back/forward history
(deferred-not-forgotten — marginal under a pinned origin); the CDP agent-browser lane (separate
pillar); any cockpit changes; the split-cell grid `b` leaf; redesigning the #405 hide shim itself.

## Headline acceptance
Loading shows during a load and clears on finish; an unreachable origin renders the error page
(reason + retry) and retry recovers once the origin returns (driven against a controllable loopback
origin); the reload verb reloads; every Browser close path routes `release_view` with the return
consumed, and the last close provably removes the webview from the view hierarchy — including
close-while-loading and close-while-hidden (driven + the inspect teardown-leak walk); the pure
loading/error state machine + reload/teardown decision table stand at cov/MSI 100; the React↔Marley
parity pair is captured. Full EARS in the pipeline spec.
