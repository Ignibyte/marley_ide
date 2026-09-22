# TICKET-075 — claim a ticket from the Forge pane

- **Forge ticket:** #75 `bc3d5e08-5079-4eaa-a8a1-e66f5c37ca21` (feature, M2.D seq-4, SECURITY-sensitive)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `2f1aa154-357d-4a21-8331-8fd1560258fd`
- **Pipeline doc:** ../../pipeline/active/claim-from-pane.spec.md
- **Source ticket:** forge sprint #12 `c93f9693-5569-4208-bb6a-20d38afec99b` (M2.D — The Controlling Cockpit)
- **Status:** closed

## Summary
cmd+left-click a ticket in the ⌘⇧F Forge overlay → claim it (the first write-from-the-UI). PURE:
TicketView.id + TicketRow.id/sprint_rows (thread the UUID). SHIM: MARLEY_OWNER + a bg-thread claim +
re-fetch (#69/#72) + the cmd-modifier branch on the #70 handler. HARNESS: a cmdclick drive verb. Only
claim_ticket (#74 closed set), localhost + bearer-guarded. cov/MSI 100 on the id-threading; the claim
masked + self-test-verified. 2 critics. Deps #74 + #70 + #69.

## Acceptance
TicketView.id parse + sprint_rows.id at cov/MSI 100; cmd-click claims (self-test + raw forge read shows
owner==MARLEY_OWNER; plain click still copies); FULL gate GREEN. Full EARS in the spec.
