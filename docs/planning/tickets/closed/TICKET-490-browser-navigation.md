# TICKET-490 — B1a: The address bar and navigation

- **Ticket:** LOCAL #490 (feature, prong 3 B1a)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/490-browser-navigation.spec.md
- **Source ticket:** docs/marley/three-prong-plan.md, prong 3 B1
- **Status:** closed

## Summary
The Browser tab gets the chrome a person needs to browse: an address bar (a Zed single-line
editor) that takes a URL, a host or a search, back, forward, reload and stop, a loading state,
the browser keys (Ctrl+L, Alt+Left and Right, Ctrl+R, F5), and JavaScript dialogs drawn by
Marley, since headless Chromium draws none and a page waiting on one would freeze.

## Acceptance
From the address bar the user reaches a page by URL, by host and by search; back, forward,
reload and stop act on the page's history; the loading state shows while a page loads; alert,
confirm and prompt appear over the page and answer it; a navigation made by an agent shows in
the address bar.
