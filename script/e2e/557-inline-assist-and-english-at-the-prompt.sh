# shellcheck shell=bash
# #557's visual check: Zed's Inline Assist in a center terminal of the Marley layout, and English
# at the prompt. A stand-in Ollama on this machine answers Inline Assist with one command:
# Ctrl+Enter opens the prompt (REQ-001), a request puts the command on the prompt line (REQ-002),
# and the prompt's run button runs it (REQ-003): on Linux Zed's prompt editor keeps Ctrl+Enter. A line typed at the prompt that reads as English shows the
# hint (REQ-004), which → does not type (REQ-011); a command shows none and Ctrl+Shift+Enter goes
# on to the shell (REQ-005). Ctrl+Shift+Enter on English starts a stand-in Claude Code with the
# line (REQ-006), and with it running, pastes the next line into it (REQ-007). A block that ended
# with 127 on English offers Ask the agent (REQ-008), whose click asks it (REQ-009). With
# `marley.english_hint` off, neither the hint nor the chip shows (REQ-010).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

TERMINAL_X=${TERMINAL_X:-800}
TERMINAL_Y=${TERMINAL_Y:-500}
# Inline Assist's run button (▶) and the exit-127 block's Ask the agent chip, from the shots.
RUN_X=${RUN_X:-1312}
RUN_Y=${RUN_Y:-109}
CHIP_X=${CHIP_X:-1252}
CHIP_Y=${CHIP_Y:-916}

# Sets the settings key path `$1` (dot-separated) to the JSON value `$2` in the run's copy of the
# settings.
set_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" "$2" <<'SETTINGS'
import json, pathlib, re, sys

path = pathlib.Path(sys.argv[1])
text = path.read_text() if path.exists() else "{}"
# Comments and trailing commas out: the settings file allows them and JSON does not.
text = re.sub(r'("(?:[^"\\]|\\.)*")|//[^\n]*|/\*.*?\*/', lambda m: m.group(1) or "", text, flags=re.S)
text = re.sub(r",(\s*[}\]])", r"\1", text)
settings = json.loads(text) if text.strip() else {}
*parents, last = sys.argv[2].split(".")
node = settings
for key in parents:
    node = node.setdefault(key, {})
node[last] = json.loads(sys.argv[3])
path.write_text(json.dumps(settings, indent=2) + "\n")
SETTINGS
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  # A stand-in Claude Code: it logs its arguments, then each line it reads.
  cat >"$bin/claude" <<SH
#!/usr/bin/env bash
exec -a claude python3 -u -c '
import sys
log = open(sys.argv[1], "a")
print("argv:", " ".join(sys.argv[2:]), file=log, flush=True)
print("stand-in agent, argv:", " ".join(sys.argv[2:]), flush=True)
for line in sys.stdin:
    print("got:", line.rstrip("\n"), file=log, flush=True)
    print("got:", line.rstrip("\n"), flush=True)
' "$E2E_WORK/agent.log" "\$@"
SH
  chmod +x "$bin/claude"
  : >"$E2E_WORK/agent.log"
  write_ollama
  python3 -u "$E2E_WORK/ollama.py" "$E2E_WORK/ollama.port" >"$E2E_WORK/ollama.out" 2>&1 &
  echo "$!" >"$E2E_WORK/ollama.pid"
  local waited=0
  until [[ -s $E2E_WORK/ollama.port ]] || ((waited > 50)); do
    sleep 0.1
    waited=$((waited + 1))
  done
  local port
  port=$(cat "$E2E_WORK/ollama.port")
  set_setting language_models.ollama "{\"api_url\": \"http://127.0.0.1:$port\", \"available_models\": [{\"name\": \"fake\", \"display_name\": \"Fake\", \"max_tokens\": 4096}]}"
  set_setting agent.enabled true
  set_setting agent.inline_assistant_model '{"provider": "ollama", "model": "fake"}'
  set_setting agent.default_model '{"provider": "ollama", "model": "fake"}'
  git init -q -b main "$repo"
  open_path "$repo"
}

teardown() {
  [[ -f $E2E_WORK/ollama.pid ]] && kill "$(cat "$E2E_WORK/ollama.pid")" 2>/dev/null
  return 0
}

