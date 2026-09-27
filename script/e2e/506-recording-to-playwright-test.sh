# shellcheck shell=bash
# #506's e2e test: a recording turned into a Playwright test the project keeps. A loopback app has
# a sign-in page (an e-mail field and a password field, each with its label, and a Sign in button
# with a test id, which goes to the welcome page) and a welcome page (a search field known only by
# its placeholder, and an Orders link that moves the page with the history API). In a Browser tab
# the user signs in, searches and follows the link, then saves the minute with Record this. The
# recording holds each click, fill and key press with its target's locators, the e-mail and the
# search as text and the password as nothing but a secret fill. The stand-in agent drafts a test
# from it with `browser_draft_test`; the scenario writes the test where the tool says, as an agent
# would, and opens it in Marley. Playwright then runs it three times: with the password in the
# environment (it passes), against a sign-in page whose button no longer navigates (it fails at
# that step), and without the password (it fails, naming the variable). No run finds the password
# in the recording or the draft. The password is made in the setup from pieces, so this file holds
# none. Chromium runs offline; Playwright comes from a project on the box, or from
# E2E_PLAYWRIGHT_MODULES, a `node_modules` holding `@playwright/test`, and runs its own headless
# Chromium from `~/.cache/ms-playwright` with its output in the scratch repository.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

SITE=
MODULES=${E2E_PLAYWRIGHT_MODULES:-/srv/stacks/rustal/node_modules}

# The page's top left in the 1600x1000 window, as #490 measured it.
PAGE_X=${PAGE_X:-260}
PAGE_Y=${PAGE_Y:-109}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo name
  if [[ ! -x $MODULES/.bin/playwright || ! -d $MODULES/@playwright/test ]]; then
    echo "#506 needs @playwright/test and its CLI; missing under $MODULES" >&2
    echo "set E2E_PLAYWRIGHT_MODULES to a node_modules folder that holds them" >&2
    return 1
  fi
  mkdir -p "$home" "$bin" "$repo/e2e" "$E2E_WORK/site"
  echo "PS1='\$ '" >"$home/.bashrc"
  terminal_env HOME "$home"
  # Nothing here opens the system browser. Should a click land on a link, these log it and
  # succeed, so neither `open`'s later commands nor gpui's desktop-portal fallback, which would
  # reach the user's own browser, runs.
  for name in xdg-open gio google-chrome firefox; do
    cat >"$bin/$name" <<SH
#!/bin/sh
printf '%s %s\n' "$name" "\$*" >>"$E2E_WORK/leak.log" || true
exit 0
SH
    chmod +x "$bin/$name"
  done
  : >"$E2E_WORK/leak.log"
  export PATH="$bin:$PATH"
  [[ $(command -v xdg-open) == "$bin/xdg-open" ]] || {
    echo "the fake xdg-open is not first on the PATH; not launching" >&2
    return 1
  }
  git init -q -b tests "$repo"
  # Playwright resolves `@playwright/test` from beside the repository, so Marley's project never
  # walks the modules.
  ln -s "$MODULES" "$E2E_WORK/node_modules"
  cat >"$repo/playwright.config.ts" <<'TS'
import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  outputDir: './test-results',
  timeout: 20000,
  expect: { timeout: 4000 },
  use: { headless: true },
});
TS
  # The password and the e-mail, made from pieces so that no file of the repository holds them.
  python3 -c 'import json, sys; json.dump({"password": "correct-" + "horse-" + "battery-" + "staple", "email": "typed." + "person@example.test"}, open(sys.argv[1], "w"))' \
    "$E2E_WORK/secrets.json"
  write_signin works
  cp "$E2E_WORK/site/signin.html" "$E2E_WORK/site/signin-working.html"
  write_signin broken
  cp "$E2E_WORK/site/signin.html" "$E2E_WORK/site/signin-broken.html"
  cp "$E2E_WORK/site/signin-working.html" "$E2E_WORK/site/signin.html"
  cat >"$E2E_WORK/site/welcome.html" <<'HTML'
