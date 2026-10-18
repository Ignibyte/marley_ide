# shellcheck shell=bash
# #805's visual check: `show` in a Marley terminal hands Marley a file or stdin to show in a tab
# beside the terminal, typed by `--as` or the file's extension. In a scratch repository:
# - `show data.csv`: Zed's table preview of the file (`805-01-table`, REQ-001).
# - CSV from `python3 -c` piped to `show --as csv --title People -`: a table tab titled People
#   (`805-02-stdin`, REQ-001).
# - `show card.json`: the component tree drawn in a Component tab (`805-03-component`, REQ-001).
# - `show notes.md` and `show logo.png`: Zed's markdown preview and image viewer (`805-04-markdown`,
#   `805-05-image`, REQ-001).
# - The program with no terminal id, as a script outside Marley runs it: the file printed in the
#   terminal (`805-06-outside`, REQ-002).
# - `show missing.csv` and `show bad.json` (a tree with an unknown type): the reason on stderr and
#   exit 1 (`805-07-error`, REQ-003).
compositor sway

# The terminal's tab, as #803's shots found it.
TERMINAL_TAB_X=${TERMINAL_TAB_X:-400}
TERMINAL_TAB_Y=${TERMINAL_TAB_Y:-51}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  python3 - "$repo" <<'PY'
import json, pathlib, struct, sys, zlib
repo = pathlib.Path(sys.argv[1])
teams = ["editor", "terminal", "browser", "agents"]
lines = ["Name,Team,Commits,Lines"]
for n in range(1, 21):
    lines.append(f"user{n:02d},{teams[n % 4]},{(n * 37) % 90 + 3},{n * 120}")
(repo / "data.csv").write_text("\n".join(lines) + "\n")
card = {"type": "card", "title": "Deploy", "tone": "success", "children": [
    {"type": "stat", "label": "Deploys today", "value": 4, "change": "+1 since noon", "tone": "success"},
    {"type": "progress", "label": "Rollout", "value": 80}]}
(repo / "card.json").write_text(json.dumps(card, indent=2) + "\n")
(repo / "bad.json").write_text(json.dumps({"type": "column", "children": [{"type": "gauge"}]}) + "\n")
(repo / "notes.md").write_text("""# Release notes

Shipped this week:

- **Terminal:** tables from block output
- **Browser:** agent pages

| Area | Owner |
|------|-------|
| Terminal | Chad |
""")
width, height = 240, 120
rows = b"".join(b"\x00" + b"".join(
    bytes((40 + x // 2, 120, 200 - y)) for x in range(width)) for y in range(height))
def chunk(kind, data):
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))
png = (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
       + chunk(b"IDAT", zlib.compress(rows)) + chunk(b"IEND", b""))
(repo / "logo.png").write_bytes(png)
(repo / "README.md").write_text("# repo\n")
PY
  open_path "$repo"
}

# Types a command at the terminal's prompt and runs it, in one `wtype`.
run_in_terminal() {
  WAYLAND_DISPLAY=$SWAY_DISPLAY wtype -d 15 "$1" -s 300 -k Return
}

# Brings the terminal's tab to the front.
terminal_tab() {
  click "$TERMINAL_TAB_X" "$TERMINAL_TAB_Y"
  settle 1
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== show data.csv"
  run_in_terminal "show data.csv"
  settle 4
  shot 805-01-table

  echo "== stdin as csv, titled"
  terminal_tab
  run_in_terminal "python3 -c \"print('city,temp'); print('Austin,31'); print('Denver,18'); print('Boston,15')\" | show --as csv --title People -"
  settle 4
  shot 805-02-stdin

  echo "== a component tree"
  terminal_tab
  run_in_terminal "show card.json"
  settle 4
  shot 805-03-component

  echo "== markdown and an image"
  terminal_tab
  run_in_terminal "show notes.md"
  settle 4
  shot 805-04-markdown
  terminal_tab
  run_in_terminal "show logo.png"
  settle 4
  shot 805-05-image

  echo "== outside Marley: the input printed"
  terminal_tab
  run_in_terminal "clear"
  settle 1
  run_in_terminal "env -u MARLEY_TERMINAL_ID \"\$MARLEY_BIN\" show notes.md; echo \"exit \$?\""
  settle 3
  shot 805-06-outside

  echo "== errors"
  run_in_terminal "show missing.csv; echo \"exit \$?\""
  settle 2
  run_in_terminal "show bad.json; echo \"exit \$?\""
  settle 3
  shot 805-07-error
}
