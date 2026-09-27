# shellcheck shell=bash
# #505's e2e test: pick, fix, check. A loopback page has a Save button (a test id, an id, 8 px of
# padding from app.css) and a Delete button (an id React's `useId` might make, a hashed class, no
# test id). The user picks both. Each "fix" rewrites the site's files and reloads the tab, as an
# agent's edit and a hot reload would, and the tray's Check (or the stand-in agent's
# `browser_check_pick`) finds the element again and says what changed:
# - Save's padding and label changed: found by its test id, the two crops side by side, the
#   changes listed;
# - Delete's id and class changed: found by its role and name;
# - a second Delete far below: the one nearest the old box taken;
# - Save pushed 2,000 px down: scrolled into view and cropped with the pick's margin;
# - both Deletes gone: not found, and no new crop;
# - the tray's verdicts, one of which opens its comparison again;
# - Save's label holding a fake token: the agent's copy of the check redacts it.
# The fake token is made in the setup from pieces, so this file holds none. Chromium runs offline.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

SITE=

# The page's top left in the 1600x1000 window, as #490 measured it, how far one row of the pick
# tray moves it down (#496), the toolbar's pick and reload buttons, and, in a tray row, Check and
# a point near the right edge of the verdict left of it, which stays put as its label grows
# (#505).
PAGE_X=${PAGE_X:-260}
PAGE_Y=${PAGE_Y:-109}
TRAY_ROW=${TRAY_ROW:-36}
PICK_X=${PICK_X:-1288}
PICK_Y=${PICK_Y:-87}
RELOAD_X=${RELOAD_X:-334}
RELOAD_Y=${RELOAD_Y:-87}
CHECK_X=${CHECK_X:-1297}
VERDICT_X=${VERDICT_X:-1255}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo name
  mkdir -p "$home" "$bin" "$repo" "$E2E_WORK/site"
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
  git init -q -b check "$repo"
  # The fake token, made from pieces so that no file of the repository holds one.
  python3 -c 'import json, sys; json.dump({"token": "gh" + "p_" + "Fix" * 12}, open(sys.argv[1], "w"))' \
    "$E2E_WORK/secrets.json"
  write_page 8 Save 0 ':r5:|css-1q2w3e|80'
  offline_chromium
  SITE=http://127.0.0.1:$(serve_site site)
  open_path "$repo"
}

teardown() {
  browser_teardown
}

# Writes the site's page: Save with `$1` px of padding and the label `$2`, below a spacer of `$3`
# px, and a Delete for each `id|class|top` after them. The stylesheet's URL changes each time, so
# a reload fetches it again.
write_page() {
  python3 - "$E2E_WORK/site" "$@" <<'PY'
import html, pathlib, sys

site = pathlib.Path(sys.argv[1])
padding, label, spacer = sys.argv[2], sys.argv[3], sys.argv[4]
counter = site / "version"
version = int(counter.read_text()) + 1 if counter.exists() else 1
counter.write_text(str(version))
deletes = []
styles = []
for entry in sys.argv[5:]:
    element_id, name, top = entry.split("|")
    deletes.append(f'<button id="{html.escape(element_id)}" class="{name} delete">Delete</button>')
    styles.append(f".{name} {{ top: {top}px; }}")
(site / "app.css").write_text(f"""body {{ margin: 0; font: 18px sans-serif; background: #f4f1ea; }}
#spacer {{ height: {spacer}px; }}
#panel {{ position: relative; margin: 40px; height: 800px; }}
#save {{ position: absolute; left: 0; top: 0; padding: {padding}px; font-size: 18px; }}
.delete {{ position: absolute; left: 0; width: 120px; height: 40px; font-size: 18px; }}
{chr(10).join(styles)}
""")
(site / "index.html").write_text(f"""<!doctype html><html><head><meta charset="utf-8"><title>Fix and check</title>
<link rel="stylesheet" href="app.css?v={version}"></head><body>
<div id="spacer"></div>
<main id="panel">
<button id="save" data-testid="save-button">{html.escape(label)}</button>
{chr(10).join(deletes)}
</main>
</body></html>
""")
PY
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
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
  settle 3
}

