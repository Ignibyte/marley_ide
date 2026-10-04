# Marley's fish integration, installed as `fish/vendor_conf.d/marley.fish` in a folder Marley puts
# first on `XDG_DATA_DIRS` for the fish it starts, with the user's own value, if they had one, in
# `MARLEY_FISH_DATA_DIRS`. fish runs vendor snippets before the user's `config.fish`, so this file
# puts the user's `XDG_DATA_DIRS` back and takes Marley's variables out of the environment first.
# The hooks are event handlers defined here, before any the user's files define, so Marley's frame
# ends a command's output before another handler prints. Each prompt and each command is reported
# to the terminal as a DCS frame, `ESC P q <payload> ESC \`, whose values are escaped the way
# Marley's decoder (`marley_terminal::dcs`) reads them.

if set -q MARLEY_FISH_DATA_DIRS
    set -gx XDG_DATA_DIRS $MARLEY_FISH_DATA_DIRS
    set -e MARLEY_FISH_DATA_DIRS
else
    set -e XDG_DATA_DIRS
end

# The terminal's nonce, which each command's frame carries so the terminal can tell the shell's
# own frames from output that prints one. It leaves the environment before the user's files run,
# so no program the shell starts inherits it.
if set -q MARLEY_SHELL_NONCE
    set -g __marley_nonce $MARLEY_SHELL_NONCE
    set -e MARLEY_SHELL_NONCE
end

# Marley's ssh (#526) has no fish function yet, and fish keeps a line typed with a leading space
# out of its history by itself (#553): both variables only leave the environment.
set -e MARLEY_SSH_COMMAND
set -e MARLEY_AGENT_HISTORY

# Marley's editor for an agent's terminal (#649), out of the environment and exported as VISUAL
# and EDITOR at the first prompt, once the user's files have run.
if set -q MARLEY_AGENT_EDITOR
    set -g __marley_agent_editor $MARLEY_AGENT_EDITOR
    set -e MARLEY_AGENT_EDITOR
end

if status is-interactive; and not set -q __marley_hooks
    set -g __marley_hooks 1
    set -g __marley_status 0

    # Prints the arguments, one value's lines, escaped: backslash first, since the later steps add
    # backslashes, then the separator and the control characters, and the lines joined by `\n`.
    function __marley_quote
        string join \n -- $argv | string replace -a '\\' '\\\\' | string replace -a ';' '\;' \
            | string replace -a \t '\\t' | string replace -a \r '\\r' \
            | string replace -a \e '\\x1b' | string join '\\n'
    end

    # After a command: its exit status, for the next prompt's frame.
    function __marley_postexec --on-event fish_postexec
        set -g __marley_status $status
    end

    # Before each prompt: the last command's exit status and the working directory. The first
    # prompt comes after the user's files have run, so the shell is announced then.
    function __marley_prompt --on-event fish_prompt
        if not set -q __marley_announced
            set -g __marley_announced 1
            printf '\033Pqinit;id=%d;nonce=%s\033\\\\' $fish_pid "$__marley_nonce"
            if set -q __marley_agent_editor
                set -gx VISUAL $__marley_agent_editor
                set -gx EDITOR $__marley_agent_editor
                set -e __marley_agent_editor
            end
            # The file fish keeps its history in, which Marley's autosuggestions read; none when
            # `fish_history` is empty, which turns the history off.
            set -l session fish
            set -q fish_history; and set session $fish_history
            if test -n "$session"
                set -l data ~/.local/share
                set -q XDG_DATA_HOME; and set data $XDG_DATA_HOME
                printf '\033Pqhistory;file=%s;nonce=%s\033\\\\' \
                    "$(__marley_quote $data/fish/$session"_history")" "$__marley_nonce"
            end
            printf '\033Pqbootstrapped;subshell=0;nonce=%s\033\\\\' "$__marley_nonce"
        end
        printf '\033Pqprecmd;exit=%d;pwd=%s;nonce=%s\033\\\\' \
            $__marley_status "$(__marley_quote $PWD)" "$__marley_nonce"
    end

    # After a command line is read and before it runs: the line as it was typed, without the
    # leading space an agent's command may carry (#553).
    function __marley_preexec --on-event fish_preexec
        printf '\033Pqpreexec;command=%s;nonce=%s\033\\\\' \
            "$(__marley_quote (string trim -l -- $argv[1]))" "$__marley_nonce"
    end
end
