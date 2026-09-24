# Marley's bash integration. Marley starts an interactive bash with `--rcfile` pointing here,
# which replaces `~/.bashrc`, so the user's own file is sourced first. Then each prompt and each
# command is reported to the terminal as a DCS frame, `ESC P q <payload> ESC \`, whose values are
# escaped the way Marley's decoder (`marley_terminal::dcs`) reads them. In the printf formats,
# `\033` is ESC and `\134` is the frame's closing backslash.

# The terminal's nonce, which each command's frame carries so the terminal can tell the shell's
# own frames from output that prints one. It leaves the environment before the user's file runs,
# so no program the shell starts inherits it.
if [ -n "${MARLEY_SHELL_NONCE+set}" ]; then
    __MARLEY_NONCE=$MARLEY_SHELL_NONCE
    unset MARLEY_SHELL_NONCE
fi

if [ -r "$HOME/.bashrc" ]; then
    . "$HOME/.bashrc"
fi

if [ -z "${__MARLEY_HOOKS-}" ]; then
    __MARLEY_HOOKS=1

    # Escapes a value into __MARLEY_REPLY without a fork: backslash first, since the later steps
    # add backslashes, then the separator and the control characters.
    __marley_quote() {
        __MARLEY_REPLY=$1
        __MARLEY_REPLY=${__MARLEY_REPLY//\\/\\\\}
        __MARLEY_REPLY=${__MARLEY_REPLY//;/\\;}
        __MARLEY_REPLY=${__MARLEY_REPLY//$'\n'/\\n}
        __MARLEY_REPLY=${__MARLEY_REPLY//$'\t'/\\t}
        __MARLEY_REPLY=${__MARLEY_REPLY//$'\r'/\\r}
        __MARLEY_REPLY=${__MARLEY_REPLY//$'\e'/\\x1b}
    }

    # Before each prompt: the last command's exit code and the working directory. It runs first
    # in PROMPT_COMMAND and returns the exit code it found, so later entries still see it.
    __marley_precmd() {
        local status=$?
        __marley_quote "$PWD"
        builtin printf '\033Pqprecmd;exit=%d;pwd=%s\033\134' "$status" "$__MARLEY_REPLY"
        return "$status"
    }

    # Printed through PS0, after a command line is read and before it runs: the line itself, as
    # history holds it.
    __marley_preexec() {
        local line
        line=$(HISTTIMEFORMAT='' builtin fc -ln -0 2>/dev/null)
        line=${line#"${line%%[![:space:]]*}"}
        __marley_quote "$line"
        builtin printf '\033Pqpreexec;command=%s;nonce=%s\033\134' \
            "$__MARLEY_REPLY" "${__MARLEY_NONCE-}"
    }

    # PROMPT_COMMAND is an array since bash 5.1; in older bash its first element is the string.
    if [[ "$(declare -p PROMPT_COMMAND 2>/dev/null)" == "declare -a"* ]]; then
        PROMPT_COMMAND=(__marley_precmd "${PROMPT_COMMAND[@]}")
    else
        PROMPT_COMMAND[0]="__marley_precmd${PROMPT_COMMAND[0]:+;${PROMPT_COMMAND[0]}}"
    fi
    PS0="${PS0-}\$(__marley_preexec)"

    builtin printf '\033Pqinit;id=%d\033\134' "$$"
    # The file bash keeps its history in, which Marley's autosuggestions read.
    if [ -n "${HISTFILE-}" ]; then
        __marley_quote "$HISTFILE"
        builtin printf '\033Pqhistory;file=%s\033\134' "$__MARLEY_REPLY"
    fi
    builtin printf '\033Pqbootstrapped;subshell=0\033\134'
fi
