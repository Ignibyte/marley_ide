# TICKET-745 — Threads in the rail

- **Ticket:** LOCAL #745 (feature, the Marley layout: tabs everywhere)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [745-threads-in-the-rail.spec.md](../../pipeline/queued/745-threads-in-the-rail.spec.md)
- **Source ticket:** Chad, 2026-10-11: "we need to have a button that shows all agent threads or
  something again … add a button in the side panel that toggles between the two? with an option on
  the home page as well", and "moving the threads back over to the top left". Plan: [tabs-everywhere shelf](../../design-notes/tabs-everywhere-2026-10-11.md).
- **Status:** open

## Summary
#737's Threads page lists every conversation, but only from a status-bar button, so the list Zed's
sidebar gave (its history switch) reads as gone. The rail's header gets PROJECTS and THREADS where
PROJECTS stands, and `ctrl-g` flips them as it flips Zed's sidebar. Threads lists the conversations
in the rail (title, agent, folder, newest first) with a search field; a click opens one in a tab of
the shown group or brings forward the tab already showing it. The Home page gets a Threads button
that opens the Threads page. The status bar's Threads button leaves. #678 took a Projects | Brain
switch out of this header because the Brain tab already held that tree; conversations have no such
tab, and Chad asked for the switch.

## Acceptance
The switch shows the conversations in the rail; the search narrows them; a click opens one in a
tab, or brings its tab forward; switching back, or `ctrl-g`, shows the projects; the Home page opens
the Threads page; the status bar has no Threads button.
