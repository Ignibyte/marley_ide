# shellcheck shell=bash
# #932's visual check: a recording's minute in the drawer, Draft a test, and the scripts' last
# runs. Chromium runs offline; #523's loopback npm registry serves Marley's pinned
# playwright-core and its login page is the tab's page. Two scripts are kept for the project,
# `log-in`, which passes, and `broken`, which fails, and `tidy` for every project, never run.
# The terminals' `.bashrc` sets the prompt and npm's registry only. No agent CLI runs.
# - `932-01-minute`: the name typed and Sign in clicked, then `marley: record this`: the drawer on
#   Recordings with the new recording expanded, its bars, its steps and the two buttons
#   (REQ-001, REQ-002).
# - `932-02-draft`: Draft a test: an unsaved TypeScript tab headed by its suggested path, and no
#   file written (REQ-003).
# - `932-03-scripts`: both project scripts run, the tray open: "passed", "failed · exit 1" and "not
#   run", each with its scope and when (REQ-004, REQ-005).
# - `932-04-day`: Marley Day, the minute and the tray (REQ-001, REQ-004).
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

STOP_AT=${STOP_AT:-}
# The page's top left in the 1600x1000 window, as #490 measured it.
PAGE_X=260
PAGE_Y=109
AWAY_X=${AWAY_X:-900}
AWAY_Y=${AWAY_Y:-500}
# From the probe runs' shots.
DRAFT_X=${DRAFT_X:-1415}
DRAFT_Y=${DRAFT_Y:-847}
RUN_X=${RUN_X:-1545}
# The Run buttons once the first run's terminal has split the window, as #523 found.
RUN_X_BESIDE=${RUN_X_BESIDE:-870}
LOG_IN_Y=${LOG_IN_Y:-174}
BROKEN_Y=${BROKEN_Y:-131}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

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
  profile_setting marley.tmux_hosts '[]'
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
  cat >"$E2E_PROFILE/config/playwright/global/tidy.mjs" <<'JS'
export default async function ({ page }) {
  await page.reload();
}
JS
  open_path "$E2E_WORK/repo"
}

teardown() {
  browser_teardown
}

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  settle 12
  # Trusts the repository.
  press "" Return
  settle 2
  palette "marley: open browser" 6
  press CTRL l
  settle 1
  type_text "$SITE/login.html"
  press "" Return
  settle 4

  echo "== the user signs in, then Record This"
  click $((PAGE_X + 306)) $((PAGE_Y + 163))
  settle 1
  type_text "Marley"
  settle 1
  click $((PAGE_X + 550)) $((PAGE_Y + 163))
  settle 2
  palette "marley: record this" 4
  away
  take 932-01-minute || return 0

  echo "== Draft a test"
  click "$DRAFT_X" "$DRAFT_Y"
  settle 4
  away
  take 932-02-draft || return 0
  expect "the draft wrote no file into the project" bash -c "! find '$E2E_WORK/repo' -name '*.spec.ts' | grep -q ."

  echo "== log-in and broken run"
  palette "marley: open browser" 2
  palette "marley: playwright scripts" 2
  away
  take 932-02b-tray || return 0
  click "$RUN_X" "$LOG_IN_Y"
  settle 25
  palette "marley: open browser" 2
  click "$RUN_X_BESIDE" "$BROKEN_Y"
  settle 12
  palette "marley: open browser" 2
  away
  cat "$E2E_PROFILE/playwright/runs.json" 2>/dev/null || echo "no runs.json"
  take 932-03-scripts || return 0

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  away
  shot 932-04-day
}
