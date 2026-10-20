# A system prompt for the Marley agent — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-824-a-system-prompt-for-the-marley-agent.md
- **Pipeline spec:** 824-a-system-prompt-for-the-marley-agent.spec.md

## Phase 1 — Plan
- **Request:** see the ticket; the default taken under Chad's /goal, his question unanswered.
- **Recall (§18.3):** #698 keeps the Marley agent to its own tools (`DISALLOWED_TOOLS`,
  `marley_tools_kept_out`), so the prompt's "what you do" must match that list; #683's and
  #696's scenarios check three phrases of the texts. Rusty's brain (consultation
  `3f67ed50d53e4cceb441bd4a6a0abab3`): nothing on this seam.

### Design
- `crates/marley_workbench/src/assistant.rs` (Marley crate): `macro_rules! answering` gives the
  shared section as a literal; `INSTRUCTIONS` and `RUSTY_INSTRUCTIONS` are `concat!`s of their own
  parts and it.

### File manifest
| File | Crate | Change |
|---|---|---|
| `crates/marley_workbench/src/assistant.rs` | Marley | the two texts |
| `script/e2e/824-a-system-prompt-for-the-marley-agent.sh` | — | the visual check |

## Phase 2 — Code

- **Built:** `answering!()`, a macro giving the shared section as a literal, and `INSTRUCTIONS`
  and `RUSTY_INSTRUCTIONS` as `concat!`s: the Marley agent's in four parts (who it is, What you
  do, How you answer, What you leave to others), Rusty's with What you do, How you answer and What
  you leave to others. The tool list is the one #698 leaves it. No deviation.
- **Review:** the three phrases older scenarios check are kept; the text reaches all three paths
  unchanged (`_meta`'s `systemPrompt.append`, Codex's `developer_instructions`, the terminal's
  instructions file), since each reads the same constant.
- **Gate:** see Phase 4.

## Phase 3 — Test
- **Scenario:** `script/e2e/824-a-system-prompt-for-the-marley-agent.sh`, one run, both checks
  pass. The run printed the prompt's non-bullet lines: "You are Marley's own agent: the guide and
  the setup hand inside Marley…", "What you do", "How you answer", "What you leave to others";
  none of the six wanted phrases missing.
- **Shot:** `824-01-thread` (REQ-001): the Marley thread under `repo`, the question sent and the
  stand-in's answer; the prompt itself is not drawn anywhere, so the run's check of the log is
  the proof. Not reached: a real model's answers in the new style (no model in a run).

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md`, `docs/marley/guide.md` (The Marley agent).
- **Knowledge:** none new.
- **Brain:** decision recorded on consultation `3f67ed50d53e4cceb441bd4a6a0abab3`.
