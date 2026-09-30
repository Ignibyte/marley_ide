# shellcheck shell=bash
# #596's visual check: ssh in an agent's terminal asks for a key's passphrase in a Marley dialog.
# A user-level sshd on a loopback port serves a bare repository to a passphrase-protected key.
# Stand-ins `claude` and `codex`, first on the terminals' PATH, print the terminal's askpass
# variables, show that git's own credential prompt still fails at once, push over ssh with that
# key, print the exit status and wait. Claude Code from the project's +: the dialog opens over its
# terminal, the passphrase typed there lets the push through (REQ-001 to REQ-004, REQ-006).
# Codex from the New Agent picker: Escape fails the push at once (REQ-005). The same push in a New
# Terminal waits on ssh's own prompt (REQ-007).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# The project's +; Down steps from New Terminal in its menu to Claude Code. From #537's shots.
PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-96}
CLAUDE_STEPS=${CLAUDE_STEPS:-3}

free_port() {
  python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])'
}
PORT=$(free_port)

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo ssh=$E2E_WORK/ssh
  mkdir -p "$home" "$bin" "$repo" "$ssh"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"

  # The key's passphrase, made at run time so no file of the repository holds one.
  printf 'e2e-%s-%s' "$RANDOM" "$RANDOM" >"$E2E_WORK/passphrase"
  ssh-keygen -q -t ed25519 -N '' -C e2e-host -f "$ssh/host_key"
  ssh-keygen -q -t ed25519 -N "$(cat "$E2E_WORK/passphrase")" -C e2e-agent -f "$ssh/agent_key"
  cp "$ssh/agent_key.pub" "$ssh/authorized_keys"
  chmod 600 "$ssh/authorized_keys"
  cat >"$ssh/sshd_config" <<CONFIG
Port $PORT
ListenAddress 127.0.0.1
HostKey $ssh/host_key
AuthorizedKeysFile $ssh/authorized_keys
PidFile $ssh/sshd.pid
StrictModes no
UsePAM no
PasswordAuthentication no
KbdInteractiveAuthentication no
PubkeyAuthentication yes
CONFIG
  printf '[127.0.0.1]:%s %s\n' "$PORT" "$(cut -d' ' -f1,2 "$ssh/host_key.pub")" >"$ssh/known_hosts"
  timeout 900 /usr/bin/sshd -D -f "$ssh/sshd_config" -E "$E2E_WORK/sshd.log" &
  SSHD_PID=$!
  git init -q --bare "$ssh/remote.git"

  # The push every terminal here makes: this key only, no agent, no user ssh config, the
  # fixture's host key.
  cat >"$bin/push-over-ssh" <<PUSH
#!/usr/bin/env bash
export GIT_SSH_COMMAND="ssh -F /dev/null -i $ssh/agent_key -o IdentitiesOnly=yes -o IdentityAgent=none -o UserKnownHostsFile=$ssh/known_hosts -o StrictHostKeyChecking=yes"
git push "ssh://\$(id -un)@127.0.0.1:$PORT$ssh/remote.git" "HEAD:\$1"
echo "push exited \$?"
PUSH
  cat >"$bin/claude" <<'STAND_IN'
#!/usr/bin/env bash
# A stand-in agent: what ssh and git do in the terminal Marley opened for it.
name=$(basename "$0")
echo "SSH_ASKPASS=${SSH_ASKPASS:+${SSH_ASKPASS##*/}} SSH_ASKPASS_REQUIRE=${SSH_ASKPASS_REQUIRE-unset}"
echo "GIT_ASKPASS=[${GIT_ASKPASS-unset}] GIT_TERMINAL_PROMPT=${GIT_TERMINAL_PROMPT-unset}"
printf 'protocol=https\nhost=example.invalid\n\n' | git -c credential.helper= credential fill
push-over-ssh "$name"
exec -a "$name" sleep 600
STAND_IN
  cp "$bin/claude" "$bin/codex"
  chmod +x "$bin/claude" "$bin/codex" "$bin/push-over-ssh"
  export MARLEY_CLAUDE=$bin/claude

  git init -q -b main "$repo"
  git -C "$repo" -c user.name=Scenario -c user.email=scenario@example.invalid commit -q --allow-empty -m Start
  open_path "$repo"
}

