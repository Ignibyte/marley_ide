# <title> — Notes

- **Local ticket doc:** <docs/planning/tickets/open/TICKET-n-slug.md>
- **Pipeline spec:** <title>.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** …
- **Classification / tier:** …
- **Recall (§18.3):** what the local ledger, the completed-pipeline archive and the brain
  surfaced.
- **Discovery:** the precise edit/file surface.
- **Decisions:** …

### Design
- Approach; file manifest (Marley crate or Zed crate per file); the ledger rows it needs.

### Test plan
- One row per acceptance criterion: unit tests, gpui driven tests, the live drive.

### Risks
- …

## Phase 2 — Code
- What was built; deviations from the plan (with reason); what the review of the diff found.

## Phase 3 — Test
- Tests RUN (with counts) + the gate result; the live drive; negative smokes; pre-existing notes.

## Phase 4 — Complete
- Docs updated; ledger appends (failures / prevention rules / lessons / ADs — codes listed);
  ticket closed; archive; the commit.
