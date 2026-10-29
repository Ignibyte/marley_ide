# shellcheck shell=bash
# #795's visual check: the first command in a new zsh is a block under its header, as on the Mac,
# where zsh starts as a login shell and the user's `.zshrc` loads iTerm2's shell integration. A
# scratch repository; a HOME whose `.zshrc` sets a plain prompt and sources a stand-in for iTerm2's
# integration that prints its marks and adds its hooks as the real one does.
# - The first zsh terminal, `zsh -l`: `echo one`, then `echo two` (`795-01-fresh`, REQ-001).
# - A quit and a launch: the restored terminal's first new command, `echo three`
#   (`795-02-restored`, REQ-002).
# Variants, each set to 1: `AGENT_FIRST` sends `echo one` by `terminal_run`, as #792's check did;
# `PLAIN_FIRST` adds a terminal Marley did not integrate (a shell given with arguments, #949)
# before the quit, so the restored output holds no command marks; `RUNNING_LAST` leaves
# `sleep 600` running at the quit. The Mac's own check is in #795's notes: no scenario reaches
# macOS from here.
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  write_iterm_stand_in "$home/.iterm2_shell_integration.zsh"
  cat >"$home/.zshrc" <<'RC'
PROMPT='%# '
test -e "${HOME}/.iterm2_shell_integration.zsh" && source "${HOME}/.iterm2_shell_integration.zsh"
RC
  terminal_env HOME "$home"
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  open_path "$repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle "${2:-2}"
}

# iTerm2's zsh integration as far as a terminal sees it: OSC 133 and 1337 marks round each prompt
# and command, a decorated PS1, and its precmd and preexec appended after the user's own.
write_iterm_stand_in() {
  cat >"$1" <<'ZSH'
if [[ -o interactive && -z ${ITERM_SHELL_INTEGRATION_INSTALLED-} ]]; then
  ITERM_SHELL_INTEGRATION_INSTALLED=Yes
  ITERM2_SHOULD_DECORATE_PROMPT=1
  iterm2_before_cmd_executes() { printf "\033]133;C;\007"; }
  iterm2_after_cmd_executes() {
    printf "\033]133;D;%s\007" "$1"
    printf "\033]1337;CurrentDir=%s\007" "$PWD"
  }
  iterm2_decorate_prompt() {
    ITERM2_PRECMD_PS1="$PS1"
    ITERM2_SHOULD_DECORATE_PROMPT=""
    PS1="%{$(printf "\033]133;A\007")%}$PS1%{$(printf "\033]133;B\007")%}"
  }
  iterm2_precmd() {
    local STATUS="$?"
    iterm2_after_cmd_executes "$STATUS"
    if [[ -n $ITERM2_SHOULD_DECORATE_PROMPT ]]; then
      iterm2_decorate_prompt
    fi
  }
  iterm2_preexec() {
    PS1="$ITERM2_PRECMD_PS1"
    ITERM2_SHOULD_DECORATE_PROMPT=1
    iterm2_before_cmd_executes
  }
  precmd_functions=($precmd_functions iterm2_precmd)
  preexec_functions=($preexec_functions iterm2_preexec)
  printf "\033]1337;ShellIntegrationVersion=14;shell=zsh\007"
fi
ZSH
}

cue() {
  type_text "$1"
  press "" Return
  settle "${2:-2}"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 4
  echo "== the first zsh"
  profile_setting terminal.shell '{"program": "zsh"}'
  settle 3
  palette "workspace: new terminal" 6
  if [[ ${AGENT_FIRST:-} == 1 ]]; then
    mcp_agent terminal-run zsh "echo one" | tee "$E2E_WORK/run1.txt"
    settle 2
  else
    cue "echo one"
  fi
  cue "echo two"
  pointer_to 1500 950
  settle 1
  shot 795-01-fresh
  if [[ ${STOP_AT:-} == 1 ]]; then
    return
  fi
  if [[ ${PLAIN_FIRST:-} == 1 ]]; then
    echo "== a plain shell's output, kept with no marks"
    profile_setting terminal.shell '{"with_arguments": {"program": "zsh", "args": ["-l"]}}'
    settle 3
    palette "workspace: new terminal" 6
    cue "echo plain"
    profile_setting terminal.shell '{"program": "zsh"}'
    settle 3
  fi
  if [[ ${RUNNING_LAST:-} == 1 ]]; then
    echo "== a command still running at the quit"
    cue "sleep 600" 3
  fi
  echo "== a quit and a launch"
  quit_marley
  launch_marley
  settle 16
  cue "echo three"
  pointer_to 1500 950
  settle 1
  shot 795-02-restored
}
