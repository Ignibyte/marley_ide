# TICKET-586 — A project's `file://` pages open in its Browser tab

- **Ticket:** LOCAL #586 (feature, prong 3 with prong 1: a follow-up slice of #561)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/586-file-pages-in-the-browser-tab.spec.md
- **Source ticket:** #561's open question (`../../pipeline/completed/561-browser-env-opener.notes.md`, "Open for Chad"), answered by Chad on 2026-09-28: "marley browser".
- **Status:** closed

## Summary
#561 exports Marley's opener as `BROWSER` in its local shells: a program that opens a local URL
gets a Browser tab of the project that owns its working directory, and anything else, `file://`
pages included, goes to the system browser. A program in a project's terminal that opens a
`file://` page, such as `cargo doc --open` or a coverage report, now gets a Browser tab of that
project too, whatever folder the file sits in (on the dev box `cargo doc` writes under the shared
target directory, outside the project). Only the opener's path changes: nothing printed into a
terminal opens a tab, a terminal outside every project still sends every URL to the system
browser, and `marley.terminal_links: system_browser` still sends every URL there.

**Changed at plan (2026-09-28).** The pages are HTML files, `.html` or `.htm`: `cargo doc
--open` hands the opener a plain path, not a `file://` URL, so a path opens too. Plan D15 keeps
agents' own navigation to http and https; since `browser_open_url` answers agents as well as the
opener, it opens only an existing HTML file, never a folder or another kind of file.

## Acceptance
`cargo doc --open` (or `webbrowser.open("file://…")`) in a project's terminal opens the page in a
Browser tab of that project, with the focus; the same call from a terminal outside every project,
or with `marley.terminal_links: system_browser`, reaches the system browser.
