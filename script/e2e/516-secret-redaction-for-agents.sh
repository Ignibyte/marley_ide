# shellcheck shell=bash
# #516's e2e test: secrets hidden from what agents read. The terminal runs a script that prints
# one fake secret of each built-in kind, a line only a user pattern matches and a plain line, with
# a secret-named variable set on its command line. The fakes are put together when the run
# starts, from pieces, so this file holds none for gitleaks to find. A stand-in agent reads the
# block through Marley's MCP server, as Claude Code does:
# - redaction on, the default: each fake comes back as `[redacted: <kind>]`, and the read counts
#   them (REQ-001);
# - `marley.redact_secrets_for_agents` false: the same read comes back as printed (REQ-002);
# - redaction on again, with a pattern in `marley.redaction_patterns` and one that is not a
#   regular expression: the pattern's line comes back as `[redacted: pattern]` (REQ-003), and a
#   notification names the bad one;
# - a page's console, read by the agent, has its two fakes hidden;
# - the Marley page shows the toggle on (REQ-004); clicked, it turns redaction off, and the
#   console comes back as printed.
# The terminal is read before the Settings window opens: sway tiles it beside the main window,
# and the narrower terminal rewraps its lines, which moves every block's rows (the plan's D2,
# TICKET-544).
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# "Redact Secrets for Agents" on the Marley page, measured from 516-01-setting-on.
TOGGLE_X=${TOGGLE_X:-1548}
TOGGLE_Y=${TOGGLE_Y:-377}

KINDS=("private key" "secret" "bearer token" "url password" "aws key id" "github token"
  "slack token" "stripe key" "google api key" "api key" "jwt")
SITE=

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home"
  cat >"$home/.bashrc" <<'RC'
PS1='$ '
RC
  terminal_env HOME "$home"
  git init -q -b redact "$E2E_WORK/repo"
  mkdir -p "$E2E_WORK/site"
  write_fakes
  offline_chromium
  SITE=http://127.0.0.1:$(serve_site site)
  open_path "$E2E_WORK/repo"
}

teardown() {
  browser_teardown
}

# The script the terminal runs, the page, and the lists the checks use: `fakes.txt`, a line of
# each built-in kind's fake as printed; `pattern.txt`, the value only the user's pattern hides;
# `deploy.txt`, the value set on the command line.
write_fakes() {
  python3 - "$E2E_WORK" <<'PY'
import pathlib, sys

work = pathlib.Path(sys.argv[1])
begin = "-----BEGIN OPENSSH " + "PRIVATE" + " KEY-----"
end = "-----END OPENSSH " + "PRIVATE" + " KEY-----"
key_body = "b3Blbn" + "Fake" * 14
password = "correct-horse-" + "battery-staple"
bearer = "Fake" * 6
url_password = "Fake" + "Pass" * 3
aws = "AK" + "IA" + "FAKE" * 4
github = "gh" + "p_" + "Fake" * 9
slack = "xo" + "xb-" + "0000000000-" + "fake" * 4
stripe = "sk" + "_live_" + "Fake" * 6
google = "AI" + "za" + "Fake" * 8 + "Fak"
anthropic = "sk" + "-ant-" + "fake" * 8
jwt = "ey" + "JhbGciOiJIUzI1NiJ9." + "ey" + "JzdWIiOiJtYXJsZXkifQ." + "fake" * 4
lines = [
    begin,
    key_body,
    "Fake" * 10 + "AAAA",
    end,
    "export DATABASE_PASS" + 'WORD="' + password + '"',
    'curl -H "Authorization: Bear' + "er " + bearer + '" https://api.example.test/v1',
    "psql postgres://marley:" + url_password + "@db.example.test:5432/app",
    "aws: " + aws,
    "gh: " + github,
    "slack: " + slack,
    "stripe: " + stripe,
    "google: " + google,
    "anthropic: " + anthropic,
    "jwt: " + jwt,
    "build id zq-7781-internal",
    "a plain line stays: 5 files built",
]
script = "cat <<'EOF'\n" + "\n".join(lines) + "\nEOF\n"
(work / "repo" / "secrets.sh").write_text(script)
fakes = [key_body, password, bearer, url_password, aws, github, slack, stripe, google, anthropic, jwt]
(work / "fakes.txt").write_text("\n".join(fakes) + "\n")
(work / "pattern.txt").write_text("zq-7781-internal\n")
(work / "deploy.txt").write_text("deploy" + "Fake" * 3)
page = (
    "<!doctype html><title>Console fakes</title><p>Two fakes in the console.</p>\n<script>\n"
    'console.log("deploying with " + ["gh", "p_", "Fake".repeat(9)].join(""));\n'
    'console.warn("db at postgres://marley:" + "Fake" + "Pass".repeat(3) + "@db.example.test/app");\n'
    "</script>\n"
)
(work / "site" / "index.html").write_text(page)
PY
}

# Sets `marley.<key>` to the JSON value in the profile copy's settings: the value replaced when
# the key is there (a scalar, as the Settings window writes one), else the key added first in
# the `marley` block, else a `marley` block added.
marley_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" "$2" <<'PY'
import pathlib, re, sys

