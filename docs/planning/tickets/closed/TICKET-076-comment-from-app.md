# TICKET-076 — comment on a ticket from the app

- **Forge ticket:** #76 `4fdd974f-20c7-40bc-bd3e-a315afd7fcad` (feature, M2.D seq-5, forge-write)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `695c166c-f900-4b4e-b599-737be9c63f63`
- **Pipeline doc:** ../../pipeline/active/comment-from-app.spec.md
- **Source ticket:** forge sprint #12 `c93f9693-5569-4208-bb6a-20d38afec99b` (M2.D — The Controlling Cockpit)
- **Status:** closed

## Summary
Compose a comment at the prompt, shift+cmd+click a forge ticket → post it (the third row modifier:
plain=copy #70, cmd=claim #75, shift+cmd=comment #76). PURE: `prepare_comment` (trim; None if blank). SHIM:
`comment_focused_on` (bg comment_ticket + re-fetch + clear-on-valid-attempt). HARNESS: a cmdshiftclick verb.
Reuses #74's closed write set (localhost, bearer never logged). cov/MSI 100 on prepare_comment; 2 critics.
Deps #74 + #75 + #69 + #72.

## Acceptance
prepare_comment at cov/MSI 100 (Some/None on blank); shift-cmd-click posts a comment (self-test + raw forge
read); plain/cmd click does NOT comment; FULL gate GREEN. Full EARS in the spec.
