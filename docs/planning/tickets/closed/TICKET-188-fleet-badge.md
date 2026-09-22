---
id: forge#188 (369dae42-9dee-44e8-b34b-8b1600780139)
title: M12 — a live fleet badge: top-bar 🤖 running-count + click → Agents tab
status: closed
milestone: M12 — The Agent Cockpit
pipeline: e99f3f31-61ac-429d-b09d-7ccfac33f371
---

The top-bar Agents cockpit icon gains a small badge counting RUNNING agents (Working+Waiting), hidden at zero;
clicking the icon (already wired) or the footer "n agents · n working" segment opens the Agents cockpit tab.
Pure `fleet_badge(agents) -> Option<String>`; shim renders the badge + the footer-segment click.
