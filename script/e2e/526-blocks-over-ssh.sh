# shellcheck shell=bash
# #526's visual check: blocks keep working over ssh. A stand-in `ssh` in C, first on the PATH,
# answers `ssh -G` with a plain config and otherwise runs the remote command it is given, its last
# argument, on a pty of its own as a host would, staying in the foreground as ssh does: in a "far" HOME of its own whose `.bash_profile` sets the prompt
# `far$ `, with `SHELL=/bin/bash` and a TMPDIR of its own. So the real bootstrap, the host's
# scripts, the connection's nonce and the terminal's handling of both shells run; only the network
# and sshd do not. Local blocks first, as before (REQ-011); then `ssh far`: the ssh block ends when
# the far shell starts (REQ-002), the far commands are blocks with their exit codes and host
# (REQ-001, REQ-010), the bootstrap's folder is gone (REQ-009); then `exit` and a local command
# again (REQ-006).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin far=$E2E_WORK/far
  mkdir -p "$home" "$bin" "$far" "$E2E_WORK/far-tmp"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
export E2E_SSH_LOG=$E2E_WORK/ssh-args.log E2E_FAR_HOME=$far E2E_FAR_TMP=$E2E_WORK/far-tmp
RC
  terminal_env HOME "$home"
  printf '%s\n' "PS1='far\$ '" >"$far/.bash_profile"
  # The stand-in ssh, in C so it runs as `ssh` and stays in the foreground as the real one does:
  # -G prints a plain config; anything else runs its last argument on a pty of its own, as a host,
  # and relays the terminal to it.
  cat >"$E2E_WORK/ssh.c" <<'C'
#include <poll.h>
#include <pty.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/wait.h>
#include <termios.h>
#include <unistd.h>

int main(int argc, char **argv) {
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-G") == 0) {
            puts("hostname far\nuser e2e\nport 22");
            return 0;
        }
    }
    FILE *log = fopen(getenv("E2E_SSH_LOG"), "a");
    if (log) {
        for (int i = 1; i < argc; i++) fprintf(log, "%.60s ", argv[i]);
        fputc('\n', log);
        fclose(log);
    }
    struct winsize size;
    ioctl(0, TIOCGWINSZ, &size);
    int master;
    pid_t child = forkpty(&master, NULL, NULL, &size);
    if (child == 0) {
        if (chdir(getenv("E2E_FAR_HOME")) != 0) _exit(1);
        setenv("HOME", getenv("E2E_FAR_HOME"), 1);
        setenv("TMPDIR", getenv("E2E_FAR_TMP"), 1);
        setenv("SHELL", "/bin/bash", 1);
        execl("/bin/sh", "sh", "-c", argv[argc - 1], (char *)NULL);
        _exit(127);
    }
    struct termios saved, raw;
    tcgetattr(0, &saved);
    raw = saved;
    cfmakeraw(&raw);
    tcsetattr(0, TCSANOW, &raw);
    char buffer[4096];
    struct pollfd fds[2] = {{0, POLLIN, 0}, {master, POLLIN, 0}};
    for (;;) {
        if (poll(fds, 2, -1) < 0) break;
        if (fds[0].revents & POLLIN) {
            ssize_t n = read(0, buffer, sizeof buffer);
            if (n <= 0 || write(master, buffer, n) != n) break;
        }
        if (fds[1].revents & (POLLIN | POLLHUP)) {
            ssize_t n = read(master, buffer, sizeof buffer);
            if (n <= 0 || write(1, buffer, n) != n) break;
        }
    }
    tcsetattr(0, TCSANOW, &saved);
    int status = 0;
    waitpid(child, &status, 0);
    return WIFEXITED(status) ? WEXITSTATUS(status) : 1;
}
C
  cc -O1 -o "$bin/ssh" "$E2E_WORK/ssh.c" -lutil
  git init -q -b ssh "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  echo "== local blocks, as before"
  type_text "echo local"
  press "" Return
  settle 1
  type_text "false"
  press "" Return
  settle 1
  shot 526-01-local
  echo "== ssh far: the host's shell with Marley's integration"
  type_text "ssh far"
  press "" Return
  settle 4
  # The tab's title: the ssh the user typed, not the bootstrap Marley added.
  shot 526-02-connected
  type_text "echo on the far side"
  press "" Return
  settle 1
  type_text "ls -A \$TMPDIR; echo tmp-listed"
  press "" Return
  settle 1
  type_text "false"
  press "" Return
  settle 1
  shot 526-03-far-blocks
  # Rerun: offered on the far `false` while the far shell waits, not on the local `echo local`.
  pointer_to 1200 "${FAR_FALSE_Y:-936}"
  settle 1
  shot 526-03b-far-rerun
  pointer_to 1200 "${LOCAL_ECHO_Y:-780}"
  settle 1
  shot 526-03c-local-no-rerun
  pointer_to 800 400
  mcp_agent blocks | tee "$E2E_WORK/blocks-far.txt"
  echo "== exit: back to the local shell"
  type_text "exit"
  press "" Return
  settle 2
  type_text "echo back home"
  press "" Return
  settle 1
  shot 526-04-back
  mcp_agent blocks | tee "$E2E_WORK/blocks-back.txt"
  cat "$E2E_WORK/ssh-args.log"
  expect "local commands are verified blocks of this machine" \
    holds "$E2E_WORK/blocks-back.txt" "'echo local', exit 0, running False, kept True, verified True, host None"
  expect "the far command is a verified block of the host" \
    holds "$E2E_WORK/blocks-far.txt" "'echo on the far side', exit 0, running False, kept True, verified True, host far"
  expect "the far false failed there" \
    holds "$E2E_WORK/blocks-far.txt" "'false', exit 1, running False, kept True, verified True, host far"
  expect "the ssh block ended when the host's shell started" \
    holds "$E2E_WORK/blocks-far.txt" "'ssh far', exit None, running False"
  expect "the bootstrap's folder is gone from the host" test -z "$(ls -A "$E2E_WORK/far-tmp")"
  expect "ssh was run with -t and the remote command" holds "$E2E_WORK/ssh-args.log" "-t far sh -c 'b="
  expect "back home, a local command is a local block again" \
    holds "$E2E_WORK/blocks-back.txt" "'echo back home', exit 0, running False, kept True, verified True, host None"
}
