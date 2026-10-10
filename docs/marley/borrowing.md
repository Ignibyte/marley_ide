# What Marley borrows from other projects

The ongoing list of projects Marley was compared with, what came of each, and what is still open.
A new survey adds a row here and files its ideas as tickets: under **Deliberate** in
`docs/planning/tickets/BACKLOG.md` when they are for later, in the Queue when Chad asks for them
now. We take ideas, never code, unless a ticket says otherwise (CONSTITUTION §20; Warp is
clean-room).

| Project | What it is | Survey | Status | Open |
|---|---|---|---|---|
| **Zed** (`zed-industries/zed`) | The editor Marley forks | `docs/zed_architecture/` (before the fork) | The base since 2026-09-18 | Upstream merges (#695 and later) |
| **Warp** | A terminal with blocks and agents | `docs/warp_architecture/`; design notes from 2026-09-25 (`warp-once-over`, `warp-second-pass`, `warp-blocks-and-natural-language`) | Built: Chad, 2026-09-26, "build every remaining finding"; 87 closed tickets name it | Nothing queued |
| **Orca** (`stablyai/orca`) | An agent IDE with worktrees, a browser and a phone app | `docs/orca_architecture/` (seven reports); `design-notes/orca-second-pass-2026-09-25.md` | Built, the same way; 47 closed tickets | Nothing queued |
| **T3 Code** (`pingdotgg/t3code`) | A control plane for headless coding agents | `docs/t3code_architecture/` (2026-10-06) | 2 of 13 items done (#668, #680 as item 4) | Items 2, 6, 8, 9 and 11 overlap #715, #714, #717, #716 and #718. Not ticketed: 1 (restarts that lose nothing), 3 (past sessions, searchable, with Resume), 5 (Codex gets what Claude Code gets), 7 (follow Omarchy's theme), 10 (the browser tools agents lack), 12 (a Usage tab, near #715), 13 (Claude Code's Remote Control). Seven open questions for Chad, at the end of its README |
| **herdr and Hermes** | An agent multiplexer, and an agent host | `design-notes/herdr-and-hermes-2026-10-02.md` | Direction: agent hosting goes to rustal-harness (Chad, 2026-10-02); harness TICKET-095 reads herdr's panes as seats (`ef803da`) | Harness-side |
| **MonoCode** (usemono.dev) | A desktop shell over agent CLIs | `docs/planning/intake/monocode-findings.md` (2026-10-09) | Eight ideas queued | #714 to #721, Deliberate |
| **plannotator** (`backnotprop/plannotator`) | Plan and diff review for agent CLIs | `docs/planning/intake/plannotator-findings.md` (2026-10-09) | Eight ideas queued | #726 to #733, Deliberate |
| **ASD-STE100 skill** (`danyuchn/asd-ste100-skill`, MIT) | Simplified Technical English for agent-facing text | Rusty's skill store: vendored at `32511c6`, adapted as `rustal-ste` (70-term glossary, 40 verbs, 13 message shapes) | Adopted for messages between agents | #725 (Marley's agents get it), #724 (the shared plugin's copy) |
