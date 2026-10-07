# shellcheck shell=bash
# #671's visual check: the rail keeps the window's order by default. It runs #542's three projects,
# a, b and c (window order a, b, c), each with a stand-in Claude Code acting through a FIFO and the
# plugin's real `event.py`, b's first terminal a plain shell, on #542's private session bus; the
# run's copy of the settings names no `rail_order`. `671-01-working`: c, the bottom project, works,
# and the rail still lists a, b, c (REQ-001). `671-02-waiting`: b waits as well, still a, b, c, with
# b's shell above its agent (REQ-001, REQ-002). `671-03-attention`: `rail_order` set to
# "attention" puts b and c first, #542's order (REQ-003).
# shellcheck source=script/e2e/542-rail-attention-order.sh
. script/e2e/542-rail-attention-order.sh

steps() {
  settle 12
  # Trusts c.
  press "" Return
  settle 3

  echo "== b, handed over: a shell, then a stand-in in a second terminal"
  hand_over "$E2E_WORK/b"
  settle 5
  press "" Return
  settle 4
  touch "$E2E_WORK/start-b"
  palette "workspace: new terminal"
  settle 4

  echo "== a, handed over"
  touch "$E2E_WORK/start-a"
  hand_over "$E2E_WORK/a"
  settle 5
  press "" Return
  settle 4

  echo "== c, the bottom project, works"
  pointer_to "$AWAY_X" "$AWAY_Y"
  step_of c working
  settle 4
  shot 671-01-working

  echo "== b waits as well"
  step_of b waiting
  settle 4
  shot 671-02-waiting

  echo "== rail_order attention"
  set_setting marley.rail_order '"attention"'
  settle 4
  shot 671-03-attention

  cat "$E2E_WORK/banners.log"
  expect "no banner reached the user's bus" bash -c "! grep -q 'STRING \"Marley\"' '$E2E_WORK/user-bus.log'"
}
