# shellcheck shell=bash
# #518's e2e test: the fuller pick bundle. Five pages, each in a Browser tab of its own, so each
# tab's pick tray starts empty:
# - `r18.html`, React 18.3.1's UMD build with a `Card` holding a `SaveButton`, each created with
#   the `__source` a JSX dev transform writes, pointing at the scratch repository's files;
# - `r19.html`, the scratch repository's `src/app19.jsx` bundled by esbuild against React 19.2.8
#   with an inline source map, a `Panel` holding a `SaveButton19`;
# - `secrets.html`, a form with an email field whose script copies what is typed into its `value`
#   attribute (as React does), a hidden CSRF input, a secret-named attribute, links and an action
#   with queries, a script, a button labelled with a token, and a Go button with a known style;
# - `long.html`, a list whose HTML carries a token across its 4,096th character;
# - `select.html`, a paragraph to select and a list beside it.
# Each pick is read by a stand-in agent through `browser_pick` and saved as JSON; the checks read
# the saved answers. The fake secrets are made in the setup from pieces and listed in the run's
# folder, so this file holds none. The React builds come from projects on the box, or from
# E2E_REACT18_MODULES and E2E_REACT19_MODULES (each a `node_modules`); the setup stops, naming
# what is missing, without them.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

SITE=
REACT18=${E2E_REACT18_MODULES:-/srv/stacks/ignibyte/Ignibyte-Marketing-Site/node_modules}
REACT19=${E2E_REACT19_MODULES:-/srv/stacks/scorchkit_home/node_modules}

