---
id: forge#183 (22a3f883-77a7-46ab-b2b7-0c75c4b1ed10)
title: M12 — Tab-completion beyond dirs: PATH commands + shell history (first word)
status: closed
milestone: M12 — The Agent Cockpit
pipeline: 5547a43c-2693-479d-b0ef-a02cc941fe19
---

At the command position (first word), Tab completes $PATH executables + session history (most-recent-first,
deduped) instead of cwd dirs; a later word keeps path completion. Pure is_command_position + merge_candidates;
shim branches complete_at_prompt on the position, gathering a cached $PATH basename set + the history words.
