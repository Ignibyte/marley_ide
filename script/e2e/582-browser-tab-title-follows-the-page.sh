# shellcheck shell=bash
# #582's e2e test: a Browser tab's title follows a title its page's script sets after the load,
# which Chromium reports through no CDP event. A page sets its title from a timer: the tab and
# its rail row follow (REQ-001), and they follow while the tab is behind another, which it went
# behind with its first title (REQ-002). A page sets its title after an IndexedDB read (REQ-003).
# An iframe's own title leaves its page's alone (REQ-004). A title holding half of a surrogate
# pair arrives with a replacement character in its place (REQ-006). Chromium runs offline.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

setup() {
  offline_chromium
  mkdir -p "$E2E_WORK/site"
  cat >"$E2E_WORK/site/timer.html" <<'HTML'
<!doctype html><html><head><title>Timer page</title></head>
<body style="margin:0;font:32px sans-serif;background:#eef1f8"><h1 style="margin:40px">A timer sets this page's title</h1>
<script>
const again = location.search === '?again';
setTimeout(() => {
  document.title = 'Changed by a timer' + (again ? ' again' : '');
}, again ? 7000 : 3000);
</script></body></html>
HTML
  cat >"$E2E_WORK/site/stored.html" <<'HTML'
<!doctype html><html><head><title>Stored page</title></head>
<body style="margin:0;font:32px sans-serif;background:#f6f1e8"><h1 style="margin:40px">Its title comes from storage</h1>
<script>
const opening = indexedDB.open('marley-582', 1);
opening.onupgradeneeded = () => opening.result.createObjectStore('store');
opening.onsuccess = () => {
  const database = opening.result;
  const writing = database.transaction('store', 'readwrite');
  writing.objectStore('store').put('stored', 'value');
  writing.oncomplete = () => {
    const reading = database.transaction('store').objectStore('store').get('value');
    reading.onsuccess = () => { document.title = 'Read from storage: ' + reading.result; };
  };
};
</script></body></html>
HTML
  cat >"$E2E_WORK/site/framed.html" <<'HTML'
<!doctype html><html><head><title>Framed page</title></head>
<body style="margin:0;font:32px sans-serif"><h1 style="margin:40px">A page with an iframe</h1>
<iframe src="inner.html" style="width:600px;height:200px;margin:0 40px"></iframe>
</body></html>
HTML
  cat >"$E2E_WORK/site/inner.html" <<'HTML'
<!doctype html><html><head><title>Inner</title></head><body>The iframe
<script>setTimeout(() => { document.title = 'Inner title'; }, 500);</script>
</body></html>
HTML
  cat >"$E2E_WORK/site/half.html" <<'HTML'
<!doctype html><html><head><title>Half page</title></head>
<body style="margin:0;font:32px sans-serif"><h1 style="margin:40px">Half a character in its title</h1>
<script>setTimeout(() => { document.title = 'Half \uD83D here'; }, 1000);</script>
</body></html>
HTML
  SITE=http://127.0.0.1:$(serve_site site)
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

# Goes to `url` in the focused Browser tab.
go_to() {
  press CTRL l
  settle 1
  type_text "$1"
  press "" Return
  settle "${2:-4}"
}

# The tabs as the agent tools list them, kept in `$E2E_WORK/tabs.txt`.
tabs() {
  mcp_agent tabs | tee "$E2E_WORK/tabs.txt"
}

steps() {
  settle 12
  # Trusts the repository.
  press "" Return
  settle 2
  palette "marley: open browser"
  settle 6

  echo "== a timer sets the page's title after the load"
  go_to "$SITE/timer.html" 5
  shot 582-01-timer
  tabs
  expect "browser_tabs names the timer's title" holds "$E2E_WORK/tabs.txt" "'Changed by a timer' at $SITE/timer.html"

  echo "== the same while the tab is behind another"
  go_to "$SITE/timer.html?again" 1
  press CTRL t
  settle 2
  tabs
  expect "the timer page went behind with its first title" holds "$E2E_WORK/tabs.txt" "'Timer page' at $SITE/timer.html?again"
  type_text "$SITE/stored.html"
  press "" Return
  settle 7
  shot 582-02-behind
  tabs
  expect "the tab behind took its page's new title" holds "$E2E_WORK/tabs.txt" "'Changed by a timer again' at $SITE/timer.html?again"

  echo "== a title set after an IndexedDB read"
  shot 582-03-stored
  expect "the stored page's title came from storage" holds "$E2E_WORK/tabs.txt" "'Read from storage: stored' at $SITE/stored.html"

  echo "== an iframe's own title leaves its page's alone"
  go_to "$SITE/framed.html" 3
  tabs
  expect "the framed page keeps its title" holds "$E2E_WORK/tabs.txt" "'Framed page' at $SITE/framed.html"
  expect "the iframe's title is nowhere" bash -c "! grep -q 'Inner title' '$E2E_WORK/tabs.txt'"

  echo "== a title with half of a surrogate pair in it"
  go_to "$SITE/half.html" 3
  tabs
  expect "the half became a replacement character" holds "$E2E_WORK/tabs.txt" "'Half � here' at $SITE/half.html"
}
