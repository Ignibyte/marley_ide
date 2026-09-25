# TICKET-488 — B0a: A Browser tab that shows Marley's own Chromium

- **Ticket:** LOCAL #488 (feature, prong 3 B0a)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/488-browser-pane.spec.md
- **Source ticket:** docs/marley/three-prong-plan.md, prong 3 (D12, D13, D16)
- **Status:** closed

## Summary
The first slice of the browser. `marley: open browser` starts Marley's own Chromium (a
transient user unit, a profile under Marley's data directory, CDP on loopback) and opens a
Browser tab in the main area that shows the page Chromium renders, at the tab's size, with
the page's title on the tab. A new crate, `marley_browser`, owns the service start, the CDP
client and the frame decoding; the tab lives in `marley_workbench`. Agents that attach to the
same Chromium drive the page the tab shows.

## Acceptance
The tab shows a fixture page with its cross-site iframe and follows another CDP client's
navigation and highlight; it re-lays the page out when the tab's size changes; closing and
reopening the tab shows the same page from the same running Chromium; with no Chromium, the
tab says so.