# The middle of the tray's row `$1`, from 0 at the top: the newest pick's.
row_y() {
  echo "$((PAGE_Y + $1 * TRAY_ROW + TRAY_ROW / 2))"
}

# Plays the fix: rewrites the page as `write_page` does, and reloads the tab.
fix() {
  write_page "$@"
  click "$RELOAD_X" "$RELOAD_Y"
  settle 4
}

# The user's Check on the pick in the tray's row `$1`. The pointer leaves the row after the click,
# so no tooltip covers the card.
check_row() {
  click "$CHECK_X" "$(row_y "$1")"
  pointer_to 800 950
  settle 4
}

# Saves the pick `$1`, with its latest check, as the stand-in agent reads it, as `pick-$2.json`.
save_pick() {
  mcp_agent pick-json "$1" "$E2E_WORK/pick-$2.json"
}

# Whether the saved answer `$1.json` makes the Python expression `$2` true, with `answer` the
# answer, `bundle` its bundle and `check` its check (none for an answer without one).
answer() {
  python3 -c '
import json, sys
answer = json.load(open(sys.argv[1]))
bundle = answer.get("bundle")
check = answer.get("check") or answer
sys.exit(0 if eval(sys.argv[2]) else 1)' "$E2E_WORK/$1.json" "$2"
}

# Whether the JPEG `$1`'s size is that of the box in the saved check `$2.json`, plus the crop's
# 16 px margin on each side, give or take a pixel.
crop_fits() {
  python3 - "$1" "$E2E_WORK/$2.json" <<'PY'
import json, struct, sys
data = open(sys.argv[1], "rb").read()
box = json.load(open(sys.argv[2]))["bundle"]["page_box"]
position = 2
while position < len(data):
    marker, length = data[position + 1], struct.unpack(">H", data[position + 2:position + 4])[0]
    if marker in (0xC0, 0xC1, 0xC2):
        height, width = struct.unpack(">HH", data[position + 5:position + 9])
        break
    position += 2 + length
print(f"  the crop: {width}x{height}; the box {box['width']:.1f}x{box['height']:.1f} and the margins")
sys.exit(0 if abs(width - (box["width"] + 32)) <= 1.5 and abs(height - (box["height"] + 32)) <= 1.5 else 1)
PY
}

