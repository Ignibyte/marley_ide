# TICKET-746 — Marley and Rusty in the title bar

- **Ticket:** LOCAL #746 (feature, the Marley layout: tabs everywhere)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [746-marley-and-rusty-in-the-title-bar.spec.md](../../pipeline/completed/746-marley-and-rusty-in-the-title-bar.spec.md)
- **Source ticket:** Chad, 2026-10-11: "the compromise is Rusty and Marley go to the top right next
  to (?) and Avatar instead. That gives them more spotlight without mucking with the left panel tabs
  that should only be for swapping." Plan: [tabs-everywhere shelf](../../design-notes/tabs-everywhere-2026-10-11.md).
- **Status:** closed

## Summary
#739 put Marley and Rusty in the status bar's right. They move to the title bar's right, after the
guide's `?` and before the user menu, in both layouts, under the same rules: Rusty only while Rusty
is on and connected, Marley only while its agent entry is there, and each opens or brings forward
its tab in the shown group. Home stays in the status bar. Zed's title bar has no seam for another
crate's buttons, so a small hook joins the `?`'s hunk in `title_bar.rs`, with its ledger row.

## Acceptance
The title bar shows Rusty and Marley beside the `?`; each opens its tab; Rusty leaves the title bar
when Rusty is turned off; the status bar no longer carries them; both layouts show them.
