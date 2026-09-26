# TICKET-503 — Localhost URLs in a terminal open in a Browser tab

- **Ticket:** LOCAL #503 (feature, prong 3 with prong 1 (the terminal), slice 1 of 2)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/503-terminal-urls-open-in-the-browser.spec.md
- **Source ticket:** Chad, 2026-09-25: "lets do 1 through 8" (item 1 of the list after the browser waves), with the Orca survey's details for #503 folded in (`docs/orca_architecture/README.md`, "What it changes in the queued sprint"; report 03 §2.3 and item 5; report 05 §2.4 and item 2)
- **Status:** closed

## Summary
A dev server prints its URL in the terminal, and Ctrl+click opens it in the system browser,
outside Marley, where agents cannot see it. Ctrl+click on a local URL (`localhost`, a name under
`.localhost`, 127.0.0.1, [::1], 0.0.0.0 or [::]) now opens it in a Browser tab of the terminal's
project, with 0.0.0.0 and [::] opened as loopback; other URLs still go to the system browser, and
Shift+Ctrl+click takes the other destination for one click. A setting on Marley's settings page
picks the default. The terminal's footer offers the local URL a program printed, only while
something listens on its port, with a menu of Browser tab, system browser and Copy. URLs printed
in an SSH session or a remote project's terminal always go to the system browser. The popover on
a link in the grid and the stitching of URLs a TUI wrapped or boxed are a second slice.

## Acceptance
Ctrl+click on `http://localhost:<port>` in a terminal opens a Browser tab on that page, and a
second click brings the same tab back; Shift+Ctrl+click and non-local URLs open the system
browser; the footer offers a printed local URL while its port listens and drops it when the port
closes; an SSH terminal's URLs never open a Browser tab.