# A stand-in Ollama on this machine: one model, and one command for every chat.
write_ollama() {
  cat >"$E2E_WORK/ollama.py" <<'PY'
import json
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

DETAILS = {"format": "gguf", "family": "llama", "families": ["llama"],
           "parameter_size": "1B", "quantization_level": "Q4_0"}


class Handler(BaseHTTPRequestHandler):
    def reply(self, body, content_type="application/json"):
        data = body.encode()
        self.send_response(200)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        if self.path.startswith("/api/tags"):
            self.reply(json.dumps({"models": [{"name": "fake", "modified_at": "2026-09-29T00:00:00Z",
                                               "size": 1, "digest": "fake", "details": DETAILS}]}))
        else:
            self.reply(json.dumps({"version": "0.9.0"}))

    def do_POST(self):
        length = int(self.headers.get("Content-Length") or 0)
        self.rfile.read(length)
        if self.path.startswith("/api/show"):
            self.reply(json.dumps({"capabilities": ["completion"],
                                   "model_info": {"general.architecture": "llama",
                                                  "llama.context_length": 4096}}))
        elif self.path.startswith("/api/chat"):
            chunks = [
                {"model": "fake", "created_at": "2026-09-29T00:00:00Z",
                 "message": {"role": "assistant", "content": "echo marley-inline-assist"}, "done": False},
                {"model": "fake", "created_at": "2026-09-29T00:00:00Z",
                 "message": {"role": "assistant", "content": ""}, "done": True, "done_reason": "stop"},
            ]
            self.reply("".join(json.dumps(chunk) + "\n" for chunk in chunks), "application/x-ndjson")
        else:
            self.reply("{}")

    def log_message(self, *_):
        pass


server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
open(sys.argv[1], "w").write(str(server.server_address[1]))
server.serve_forever()
PY
}

# The last row of the terminal titled with `$1` whose text is a prompt, to `$2`.
prompt_line() {
  mcp_agent terminal-screen "$1" | grep -E '^  \| \$' | tail -1 >"$2"
  cat "$2"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1

  echo "== Inline Assist in a center terminal"
  press "CTRL" Return
  settle 3
  shot 557-01-inline-prompt
  type_text "print a marker"
  press "" Return
  settle 4
  shot 557-02-generated
  click "$RUN_X" "$RUN_Y"
  settle 3
  shot 557-03-ran

  echo "== English typed at the prompt: the hint"
  click "$TERMINAL_X" "$TERMINAL_Y"
  settle 1
  type_text "what is using port 3000"
  settle 2
  shot 557-04-hint
  press "" Right
  settle 1
  prompt_line repo "$E2E_WORK/after-right.txt"
  expect "→ typed nothing of the hint" grep -qE '\$ what is using port 3000 *$' "$E2E_WORK/after-right.txt"
  press "CTRL" u
  settle 1

  echo "== a command: no hint, and the key goes to the shell"
  type_text "ls -la"
  settle 2
  shot 557-05-no-hint
  press "CTRL SHIFT" Return
  settle 2
  expect "no agent was asked" test ! -s "$E2E_WORK/agent.log"
  press "CTRL" u
  settle 1

  echo "== Ctrl+Shift+Enter with no agent: Claude Code starts with the line"
  type_text "what is using port 3000"
  settle 1
  press "CTRL SHIFT" Return
  settle 5
  shot 557-06-asked-new
  cat "$E2E_WORK/agent.log"
  # Marley's flags for the agent come first; the line is its last argument.
  expect "the agent started with the line" grep -qE '^argv: .*what is using port 3000$' "$E2E_WORK/agent.log"

  echo "== with the agent running, the next line goes to it"
  press ALT 1
  settle 1
  type_text "find all the large files in this repo"
  settle 1
  press "CTRL SHIFT" Return
  settle 3
  shot 557-07-asked-running
  cat "$E2E_WORK/agent.log"
  expect "the running agent got it" grep -qF "got: find all the large files in this repo" "$E2E_WORK/agent.log"

  echo "== exit 127 on English offers Ask the agent"
  press ALT 1
  settle 1
  type_text "show me the biggest folders"
  press "" Return
  settle 2
  shot 557-08-exit-127
  click "$CHIP_X" "$CHIP_Y"
  settle 3
  shot 557-09-button
  cat "$E2E_WORK/agent.log"
  expect "the chip asked with the command" grep -qF "got: show me the biggest folders" "$E2E_WORK/agent.log"

  echo "== the setting off: no hint, no chip"
  # The stand-in ends, so #555's own chip, which needs an agent elsewhere, stays away too.
  press ALT 2
  settle 1
  press "CTRL" d
  settle 2
  set_setting marley.english_hint false
  settle 3
  press ALT 1
  settle 1
  type_text "show me the biggest folders"
  press "" Return
  settle 2
  type_text "what is using port 3000"
  settle 2
  shot 557-10-off
  press "CTRL SHIFT" Return
  settle 5
  cat "$E2E_WORK/agent.log"
  expect "the key still asks with the setting off" test "$(grep -c '^argv:' "$E2E_WORK/agent.log")" = 2
}
