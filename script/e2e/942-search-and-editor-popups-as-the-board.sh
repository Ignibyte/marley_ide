# shellcheck shell=bash
# #942's visual check: search and the editor's pop-ups in the board's words. #856's scratch crate in
# a git repository with a second branch; `rows` is an unused variable. The user's HTML extension is
# copied into the run's profile with its version lowered, so the registry's newer one is an update.
# An Ollama stand-in answers `/api/generate` with a fixed completion, and edit predictions use it.
# - `pane: deploy search`: the Replace field under the query, "Replace with" (`942-01-search`,
#   REQ-001).
# - `projects: open recent`: "Open a recent project" (`942-02-recent`, REQ-002).
# - `git: branch`: "Switch to a branch, or name a new one" (`942-03-branches`, REQ-003).
# - `zed: extensions`, "html": its card offers Update (`942-04-extensions`, REQ-004).
# - `rail.wid` and Ctrl+Space: kind words such as "fn" and "fld" (`942-05-completions`, REQ-005).
# - A line typed: the stand-in's grey text and "Tab takes the prediction" after it
#   (`942-06-prediction`, REQ-006).
# - `rows`' warning at its line's end (`942-07-inline-error`, REQ-007).
# - Marley Day: completions over the line with its warning (`942-08-day`, REQ-005, REQ-007).
compositor sway

STOP_AT=${STOP_AT:-0}
AWAY_X=${AWAY_X:-1500}
AWAY_Y=${AWAY_Y:-950}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo html=$E2E_PROFILE/extensions/installed/html
  mkdir -p "$home" "$repo/src"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  cat >"$E2E_WORK/gitconfig" <<'CONFIG'
[user]
	name = Scenario
	email = scenario@example.invalid
CONFIG
  export GIT_CONFIG_GLOBAL=$E2E_WORK/gitconfig GIT_CONFIG_NOSYSTEM=1
  printf '[package]\nname = "rail"\nversion = "0.1.0"\nedition = "2021"\n' >"$repo/Cargo.toml"
  cat >"$repo/src/main.rs" <<'RS'
/// How wide the rail draws.
pub struct Rail {
    width: u32,
    shrunk: bool,
}

impl Rail {
    /// The rail's width, shrunk or not.
    pub fn width_for(&self, shrunk: bool) -> u32 {
        if shrunk { 48 } else { self.width }
    }
}

fn main() {
    let rail = Rail { width: 260, shrunk: false };
    let rows = rail.width_for(rail.shrunk);
    rail.wid
}
RS
  git init -q -b main "$repo"
  git -C "$repo" add -A
  git -C "$repo" commit -q -m "Start"
  git -C "$repo" branch -q feature/one
  # The user's HTML extension, its version lowered so the registry's is newer (REQ-004).
  if [[ -d $HOME/.local/share/marley/extensions/installed/html ]]; then
    mkdir -p "$(dirname "$html")"
    cp -r "$HOME/.local/share/marley/extensions/installed/html" "$html"
    sed -i '0,/^version = /s/^version = .*/version = "0.0.1"/' "$html/extension.toml"
  fi
  write_ollama
  python3 -u "$E2E_WORK/ollama.py" "$E2E_WORK/ollama.port" >"$E2E_WORK/ollama.out" 2>&1 &
  echo "$!" >"$E2E_WORK/ollama.pid"
  local waited=0 port
  until [[ -s $E2E_WORK/ollama.port ]] || ((waited > 50)); do
    sleep 0.1
    waited=$((waited + 1))
  done
  port=$(cat "$E2E_WORK/ollama.port")
  profile_setting edit_predictions "{\"provider\": \"ollama\", \"ollama\": {\"api_url\": \"http://127.0.0.1:$port\", \"model\": \"codellama:7b\"}}"
  # Zed updates an installed extension at start; the lowered one must stay as it is.
  profile_setting auto_update_extensions '{"html": false}'
  profile_setting marley.tmux_hosts '[]'
  open_path "$repo"
}

# An Ollama stand-in: `/api/generate` answers a fixed completion, `/api/tags` names one FIM model.
write_ollama() {
  cat >"$E2E_WORK/ollama.py" <<'PY'
import http.server
import json
import sys


class Handler(http.server.BaseHTTPRequestHandler):
    def answer(self, body):
        data = json.dumps(body).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        self.answer({"models": [{"name": "codellama:7b", "model": "codellama:7b"}]})

    def do_POST(self):
        length = int(self.headers.get("Content-Length", 0))
        self.rfile.read(length)
        if self.path.endswith("/api/generate"):
            self.answer({"model": "codellama:7b", "created_at": "2026-10-28T00:00:00Z", "response": " = rows * 2;", "done": True})
        else:
            self.answer({"models": []})

    def log_message(self, *args):
        print("ollama:", self.path, flush=True)


server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
with open(sys.argv[1], "w") as port:
    port.write(str(server.server_address[1]))
server.serve_forever()
PY
}

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

goto() {
  palette "go to line"
  type_text "$1"
  press "" Return
  settle 2
}

steps() {
  settle 15
  # Trusts the scratch crate.
  press "" Return
  settle 4

  echo "== project search"
  palette "pane: deploy search" 3
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 942-01-search
  press "CTRL" w
  settle 1

  echo "== Open Recent"
  palette "projects: open recent" 3
  shot 942-02-recent
  press "" Escape
  settle 1

  echo "== the branch picker"
  palette "git: branch" 3
  shot 942-03-branches
  press "" Escape
  settle 1

  echo "== the extensions"
  palette "zed: extensions" 6
  type_text "html"
  settle 4
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 942-04-extensions
  press "CTRL" w
  settle 1
  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== the crate"
  press CTRL p
  settle 2
  type_text "main.rs"
  settle 2
  press "" Return
  # rust-analyzer reads the crate and std.
  settle 40
  goto 16:5
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
  shot 942-07-inline-error

  echo "== completions"
  goto 17:13
  press CTRL space
  settle 4
  shot 942-05-completions
  press "" Escape
  settle 1

  echo "== a prediction"
  goto 16:44
  press "" Return
  type_text "    let total"
  settle 6
  shot 942-06-prediction
  press "" Escape
  settle 1

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  goto 17:13
  press CTRL space
  settle 4
  shot 942-08-day
  press "" Escape
  cat "$E2E_WORK/ollama.out"
  kill "$(cat "$E2E_WORK/ollama.pid")" 2>/dev/null || true
}
