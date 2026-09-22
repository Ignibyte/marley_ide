# TICKET-196 — Clickable links (URLs + file paths in terminal output)

- **Forge ticket:** #196 (b76e42db-b8b2-4e44-8ecc-d2bc5e65c1ae) (feature, M12.2)
- **Owner:** c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** 189c5b14-1076-4161-b65f-5aa88cd632fe
- **Pipeline doc:** ../../pipeline/active/clickable-links.spec.md
- **Source ticket:** M12.2 "Terminal fidelity & cockpit UX" (sprint #25); also the front half of the M13 wedge (#212)
- **Status:** closed

## Summary
URLs and file paths in terminal block output become clickable — a URL opens in the system
browser (via a `marley_command` adapter), a file opens in Marley's code view (via
`open_file_in_viewer` + `resolve_under_root`). A pure `scan_links` scanner (cov/MSI 100)
finds the link spans over a rendered line; the render underlines/hover-highlights them and
composes with the #31 styled-run render + text selection. OSC 8 explicit hyperlink escapes
are deferred (the Block model doesn't carry hyperlinks today — a separate follow-up).

## Acceptance
Scanner bounds URLs + file paths (incl. multiple per line, punctuation-aware); a click opens
the URL / file; rendering composes with styled runs + selection; spawn confined to
marley_command. Full EARS (REQ-001..006) in the pipeline spec.
