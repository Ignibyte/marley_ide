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

# The file holding the command Marley's ssh runs on a host (#526), kept out of the environment
# the same way.
if [[ -n ${MARLEY_SSH_COMMAND+set} ]]; then
    typeset -g __MARLEY_SSH_COMMAND=$MARLEY_SSH_COMMAND
    unset MARLEY_SSH_COMMAND
fi

# Whether a line typed with a leading space stays out of the history, as an agent's command is
# typed when the user keeps agents out of it (#553); out of the environment the same way.
if [[ -n ${MARLEY_AGENT_HISTORY+set} ]]; then
    [[ $MARLEY_AGENT_HISTORY == 0 ]] && typeset -g __MARLEY_SPACED_OUT=1
    unset MARLEY_AGENT_HISTORY
fi

# Marley's editor for an agent's terminal (#649), out of the environment the same way and
# exported as VISUAL and EDITOR at the first prompt, once the user's files have run.
if [[ -n ${MARLEY_AGENT_EDITOR+set} ]]; then
    typeset -g __MARLEY_AGENT_EDITOR=$MARLEY_AGENT_EDITOR
    unset MARLEY_AGENT_EDITOR
fi

# On a host Marley's ssh reached (#526): the folder the bootstrap wrote this file to goes now;
# zsh keeps the open file readable to its end.
if [[ -n ${__MARLEY_CLEANUP+set} ]]; then
    rm -rf -- "$__MARLEY_CLEANUP"
    unset __MARLEY_CLEANUP
    typeset -g __MARLEY_REMOTE=1
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
        builtin printf '\033Pqprecmd;exit=%d;pwd=%s;nonce=%s\033\\' \
            "$exit_code" "$__MARLEY_REPLY" "${__MARLEY_NONCE-}"
    }

    # After a command line is read and before it runs: the line as it was typed, without the
    # leading space an agent's command may carry (#553).
    __marley_preexec() {
        emulate -L zsh
        __marley_quote "${1#"${1%%[![:space:]]*}"}"
        builtin printf '\033Pqpreexec;command=%s;nonce=%s\033\\' \
            "$__MARLEY_REPLY" "${__MARLEY_NONCE-}"
    }

    # Returns 1, which keeps the line out of the history, for a line starting with a space.
    __marley_addhistory() {
        [[ $1 != [[:space:]]* ]]
    }

    # At the first prompt the user's files have run and set their own hooks, so Marley's precmd
    # goes first, where it reads the exit code before another hook can change it.
    __marley_install() {
        emulate -L zsh
        precmd_functions=(__marley_precmd ${precmd_functions:#__marley_install})
        preexec_functions+=(__marley_preexec)
        # A line typed with a leading space is never saved; preexec still gets it (#553).
        if [[ -n ${__MARLEY_SPACED_OUT-} ]]; then
            zshaddhistory_functions+=(__marley_addhistory)
        fi
        builtin printf '\033Pqinit;id=%d;nonce=%s\033\\' "$$" "${__MARLEY_NONCE-}"
        if [[ -n ${__MARLEY_AGENT_EDITOR-} ]]; then
            export VISUAL=$__MARLEY_AGENT_EDITOR EDITOR=$__MARLEY_AGENT_EDITOR
            unset __MARLEY_AGENT_EDITOR
        fi
        # The file zsh keeps its history in, which Marley's autosuggestions read.
        if [[ -n ${HISTFILE-} ]]; then
            __marley_quote "$HISTFILE"
            builtin printf '\033Pqhistory;file=%s;nonce=%s\033\\' \
                "$__MARLEY_REPLY" "${__MARLEY_NONCE-}"
        fi
        builtin printf '\033Pqbootstrapped;subshell=%d;nonce=%s\033\\' \
            "${__MARLEY_REMOTE:-0}" "${__MARLEY_NONCE-}"
        # Here, after the user's files, so a user's own ssh function is seen (#526).
        if [[ -z ${__MARLEY_REMOTE-} && -n ${__MARLEY_SSH_COMMAND-} ]] \
            && (( ! ${+functions[ssh]} )); then
            functions[ssh]=$functions[__marley_ssh]
        fi
        __marley_precmd
    }

    # Marley's ssh (#526): an interactive login starts the host's bash or zsh with this
    # integration, carried in the ssh command and gone from the host once read, after a frame
    # that announces the connection's own nonce. A remote command, a flag that makes the session
    # non-interactive, no terminal, a host whose config sets RemoteCommand or the tag
    # marley-plain, and `command ssh` run as plain ssh. __marley_install names it ssh.
    __marley_ssh() {
        emulate -L zsh
        local destination='' plain='' wants='' word letters letter config line session remote
        local -i index=1
        local -a words
        words=("$@")
        while (( index <= $#words )); do
            word=$words[index]
            index+=1
            if [[ -n $wants ]]; then
                wants=''
                continue
            fi
            case $word in
                (--)
                    if (( index <= $#words )); then
                        destination=$words[index]
                        index+=1
                    fi
                    break
                    ;;
                (-?*)
                    letters=${word#-}
                    while [[ -n $letters ]]; do
                        letter=${letters[1]}
                        letters=${letters[2,-1]}
                        case $letter in
                            ([NTWfGVOQsn]) plain=1 ;;
                        esac
                        case $letter in
                            ([BbcDEeFIiJLlmOoPpQRSWw])
                                if [[ -z $letters ]]; then
                                    wants=1
                                fi
                                letters=''
                                ;;
                            ([46AaCfGgKkMNnqsTtVvXxYy]) ;;
                            (*) plain=1 ;;
                        esac
                    done
                    ;;
                (*)
                    destination=$word
                    break
                    ;;
            esac
        done
        # A word after the destination is a remote command.
        if [[ -z $destination || -n $plain || ! -t 0 || ! -t 1 ]] || (( index <= $#words )); then
            command ssh "$@"
            return
        fi
        if ! config=$(command ssh -G "$@" 2>/dev/null); then
            command ssh "$@"
            return
        fi
        # The loop only reads the host's config: ssh must not run inside it.
        while IFS= read -r line; do
            case $line in
                ('remotecommand none') ;;
                ('tag marley-plain' | 'remotecommand '*) plain=1 ;;
            esac
        done <<<"$config"
        if [[ -n $plain ]]; then
            command ssh "$@"
            return
        fi
        session=$(od -An -N16 -tx1 /dev/urandom 2>/dev/null | tr -d ' \n')
        remote=$(cat -- "$__MARLEY_SSH_COMMAND" 2>/dev/null)
        if (( $#session != 32 )) || [[ -z $remote ]]; then
            command ssh "$@"
            return
        fi
        __marley_quote "$destination"
        builtin printf '\033Pqremote;host=%s;session=%s;nonce=%s\033\\' \
            "$__MARLEY_REPLY" "$session" "${__MARLEY_NONCE-}"
        command ssh -t "$@" "$remote $session"
    }
    precmd_functions+=(__marley_install)
fi
