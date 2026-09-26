# TICKET-503 — Localhost URLs in a terminal open in a Browser tab

- **Ticket:** LOCAL #503 (feature, prong 3 with prong 1 (the terminal))
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/503-terminal-urls-open-in-the-browser.spec.md
- **Source ticket:** Chad, 2026-09-25: "lets do 1 through 8" (item 1 of the list after the browser waves)
- **Status:** open

## Summary
A dev server prints its URL in the terminal, and a click on it opens the system browser, outside Marley, where agents cannot see it. A click on a local URL (localhost, 127.0.0.1, 0.0.0.0, [::1] and the like) opens it in a Browser tab of the terminal's project instead, and the terminal offers the URL a dev server printed, so one click takes the page into Marley.

## Acceptance
Ctrl+click on `http://localhost:<port>` in a terminal opens a Browser tab on that page; other URLs still open the system browser; the terminal offers the last local URL a command printed.
