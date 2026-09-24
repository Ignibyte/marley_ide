# TICKET-489 — B0b: Typing and clicking in the page

- **Ticket:** LOCAL #489 (feature, prong 3 B0b)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/489-browser-input.spec.md
- **Source ticket:** docs/marley/three-prong-plan.md, prong 3 (D12; the amendment's five questions)
- **Status:** open

## Summary
The input half of the B0 spike. With the Browser tab focused, clicks, drags, the wheel and the
keyboard reach the page through CDP: printable keys as key events with their text, named keys
and Ctrl or Alt chords as raw key events, compose sequences and input-method text through
gpui's input handler, and the system clipboard bridged in both directions. Super chords never
reach the page. Each input's time to the next frame is logged, the spike's latency answer.

## Acceptance
In a fixture page: a click lands where it was made with its button and count, typing and
editing keys act as in Chromium, a compose sequence types its character once, the wheel
scrolls 100 CSS pixels per detent, input reaches a cross-site iframe, Ctrl+V pastes the system
clipboard and Ctrl+C copies the page's selection to it, and the run reports the latency.
