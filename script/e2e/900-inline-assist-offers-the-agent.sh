# shellcheck shell=bash
# #900's visual check: inline assist offers the window's terminal agent the selection. A stand-in
# Ollama (#557's) makes Zed's default model ready, so Ctrl+Enter opens inline assist's prompt; a
# stand-in `claude` (#557's) runs in `repo`'s terminal and logs what it reads.
# - `900-01-line`: lines 20 to 22 of `src/main.rs` selected, inline assist open: under the prompt,
#   "Or send lines 20–22 to Claude Code in …" and the action's name (REQ-001).
# - `900-03-day`: the same in Marley Day (REQ-001).
# - `900-02-sent`: the line clicked: the stand-in's terminal holds the reference (REQ-002).
compositor sway

AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-700}
# From the probe run's shots: the line under the prompt, and the terminal's tab.
LINE_X=${LINE_X:-500}
LINE_Y=${LINE_Y:-580}
TERMINAL_TAB_X=${TERMINAL_TAB_X:-480}
TERMINAL_TAB_Y=${TERMINAL_TAB_Y:-51}
# A first run that stops after the first shot, to read the positions.
STOP_AT=${STOP_AT:-}

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
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo line
  mkdir -p "$home" "$bin" "$repo/src"
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
print("stand-in agent, argv:", " ".join(sys.argv[2:]), flush=True)
for line in sys.stdin:
    print("got:", line.rstrip("\\n"), file=log, flush=True)
    print("got:", line.rstrip("\\n"), flush=True)
' "$E2E_WORK/agent.log" "\$@"
SH
  chmod +x "$bin/claude"
  : >"$E2E_WORK/agent.log"
  for line in $(seq 1 30); do
    printf 'fn step_%02d() -> u32 {\n    %d\n}\n' "$line" "$line"
  done >"$repo/src/main.rs"
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

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

# Opens `src/main.rs`, selects its lines 20 to 22 and opens inline assist.
assist() {
  press "CTRL" p
  settle 1
  type_text "main.rs"
  settle 2
  press "" Return
  settle 3
  press "CTRL" g
  settle 1
  type_text "20"
  press "" Return
  settle 1
  press "" Home
  press "SHIFT" Down
  press "SHIFT" Down
  press "SHIFT" Down
  settle 1
  press "CTRL" Return
  settle 3
}

steps() {
  settle 12
  # Trusts repo.
  press "" Return
  settle 3

  echo "== the stand-in claude in repo's terminal"
  type_text "claude"
  press "" Return
  settle 4

  echo "== inline assist on lines 20 to 22"
  assist
  away
  shot 900-01-line
  [[ $STOP_AT == 1 ]] && return 0

  echo "== Marley Day"
  set_setting theme '"Marley Day"'
  settle 4
  away
  shot 900-03-day

  echo "== the line clicked"
  click "$LINE_X" "$LINE_Y"
  settle 3
  click "$TERMINAL_TAB_X" "$TERMINAL_TAB_Y"
  settle 2
  away
  shot 900-02-sent
}
