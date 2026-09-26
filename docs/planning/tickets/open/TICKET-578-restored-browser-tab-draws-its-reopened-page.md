# TICKET-578 — A restored Browser tab draws the page it reopens

- **Ticket:** LOCAL #578 (bug, prong 3, after #494)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** (none yet)
- **Source ticket:** found in TICKET-576's Test, 2026-09-26 (the shot `576-03-a-again`)
- **Status:** open

## Summary
A Browser tab restored at a launch paints before its page is back, and `BrowserView::start_viewing` then counts it among the viewers of its saved page id, which the browser no longer has when Marley's Chromium stopped. When the tab reopens its saved URL in a new page, `show_page` gives it the new page's id but leaves `viewing` set, so `start_viewing` returns early from then on and the new page never gets the tab as a viewer: its tab stays blank, though its title, its URL and `browser_tabs` say the page loaded. #494's scenario did not show it because it clicked a tab behind the one in front, whose first paint came after its page did. `show_page` should move the tab's viewing from the old page to the new one.

## Acceptance
After Marley and its Chromium stop and Marley starts again, a restored Browser tab in front of its pane draws the page it reopened.
