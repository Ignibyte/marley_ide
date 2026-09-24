# Marley's zsh integration, installed as `.zshenv` in a directory of its own. Marley starts an
# interactive zsh with `ZDOTDIR` pointing at that directory, and the user's own `ZDOTDIR`, if they
# had one, in `MARLEY_ZSH_ZDOTDIR`. zsh reads `$ZDOTDIR/.zshenv` first, and this file puts the
# user's `ZDOTDIR` back and sources their `.zshenv`, so zsh goes on to read their `.zprofile`,
# `.zshrc` and `.zlogin` as it would have.
# The hooks are installed at the first prompt, once those files have run. Each prompt and each
# command is then reported to the terminal as a DCS frame, `ESC P q <payload> ESC \`, whose values
# are escaped the way Marley's decoder (`marley_terminal::dcs`) reads them.

if [[ -n ${MARLEY_ZSH_ZDOTDIR+set} ]]; then
    ZDOTDIR=$MARLEY_ZSH_ZDOTDIR
    unset MARLEY_ZSH_ZDOTDIR
else
    unset ZDOTDIR
fi

# The terminal's nonce, which each command's frame carries so the terminal can tell the shell's
# own frames from output that prints one. It leaves the environment before the user's files run,
# so no program the shell starts inherits it.
if [[ -n ${MARLEY_SHELL_NONCE+set} ]]; then
    typeset -g __MARLEY_NONCE=$MARLEY_SHELL_NONCE
    unset MARLEY_SHELL_NONCE
fi

if [[ -r ${ZDOTDIR:-$HOME}/.zshenv ]]; then
    builtin source "${ZDOTDIR:-$HOME}/.zshenv"
fi

if [[ -o interactive && -z ${__MARLEY_HOOKS-} ]]; then
    typeset -g __MARLEY_HOOKS=1

    # Escapes a value into __MARLEY_REPLY without a fork: backslash first, since the later steps
    # add backslashes, then the separator and the control characters. Each expansion is quoted,
    # which zsh reads the same everywhere; unquoted, a script's top level drops the backslash
    # from the replacement.
    __marley_quote() {
        emulate -L zsh
        typeset -g __MARLEY_REPLY="$1"
        __MARLEY_REPLY="${__MARLEY_REPLY//\\/\\\\}"
        __MARLEY_REPLY="${__MARLEY_REPLY//;/\\;}"
        __MARLEY_REPLY="${__MARLEY_REPLY//$'\n'/\\n}"
        __MARLEY_REPLY="${__MARLEY_REPLY//$'\t'/\\t}"
        __MARLEY_REPLY="${__MARLEY_REPLY//$'\r'/\\r}"
        __MARLEY_REPLY="${__MARLEY_REPLY//$'\e'/\\x1b}"
    }

    # Before each prompt: the last command's exit code, read before anything else runs, and the
    # working directory.
    __marley_precmd() {
        local exit_code=$?
        emulate -L zsh
        __marley_quote "$PWD"
        builtin printf '\033Pqprecmd;exit=%d;pwd=%s\033\\' "$exit_code" "$__MARLEY_REPLY"
    }

    # After a command line is read and before it runs: the line as it was typed.
    __marley_preexec() {
        emulate -L zsh
        __marley_quote "$1"
        builtin printf '\033Pqpreexec;command=%s;nonce=%s\033\\' \
            "$__MARLEY_REPLY" "${__MARLEY_NONCE-}"
    }

    # At the first prompt the user's files have run and set their own hooks, so Marley's precmd
    # goes first, where it reads the exit code before another hook can change it.
    __marley_install() {
        emulate -L zsh
        precmd_functions=(__marley_precmd ${precmd_functions:#__marley_install})
        preexec_functions+=(__marley_preexec)
        builtin printf '\033Pqinit;id=%d\033\\' "$$"
        # The file zsh keeps its history in, which Marley's autosuggestions read.
        if [[ -n ${HISTFILE-} ]]; then
            __marley_quote "$HISTFILE"
            builtin printf '\033Pqhistory;file=%s\033\\' "$__MARLEY_REPLY"
        fi
        builtin printf '\033Pqbootstrapped;subshell=0\033\\'
        __marley_precmd
    }
    precmd_functions+=(__marley_install)
fi