<!doctype html><html><head><meta charset="utf-8"><title>Welcome</title><style>
body { margin: 0; font: 18px sans-serif; }
h1 { position: absolute; left: 40px; top: 20px; margin: 0; font-size: 28px; }
input { position: absolute; left: 40px; top: 90px; width: 300px; height: 32px; font-size: 18px; }
#orders { position: absolute; left: 40px; top: 150px; }
#view { position: absolute; left: 40px; top: 200px; margin: 0; }
</style></head><body>
<h1>Welcome</h1>
<input type="search" placeholder="Search orders">
<a id="orders" href="/orders">Orders</a>
<p id="view"></p>
<script>
document.getElementById('orders').addEventListener('click', (event) => {
  event.preventDefault();
  history.pushState({}, '', '/orders');
  document.getElementById('view').textContent = 'Orders';
});
document.querySelector('input[type=search]').addEventListener('keydown', (event) => {
  if (event.key === 'Enter') document.getElementById('view').textContent = 'Results for ' + event.target.value;
});
</script></body></html>
HTML
  offline_chromium
  SITE=http://127.0.0.1:$(serve_site site)
  open_path "$repo"
}

teardown() {
  browser_teardown
}

# Writes the sign-in page as `signin.html`: its Sign in button goes to the welcome page, or, for
# `broken`, does nothing.
write_signin() {
  local handler="location.href = '/welcome.html';"
  [[ $1 == broken ]] && handler="document.getElementById('status').textContent = 'Signing in…';"
  cat >"$E2E_WORK/site/signin.html" <<HTML
<!doctype html><html><head><meta charset="utf-8"><title>Sign in</title><style>
body { margin: 0; font: 18px sans-serif; }
label, input, button, p { position: absolute; left: 40px; margin: 0; }
input { width: 300px; height: 32px; font-size: 18px; }
button { width: 160px; height: 40px; font-size: 18px; }
#email-label { top: 40px; } #email { top: 70px; }
#password-label { top: 120px; } #password { top: 150px; }
#sign-in { top: 210px; } #status { top: 270px; }
</style></head><body>
<label id="email-label" for="email">E-mail</label><input id="email" type="email" name="email">
<label id="password-label" for="password">Password</label><input id="password" type="password" name="password">
<button type="button" id="sign-in" data-testid="sign-in">Sign in</button>
<p id="status"></p>
<script>document.getElementById('sign-in').addEventListener('click', () => { $handler });</script>
</body></html>
HTML
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Clicks the page at (x, y) in its own pixels.
click_page() {
  click $((PAGE_X + $1)) $((PAGE_Y + $2))
}

# The secret `$1` from the run's secrets.
secret() {
  python3 -c 'import json, sys; print(json.load(open(sys.argv[1]))[sys.argv[2]])' "$E2E_WORK/secrets.json" "$1"
}

# Whether the saved JSON `$1` makes the Python expression `$2` true, with `data` the JSON,
# `actions` a timeline's action entries and `secrets` the run's secrets.
answer() {
  python3 -c '
import json, sys
data = json.load(open(sys.argv[1]))
secrets = json.load(open(sys.argv[3]))
actions = [entry for entry in data.get("entries", []) if entry.get("kind") == "action"]
sys.exit(0 if eval(sys.argv[2]) else 1)' "$1" "$2" "$E2E_WORK/secrets.json"
}

# Runs the drafted test `$1` with Playwright in the scratch repository, the password in the
# environment unless `$2` is `unset`; its output goes to `$3` and its exit code after it.
playwright_run() {
  local status=0
  if [[ $2 == unset ]]; then
    (cd "$E2E_WORK/repo" && env -u PASSWORD "$E2E_WORK/node_modules/.bin/playwright" test "$1" --reporter=line) >"$3" 2>&1 || status=$?
  else
    (cd "$E2E_WORK/repo" && PASSWORD=$(secret password) "$E2E_WORK/node_modules/.bin/playwright" test "$1" --reporter=line) >"$3" 2>&1 || status=$?
  fi
  echo "  playwright exit $status"
  sed 's/^/  | /' "$3" | tail -12
  return "$status"
}

# Whether no file under the folders and files given holds the password.
no_password_in() {
  ! grep -rqF "$(secret password)" "$@"
}

steps() {
  local id file recordings timeline
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  echo "== sign in, search, follow a link, in a Browser tab"
  palette "marley: new browser tab"
  settle 3
  type_text "$SITE/signin.html"
  press "" Return
  settle 5
  click_page 190 86
  settle 1
  type_text "$(secret email)"
  settle 1
  click_page 190 166
  settle 1
  type_text "$(secret password)"
  settle 1
  click_page 120 230
  settle 4
  click_page 190 106
  settle 1
  type_text 4512
  press "" Return
  settle 1
  click_page 70 161
  settle 2
  echo "== Record this saves the minute"
  palette "marley: record this"
  settle 3
  shot 506-01-recorded
  echo "== the agent reads the recording"
  id=$(mcp_agent recordings | sed -n 's/^  recording \([0-9A-Za-z-]*\):.*/\1/p' | tail -1)
  echo "  recording $id"
  mcp_agent recording "$id" | grep -E ' (action|navigation):'
  recordings="$E2E_PROFILE/browser/recordings"
  timeline="$recordings/$id/timeline.json"
  expect "each click, fill and press is an action with its locators" answer "$timeline" \
    '[entry["action"] for entry in actions] == ["click", "fill", "click", "fill", "click", "click", "fill", "press", "click"] and all(entry["locators"] for entry in actions)'
  expect "the e-mail and the search kept as fills, once each" answer "$timeline" \
    '[entry.get("text") for entry in actions if entry["action"] == "fill"] == [secrets["email"], None, "4512"]'
  expect "the password's fill is secret, with no text" answer "$timeline" \
    '[(entry["secret"], "text" in entry) for entry in actions if entry["action"] == "fill"][1] == (True, False)'
  expect "Sign in's test id found it alone" answer "$timeline" \
    'actions[4]["locators"][0] == {"kind": "test id", "attribute": "data-testid", "value": "sign-in", "unique": True}'
  expect "the link's move is a navigation within the document" answer "$timeline" \
    'any(entry.get("kind") == "navigation" and entry["url"].endswith("/orders") and entry["within"] for entry in data["entries"])'
  echo "== the agent drafts a test"
  mcp_agent draft-test "$id" "$E2E_WORK/draft.json"
  expect "the draft locates each target by the first locator found alone" answer "$E2E_WORK/draft.json" \
    'all(step in data["test"] for step in ["getByRole(\x27textbox\x27, { name: \x27E-mail\x27, exact: true })", "getByLabel(\x27Password\x27, { exact: true })", "getByTestId(\x27sign-in\x27).click()", "getByRole(\x27searchbox\x27, { name: \x27Search orders\x27, exact: true })", "getByRole(\x27link\x27, { name: \x27Orders\x27, exact: true }).click()"])'
  expect "it expects the URL after each navigation, as a path" answer "$E2E_WORK/draft.json" \
    '"toHaveURL(\x27/welcome.html\x27)" in data["test"] and "toHaveURL(\x27/orders\x27)" in data["test"]'
  expect "its baseURL is the recording's origin, and it starts by a path" answer "$E2E_WORK/draft.json" \
    '"baseURL: \x27" + data["start"].split("/signin.html")[0] + "\x27" in data["test"] and "page.goto(\x27/signin.html\x27)" in data["test"]'
  expect "it reads the password from PASSWORD, and goes in the project's e2e folder" answer "$E2E_WORK/draft.json" \
    'data["env"] == ["PASSWORD"] and "fill(secret(\x27PASSWORD\x27))" in data["test"] and data["path"].startswith(data["project"] + "/e2e/") and data["path"].endswith("-" + data["id"] + ".spec.ts")'
  file=$(python3 -c 'import json, sys; draft = json.load(open(sys.argv[1])); open(draft["path"], "w").write(draft["test"]); print(draft["path"].rsplit("/", 1)[1])' "$E2E_WORK/draft.json")
  echo "== the draft, written where the tool said, opened in Marley"
  press CTRL p
  settle 1
  type_text "$file"
  settle 2
  press "" Return
  settle 3
  shot 506-02-draft
  echo "== Playwright runs the draft with the password set"
  expect "the drafted test passes" playwright_run "e2e/$file" set "$E2E_WORK/playwright-pass.log"
  echo "== against a sign-in page whose button no longer navigates"
  cp "$E2E_WORK/site/signin-broken.html" "$E2E_WORK/site/signin.html"
  expect "the drafted test fails" not playwright_run "e2e/$file" set "$E2E_WORK/playwright-broken.log"
  expect "at the step that expects the welcome page" grep -q "toHaveURL" "$E2E_WORK/playwright-broken.log"
  cp "$E2E_WORK/site/signin-working.html" "$E2E_WORK/site/signin.html"
  echo "== without the password"
  expect "the test fails without PASSWORD" not playwright_run "e2e/$file" unset "$E2E_WORK/playwright-unset.log"
  expect "naming the variable" grep -q "set PASSWORD to run this test" "$E2E_WORK/playwright-unset.log"
  echo "== the password is nowhere Marley wrote"
  expect "not in the recording, the draft or the test file" \
    no_password_in "$recordings" "$E2E_WORK/draft.json" "$E2E_WORK/repo/e2e"
  cat "$E2E_WORK/leak.log"
  expect "nothing reached the system browser" test ! -s "$E2E_WORK/leak.log"
}

# Whether the command after it fails.
not() {
  ! "$@"
}
