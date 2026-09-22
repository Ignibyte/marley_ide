# TICKET-240 — Editor surface rail row shows a stable "Editor" label

- **Forge ticket:** #240 (c5033176-df15-48ed-adbb-b3eaa25a8848) (chore, M14 sprint #27)
- **Owner:** session c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** 934cef3b-a029-45ce-99a2-4114c2a24b70
- **Pipeline doc:** ../../pipeline/completed/editor-rail-label.spec.md
- **Source:** chad live feedback #2 (2026-07-10) — the M14 round; 3rd ticket. His pick: stable "Editor" label.
- **Status:** closed

## Summary
The editor surface's left-rail row is frozen at the FIRST file's name (`open_or_switch_code` sets
`Tab::code(first_file_name, …)` and never updates it), so it doesn't reflect the active file. Fix: the editor
tab gets a stable `"Editor"` title; the file-tab strip stays the source of truth for which file. The rail label
flows through `live_tab_title`'s fallback (an editor tab has no terminal), so setting the title = "Editor" at
creation is enough; a #177 rename still overrides; persistence is path-based (unaffected).

## Acceptance
The editor tab's title is "Editor" regardless of which/how many files are open (`open_or_switch_code_cases`);
the strip shows per-file names (`surface.files()`, unchanged); the rail reads "Editor" (driven). Pure seam
cov/MSI 100. Full EARS in the spec.
