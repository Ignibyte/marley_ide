# shellcheck shell=bash
# #687's visual check, Zed's agent: Marley's own agent for someone with neither Claude Code nor
# Codex signed in, and a Zed model set up. Fakes say both are signed out; `OPENAI_API_KEY` set to a
# dummy and the copy's `agent.default_model` naming OpenAI make Zed's default model ready, and
# nothing is ever sent to it. The offer names Zed's agent (`zed-offer`, REQ-006); Turn On writes
# the switch with the agent, and the profile selector of a Zed Agent thread lists Marley, whose
# settings hold no built-in tool and only Marley's seven (`profile`, REQ-007). Picked, it becomes
# the user's default profile; the switch off, the profile goes and the default is `write` again
# (REQ-008).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The offer's Turn On, as #683's run found it.
TURN_ON_X=${TURN_ON_X:-958}
TURN_ON_Y=${TURN_ON_Y:-901}
# The project's +, as #683's run found it.
PLUS_X=${PLUS_X:-224}
PLUS_Y=${PLUS_Y:-123}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  # A `claude` and a `codex` both signed out.
  cat >"$bin/claude" <<'SH'
#!/bin/sh
case "$1 $2" in
  "auth status") printf '{"loggedIn": false}\n' ;;
  *) printf '2.1.293 (Claude Code)\n' ;;
esac
SH
  cat >"$bin/codex" <<'SH'
#!/bin/sh
case "$1 $2" in
  "login status") printf 'Not logged in\n' >&2 ; exit 1 ;;
  *) printf 'codex-cli 0.160.0\n' ;;
esac
SH
  chmod +x "$bin/claude" "$bin/codex"
  export MARLEY_CLAUDE=$bin/claude MARLEY_CODEX=$bin/codex
  export OPENAI_API_KEY=e2e-not-a-key MCP_CLIENT_NAME="Stand-in agent"
  profile_setting agent.default_model '{"provider": "openai", "model": "gpt-4o-mini"}'
  profile_setting agent.default_profile '"write"'
  # The run starts undecided, so the offer comes.
  profile_setting marley.assistant '{}'
  open_path "$E2E_WORK/repo"
}

# Whether the copy's `marley.assistant` holds `enabled: $1` and `agent: $2`.
assistant_is() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" "$2" <<'PY'
import json, re, sys
text = open(sys.argv[1]).read()
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
assistant = json.loads(text).get("marley", {}).get("assistant", {})
print("  marley.assistant:", assistant)
sys.exit(0 if json.dumps(assistant.get("enabled")) == sys.argv[2]
         and json.dumps(assistant.get("agent")) == sys.argv[3] else 1)
PY
}

# Whether the copy's `agent.default_profile` is `$1`.
default_profile_is() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" <<'PY'
import json, re, sys
text = open(sys.argv[1]).read()
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
profile = json.loads(text).get("agent", {}).get("default_profile")
print("  agent.default_profile:", profile)
sys.exit(0 if profile == sys.argv[2] else 1)
PY
}

# Whether `settings_read` finds the Marley profile with no built-in tool and Marley's seven.
profile_is_right() {
  python3 - "$E2E_WORK/profile.txt" <<'PY'
import json, sys
text = open(sys.argv[1]).read()
start = text.find("{")
answer = json.JSONDecoder().raw_decode(text[start:])[0] if start >= 0 else {}
profile = answer.get("global") or answer.get("effective") or {}
servers = profile.get("context_servers") or {}
marley_tools = (servers.get("marley") or {}).get("tools") or {}
print("  name:", profile.get("name"), "tools:", profile.get("tools"),
      "enable_all_context_servers:", profile.get("enable_all_context_servers"))
print("  context servers:", sorted(servers), "marley's:", sorted(marley_tools))
expected = {"docs_search", "docs_read", "settings_schema", "settings_read", "settings_change",
            "keymap_change", "actions_list"}
sys.exit(0 if profile.get("name") == "Marley" and not profile.get("tools")
         and profile.get("enable_all_context_servers") is False
         and set(servers) == {"marley"}
         and {name for name, on in marley_tools.items() if on} == expected else 1)
PY
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== the offer names Zed's agent"
  shot 687-05-zed-offer

  echo "== Turn On"
  click "$TURN_ON_X" "$TURN_ON_Y"
  settle 3
  expect "the switch is written on with Zed's agent" assistant_is true '"zed"'
  mcp_agent tool settings_read '{"key": "agent.profiles.marley"}' >"$E2E_WORK/profile.txt" 2>&1
  cat "$E2E_WORK/profile.txt"
  expect "the Marley profile has no built-in tool and Marley's seven" profile_is_right

  echo "== the profile selector lists Marley"
  # A Zed Agent thread from the project's +: New Agent Thread's submenu lists Zed Agent first,
  # where `agent: new thread` would open the panel's last agent.
  click "$PLUS_X" "$PLUS_Y"
  settle 1
  press "" Down
  press "" Down
  press "" Right
  settle 1
  press "" Return
  settle 4
  press "CTRL" i
  settle 2
  shot 687-06-profile

  echo "== picked, then the switch off"
  type_text "Marley"
  settle 1
  press "" Return
  settle 3
  expect "the picked profile is the user's default" default_profile_is marley
  profile_setting marley.assistant.enabled false
  settle 4
  mcp_agent tool settings_read '{"key": "agent.profiles.marley"}' >"$E2E_WORK/profile-off.txt" 2>&1
  cat "$E2E_WORK/profile-off.txt"
  expect "the profile is gone" holds "$E2E_WORK/profile-off.txt" "no_setting"
  expect "the default profile is write again" default_profile_is write
}
