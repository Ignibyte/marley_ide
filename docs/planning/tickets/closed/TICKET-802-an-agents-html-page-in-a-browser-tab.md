# TICKET-802 — An agent's HTML page in a Browser tab

- **Ticket:** LOCAL #802 (feature, agent components)
- **Owner:** b6416b37-a6bf-4172-91e4-8d05bc38cda5
- **Pipeline doc:** ../../pipeline/completed/802-an-agents-html-page-in-a-browser-tab.spec.md
- **Source ticket:** Chad, 2026-10-17: "So the options can be inline editing for agents to show components or static html in a scratch file or existing project for that sort of style."
- **Status:** closed

## Summary
For what the catalog cannot draw, an agent writes a page. A tool, `browser_show_page`, takes the
page's HTML and writes it to a scratch folder Marley keeps per project, or takes the path of an
HTML file already in the project. It opens the page in a Browser tab of the caller's project
(as `browser_open_url` does for a local file since #586) and reloads the tab when the file
changes. Marley keeps a stylesheet of its theme's colors and fonts as CSS variables, which the
tool's answer names, so a page can look like Marley.

## Acceptance
A page the tool writes opens in a Browser tab, takes Marley's colors through the stylesheet, and
reloads when the agent writes the file again.