path, key, value = pathlib.Path(sys.argv[1]), sys.argv[2], sys.argv[3]
text = path.read_text()
existing = re.search(r'"%s"\s*:\s*(true|false|"[^"\n]*"|-?[0-9.]+)' % re.escape(key), text)
if existing:
    text = text[: existing.start(1)] + value + text[existing.end(1) :]
else:
    block = re.search(r'"marley"\s*:\s*\{', text)
    if block:
        text = text[: block.end()] + f'\n    "{key}": {value},' + text[block.end() :]
    else:
        brace = text.index("{")
        text = text[: brace + 1] + f'\n  "marley": {{ "{key}": {value} }},' + text[brace + 1 :]
path.write_text(text)
PY
}

# Reads the script's block as the stand-in agent and checks it against what `$1` names: `on`,
# each built-in fake hidden and named, the pattern's value as printed; `off`, every fake as
# printed; `pattern`, the pattern's value hidden too.
read_block() {
  local expect=$1 out=$E2E_WORK/read-$1.txt kind failed=0 counts
  if ! mcp_agent terminal-read secrets.sh >"$out"; then
    cat "$out"
    echo "the read failed" >&2
    return 1
  fi
  cat "$out"
  counts=$(grep -m1 '^  redacted:' "$out" || true)
  case $expect in
    on | pattern)
      for kind in "${KINDS[@]}"; do
        grep -qF "[redacted: $kind]" "$out" || {
          echo "no [redacted: $kind]" >&2
          failed=1
        }
      done
      if grep -qFf "$E2E_WORK/fakes.txt" "$out"; then
        echo "a fake came back as printed" >&2
        failed=1
      fi
      ;;
    off)
      if grep -qF "[redacted" "$out"; then
        echo "redaction off still hid something" >&2
        failed=1
      fi
      while read -r fake; do
        grep -qF "$fake" "$out" || {
          echo "a fake did not come back as printed" >&2
          failed=1
        }
      done <"$E2E_WORK/fakes.txt"
      ;;
  esac
  case $expect in
    on)
      grep -qFf "$E2E_WORK/pattern.txt" "$out" || {
        echo "a built-in rule took the pattern's line" >&2
        failed=1
      }
      [[ $counts == "  redacted: 12 in the read, 1 in the list" ]] || failed=1
      ;;
    off) [[ $counts == "  redacted: 0 in the read, 0 in the list" ]] || failed=1 ;;
    pattern)
      if ! grep -qF "[redacted: pattern]" "$out" || grep -qFf "$E2E_WORK/pattern.txt" "$out"; then
        echo "the user's pattern did not hide its line" >&2
        failed=1
      fi
      [[ $counts == "  redacted: 13 in the read, 1 in the list" ]] || failed=1
      ;;
  esac
  if [[ $failed -eq 0 ]]; then
    echo "check $expect: pass ($counts)"
  else
    echo "check $expect: FAIL ($counts)"
  fi
  return "$failed"
}

# Reads the page's console as the stand-in agent: `on`, both fakes hidden; `off`, both as printed.
read_console() {
  local out=$E2E_WORK/console-$1.txt hidden=0
  mcp_agent console >"$out"
  cat "$out"
  grep -qF "[redacted: github token]" "$out" && hidden=$((hidden + 1))
  grep -qF "[redacted: url password]" "$out" && hidden=$((hidden + 1))
  if [[ $1 == on && $hidden -eq 2 ]] && ! grep -qFf "$E2E_WORK/fakes.txt" "$out"; then
    echo "check console $1: pass"
  elif [[ $1 == off && $hidden -eq 0 ]] && grep -qF "$(sed -n 6p "$E2E_WORK/fakes.txt")" "$out"; then
    echo "check console $1: pass"
  else
    echo "check console $1: FAIL"
    return 1
  fi
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  echo "== the script, with a secret-named variable on its command line"
  type_text "DEPLOY_TOKEN=$(cat "$E2E_WORK/deploy.txt") sh secrets.sh"
  press "" Return
  settle 3
  shot 516-00-printed
  echo "== redaction on, the default"
  read_block on
  echo "== marley.redact_secrets_for_agents false"
  marley_setting redact_secrets_for_agents false
  settle 3
  read_block off
  echo "== redaction on again, with a pattern of the user's and one that is not a regex"
  marley_setting redact_secrets_for_agents true
  marley_setting redaction_patterns '["zq-[0-9]{4}-internal", "("]'
  settle 3
  shot 516-02-bad-pattern
  read_block pattern
  echo "== the console"
  mcp_agent navigate "$SITE/index.html"
  settle 3
  read_console on
  echo "== the Marley page"
  press "CTRL SHIFT" p
  settle 1
  type_text "marley: open settings"
  settle 1
  press "" Return
  settle 4
  shot 516-01-setting-on
  echo "== the toggle clicked off"
  click "$TOGGLE_X" "$TOGGLE_Y"
  settle 3
  shot 516-03-setting-off
  grep -n '"redact_secrets_for_agents"' "$E2E_PROFILE/config/settings.json"
  read_console off
}
