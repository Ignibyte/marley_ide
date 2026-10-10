# A new agent on a remote host — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-741-a-new-agent-on-a-remote-host.md
- **Pipeline spec:** 741-a-new-agent-on-a-remote-host.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-10)
- **Request:** Chad, 2026-10-10, the agents-anywhere plan
  (`docs/planning/design-notes/agents-anywhere-2026-10-10.md`), with the goal "lets make tickets
  and build it".
- **Classification:** feature.
- **Recall (§18.3):**
  - The guide's own note (`guide.md:2378`) and AD at `architecture-decisions.md:4444`: Views' commands run on this machine only.
  - `rh attach WORKSPACE` takes a workspace alias or id and needs a terminal; `seat start` waits up to 60 s for the seat's first report (TICKET-109).
  - #541's spawn-site rule: the ssh runs as a terminal's task, not a new spawn site.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **The design** is written at promotion (`/pipeline:plan`), when every cited seam is checked
  against the code again.