teardown() {
  if [[ -n ${SSHD_PID:-} ]]; then
    kill "$SSHD_PID" 2>/dev/null || true
  fi
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# The newest terminal's screen of the project.
screen_of() {
  mcp_agent terminal-screen repo | tee "$E2E_WORK/$1.txt"
}

# Whether the passphrase is absent from `file`.
lacks_passphrase() {
  ! grep -qF -- "$(cat "$E2E_WORK/passphrase")" "$1"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== the terminal's claude and codex are the stand-ins"
  type_text "command -v claude codex > $E2E_WORK/which.txt"
  press "" Return
  settle 2
  cat "$E2E_WORK/which.txt"
  if ! grep -qx "$E2E_WORK/bin/claude" "$E2E_WORK/which.txt" ||
    ! grep -qx "$E2E_WORK/bin/codex" "$E2E_WORK/which.txt"; then
    echo "check the stand-ins come first on the terminal's PATH: FAIL (stopping before any launch)"
    return 1
  fi

  echo "== Claude Code from the project's +: ssh asks in Marley's dialog"
  click "$PLUS_X" "$PLUS_Y"
  settle 1
  local step
  for ((step = 0; step < CLAUDE_STEPS; step++)); do
    press "" Down
  done
  settle 1
  shot 596-00-menu
  press "" Return
  settle 8
  shot 596-01-dialog
  screen_of asking
  expect "the agent's terminal holds the askpass variables" \
    holds "$E2E_WORK/asking.txt" '$ claude' 'SSH_ASKPASS=askpass.sh SSH_ASKPASS_REQUIRE=force' \
    'GIT_ASKPASS=[] GIT_TERMINAL_PROMPT=0'
  expect "git's own prompt still fails at once" \
    holds "$E2E_WORK/asking.txt" "could not read Username for 'https://example.invalid': terminal prompts disabled"
  expect "ssh's prompt is not on the terminal" bash -c "! grep -q 'Enter passphrase' '$E2E_WORK/asking.txt'"

  type_text "$(cat "$E2E_WORK/passphrase")"
  settle 1
  shot 596-02-typed
  press "" Return
  settle 5
  shot 596-03-pushed
  screen_of pushed
  expect "the push went through" holds "$E2E_WORK/pushed.txt" 'HEAD -> claude' 'push exited 0'
  expect "the server took the key" holds "$E2E_WORK/sshd.log" 'Accepted publickey'
  expect "the terminal never shows the passphrase" lacks_passphrase "$E2E_WORK/pushed.txt"
  expect "Marley's log never holds the passphrase" lacks_passphrase "$E2E_LOG"

  echo "== Codex from the New Agent picker: Escape fails the push at once"
  press "CTRL ALT" n
  settle 2
  type_text "Codex"
  settle 1
  press "" Return
  settle 8
  shot 596-04-codex-dialog
  press "" Escape
  settle 4
  shot 596-05-cancelled
  screen_of cancelled
  # The screen's read starts at git's first line; the shot shows the typed `$ codex`.
  expect "the push failed at once" \
    holds "$E2E_WORK/cancelled.txt" 'Permission denied' 'push exited 128'

  echo "== a New Terminal: ssh asks on the terminal"
  palette "workspace: new terminal"
  settle 3
  type_text "push-over-ssh plain"
  press "" Return
  settle 3
  shot 596-06-plain-terminal
  screen_of plain
  expect "ssh asks on the terminal there" holds "$E2E_WORK/plain.txt" 'Enter passphrase for key'
  press CTRL c
  settle 1

  echo "== sshd's log"
  cat "$E2E_WORK/sshd.log"
}
