# shellcheck shell=bash
# #539's e2e test: every page Marley's Chromium shows is told it runs Chrome, not HeadlessChrome.
# A page in a Browser tab, loaded twice so its high-entropy hints arrive, prints what its request
# said and what its script reads, and holds a cross-site iframe that does the same; each script
# sends what it read back to the server. The user agent says Chrome/ (REQ-001); the brands agree
# between the header and the script and name no headless brand, on Linux (REQ-002); the full
# version list is there and agrees (REQ-003); the iframe says the same (REQ-004). An agent's new
# tab (REQ-005) and a tab Marley opens at a URL itself (REQ-006) carry the identity on their first
# request, and the opened tab has no blank page to go back to. Every shot and the server's log
# come first and the checks after, so the build before the change shows each step before it fails
# (L-claude-512). Chromium runs offline.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

setup() {
  offline_chromium
  mkdir -p "$E2E_WORK/site"
  echo '<!doctype html><title>Site</title>' >"$E2E_WORK/site/index.html"
  PORT=$(serve_site site)
  SITE=http://127.0.0.1:$PORT
  write_mcp_agent
  git init -q -b main "$E2E_WORK/repo"
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

# Keeps `command`'s output in `$E2E_WORK/out.txt` as the log shows it.
out() {
  "$@" | tee "$E2E_WORK/out.txt"
}

# What the server heard and what the pages' scripts reported, one line each, from its log.
heard() {
  python3 - "$E2E_WORK/site.log" <<'PY'
import json, sys
for line in open(sys.argv[1]):
    kind, _, rest = line.partition(" ")
    if kind == "headers":
        entry = json.loads(rest)
        print(f"  request {entry['path']}")
        for name in ("User-Agent", "Sec-CH-UA", "Sec-CH-UA-Platform", "Sec-CH-UA-Full-Version-List"):
            print(f"    {name}: {entry.get(name)}")
    elif kind == "report":
        entry = json.loads(rest)
        print(f"  script at {entry['page']}")
        for name in ("ua", "brands", "full", "platform"):
            print(f"    {name}: {entry.get(name)}")
PY
}

# Checks the log: `check <what>` prints `yes` or `no` and why.
check() {
  python3 - "$E2E_WORK/site.log" "$1" <<'PY'
import json, re, sys
requests, reports = [], []
for line in open(sys.argv[1]):
    kind, _, rest = line.partition(" ")
    if kind == "headers":
        requests.append(json.loads(rest))
    elif kind == "report":
        reports.append(json.loads(rest))
what = sys.argv[2]
chrome = lambda agent: bool(agent) and "Chrome/" in agent and "HeadlessChrome" not in agent
page = [entry for entry in requests if "frame=" in entry["path"]]
first = [entry for entry in requests if entry["path"].endswith("?first")]
opened = [entry for entry in requests if entry["path"].endswith("?opened")]
framed = [entry for entry in requests if entry["path"] == "/headers"]
top = [entry for entry in reports if "frame=" in entry["page"]]
inner = [entry for entry in reports if "frame=" not in entry["page"] and "?" not in entry["page"]]
if what == "user-agent":
    ok = page and all(chrome(entry["User-Agent"]) for entry in requests) and top and all(chrome(entry["ua"]) for entry in reports)
elif what == "brands":
    ok = page and top and all("Headless" not in (entry["Sec-CH-UA"] or "Headless") for entry in page) \
        and top[-1]["brands"] == page[-1]["Sec-CH-UA"] and all(entry["Sec-CH-UA-Platform"] == '"Linux"' for entry in page)
elif what == "full-version-list":
    full = page[-1]["Sec-CH-UA-Full-Version-List"] if page else None
    ok = bool(full) and "Headless" not in full and re.search(r'"Chromium";v="\d+\.\d+\.\d+\.\d+"', full) and top and top[-1]["full"] == full
elif what == "iframe":
    ok = framed and all(chrome(entry["User-Agent"]) for entry in framed) and inner and all(chrome(entry["ua"]) for entry in inner)
elif what == "agent-tab":
    ok = first and chrome(first[0]["User-Agent"])
elif what == "opened-url":
    ok = opened and chrome(opened[0]["User-Agent"])
else:
    sys.exit(f"no check {what}")
print(f"  {what}: {'yes' if ok else 'no'}")
PY
}

steps() {
  settle 12
  # Trusts the repository.
  press "" Return
  settle 2

  echo "== a page and its cross-site iframe"
  palette "marley: open browser"
  settle 6
  press CTRL l
  settle 1
  type_text "$SITE/headers?frame=http://localhost:$PORT/headers"
  press "" Return
  settle 4
  # The second load carries the high-entropy hints the first answer asked for.
  press CTRL r
  settle 4
  shot 539-01-page

  echo "== an agent's new tab"
  out mcp_agent navigate --new-tab "$SITE/headers?first"
  settle 3
  # Its tab opens beside the focused one without the focus; the next tab is it.
  press CTRL Page_Down
  settle 2
  shot 539-02-agent-tab

  echo "== a URL Marley opens itself"
  out mcp_agent open-url "$SITE/headers?opened" "$E2E_WORK/repo"
  settle 4
  shot 539-03-opened-url

  echo "== what the server heard"
  heard
  for what in user-agent brands full-version-list iframe agent-tab opened-url; do
    check "$what"
  done | tee "$E2E_WORK/checks.txt"
  expect "the user agent says Chrome/, not HeadlessChrome, in every request and script" \
    holds "$E2E_WORK/checks.txt" "user-agent: yes"
  expect "the header's brands are the script's, no headless one, on Linux" \
    holds "$E2E_WORK/checks.txt" "brands: yes"
  expect "the full version list is there, Chromium's, and the script's agrees" \
    holds "$E2E_WORK/checks.txt" "full-version-list: yes"
  expect "the cross-site iframe says the same" holds "$E2E_WORK/checks.txt" "iframe: yes"
  expect "an agent's new tab's first request says Chrome/" holds "$E2E_WORK/checks.txt" "agent-tab: yes"
  expect "a tab Marley opened at a URL says Chrome/ on its first request" \
    holds "$E2E_WORK/checks.txt" "opened-url: yes"
}
