# TICKET-535 — Agent events pushed to the phone through ntfy on the dev box

- **Ticket:** LOCAL #535 (feature, prong 2 remote control: the phone path's first slice)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/535-phone-push-notifications.spec.md
- **Source ticket:** Chad, 2026-09-25, on the phone: a full native app eventually, and "we could potentially have the app drive it or a small native rust relayer". The Orca survey puts push first on that path (docs/orca_architecture/04-remote-control-and-mobile.md §2.6, §3.2 item 1, §3.3).
- **Status:** open
- **Backlog:** Deliberate. It is the first step of the phone path, which starts when Chad picks it. It also needs #519's agent events: the three event kinds, "failed" above all, come from them.

## Summary
When a Claude Code agent in one of Marley's terminals needs input, finishes or fails, Marley posts
the event to an ntfy server on the dev box, and the ntfy app on Chad's phone shows it over the
tailnet. Marley posts to ntfy on loopback, so the post itself never leaves the machine. The push is
one line, `<project>: Claude needs input` (or finished, failed), under the title `Marley`; the
agent's own words, the tool and its input stay on the desktop. On iOS, ntfy.sh and Apple's push
service see a message id, a hash of the topic URL and the fixed text "New message", never the
line itself. The phone endpoint, the native app and a small Rust relay are later slices, designed
in the notes.

## Acceptance
With `marley.push` set, a permission request, a finished turn and a failed turn from Claude Code in
a terminal Chad is not looking at each reach ntfy as one push naming the project, the agent and the
event, with no body; an event in the focused terminal of the active window, and any OSC 9 or 777
from a program that is not an agent, push nothing.
