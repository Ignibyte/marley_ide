# shellcheck shell=bash
# #499's e2e test: the flight recorder keeps a page's last minute and Record this saves it. A
# loopback sign-in page logs to the console and, on Sign in, fetches with a token in its query.
# A click on the page's "Early" box, then more than a minute of waiting, rolls that click out of
# the minute; an e-mail and a password are typed, Sign in is clicked and the page scrolled; then
# `marley: record this` saves the minute and says so in a toast. The stand-in agent lists the
# recordings and reads the new one with a frame; the run log shows the timeline (presses, keys by
# name, typing as counts, the console, the request with its token hidden, the snapshots, the
# frames at least half a second apart) and a grep that finds neither the password nor the token
# in the recording's files, and the e-mail kept as a fill: since #506 an ordinary field's text is
# recorded, a password field's never. Chromium runs offline.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The page's top left in the 1600x1000 window, as #490 measured it.
PAGE_X=260
PAGE_Y=109
EMAIL=chad@example.com
PASSWORD=hunter2-secret
TOKEN=s3cr3t-token-42

setup() {
  offline_chromium
  mkdir -p "$E2E_WORK/site"
  cat >"$E2E_WORK/site/index.html" <<HTML
<!doctype html><html><head><title>Sign in</title><style>
body { margin: 0; font: 18px sans-serif; background: #f5f5ff; height: 2000px }
label { position: absolute; left: 40px }
#email-label { top: 42px } #password-label { top: 102px }
input { position: absolute; left: 160px; width: 260px; height: 32px; font-size: 18px }
#email { top: 36px } #password { top: 96px }
#go { position: absolute; left: 160px; top: 160px; width: 120px; height: 40px; font-size: 18px }
#early { position: absolute; left: 600px; top: 40px; width: 120px; height: 60px; background: #fdd }
#status { position: absolute; left: 40px; top: 230px; margin: 0 }
</style></head><body>
<label id="email-label" for="email">E-mail</label><input id="email" type="email">
<label id="password-label" for="password">Password</label><input id="password" type="password">
<button id="go">Sign in</button>
<div id="early">Early</div>
<p id="status">Not signed in.</p>
<script>
console.log('the sign-in page is ready');
document.getElementById('go').addEventListener('click', () => {
  fetch('/api/session?token=$TOKEN&page=2').catch(() => {});
  console.warn('signing in');
  document.getElementById('status').textContent = 'Signed in.';
});
</script>
</body></html>
HTML
  SITE=http://127.0.0.1:$(serve_site site)
  git init -q -b browser "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

teardown() {
  browser_teardown
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# The window's point for (x, y) in the page's viewport.
page_point() {
  echo "$((PAGE_X + $1)) $((PAGE_Y + $2))"
}

steps() {
  local listing id recordings
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  palette "marley: open browser"
  settle 5
  press CTRL l
  settle 1
  type_text "$SITE/index.html"
  press "" Return
  settle 4
  echo "== a click on the Early box, then more than a minute"
  # shellcheck disable=SC2046
  click $(page_point 660 70)
  settle 65
  echo "== an e-mail and a password, Sign in, a scroll"
  # shellcheck disable=SC2046
  click $(page_point 290 52)
  settle 1
  type_text "$EMAIL"
  # shellcheck disable=SC2046
  click $(page_point 290 112)
  settle 1
  type_text "$PASSWORD"
  # shellcheck disable=SC2046
  click $(page_point 220 180)
  settle 2
  # shellcheck disable=SC2046
  pointer_to $(page_point 700 500)
  scroll 3
  settle 2
  echo "== Record this saves the minute, and a toast names it"
  palette "marley: record this"
  settle 3
  shot 499-01-recorded
  echo "== the agent lists the recordings and reads the new one"
  listing=$(mcp_agent recordings)
  echo "$listing"
  id=$(printf '%s\n' "$listing" | sed -n 's/^  recording \([0-9A-Za-z-]*\):.*/\1/p' | tail -1)
  mcp_agent recording "$id" 2 "$(shot_file 499-frame-2.jpg)"
  echo "== the recording's files, and what is not in them"
  recordings="$E2E_PROFILE/browser/recordings"
  find "$recordings" -type f | sed "s|^$recordings/||" | sort | head -5
  echo "  $(find "$recordings" -name '*.jpg' | wc -l) frames on disk"
  expect "neither the password nor the token is in the recording" none_in "$recordings" "$PASSWORD" "$TOKEN"
  expect "the e-mail is kept as a fill (#506)" grep -rqF "\"text\": \"$EMAIL\"" "$recordings"
}

# Whether none of the strings after the folder `$1` is in a file under it.
none_in() {
  local dir=$1 pattern
  local -a patterns=()
  shift
  for pattern; do
    patterns+=(-e "$pattern")
  done
  ! grep -rqF "${patterns[@]}" "$dir"
}
