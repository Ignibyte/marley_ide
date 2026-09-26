# TICKET-566 — What a stopped turn needs

- **Ticket:** LOCAL #566 (feature, prong 2; the Jev note's use 1, on #519's seats and #565's layer)
- **Owner:** spec-drafter-2026-09-26
- **Pipeline doc:** ../../pipeline/queued/566-stop-kind.spec.md
- **Source ticket:** Chad, 2026-09-26, approving the seven ranked uses of
  `docs/planning/design-notes/jev-system-one-2026-09-25.md`; use 1, "What a stopped turn
  needs", the first use the note recommends turning on, under his rules "local first and then
  jev second" and every use off by default with its own switch and mode
- **Status:** open

## Summary
When Claude Code's turn ends with `Stop`, its rail row reads `idle` and shows the last message
(#519). This use says what the stop needs from Chad: done with evidence, done by claim only, asks
you, blocked, still going, or cannot tell. Code decides first, from the turn's own events: a
message that asks a question is `asks you`, a permission asked and never finished is `blocked`,
an interrupt is `interrupted`, and `done with evidence` is allowed only when a command ran after
the turn's last edit and did not fail. The model is asked the rest through #565's layer, with the
state Chad allowed: the prompt and the message cut to 300 characters and masked, the tools' names
and counts, and the facts as a phrase; a noul per part of the prompt says which parts the message
covers. The kind is a label on the seat, so `fleet_snapshot`, #538's banner and #542's order can
read it. Off by default; in `shadow` it is only logged; in `suggest` the word follows `idle`
with a question mark; in `act` it replaces `idle`. Nothing acts on it.

## Acceptance
With the use in `act` and the layer on the `replay` provider, a stop whose replay answer is
`done_checked` at 0.91 reads `done · checked` on the row, one whose message ends in a question
reads `asks you` with no call, a stop with a permission still pending reads `blocked` with no
call, a stop with no replay row stays `idle`, and a part of the prompt the message does not cover
shows on the activity line; in `shadow` the row stays `idle` and the Decisions view holds the
call; in `off` nothing is logged; `fleet_snapshot` carries the `stop_kind` label.
