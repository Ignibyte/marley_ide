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

# The file holding the command Marley's ssh runs on a host (#526), kept out of the environment
# the same way.
if [ -n "${MARLEY_SSH_COMMAND+set}" ]; then
    __MARLEY_SSH_COMMAND=$MARLEY_SSH_COMMAND
    unset MARLEY_SSH_COMMAND
fi

# On a host Marley's ssh reached (#526): the folder the bootstrap wrote this file to goes now,
# since bash has read the file whole, and a login shell's files run in place of ~/.bashrc, since
# ssh gives a login shell and --rcfile makes bash skip them.
if [ -n "${__MARLEY_CLEANUP+set}" ]; then
    rm -rf -- "$__MARLEY_CLEANUP"
    unset __MARLEY_CLEANUP
    __MARLEY_REMOTE=1
fi
if [ -n "${__MARLEY_LOGIN+set}" ]; then
    unset __MARLEY_LOGIN
    if [ -r /etc/profile ]; then
        . /etc/profile
    fi
    if [ -r "$HOME/.bash_profile" ]; then
        . "$HOME/.bash_profile"
    elif [ -r "$HOME/.bash_login" ]; then
        . "$HOME/.bash_login"
    elif [ -r "$HOME/.profile" ]; then
        . "$HOME/.profile"
    fi
elif [ -r "$HOME/.bashrc" ]; then
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
        builtin printf '\033Pqprecmd;exit=%d;pwd=%s;nonce=%s\033\134' \
            "$status" "$__MARLEY_REPLY" "${__MARLEY_NONCE-}"
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

    # Marley's ssh (#526): an interactive login starts the host's bash or zsh with this
    # integration, carried in the ssh command and gone from the host once read, after a frame
    # that announces the connection's own nonce. A remote command, a flag that makes the session
    # non-interactive, no terminal, a host whose config sets RemoteCommand or the tag
    # marley-plain, and `command ssh` run as plain ssh. A shell on a host, and a user's own ssh
    # function, leave it undefined.
    if [ -z "${__MARLEY_REMOTE-}" ] && [ -n "${__MARLEY_SSH_COMMAND-}" ] \
        && ! declare -F ssh >/dev/null; then
        ssh() {
            local destination='' plain='' wants='' word letters letter index=0
            local -a words=("$@")
            while [ "$index" -lt "${#words[@]}" ]; do
                word=${words[index]}
                index=$((index + 1))
                if [ -n "$wants" ]; then
                    wants=''
                    continue
                fi
                case $word in
                    --)
                        if [ "$index" -lt "${#words[@]}" ]; then
                            destination=${words[index]}
                            index=$((index + 1))
                        fi
                        break
                        ;;
                    -?*)
                        letters=${word#-}
                        while [ -n "$letters" ]; do
                            letter=${letters:0:1}
                            letters=${letters:1}
                            case $letter in
                                [NTWfGVOQsn]) plain=1 ;;
                            esac
                            case $letter in
                                [BbcDEeFIiJLlmOoPpQRSWw])
                                    if [ -z "$letters" ]; then
                                        wants=1
                                    fi
                                    letters=''
                                    ;;
                                [46AaCfGgKkMNnqsTtVvXxYy]) ;;
                                *) plain=1 ;;
                            esac
                        done
                        ;;
                    *)
                        destination=$word
                        break
                        ;;
                esac
            done
            # A word after the destination is a remote command.
            if [ -z "$destination" ] || [ -n "$plain" ] || [ "$index" -lt "${#words[@]}" ] \
                || [ ! -t 0 ] || [ ! -t 1 ]; then
                command ssh "$@"
                return
            fi
            local config line
            if ! config=$(command ssh -G "$@" 2>/dev/null); then
                command ssh "$@"
                return
            fi
            # The loop only reads the host's config: ssh must not run inside it, where its stdin
            # would be the config rather than the terminal.
            while IFS= read -r line; do
                case $line in
                    'remotecommand none') ;;
                    'tag marley-plain' | 'remotecommand '*) plain=1 ;;
                esac
            done <<<"$config"
            if [ -n "$plain" ]; then
                command ssh "$@"
                return
            fi
            local session remote
            session=$(od -An -N16 -tx1 /dev/urandom 2>/dev/null | tr -d ' \n')
            remote=$(cat -- "$__MARLEY_SSH_COMMAND" 2>/dev/null)
            if [ "${#session}" -ne 32 ] || [ -z "$remote" ]; then
                command ssh "$@"
                return
            fi
            __marley_quote "$destination"
            builtin printf '\033Pqremote;host=%s;session=%s;nonce=%s\033\134' \
                "$__MARLEY_REPLY" "$session" "${__MARLEY_NONCE-}"
            command ssh -t "$@" "$remote $session"
        }
    fi

    builtin printf '\033Pqinit;id=%d;nonce=%s\033\134' "$$" "${__MARLEY_NONCE-}"
    # The file bash keeps its history in, which Marley's autosuggestions read.
    if [ -n "${HISTFILE-}" ]; then
        __marley_quote "$HISTFILE"
        builtin printf '\033Pqhistory;file=%s;nonce=%s\033\134' \
            "$__MARLEY_REPLY" "${__MARLEY_NONCE-}"
    fi
    builtin printf '\033Pqbootstrapped;subshell=%d;nonce=%s\033\134' \
        "${__MARLEY_REMOTE:-0}" "${__MARLEY_NONCE-}"
fi
