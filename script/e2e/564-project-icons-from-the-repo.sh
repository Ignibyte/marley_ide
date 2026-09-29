# shellcheck shell=bash
# #564's visual check: a project's own icon on its rail header, read from its repository alone. A
# repository with no icon keeps today's header (REQ-003); a `favicon.png` put there while Marley
# runs shows within five seconds (REQ-001, REQ-004); with it gone, the SVG an `index.html`
# declares shows (REQ-002); a `favicon.png` that does not decode is skipped for that SVG, and one
# over 256 KiB with no page left leaves the header as today, each skip logged (REQ-005).
compositor sway

setup() {
  local repo=$E2E_WORK/repo
  mkdir -p "$repo"
  write_png_maker
  git init -q -b main "$repo"
  open_path "$repo"
}

# `png <file> <side> <red,green,blue> [noise]`: a PNG of that side, one colour or random noise.
write_png_maker() {
  cat >"$E2E_WORK/png.py" <<'PY'
import os
import struct
import sys
import zlib

path, side, colour = sys.argv[1], int(sys.argv[2]), [int(c) for c in sys.argv[3].split(",")]
noise = len(sys.argv) > 4


def chunk(kind, data):
    body = kind + data
    return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body) & 0xFFFFFFFF)


rows = b"".join(
    b"\x00" + (os.urandom(side * 4) if noise else bytes(colour + [255]) * side) for _ in range(side)
)
png = (
    b"\x89PNG\r\n\x1a\n"
    + chunk(b"IHDR", struct.pack(">IIBBBBB", side, side, 8, 6, 0, 0, 0))
    + chunk(b"IDAT", zlib.compress(rows, 9))
    + chunk(b"IEND", b"")
)
open(path, "wb").write(png)
PY
}

png() { python3 "$E2E_WORK/png.py" "$@"; }

header() { shot "$1"; }

steps() {
  local repo=$E2E_WORK/repo
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== no icon: the header as today"
  header 564-01-none

  echo "== a favicon.png appears"
  png "$repo/favicon.png" 64 220,40,40
  settle 5
  header 564-02-png

  echo "== it goes; the page's SVG shows"
  rm "$repo/favicon.png"
  cat >"$repo/brand.svg" <<'SVG'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><circle cx="16" cy="16" r="14" fill="#2f7dd8"/></svg>
SVG
  cat >"$repo/index.html" <<'HTML'
<!doctype html>
<html><head><title>repo</title><link href="/brand.svg?v=2" rel='icon' type="image/svg+xml"></head><body></body></html>
HTML
  settle 5
  header 564-03-svg

  echo "== a favicon.png that does not decode: skipped for the SVG"
  printf 'not a png' >"$repo/favicon.png"
  settle 5
  header 564-04-junk

  echo "== one over 256 KiB, and no page: the header as today"
  rm "$repo/index.html"
  png "$repo/favicon.png" 400 0,0,0 noise
  ls -l "$repo/favicon.png"
  settle 5
  header 564-05-too-big
}
