# shellcheck shell=bash
# #668's check: a test run acts on nothing of the user's. The harness copies the user's settings
# into the run's profile; this scenario stands in for a user whose settings reach outside the run
# (a phone push topic, an agent harness, fleet hosts, a System One account) and whose environment
# holds both System One keys, all made up. Its checks read the run's copy and its own environment,
# which Marley inherits: none of those may reach the run, and a harmless setting must.
#
# One shot, `668-01-started`: Marley started on that profile.

# The harness's own `config`, the folder it copies `settings.json` from, read after this file is
# sourced: here a scratch one under the shots, never the user's.
config=${SHOT_DIR:-${TMPDIR:-/tmp}/marley-shots}/668-config/marley
mkdir -p "$config"
cat >"$config/settings.json" <<'JSON'
{
  // A comment and a trailing comma, as a user's file may hold.
  "theme": "One Dark",
  "marley": {
    "push": { "topic": "made-up-topic", "token_file": "/nonexistent/push-token" },
    "harness": { "command": "/nonexistent/harness", "args": [] },
    "embedded_harness": true,
    "fleet": { "providers": [], "hosts": [] },
    "system_one": { "enabled": true },
  },
}
JSON
export MARLEY_SYSTEM_ONE_KEY=made-up-key MARLEY_CLOUDFLARE_API_TOKEN=made-up-token

# Whether the run's copy of the settings holds nothing at the dotted key path `$1`.
setting_absent() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" <<'SETTINGS'
import json, pathlib, sys

node = json.loads(pathlib.Path(sys.argv[1]).read_text())
for key in sys.argv[2].split("."):
    node = node.get(key) if isinstance(node, dict) else None
sys.exit(0 if node is None else 1)
SETTINGS
}

# Whether the run's copy holds the JSON value `$2` at the dotted key path `$1`.
setting_is() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" "$2" <<'SETTINGS'
import json, pathlib, sys

node = json.loads(pathlib.Path(sys.argv[1]).read_text())
for key in sys.argv[2].split("."):
    node = node.get(key) if isinstance(node, dict) else None
sys.exit(0 if node == json.loads(sys.argv[3]) else 1)
SETTINGS
}

setup() {
  local key
  mkdir -p "$E2E_WORK/repo"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  for key in push harness embedded_harness fleet system_one; do
    expect "the run's copy leaves out marley.$key" setting_absent "marley.$key"
  done
  expect "the run's copy keeps the rest" setting_is theme '"One Dark"'
  expect "the run holds no System One key" test -z "${MARLEY_SYSTEM_ONE_KEY+set}"
  expect "the run holds no Cloudflare token" test -z "${MARLEY_CLOUDFLARE_API_TOKEN+set}"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  shot 668-01-started
}