# Whether the shot `$4` shows the page drawn at (x, y) of the page, with `rows` rows in the tray:
# the page's light background there, not the tab's dark one around a lone crop, which a crop's
# capture once left in place of the page.
page_drawn() {
  local point
  point=$(page_point "$1" "$2" "$3" | tr ' ' '+')
  magick "$(shot_file "$4.png")" -crop "1x1+$point" +repage -format "%[fx:luminance]" info: |
    awk '{ print "  luminance " $1; exit !($1 > 0.8) }'
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
  palette "marley: new browser tab"
  settle 3
  type_text "$SITE/index.html"
  press "" Return
  settle 5
  echo "== the user picks Save, then Delete"
  pick_at 60 60 0
  pick_at 80 140 1
  shot 505-00-picks
  mcp_agent pick 2 "$(shot_file 505-pick-2.jpg)"
  save_pick 2 delete
  expect "Delete's locators hold no generated id and no class" answer pick-delete \
    'not any(":r5:" in locator["value"] or "css-" in locator["value"] or locator["kind"] == "id" for locator in bundle["locators"])'
  echo "== the fix: Save's padding 16 px and its label Save changes; Check on Save"
  fix 16 "Save changes" 0 ':r5:|css-1q2w3e|80'
  check_row 1
  shot 505-01-check
  expect "the page is drawn whole after the check" page_drawn 300 400 2 505-01-check
  save_pick 1 save-1
  expect "Save found by its test id" answer pick-save-1 'check["found_by"] == "test id"'
  expect "its padding, text and size changes listed" answer pick-save-1 \
    '"padding: 8px → 16px" in check["changes"] and any(line.startswith("text: ") and "Save changes" in line for line in check["changes"]) and any(line.startswith("box: ") and "×" in line for line in check["changes"])'
  echo "== the fix: Delete's id and class made again; Check on Delete"
  fix 16 "Save changes" 0 '«r9»|css-9o8i7u|80'
  check_row 0
  shot 505-02-generated-names
  expect "the page is drawn whole after the check" page_drawn 300 400 2 505-02-generated-names
  save_pick 2 delete-1
  expect "Delete found by its role and name" answer pick-delete-1 'check["found_by"] == "role and name"'
  echo "== the fix: a second Delete 600 px below; Check on Delete"
  fix 16 "Save changes" 0 '«r9»|css-9o8i7u|80' '«r10»|css-7y6t5r|680'
  check_row 0
  shot 505-03-nearest
  expect "the page is drawn whole after the check" page_drawn 300 400 2 505-03-nearest
  save_pick 2 delete-2
  expect "the Delete nearest the old box taken" answer pick-delete-2 \
    'check["found_by"] == "role and name" and abs(check["bundle"]["page_box"]["y"] - bundle["page_box"]["y"]) < 5'
  echo "== the agent checks Save"
  mcp_agent check-pick 1 "$(shot_file 505-agent-check-1.jpg)" "$E2E_WORK/agent-check-1.json"
  expect "the agent's check: found by test id, with its changes" answer agent-check-1 \
    'check["found"] and check["found_by"] == "test id" and "padding: 8px → 16px" in check["changes"]'
  echo "== the fix: 2,000 px above Save; the page opens at its top"
  fix 16 "Save changes" 2000 '«r9»|css-9o8i7u|80'
  mcp_agent check-pick 1 "$(shot_file 505-agent-check-2.jpg)" "$E2E_WORK/agent-check-2.json"
  mcp_agent look | tee "$E2E_WORK/look.txt"
  expect "Save scrolled into view" python3 -c \
    'import json, re, sys; look = json.loads(re.search(r"\{.*\}", open(sys.argv[1]).read()).group(0)); sys.exit(0 if look["viewport"]["scroll_y"] > 1000 else 1)' \
    "$E2E_WORK/look.txt"
  expect "its crop is its box and the pick's margins" crop_fits "$(shot_file 505-agent-check-2.jpg)" agent-check-2
  check_row 1
  shot 505-05-scrolled
  expect "the page is drawn whole after the check" page_drawn 300 400 2 505-05-scrolled
  echo "== the fix: both Deletes gone; Check on Delete"
  fix 16 "Save changes" 0
  check_row 0
  shot 505-04-not-found
  save_pick 2 delete-3
  expect "Delete not found, and no new crop" answer pick-delete-3 \
    'check["found_by"] is None and check.get("bundle") is None and check["changes"] == []'
  echo "== the tray's verdicts; Save's reopens its comparison"
  click "$VERDICT_X" "$(row_y 1)"
  pointer_to 800 950
  settle 2
  shot 505-06-verdict
  echo "== the fix: Save's label holds a token; the agent checks it"
  fix 16 "Save $(python3 -c 'import json, sys; print(json.load(open(sys.argv[1]))["token"])' "$E2E_WORK/secrets.json")" 0
  mcp_agent check-pick 1 "$(shot_file 505-agent-check-3.jpg)" "$E2E_WORK/agent-check-3.json"
  save_pick 1 save-2
  expect "the check reaches the agent with the token redacted" answer agent-check-3 \
    'any("[redacted" in line for line in check["changes"])'
  expect "no piece of the token in the check or the pick" \
    no_secrets_in "$E2E_WORK/agent-check-3.json" "$E2E_WORK/pick-save-2.json"
  cat "$E2E_WORK/leak.log"
  expect "nothing reached the system browser" test ! -s "$E2E_WORK/leak.log"
}
