# TICKET-204 — command-palette workflows: save + re-run named commands (bounded v1)

- **Forge ticket:** #204 (cb29fc31-4d22-4121-9375-1fc376a5c84c) (feature, M12.2)
- **Owner:** autonomous (/work 195-222)
- **AAR:** 23fd1e42-fe64-4c50-bc02-b5d19865da72
- **Pipeline doc:** ../../pipeline/active/warp-workflows.spec.md
- **Source ticket:** sprint #25 (f3ecc094) — M12.2 cockpit polish
- **Status:** closed

## Summary
A Warp Workflows analog: save a command under a name (with optional `{{param}}` placeholders) and
re-run it from the command palette. Bounded v1 — the pure `substitute(template, args)` engine + a
`Workflow{name,command,params}` model (new `workflows.rs`, cov/MSI 100), a marley_settings
`Workflows: Vec<Workflow>` round-trip mirroring the #87 `RemoteHosts` pattern, and a masked shim: the
palette lists saved workflows (a `WORKFLOW_BASE` dynamic range like #199 themes), invoking one inserts
its command into the prompt buffer (substitute for no-param; the template to fill inline for param
workflows), and a "Save command as workflow…" entry names + persists it. The interactive multi-param
prompt modal + an edit/delete UI are deferred to a follow-up.

## Acceptance
Save `deploy = git push origin` as a workflow → it appears as "Workflow: deploy" in the palette and
persists across a reload; invoking it inserts `git push origin` at the prompt ready to run; a
`{{param}}` workflow inserts its template to fill inline. Full EARS criteria (REQ-001…006) live in the
pipeline spec.
