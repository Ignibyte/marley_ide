# TICKET-641 — SSH links that know they are dead

- **Ticket:** LOCAL #641 (feature, prong 2: C3's remote terminals after #543, D20's collector after #610)
- **Owner:** claude-opus-5-5, 2026-10-02 (/spec)
- **Pipeline doc:** ../../pipeline/completed/641-ssh-links-that-know-they-are-dead.spec.md
- **Source ticket:** herdr's remote-link habits, withdrawn from rustal-harness (its TICKET-094, D164)
  and given to Marley's own SSH connections; `docs/planning/design-notes/herdr-and-hermes-2026-10-02.md`
  Part 1 item 6
- **Status:** closed

## Summary
Marley opens two kinds of SSH connection: a remote terminal (#543, ssh into a tmux session per
terminal) and the host collector (#610, a script run over SSH every 5 s). Neither asks ssh to
check its link, so a link that goes silent without closing holds ssh open until TCP gives up:
13 to 15 minutes on this box (its keepalive and retransmit limits), hours on a kernel with the
default two-hour keepalive. Meanwhile the remote terminal reads live and swallows keys, and a hung
collection stops the whole Fleet poll, which shows the last reading as current. A link that closes
outright ends the remote terminal's task, and nothing reconnects until the user runs Rerun.

This ticket brings herdr's three habits to the remote terminals and the first of them to the
collector. Both argvs get ssh's own keepalive, so a silent link ends ssh within 20 s. When a remote
terminal's ssh ends with its error status (255), the terminal keeps its last screen, drawn dimmed
with input off and a line saying why and when Marley tries next. Marley checks the link on a backoff
that doubles from 1 s up to two minutes and reattaches the same tmux session with Zed's Rerun once
the host answers. Keys pressed while the terminal is dimmed are not sent; the line counts them, and
nothing is kept to replay later.

The collector's backoff and a Fleet host that keeps its last reading dimmed are the next slice
(Out in the spec).

## Acceptance
A remote terminal whose link goes silent dims within 20 s, says the link is down, refuses and counts
typed keys, and reattaches the same session on its own once the host answers again; one the user
ended with `exit` does not reconnect. A listed host whose link goes silent reads unreachable within
20 s, and the rest of the Fleet panel keeps refreshing.
