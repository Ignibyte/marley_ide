# TICKET-467 — A Marley shell's title leaves out Marley's arguments

- **Ticket:** LOCAL #467 (bug, prong 1 T0c)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/467-shell-title-hides-marleys-arguments.spec.md
- **Source ticket:** ../closed/TICKET-463-shell-integration-scripts.md
- **Status:** closed

## Summary
Since #463 every local bash in Marley starts as `bash --rcfile <data dir>/shell_integration/marley.bash`,
and Zed's terminal title names the foreground process with its arguments
(`Terminal::title`, `crates/terminal/src/terminal.rs:3060`). So every shell tab and every
terminal row in the rail reads `marley_ide — bash --rcfile /home/…/marley.bash` where it read
`marley_ide — bash`. Found in the capture before the rail restyle, 2026-09-23. The arguments
are Marley's, not the user's, and the title should leave them out.

## Acceptance
A shell Marley started with its integration is titled as the same shell started without it;
any other process's title keeps every argument. The EARS criteria are in the spec.