# The page's top left in the 1600x1000 window, as #490 measured it, how far one row of the pick
# tray moves it down (#496), and the toolbar's pick button.
PAGE_X=${PAGE_X:-260}
PAGE_Y=${PAGE_Y:-109}
TRAY_ROW=${TRAY_ROW:-36}
PICK_X=${PICK_X:-1288}
PICK_Y=${PICK_Y:-87}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo name missing=()
  for name in "$REACT18/react/umd/react.development.js" "$REACT18/react-dom/umd/react-dom.development.js" \
    "$REACT19/react/jsx-dev-runtime.js" "$REACT19/react-dom/client.js" "$REACT19/.bin/esbuild"; do
    [[ -e $name ]] || missing+=("$name")
  done
  if ((${#missing[@]} > 0)); then
    echo "#518 needs React 18's UMD build and React 19 with esbuild; missing: ${missing[*]}" >&2
    echo "set E2E_REACT18_MODULES and E2E_REACT19_MODULES to node_modules folders that hold them" >&2
    return 1
  fi
  mkdir -p "$home" "$bin" "$repo/src" "$E2E_WORK/site"
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
  git init -q -b pick "$repo"
  cp "$REACT18/react/umd/react.development.js" "$REACT18/react-dom/umd/react-dom.development.js" \
    "$E2E_WORK/site/"
  python3 - "$E2E_WORK" <<'PY'
import json, pathlib, sys

work = pathlib.Path(sys.argv[1])
repo, site = work / "repo", work / "site"

# The fake secrets, made from pieces so that no file of the repository holds one.
secrets = {
    "button token": "gh" + "p_" + "Fake" * 9,
    "list token": "gh" + "p_" + "Long" * 9,
    "api key": "k" + "ey-" + "Fake" * 6,
    "csrf": "csrf" + "Fake" * 6,
    "session": "sess" + "Fake" * 5,
    "access token": "tok" + "Fake" * 5,
    "reference": "ref" + "Fake" * 4,
    "email": "typed." + "person@example.test",
}
(work / "secrets.json").write_text(json.dumps(secrets))

(repo / "src" / "SaveButton.jsx").write_text(
    "export function SaveButton() {\n"
    "  const label = 'Save 18';\n"
    "  return (\n"
    '    <button id="save18">{label}</button>\n'
    "  );\n"
    "}\n"
)
(repo / "src" / "Card.jsx").write_text(
    "export function Card({ children }) {\n"
    '  return <section className="card">{children}</section>;\n'
    "}\n"
)
(repo / "src" / "SaveButton19.jsx").write_text(
    "export function SaveButton19() {\n"
    "  const label = 'Save 19';\n"
    "  return (\n"
    '    <button id="save19">{label}</button>\n'
    "  );\n"
    "}\n"
)
(repo / "src" / "app19.jsx").write_text(
    "import { createRoot } from 'react-dom/client';\n"
    "import { SaveButton19 } from './SaveButton19.jsx';\n"
    "\n"
    "function Panel() {\n"
    '  return <main className="panel"><SaveButton19 /></main>;\n'
    "}\n"
    "\n"
    "createRoot(document.getElementById('root')).render(<Panel />);\n"
)

style = """<style>
body { margin: 0; font: 18px sans-serif; }
.card, .panel { position: absolute; left: 40px; top: 40px; width: 300px; height: 120px; border: 1px solid #ccc; }
#save18, #save19 { position: absolute; left: 20px; top: 30px; width: 200px; height: 44px; font-size: 18px; }
</style>"""
source = lambda name, line, column: json.dumps(
    {"fileName": str(repo / "src" / name), "lineNumber": line, "columnNumber": column}
)
(site / "r18.html").write_text(f"""<!doctype html><html><head><title>React 18</title>{style}
<script src="react.development.js"></script><script src="react-dom.development.js"></script>
</head><body><div id="root"></div><script>
const e = React.createElement;
function SaveButton() {{
  return e('button', {{ id: 'save18', __source: {source("SaveButton.jsx", 4, 5)} }}, 'Save 18');
}}
function Card() {{
  return e('section', {{ className: 'card', __source: {source("Card.jsx", 2, 10)} }},
    e(SaveButton, {{ __source: {source("Card.jsx", 2, 10)} }}));
}}
ReactDOM.createRoot(document.getElementById('root')).render(e(Card, null));
</script></body></html>
""")
(site / "r19.html").write_text(
    f'<!doctype html><html><head><title>React 19</title>{style}</head>'
    '<body><div id="root"></div><script src="app19.js"></script></body></html>\n'
)

(site / "secrets.html").write_text(f"""<!doctype html><html><head><title>Secrets</title><style>
body {{ margin: 0; font: 16px sans-serif; }}
#checkout {{ position: absolute; left: 40px; top: 40px; width: 500px; height: 300px; margin: 0; border: 1px solid #999; background: #f7f7f7; }}
#email {{ position: absolute; left: 20px; top: 20px; width: 260px; height: 30px; }}
#next {{ position: absolute; left: 20px; top: 70px; }}
#auth {{ position: absolute; left: 160px; top: 70px; }}
#token-button {{ position: absolute; left: 20px; top: 110px; width: 420px; height: 36px; }}
#go {{ position: absolute; left: 20px; top: 170px; width: 120px; height: 40px; padding: 8px 16px; color: rgb(10, 20, 30); }}
</style></head><body>
<form id="checkout" data-api-key="{secrets['api key']}" action="/submit?session={secrets['session']}#done">
<input type="email" id="email" name="email" autocomplete="email">
<input type="hidden" name="csrf_token" value="{secrets['csrf']}">
<a id="next" href="https://example.com/next?ref={secrets['reference']}#top">Next page</a>
<a id="auth" href="https://example.com/cb?access_token={secrets['access token']}">Sign in</a>
<button type="button" id="token-button">Token {secrets['button token']}</button>
<button type="button" id="go">Go</button>
<script>document.getElementById('email').addEventListener('input', (event) => event.target.setAttribute('value', event.target.value));</script>
</form></body></html>
""")

# The list's HTML, as Chromium serializes it, puts the token across its 4,096th character.
opening = '<ul id="long">'
items = []
length = len(opening)
number = 1
while length < 4000:
    item = f"<li>Item {number:04} of the long list</li>"
    items.append(item)
    length += len(item)
    number += 1
lead = "<li>Token: "
start = 4080
pad = start - length - len(lead) - len("<li></li>")
items.append("<li>" + "x" * pad + "</li>")
items.append(lead + secrets["list token"] + "</li>")
serialized = opening + "".join(items)
assert serialized.index(secrets["list token"]) == start, serialized.index(secrets["list token"])
items.extend(f"<li>Item {tail:04} after the token</li>" for tail in range(1, 40))
(site / "long.html").write_text(
    "<!doctype html><html><head><title>Long list</title><style>"
    "body { margin: 0; font: 16px sans-serif; }"
    " #long { position: absolute; left: 40px; top: 40px; width: 400px; height: 80px; margin: 0;"
    " padding: 0; background: #eef; }"
    " #long li { display: none; }"
    "</style></head><body>" + opening + "".join(items) + "</ul></body></html>\n"
)

(site / "select.html").write_text("""<!doctype html><html><head><title>Selection</title><style>
body { margin: 0; font: 20px sans-serif; }
#para { position: absolute; left: 40px; top: 40px; margin: 0; }
#small { position: absolute; left: 40px; top: 120px; margin: 0; padding: 0 0 0 24px; line-height: 30px; }
</style></head><body>
<p id="para">Select this sentence for the agent.</p>
<ul id="small"><li>First item</li><li id="second">Second item</li><li>Third item</li></ul>
</body></html>
""")
PY
  # React 19 has no UMD build: esbuild bundles the app with the dev JSX transform and an inline
  # map. Its modules resolve through a link beside the repository, never inside it, so the
  # project's scan never walks them.
  ln -s "$REACT19" "$E2E_WORK/node_modules"
  "$REACT19/.bin/esbuild" "$repo/src/app19.jsx" --bundle --jsx=automatic --jsx-dev \
    --sourcemap=inline "--define:process.env.NODE_ENV=\"development\"" \
    --outfile="$E2E_WORK/site/app19.js" --log-level=warning || return 1
  offline_chromium
  SITE=http://127.0.0.1:$(serve_site site)
  open_path "$repo"
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

# Opens `$1` in a new Browser tab, whose address bar has the focus.
new_tab() {
  palette "marley: new browser tab"
  settle 3
  type_text "$1"
  press "" Return
  settle "${2:-4}"
}

# The window's point for (x, y) in the page, with `rows` rows in the tab's pick tray above it.
page_point() {
  local rows=${3:-0} tray=0
  ((rows > 0)) && tray=$((rows * TRAY_ROW + 1))
  echo "$((PAGE_X + $1)) $((PAGE_Y + tray + $2))"
}

# Picks the element at (x, y) in the page, with `rows` rows in the tray, through the toolbar's
# pick button.
pick_at() {
  click "$PICK_X" "$PICK_Y"
  settle 1
  # shellcheck disable=SC2046
  click $(page_point "$1" "$2" "${3:-0}")
  settle "${4:-3}"
}

# Drags along the page's row `$1` from x `$2` to x `$3`, in steps with pauses, as a hand does. It
# ends inside the text: on this page, whose paragraph and list are placed absolutely over a body
# with no height, a move past a line's end collapses the selection in Chromium itself (#580).
drag_across() {
  local y=$1 x=$2
  # shellcheck disable=SC2046
  pointer_to $(page_point "$x" "$y")
  settle 0.5
  pointer_down
  settle 0.3
  while ((x < $3)); do
    x=$((x + 60 < $3 ? x + 60 : $3))
    # shellcheck disable=SC2046
    pointer_to $(page_point "$x" "$y")
    settle 0.1
  done
  settle 0.5
  pointer_up
  settle 1
}

# Reads pick `$1` as the stand-in agent does and saves its answer as `pick-$2.json`.
save_pick() {
  mcp_agent pick "$1" "$(shot_file "518-pick-$1.jpg")"
  mcp_agent pick-json "$1" "$E2E_WORK/pick-$2.json"
}

# Whether the saved answer `pick-$1.json` makes the Python expression `$2` true, with `answer`
# the answer and `bundle` its bundle.
answer() {
  python3 -c '
import json, sys
answer = json.load(open(sys.argv[1]))
bundle = answer["bundle"]
sys.exit(0 if eval(sys.argv[2]) else 1)' "$E2E_WORK/pick-$1.json" "$2"
}

# Whether no fake secret, and no piece of one twelve characters long, is in any of the files.
no_secrets_in() {
  python3 - "$E2E_WORK/secrets.json" "$@" <<'PY'
import json, sys
secrets = json.load(open(sys.argv[1]))
found = []
for path in sys.argv[2:]:
    text = open(path).read()
    for what, secret in secrets.items():
        pieces = {secret[start:start + 12] for start in range(0, max(1, len(secret) - 11))}
        if any(piece in text for piece in pieces):
            found.append(f"{what} in {path.rsplit('/', 1)[-1]}")
print("  found: " + ("; ".join(found) or "nothing"))
sys.exit(1 if found else 0)
PY
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  echo "== React 18: the button of a Card, created with __source"
  new_tab "$SITE/r18.html" 5
  pick_at 160 92 0 4
  shot 518-01-react18-pick
  save_pick 1 r18
  expect "React 18: the chain, outermost first" answer r18 'bundle["component"]["chain"] == ["Card", "SaveButton"]'
  expect "React 18: the debug source, in the project" answer r18 \
    'bundle["component"]["source"]["from"] == "debug source" and bundle["component"]["source"]["file"] == "src/SaveButton.jsx" and bundle["component"]["source"]["line"] == 4 and bundle["component"]["source"]["column"] == 5'
  echo "== React 19: the button of a Panel, from an esbuild bundle with an inline map"
  new_tab "$SITE/r19.html" 6
  pick_at 160 92 0 6
  shot 518-02-react19-pick
  save_pick 2 r19
  expect "React 19: the chain, outermost first" answer r19 'bundle["component"]["chain"] == ["Panel", "SaveButton19"]'
  expect "React 19: the debug stack's frame, through the map, in the project" answer r19 \
    'bundle["component"]["source"]["from"] == "debug stack" and bundle["component"]["source"]["file"] == "src/SaveButton19.jsx" and bundle["component"]["source"]["line"] == 4'
  echo "== the secrets page: an e-mail typed, then the form picked by its padding"
  new_tab "$SITE/secrets.html" 4
  # shellcheck disable=SC2046
  click $(page_point 190 75)
  settle 1
  type_text "$(python3 -c 'import json, sys; print(json.load(open(sys.argv[1]))["email"])' "$E2E_WORK/secrets.json")"
  settle 1
  pick_at 480 300 0
  shot 518-03-secrets-pick
  save_pick 3 form
  expect "the form's HTML holds no script" answer form '"<script" not in bundle["html"] and "<form" in bundle["html"]'
  expect "the secret-named attribute, the hidden field and the e-mail's value are gone" answer form \
    '"data-api-key=\"[redacted]\"" in bundle["html"] and "value=" not in bundle["html"]'
  expect "the links and the action lose their queries; the token's link is redacted" answer form \
    '"href=\"https://example.com/next\"" in bundle["html"] and "href=\"[redacted]\"" in bundle["html"] and "action=\"/submit\"" in bundle["html"]'
  expect "a page without React has no component" answer form 'bundle["component"] is None'
  pick_at 120 230 1
  save_pick 4 go
  expect "the Go button's styles as its style sets them" answer go \
    'bundle["styles"]["padding"] == "8px 16px" and bundle["styles"]["color"] == "rgb(10, 20, 30)" and len(bundle["styles"]) == 16'
  pick_at 270 168 2
  save_pick 5 token
  mcp_agent picks | tee "$E2E_WORK/picks.txt"
  expect "the token's button reaches the agent redacted" answer token \
    '"[redacted" in answer["summary"] and "[redacted" in (bundle["name"] or "") and "[redacted" in bundle["html"]'
  expect "and the list of picks shows it redacted" holds "$E2E_WORK/picks.txt" "[redacted"
  echo "== a long list, whose HTML carries a token across its 4,096th character"
  new_tab "$SITE/long.html" 4
  pick_at 200 80 0
  save_pick 6 long
  expect "the HTML is cut at 4,096 characters and marked" answer long \
    'len(bundle["html"]) == 4096 + len(" (truncated)") and bundle["html"].endswith(" (truncated)")'
  echo "== a selection by a drag, then a list item beside it"
  new_tab "$SITE/select.html" 4
  drag_across 52 42 300
  pick_at 100 165 0
  shot 518-04-selection-pick
  save_pick 7 item
  expect "the page's selection" answer item '(bundle["selected_text"] or "").startswith("Select this sentence")'
  expect "the siblings' texts" answer item 'bundle["nearby_text"] == ["First item", "Third item"]'
  echo "== no secret, and no piece of one, reached the agent"
  expect "the saved answers and the list hold none of the fake secrets" \
    no_secrets_in "$E2E_WORK"/pick-*.json "$E2E_WORK/picks.txt"
  cat "$E2E_WORK/leak.log"
  expect "nothing reached the system browser" test ! -s "$E2E_WORK/leak.log"
}
