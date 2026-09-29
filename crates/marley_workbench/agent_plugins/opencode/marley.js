// marley-opencode-plugin 1
// Marley's plugin for OpenCode (#552): a desktop notification through the Marley terminal OpenCode
// runs in when a session goes idle, asks for a permission or fails. It writes an OSC 777 notify to
// the controlling terminal, whatever the process's stdout is, only when TERM_PROGRAM is `zed`, as
// Marley's terminals set it, and it never throws: a terminal it cannot reach gets nothing.
import { closeSync, openSync, writeSync } from "node:fs";

const MESSAGES = {
  "session.idle": "finished",
  "permission.asked": "needs your permission",
  "permission.updated": "needs your permission",
  "session.error": "hit an error",
};

export const MarleyNotify = async ({ directory } = {}) => {
  const project =
    String(directory || process.cwd())
      .split("/")
      .filter(Boolean)
      .pop() || "OpenCode";
  const notify = (message) => {
    try {
      if (process.env.TERM_PROGRAM !== "zed") return;
      // A body must not end the sequence early.
      const body = `${project} ${message}`.replace(/[\x00-\x1f\x7f;]/g, " ");
      const tty = openSync("/dev/tty", "w");
      try {
        writeSync(tty, `\x1b]777;notify;OpenCode;${body}\x07`);
      } finally {
        closeSync(tty);
      }
    } catch {
      // No controlling terminal, as under `opencode serve`: nothing to tell.
    }
  };
  return {
    event: async ({ event } = {}) => {
      try {
        const message = MESSAGES[event && event.type];
        if (message) notify(message);
      } catch {
        // A hook never breaks OpenCode.
      }
    },
  };
};
