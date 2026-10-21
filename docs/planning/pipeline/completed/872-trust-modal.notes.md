# Zed's trust modal as the Dialogs board draws it — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-872-the-trust-modal.md
- **Pipeline spec:** 872-trust-modal.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #871.
- **Classification / tier:** feature, S. One Zed hunk (`workspace/src/security_modal.rs`), a
  Marley module.
- **Recall (§18.3):** #870's prompt hook and button roles; every scenario trusts its project by
  pressing Enter at start, so the modal is on screen in each.
- **Discovery:** `SecurityModal::render` (security_modal.rs:83) builds an `AlertModal` with
  `.header(...)` and `.footer(...)`; `AlertModal::header`/`footer` set an `Option`, so a second
  call replaces the first. `trust_and_dismiss` and `trusted` are private.
- **Decisions:** D1–D3 in the spec.

### Design
- **Zed hunk:** `pub struct MarleyTrustLook(pub fn(&MarleyTrust<'_>, &mut Window, &mut App) ->
  Option<(AnyElement, AnyElement)>)` and `pub struct MarleyTrust<'a> { folders: &'a [String],
  trust: Rc<dyn Fn(&mut Window, &mut App)>, stay: Rc<dyn Fn(&mut Window, &mut App)> }`. In
  `render`, before the `AlertModal`: the folders shortened as Zed's header shows them, the two
  callbacks updating the modal through a weak handle (`trust_and_dismiss`; `trusted = Some(false)`
  and `dismiss`), the hook asked; after Zed's `.header(...)` and `.footer(...)`,
  `.when_some` replaces each. Ledger row.
- **Marley `trust_card.rs`:** `init` sets the hook; `look`: none outside the Marley layout; the
  header (title semibold, the line muted, the folders when several) and the footer (Not Now quiet,
  Trust primary, through `prompts::render_button`).
- **File manifest:** `crates/workspace/src/security_modal.rs` (Zed), `trust_card.rs` (new),
  `marley_workbench.rs`; the scenario.
- **Visual check plan:** REQ-001 at start, and again in Day with a second folder handed over
  (#513); REQ-002 by Enter. Not reached: several folders at once.

## Phase 2 — Code
- **Built:** in `security_modal.rs`, `MarleyTrustLook` and `MarleyTrust`, the hook asked after
  `trust_input` (the folders shortened as Zed's header shows them; `trust` calling
  `trust_and_dismiss`, `stay` setting `trusted = Some(false)` and dismissing, both through a weak
  handle), and two `when_some` calls after Zed's `.footer(...)` that replace the header and the
  footer. Marley's `trust_card.rs` (`look`), registered in `init`.
- **Deviation:** the title names the folder by its last component ("Trust repo?"), its path a
  muted line under it: the first run titled it with the whole path, two lines long outside the
  home folder.
- **Gate:** red once (rustdoc `invalid_html_tags` on a `<name>` in the module comment, reworded), then green.
- **Review:** the callbacks run after the click, so nothing updates the modal during its render;
  Enter and `ToggleWorktreeSecurity` stay on Zed's actions; the body is untouched.


## Phase 3 — Test
- **Scenario:** `script/e2e/872-trust-modal.sh` (sway): `repo` at start, `second` handed over.
- `872-01-night`: "Trust repo?", its path muted, the board's line, Zed's restricted-mode text,
  list and "Trust all projects in" with its field, Not Now quiet and Trust solid orange at the
  right. REQ-001 met.
- `872-02-trusted`: after Enter the modal is gone and the title bar's Restricted Mode badge with
  it. REQ-002 met.
- `872-03-day`: Marley Day, "Trust second?" for the handed-over folder, the same card. REQ-001
  met.
- Not reached: several folders at once ("Trust these N projects?").


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md` (The trust card), look-plan W3.4 done, the ledger row.
- Knowledge: AD-claude-872-trust-modal-ends-by-hook-001. Brain: `brain decide` on consultation
  a9e65840.
- Ticket closed; pipeline archived.
