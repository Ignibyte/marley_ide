# shellcheck shell=bash
# #523's e2e test: Playwright scripts kept in Marley and run on a Browser tab. Setup keeps two
# scripts for the repository's project, `log-in` and `broken`, and one for every project,
# `where`, and serves Marley's pinned playwright-core from a loopback npm registry, packed from a
# copy on the box, so the first run's real install needs no network. The Scripts button opens the
# tray, which lists the three with their scopes (REQ-001), and a name typed there makes a script
# from Marley's template for every project, open in an editor tab (REQ-002). The first run,
# `where`, installs Marley's Playwright in its own block (REQ-008) and prints the tab's page id
# and URL from the run's environment (REQ-005). `log-in` signs the page in while the tab stays in
# front, its block's output in a terminal beside it, and saves no recording (REQ-003, REQ-004,
# REQ-007). `broken` fails, and its tab's last minute is saved as a recording with the run's
# start and end in its timeline, named in a toast (REQ-006). Chromium runs offline.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The toolbar's Scripts button, left of Pick (PICK_X=1288 in #496), measured from the first run.
SCRIPTS_X=1260
TOOLBAR_Y=87
# The tray's rows, 36 pixels apart from the first; the Run button's and the name field's x, and
# the "For All Projects" button's, measured from the first run.
ROW_Y=122
ROW_HEIGHT=36
RUN_X=1300
FIELD_X=600
FOR_ALL_X=1290
# The Run buttons once the first run's terminal has split the window: the tab's pane is the left
# half.
RUN_X_BESIDE=750

