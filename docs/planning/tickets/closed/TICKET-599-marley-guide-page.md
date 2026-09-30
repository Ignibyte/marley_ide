# TICKET-599 — The Marley guide as a page, opened from a ? in the title bar

- **Ticket:** LOCAL #599 (feature, workbench shell: help)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** ../../pipeline/completed/599-marley-guide-page.spec.md
- **Source ticket:** Chad, 2026-09-30: "Lets create a documentation html page that is a detailed documentation guide static html (for now) and then at the top where the + below the sign in lets add a ? that opens it in a browser. Fill this out entirely with everything that we've done. Have a summary of what it is and then detailed explanation on how to do it." He chose the title bar beside Sign In, and Marley's Browser tab, with the system browser when no project is open.
- **Status:** closed

## Summary
Marley's user documentation lives in Markdown files in the repository (`docs/marley/guide.md`,
which stops at #516, and `docs/marley/walkthrough.md`), which a user of the installed app never
sees. Marley ships one self-contained HTML page, its guide: for every feature a short summary of
what it is, then a detailed how-to, current through everything shipped. A `?` button in the title
bar, beside Sign In, and the command `marley: open guide` open it in a Browser tab of the project
on screen, or in the system browser when no project is open.

## Acceptance
Clicking the `?` in the title bar opens the guide in a Browser tab beside the work; a second click
brings the same tab forward; every feature area reads as a summary followed by steps, and nothing
shipped through #598 is missing.
