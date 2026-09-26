# TICKET-561 — Programs that open a browser land in Marley's Browser tab

- **Ticket:** LOCAL #561 (feature, prong 3 with prong 1: a follow-up slice of #503)
- **Owner:** spec-drafter-2026-09-26
- **Pipeline doc:** ../../pipeline/queued/561-browser-env-opener.spec.md
- **Source ticket:** The Orca second pass of 2026-09-25 (`docs/planning/design-notes/orca-second-pass-2026-09-25.md`), finding 3; Chad decided on 2026-09-26 that every remaining Orca and Warp finding gets built.
- **Status:** open

## Summary
#503 routes the URLs a terminal prints and the ones Chad clicks. A program that opens a URL
itself (`gh pr view --web`, Vite's `--open`, Python's `webbrowser`, a Jupyter server's token
URL) still opens the system browser, where no agent tool sees the page. `BROWSER` is the Unix
convention for the program that opens URLs, and `gh`, Python, Vite and cargo read it. Marley
ships a small opener beside the plugin's bridge and exports it as `BROWSER` in the environment
its local shells get, only when the user has none of their own. The opener sends the URL and its
working directory to Marley over the MCP endpoint with the bearer; Marley applies #503's rule
and opens a local URL in a Browser tab of the project that owns that directory, with the focus,
and the opener hands anything else to `xdg-open` itself with `BROWSER` unset, so nothing loops
back. Under #507 that tab is in the project's own Chromium, where an agent can then drive the
page.

## Acceptance
`python3 -c 'import webbrowser; webbrowser.open("http://127.0.0.1:<port>/")'` in a project's
terminal opens a Browser tab of that project on the page, with the focus; the same call with
`https://example.com/` or a `file://` URL reaches the fake `xdg-open` and opens no tab; a shell
whose environment already carried `BROWSER` keeps it; a terminal outside every project sends
every URL to the system browser; `marley.terminal_links: system_browser` sends every URL there;
nothing printed into a terminal can open a tab.