setup() {
  local home=$E2E_WORK/home core node_dir key registry
  offline_chromium
  core=${E2E_PLAYWRIGHT_CORE:-/srv/stacks/rustal/node_modules/playwright-core}
  if ! grep -qs '"version": "1.63.0"' "$core/package.json"; then
    echo "setup: no playwright-core 1.63.0 at $core; E2E_PLAYWRIGHT_CORE names one" >&2
    return 1
  fi
  node_dir=$(dirname "$(command -v node)")
  mkdir -p "$home" "$E2E_WORK/site" "$E2E_WORK/registry/playwright-core/-"
  cat >"$E2E_WORK/site/login.html" <<'HTML'
<!doctype html><html><head><title>Log in</title></head>
<body style="margin:0;font:28px sans-serif;background:#eef6ee">
<h1 style="margin:40px">Log in</h1>
<p style="margin:0 40px"><label>Name <input id="name" style="font:inherit"></label>
<button style="font:inherit" onclick="document.getElementById('out').textContent =
  'Signed in as ' + document.getElementById('name').value">Sign in</button></p>
<p id="out" style="margin:24px 40px;font-weight:bold"></p>
</body></html>
HTML
  SITE=http://127.0.0.1:$(serve_site site)
  # The registry: the copy packed as npm packs it, and a document naming it, as the npm registry
  # answers for a package.
  npm_config_update_notifier=false npm pack "$core" --pack-destination \
    "$E2E_WORK/registry/playwright-core/-" --cache "$E2E_WORK/npm-cache" --loglevel=error >/dev/null
  registry=http://127.0.0.1:$(serve_site registry)
  python3 - "$E2E_WORK/registry" "$registry" "$core/package.json" <<'PY'
import base64, hashlib, json, pathlib, sys
root, registry, manifest = pathlib.Path(sys.argv[1]), sys.argv[2], sys.argv[3]
tarball = root / "playwright-core/-/playwright-core-1.63.0.tgz"
data = tarball.read_bytes()
version = json.loads(pathlib.Path(manifest).read_text())
version["dist"] = {
    "tarball": f"{registry}/playwright-core/-/playwright-core-1.63.0.tgz",
    "integrity": "sha512-" + base64.b64encode(hashlib.sha512(data).digest()).decode(),
    "shasum": hashlib.sha1(data).hexdigest(),
}
document = {"name": "playwright-core", "dist-tags": {"latest": "1.63.0"}, "versions": {"1.63.0": version}}
(root / "playwright-core/index.html").write_text(json.dumps(document))
PY
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$node_dir:\$PATH"
export npm_config_registry="$registry/"
export npm_config_cache="$E2E_WORK/npm-cache"
export npm_config_update_notifier=false
RC
  terminal_env HOME "$home"
  write_mcp_agent
  git init -q -b main "$E2E_WORK/repo"
  key=$(basename "$(browser_project_dir "$E2E_WORK/repo")")
  mkdir -p "$E2E_PROFILE/config/playwright/projects/$key" "$E2E_PROFILE/config/playwright/global"
  cat >"$E2E_PROFILE/config/playwright/projects/$key/log-in.mjs" <<'JS'
export default async function ({ page }) {
  await page.getByLabel('Name').fill('Marley');
  await page.getByRole('button', { name: 'Sign in' }).click();
  await page.getByText('Signed in as Marley').waitFor();
}
JS
  cat >"$E2E_PROFILE/config/playwright/projects/$key/broken.mjs" <<'JS'
export default async function ({ page }) {
  await page.getByRole('button', { name: 'Check out' }).click({ timeout: 2000 });
}
JS
  cat >"$E2E_PROFILE/config/playwright/global/where.mjs" <<'JS'
export default async function ({ page }) {
  console.log(`tab ${process.env.MARLEY_TAB} at ${page.url()}`);
}
JS
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

# Clicks the Run button of the tray's row `$1`, from 0, at `$2` (the full-width tab's by default).
run_row() {
  click "${2:-$RUN_X}" $((ROW_Y + $1 * ROW_HEIGHT))
}

# How many recordings the stand-in agent lists.
recording_count() {
  mcp_agent recordings | grep -c "^  recording " || true
}

steps() {
  local tab before
  settle 12
  # Trusts the repository.
  press "" Return
  settle 2
  palette "marley: open browser"
  settle 6
  press CTRL l
  settle 1
  type_text "$SITE/login.html"
  press "" Return
  settle 4
  mcp_agent tabs | tee "$E2E_WORK/tabs.txt"
  tab=$(sed -nE 's/^  tab ([^:]*):.*/\1/p' "$E2E_WORK/tabs.txt" | head -1)
  echo "the tab: $tab"

  echo "== the Scripts button opens the tray"
  click "$SCRIPTS_X" "$TOOLBAR_Y"
  settle 2
  shot 523-01-tray

  echo "== a new script for every project, from the template"
  click "$FIELD_X" $((ROW_Y + 3 * ROW_HEIGHT))
  settle 1
  type_text "check title"
  settle 1
  click "$FOR_ALL_X" $((ROW_Y + 3 * ROW_HEIGHT))
  settle 3
  shot 523-02-new
  find "$E2E_PROFILE/config/playwright" -type f | sed "s#$E2E_PROFILE/##" | sort
  expect "check-title.mjs is kept for every project" test -f "$E2E_PROFILE/config/playwright/global/check-title.mjs"
  expect "it starts from Marley's template" holds "$E2E_PROFILE/config/playwright/global/check-title.mjs" "export default async function ({ page, context, browser })"

  echo "== back in the tab: the first run, where, installs Marley's Playwright first"
  palette "marley: open browser"
  settle 2
  # The rows now: broken, log-in (this project), check-title, where (all projects).
  run_row 3
  settle 15
  shot 523-02b-installed
  mcp_agent terminal-read where.mjs | tee "$E2E_WORK/where.txt"
  expect "the first run installed playwright-core from the registry" holds "$E2E_WORK/where.txt" "npm install --prefix" "added 1 package"
  expect "the run's environment names the tab's page and its URL" holds "$E2E_WORK/where.txt" "tab $tab at $SITE/login.html"
  expect "Marley's Playwright is installed" test -f "$E2E_PROFILE/playwright/node_modules/playwright-core/package.json"

  echo "== log-in signs the page in, the tab in front, and saves nothing"
  before=$(recording_count)
  palette "marley: open browser"
  settle 2
  run_row 1 "$RUN_X_BESIDE"
  settle 10
  shot 523-03-passed
  mcp_agent terminal-read log-in.mjs | tee "$E2E_WORK/log-in.txt"
  mcp_agent tabs | tee "$E2E_WORK/tabs.txt"
  expect "log-in passed" holds "$E2E_WORK/log-in.txt" "✓ log-in passed"
  expect "the run typed no install this time" bash -c "! grep -q 'npm install' '$E2E_WORK/log-in.txt'"
  expect "a passing run saved no recording" test "$(recording_count)" -eq "$before"

  echo "== broken fails, and the tab's last minute is saved"
  palette "marley: open browser"
  settle 2
  run_row 0 "$RUN_X_BESIDE"
  settle 10
  shot 523-04-failed
  mcp_agent terminal-read broken.mjs | tee "$E2E_WORK/broken.txt"
  expect "broken failed" holds "$E2E_WORK/broken.txt" "✗ broken failed"
  expect "the failed run saved a recording" test "$(recording_count)" -eq $((before + 1))
  local newest
  newest=$(mcp_agent recordings | sed -nE 's/^  recording ([^ :]*).*/\1/p' | head -1)
  mcp_agent recording "$newest" | tee "$E2E_WORK/recording.txt"
  expect "the recording holds the run's start and its end with exit code 1" \
    holds "$E2E_WORK/recording.txt" 'script: {"name": "broken"}' \
    'script: {"name": "broken", "exit_code": 1}'
}
