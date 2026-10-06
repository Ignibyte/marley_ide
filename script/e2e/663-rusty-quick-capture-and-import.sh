# shellcheck shell=bash
# #663's visual check: capture a line or a URL into Rusty's brain and import a vault, from the
# palette. `marley_rusty`'s stand-in `rusty-mcp`, named by `MARLEY_RUSTY_MCP`, over a scratch
# state folder (`RUSTY_STAND_IN_STATE`) whose `today` file fixes the day and whose `vault/` holds
# made-up pages, never the user's brain (R-D8). A fixture web server on 127.0.0.1 serves a page
# that answers after 7 s, past the 5 s every other call to Rusty keeps, and nothing else; a
# scratch Obsidian vault is the import's source. The run's settings turn `use_system_path_prompts`
# off, so the folder comes from Zed's own prompt.
#
# The form (`663-01-capture-form`), a refusal (`663-02-refused`), a line captured
# (`663-03-captured`), today's note (`663-04-today`), the inbox (`663-05-inbox`), a URL while
# Rusty fetches (`663-06-capturing`) and its page (`663-07-source`), a failed fetch
# (`663-08-fetch-failed`), a refused URL (`663-09-url-refused`), the folder prompt
# (`663-10-folder`), the plan (`663-11-plan`), the report (`663-12-imported`) and its page with
# the vault in the Brain view (`663-13-report`).
compositor sway

# Where things sit, from the first run's shots: the rail's Brain button, a toast's Open, and the
# import form's Open Report.
RAIL_HEADER_Y=${RAIL_HEADER_Y:-16}
BRAIN_X=${BRAIN_X:-42}
TOAST_OPEN_X=${TOAST_OPEN_X:-1171}
TOAST_OPEN_Y=${TOAST_OPEN_Y:-933}
OPEN_REPORT_X=${OPEN_REPORT_X:-1011}
OPEN_REPORT_Y=${OPEN_REPORT_Y:-512}
TODAY=2026-10-06

vault() { echo "$E2E_WORK/rusty/vault"; }

calls() { cat "$E2E_WORK/rusty/calls" 2>/dev/null; }

# Whether the stand-in was asked to run tool `$1` with exactly the arguments `$2`.
called_with() { calls | grep -qF " tools/call $1 $2"; }

# A made-up page `$1` of the brain with the body `$2`.
vault_page() {
  local path
  path=$(vault)/$1.md
  mkdir -p "$(dirname "$path")"
  printf -- '---\ntitle: %s\n---\n%s\n' "$(basename "$1")" "$2" >"$path"
}

# A page `$1` of the Obsidian vault, its text from stdin.
obsidian_page() {
  mkdir -p "$(dirname "$E2E_WORK/obsidian/$1.md")"
  cat >"$E2E_WORK/obsidian/$1.md"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle 2
}

# The fixture web server: `/slow` answers after 7 s with an article, anything else is missing.
start_web() {
  cat >"$E2E_WORK/web.py" <<'PY'
import http.server, sys, time

class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path != "/slow":
            self.send_error(404, "Not Found")
            return
        time.sleep(7)
        body = (b"<!doctype html><html><head><title>Notes on Slow Pages</title></head><body>"
                b"<h1>Notes on Slow Pages</h1><p>A page that takes its time to answer.</p>"
                b"<p>Rusty waits for it, and so does Marley.</p></body></html>")
        self.send_response(200)
        self.send_header("Content-Type", "text/html")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *_):
        pass

server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
with open(sys.argv[1], "w") as out:
    out.write(str(server.server_address[1]))
server.serve_forever()
PY
  python3 "$E2E_WORK/web.py" "$E2E_WORK/web.port" &
  WEB_PID=$!
  local tries=0
  until [[ -s $E2E_WORK/web.port ]] || ((tries++ > 50)); do
    sleep 0.1
  done
  WEB=http://127.0.0.1:$(cat "$E2E_WORK/web.port")
}

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home" "$(vault)/archive"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  profile_setting marley.rusty.enabled true
  profile_setting marley.rusty.connection '"embedded"'
  profile_setting use_system_path_prompts false
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  echo "$TODAY" >"$E2E_WORK/rusty/today"
  vault_page home "Where the day starts."
  vault_page notes/old "A note the brain already has."
  printf -- '---\ntags: [idea]\n---\n# Atlas\n\nThe project. Its [[plan]], and [[nowhere]] yet.\n' |
    obsidian_page projects/atlas
  printf '# Plan\n\nWhat comes first. #roadmap\n' | obsidian_page projects/plan
  printf '# Old\n\nThe vault'"'"'s own old note.\n' | obsidian_page notes/old
  mkdir -p "$E2E_WORK/obsidian/assets" "$E2E_WORK/obsidian/.obsidian"
  printf 'not really a picture\n' >"$E2E_WORK/obsidian/assets/map.png"
  printf '{"items": [{"type": "file", "path": "projects/atlas.md", "title": "Atlas"}, {"type": "graph", "title": "Graph view"}]}\n' \
    >"$E2E_WORK/obsidian/.obsidian/bookmarks.json"
  start_web
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

teardown() {
  if [[ -n ${WEB_PID:-} ]]; then
    kill "$WEB_PID" 2>/dev/null || true
  fi
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2

  echo "== a line to today's note"
  palette "rusty: capture to today"
  shot 663-01-capture-form
  press "" Return
  settle 2
  shot 663-02-refused
  type_text "Call the printer about the invoice"
  press "" Return
  settle 2
  shot 663-03-captured
  expect "the line went to the daily note" \
    called_with brain_capture '{"target": "daily", "text": "Call the printer about the invoice"}'

  echo "== today's note"
  palette "rusty: open today"
  settle 2
  shot 663-04-today
  expect "the line is under the timeline" \
    holds "$(vault)/daily/$TODAY.md" "## Timeline" "(mcp) — Call the printer about the invoice"

  echo "== the inbox"
  palette "rusty: capture to inbox"
  type_text "Read the Orca notes again"
  press "" Return
  settle 2
  click "$TOAST_OPEN_X" "$TOAST_OPEN_Y"
  settle 3
  shot 663-05-inbox

  echo "== a slow URL"
  palette "rusty: capture url"
  type_text "$WEB/slow"
  press "" Return
  settle 2
  shot 663-06-capturing
  settle 8
  shot 663-07-source
  expect "the page was kept under sources" test -f "$(vault)/sources/notes-on-slow-pages.md"

  echo "== a missing URL"
  palette "rusty: capture url"
  type_text "$WEB/missing"
  press "" Return
  settle 3
  shot 663-08-fetch-failed

  echo "== a refused URL"
  palette "rusty: capture url"
  type_text "ftp://example.com/file"
  press "" Return
  settle 2
  shot 663-09-url-refused
  press "" Escape
  settle 1

  echo "== the import"
  click "$BRAIN_X" "$RAIL_HEADER_Y"
  settle 2
  palette "rusty: import vault"
  press CTRL a
  type_text "$E2E_WORK/obsidian"
  settle 2
  shot 663-10-folder
  press "" Return
  settle 3
  shot 663-11-plan
  press "" Return
  settle 3
  shot 663-12-imported
  expect "the vault's pages came in" test -f "$(vault)/projects/atlas.md"
  expect "the brain's own note was left" holds "$(vault)/notes/old.md" "A note the brain already has."
  click "$OPEN_REPORT_X" "$OPEN_REPORT_Y"
  settle 3
  shot 663-13-report
  calls | grep -E 'brain_capture|source_capture|brain_import'
}
