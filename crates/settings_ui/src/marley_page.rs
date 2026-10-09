// Marley: the Settings window's Marley page (#515). It holds Marley's own settings, and each
// Marley feature with a setting adds its section here.

use std::sync::Arc;

use collections::HashMap;
use gpui::{AnyView, Global, ScrollHandle};
use settings::SettingsStore;
use ui::prelude::*;
use util::ResultExt as _;

use crate::{
    ActionLink, SettingField, SettingItem, SettingsPage, SettingsPageItem, SettingsWindow,
    SubPageLink, USER,
};

/// The views Marley's crates draw on the Marley page's sub-pages, by the sub-page's name (#643):
/// the settings UI cannot depend on a Marley crate, so the Marley crate registers its view here.
#[derive(Default)]
pub struct MarleyPageViews(HashMap<&'static str, AnyView>);

impl Global for MarleyPageViews {}

impl MarleyPageViews {
    /// Registers `view` as the content of the sub-page `name`.
    pub fn set(name: &'static str, view: AnyView, cx: &mut App) {
        cx.default_global::<Self>().0.insert(name, view);
    }

    fn get(name: &str, cx: &App) -> Option<AnyView> {
        cx.try_global::<Self>()?.0.get(name).cloned()
    }
}

pub(crate) fn marley_page(cx: &App) -> SettingsPage {
    SettingsPage {
        title: "Marley",
        items: layout_section()
            .into_iter()
            .chain(agents_section())
            .chain(agent_versions_section())
            .chain(terminal_section())
            .chain(push_section())
            .chain(voice_section())
            .chain(agent_control_section())
            .chain(assistant_section())
            .chain(system_one_section())
            // While Rusty is off its section holds the header and the switch alone (#661).
            .chain(
                rusty_section()
                    .into_iter()
                    .take(if rusty_on(cx) { usize::MAX } else { 2 }),
            )
            .chain(privacy_section())
            .collect(),
    }
}

fn layout_section() -> [SettingsPageItem; 6] {
    [
        SettingsPageItem::SectionHeader("Layout"),
        SettingsPageItem::SettingItem(SettingItem {
            title: "Layout",
            description: "Marley's rail of projects with their terminals, or Zed's own layout.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.layout"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.layout.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content.marley.get_or_insert_default().layout = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: the rail's order (#542).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Rail Order",
            description: "The order the rail lists projects and the rows under them in: what needs you first (an agent waiting on you or failed, then one that finished while you looked elsewhere, then working, then not reporting, then the rest), or the window's order. Nothing moves while the pointer is over the rail.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.rail_order"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.rail_order.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content.marley.get_or_insert_default().rail_order = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: the machine's containers, in the rail (#669) and in a panel since #673.
        SettingsPageItem::SettingItem(SettingItem {
            title: "Containers Panel",
            description: "Show the Containers panel's button in the status bar: the panel, on the right, lists the ports of the containers running on this machine that no project's folder holds, each with Stop. A container whose Compose folder is in a project shows under that project in the rail either way.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.rail_containers"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.rail_containers.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .rail_containers = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: the shell's prompt editor at every prompt (#627).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Prompt Editor",
            description: "At a terminal's shell prompt, type in a Zed editor docked under the terminal, with completions and colours, and send the command with Enter. A running command or a full-screen program still gets every key. Off, Ctrl+G at a prompt still opens it.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.prompt_editor"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.prompt_editor.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .prompt_editor = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: the hint for English at a shell's prompt (#557).
        SettingsPageItem::SettingItem(SettingItem {
            title: "English at the Prompt",
            description: "When a line typed at a terminal's prompt reads as a request in words, such as what is using port 3000, show that Ctrl+Shift+Enter asks the agent, and offer Ask the agent on a block the shell could not run (exit 127). Marley's own rules decide; nothing is sent until you ask, and the key works either way.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.english_hint"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.english_hint.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content.marley.get_or_insert_default().english_hint = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
    ]
}

fn agents_section() -> [SettingsPageItem; 18] {
    [
        SettingsPageItem::SectionHeader("Agents"),
        SettingsPageItem::SettingItem(SettingItem {
            title: "Redact Secrets for Agents",
            description: "Hide keys, tokens and passwords in what Marley's tools give agents from terminals and the browser's console (#516). Add your own patterns as `marley.redaction_patterns` in settings.json.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.redact_secrets_for_agents"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.redact_secrets_for_agents.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .redact_secrets_for_agents = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: when a working Claude Code's rail row says `no update in N m` (#547).
        SettingsPageItem::SettingItem(SettingItem {
            title: "No Update After Minutes",
            description: "Minutes a working Claude Code may go without reporting a hook event before its row in the rail says \"no update in N m\" instead of \"working\". 0 turns this off.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.no_update_after_minutes"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.no_update_after_minutes.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .no_update_after_minutes = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: Claude Code sessions resumed after a restart (#540).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Resume Claude Code Sessions",
            description: "When Marley starts, a terminal that was running a Claude Code session at the last quit runs claude --resume with that session, in the folder the session started in, so the conversation comes back.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.resume_agents"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.resume_agents.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .resume_agents = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: when the stall kind's quiet checks come (#569).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Stall Check After Seconds",
            description: "Seconds a working Claude Code may be quiet before the stall kind's first check, with the next at twice, four and eight times it. The checks run only while the stall kind's mode in the System One section is on. 0 turns them off.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.stall_check_after_seconds"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.stall_check_after_seconds.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .stall_check_after_seconds = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: asking before a close ends a working agent, and holding it for undo (#550).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Ask Before Ending a Working Agent",
            description: "Whether closing a terminal, a window or Marley asks first while an agent in it is working, and names each one.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.ask_before_ending_a_working_agent"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.ask_before_ending_a_working_agent.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .ask_before_ending_a_working_agent = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: "Undo Close Seconds",
            description: "Seconds a working agent's terminal closed from its tab is kept, still running, for Undo or Ctrl-Shift-T. 0 ends it at once.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.undo_close_seconds"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.undo_close_seconds.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .undo_close_seconds = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: which agents' consequential clicks in the Browser tab wait for Allow (#571).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Browser Click Pause Agents",
            description: "Whose clicks in a Browser tab that pay, delete, send in your name or change an account wait for Allow or Refuse, while the Click Consequence mode in the System One section is on: agents that run without a permission prompt of their own and callers Marley cannot name, or every agent.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.browser_click_pause_agents"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.browser_click_pause_agents.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .browser_click_pause_agents = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: when an agent's writes into a running program ask the user (#525).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Agent Terminal Writes",
            description: "When an agent that types into a program running in a terminal (psql, a debugger, a REPL) asks you first: its first write to each program, every write, or never.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.agent_terminal_writes"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.agent_terminal_writes.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .agent_terminal_writes = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: whether an agent's commands outside both lists ask (#556); the lists are set in
        // settings.json.
        SettingsPageItem::SettingItem(SettingItem {
            title: "Agent Commands Outside Lists",
            description: "What happens to a command an agent runs at a terminal's prompt (terminal_run) that neither marley.agent_command_allowlist nor marley.agent_command_denylist in settings.json matches: it runs, since the agent's own permission prompt already asked, or it waits for Run or Refuse under the terminal. A command the allowlist matches runs at once; one the denylist matches always asks.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.agent_commands_outside_lists"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.agent_commands_outside_lists.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .agent_commands_outside_lists = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: whether an agent's commands enter the shell's history (#553).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Agent Commands in History",
            description: "Whether commands an agent runs in your terminal enter your shell history and Marley's suggestions. Applies to terminals opened after a change.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.agent_commands_in_history"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.agent_commands_in_history.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .agent_commands_in_history = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: what Marley starts Claude Code and Codex with (#532); the per-project entries
        // are set in settings.json.
        SettingsPageItem::SettingItem(SettingItem {
            title: "Claude Code Permissions",
            description: "What Marley starts Claude Code with: its own permission prompts, or --dangerously-skip-permissions, which asks for none. An entry of agent_permissions_by_project in settings.json wins for its project. A rail row marks an agent that runs without its prompts.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.claude_code_permissions"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.claude_code_permissions.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .claude_code_permissions = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: "Codex Permissions",
            description: "What Marley starts Codex with: its own approvals and sandbox, or full access (--sandbox danger-full-access --ask-for-approval never), with no sandbox and no approvals. An entry of agent_permissions_by_project in settings.json wins for its project.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.codex_permissions"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.codex_permissions.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .codex_permissions = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: Codex runs against an App Server of its own, which Marley joins (#650).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Codex App Server",
            description: "Run each Codex Marley launches in a local project against a Codex App Server of its own, and join it as a second client: the rail, the inbox and the close guard then read the thread's own state, its token use and its sandbox, not the terminal's quiet. Codex's requests then name Marley in their user agent. Applies to Codex launched after the change.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.codex_app_server"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.codex_app_server.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .codex_app_server = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: Marley as Claude Code's IDE for each local project (#653).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Claude Code IDE Link",
            description: "Serve Claude Code's IDE link for each local project: Marley writes a lock file into Claude Code's ide folder and names the project's port in its new terminals, so a claude started there gets your selection and open file with each prompt and the language servers' diagnostics, and send selection mentions the lines in its prompt. /ide reaches it from other terminals. Any program of yours that can read the lock file can connect and read the selection.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.claude_code_ide"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.claude_code_ide.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .claude_code_ide = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: the Claude Code plugin shared with rustal-harness, off by default (#709).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Shared Claude Code Plugin",
            description: "Load the Claude Code plugin Marley shares with rustal-harness in Marley's local terminals, so a claude started there reports its state (working, waiting on you and for what, idle) to the rail. Applies to terminals opened after the change. A Claude Code under a managed disableSideloadFlags refuses to start with it.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.claude_code_shared_plugin"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.claude_code_shared_plugin.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .claude_code_shared_plugin = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: who answers Claude Code's trust question in a new worktree (#587).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Worktree Trust Question",
            description: "Who answers Claude Code's question whether to trust a folder when it asks it in a worktree New Agent in Worktree made, as it does for a repository it has not trusted yet: you, from a notification whose Trust Folder answers it, or Marley, when Zed trusts the worktree's folder.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.claude_code_worktree_trust"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.claude_code_worktree_trust.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .claude_code_worktree_trust = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: agent prompts through the agent's own editor key, in a tab (#649).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Agent Prompts in a Tab",
            description: "Give the terminals Marley opens for agents Marley's editor: Ctrl-G, the Rich Input button and the agent's own editor key open the prompt in a tab, and closing the tab hands it back to the agent, which sends nothing until you do. Off, Ctrl-G opens Marley's Rich Input overlay. Applies to agent terminals opened after the change.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.agent_editor_in_tab"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.agent_editor_in_tab.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .agent_editor_in_tab = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
    ]
}

// Marley: where a URL clicked in a terminal opens (#503).
fn terminal_section() -> [SettingsPageItem; 6] {
    [
        SettingsPageItem::SectionHeader("Terminal"),
        SettingsPageItem::SettingItem(SettingItem {
            title: "Terminal Links",
            description: "Where a URL Ctrl+clicked in a terminal opens: a local one, such as a dev server's localhost address, in a Browser tab of the terminal's project; every one in a Browser tab; or every one in the system browser. Shift+Ctrl+click opens it in the other place. A terminal running ssh always uses the system browser.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.terminal_links"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.terminal_links.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .terminal_links = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: a scrolled-back block's command pinned over the terminal's top row (#529).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Sticky Command Header",
            description: "Whether a block's command is pinned over the terminal's top row while you scroll back through its output. A click on it scrolls to the block's start.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.sticky_command_header"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.sticky_command_header.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .sticky_command_header = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: a block's prompt rows drawn as Marley's header (#628).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Block Headers",
            description: "Whether a block's prompt rows are drawn as Marley's header, the command and its status in place of the shell's prompt. The prompt waiting for your next command stays the shell's.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.block_headers"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.block_headers.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .block_headers = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: how blocks are spaced in a terminal (#631).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Block Density",
            description: "How blocks are spaced in a terminal: comfortable puts half a row between blocks and gives a one-row prompt's header two lines, the folder and branch over the command; compact draws no gaps and keeps each header in its prompt's own rows.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.block_density"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.block_density.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .block_density = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: a long command's end, notified from a terminal not in front (#551).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Long Command Seconds",
            description: "How many seconds a command runs before its end shows a desktop notification, when its terminal is not the one in front. 0 turns this off.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.long_command_seconds"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.long_command_seconds.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .long_command_seconds = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
    ]
}

// Marley: the integrations turned on outside the agent versions Marley tested them on (#648).
fn agent_versions_section() -> [SettingsPageItem; 7] {
    [
        SettingsPageItem::SectionHeader("Agent Versions"),
        SettingsPageItem::SettingItem(SettingItem {
            title: "Prompt Tags on Untested Claude Code",
            description: "Whether Marley tells your prompts from the ones Claude Code adds itself (task notifications, system reminders) by their tags on a Claude Code version it has not checked them on. Off, it reads every prompt there as yours; the agent bar says which version it found and which it checked.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.allow_untested_versions.claude_prompt_tags"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.allow_untested_versions.as_ref())
                        .and_then(|allowed| allowed.get("claude_prompt_tags"))
                },
                write: |settings_content, value, _| {
                    let allowed = settings_content
                        .marley
                        .get_or_insert_default()
                        .allow_untested_versions
                        .get_or_insert_default();
                    let _before = match value {
                        Some(on) => allowed.insert("claude_prompt_tags".to_string(), on),
                        None => allowed.remove("claude_prompt_tags"),
                    };
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: Codex's App Server on a Codex version Marley has not tested it on (#650).
        SettingsPageItem::SettingItem(SettingItem {
            title: "App Server on Untested Codex",
            description: "Whether Codex App Server runs on a Codex version Marley has not tested it on. Off, such a Codex starts as it does with Codex App Server off, and the agent bar says which version it found and which Marley tested.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.allow_untested_versions.codex_app_server"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.allow_untested_versions.as_ref())
                        .and_then(|allowed| allowed.get("codex_app_server"))
                },
                write: |settings_content, value, _| {
                    let allowed = settings_content
                        .marley
                        .get_or_insert_default()
                        .allow_untested_versions
                        .get_or_insert_default();
                    let _before = match value {
                        Some(on) => allowed.insert("codex_app_server".to_string(), on),
                        None => allowed.remove("codex_app_server"),
                    };
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: Claude Code's IDE link on a version Marley has not checked it on (#653).
        SettingsPageItem::SettingItem(SettingItem {
            title: "IDE Link on Untested Claude Code",
            description: "Whether Claude Code IDE Link serves a Claude Code version Marley has not checked the link on. Off, Marley serves no IDE link to it, and the agent bar says which version it found and which Marley checked.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.allow_untested_versions.claude_ide_connection"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.allow_untested_versions.as_ref())
                        .and_then(|allowed| allowed.get("claude_ide_connection"))
                },
                write: |settings_content, value, _| {
                    let allowed = settings_content
                        .marley
                        .get_or_insert_default()
                        .allow_untested_versions
                        .get_or_insert_default();
                    let _before = match value {
                        Some(on) => allowed.insert("claude_ide_connection".to_string(), on),
                        None => allowed.remove("claude_ide_connection"),
                    };
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: the IDE link's selection on an unchecked version (#653).
        SettingsPageItem::SettingItem(SettingItem {
            title: "IDE Selection on Untested Claude Code",
            description: "Whether the IDE link sends your selection and open file to a Claude Code version Marley has not checked them on. Off, such a Claude Code gets diagnostics from Marley but no selection.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.allow_untested_versions.claude_ide_selection"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.allow_untested_versions.as_ref())
                        .and_then(|allowed| allowed.get("claude_ide_selection"))
                },
                write: |settings_content, value, _| {
                    let allowed = settings_content
                        .marley
                        .get_or_insert_default()
                        .allow_untested_versions
                        .get_or_insert_default();
                    let _before = match value {
                        Some(on) => allowed.insert("claude_ide_selection".to_string(), on),
                        None => allowed.remove("claude_ide_selection"),
                    };
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: send selection's mention on an unchecked version (#653).
        SettingsPageItem::SettingItem(SettingItem {
            title: "IDE Mentions on Untested Claude Code",
            description: "Whether send selection mentions the lines in the prompt of a Claude Code version Marley has not checked mentions on. Off, it types @path#La-b at the prompt instead.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.allow_untested_versions.claude_ide_mention"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.allow_untested_versions.as_ref())
                        .and_then(|allowed| allowed.get("claude_ide_mention"))
                },
                write: |settings_content, value, _| {
                    let allowed = settings_content
                        .marley
                        .get_or_insert_default()
                        .allow_untested_versions
                        .get_or_insert_default();
                    let _before = match value {
                        Some(on) => allowed.insert("claude_ide_mention".to_string(), on),
                        None => allowed.remove("claude_ide_mention"),
                    };
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: the shared plugin on an unchecked version (#709).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Shared Plugin on Untested Claude Code",
            description: "Whether Marley's terminals load the shared Claude Code plugin for a Claude Code version before the one the plugin was tested on. Off, they don't, and the rail reads Claude Code's state from its hooks.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.allow_untested_versions.claude_shared_plugin"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.allow_untested_versions.as_ref())
                        .and_then(|allowed| allowed.get("claude_shared_plugin"))
                },
                write: |settings_content, value, _| {
                    let allowed = settings_content
                        .marley
                        .get_or_insert_default()
                        .allow_untested_versions
                        .get_or_insert_default();
                    let _before = match value {
                        Some(on) => allowed.insert("claude_shared_plugin".to_string(), on),
                        None => allowed.remove("claude_shared_plugin"),
                    };
                },
            }),
            metadata: None,
            files: USER,
        }),
    ]
}

// Marley: the master switch of Marley's voice features, off by default (#642).
fn voice_section() -> [SettingsPageItem; 2] {
    [
        SettingsPageItem::SectionHeader("Voice"),
        SettingsPageItem::SettingItem(SettingItem {
            title: "Voice",
            description: "Dictate into a terminal through Voxtype, where it is installed: the agent bar's microphone and marley: toggle dictation. Off, Marley shows no microphone and starts no voxtype. Marley never touches the audio.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.voice.enabled"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.voice.as_ref())
                        .and_then(|voice| voice.enabled.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .voice
                        .get_or_insert_default()
                        .enabled = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
    ]
}

// Marley: the kill switch of agents' write tools (#703).
fn agent_control_section() -> [SettingsPageItem; 5] {
    [
        SettingsPageItem::SectionHeader("Agent Control"),
        SettingsPageItem::SettingItem(SettingItem {
            title: "Stopped",
            description: "Stop every agent's use of Marley's tools that act: typing into or running in a terminal, the browser's clicks and typing, settings and keymap changes, harness seats. The tools that only read keep working. Every call is listed in the Agent Activity tab (marley: open agent activity).",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.agent_control.stopped"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.agent_control.as_ref())
                        .and_then(|agent_control| agent_control.stopped.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .agent_control
                        .get_or_insert_default()
                        .stopped = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: what an agent may do to Zed's editors (#704).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Editors",
            description: "What an agent may do to Zed's editors through Marley's tools: Off refuses them, reads too; Ask First asks before the first tool that acts in an agent's session (Allow for This Session, Always for This Project, Deny); Ask Every asks each time; Allow never asks.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.agent_control.editors"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.agent_control.as_ref())
                        .and_then(|agent_control| agent_control.editors.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .agent_control
                        .get_or_insert_default()
                        .editors = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: what an agent may do to the Agent Panel's threads (#706).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Threads",
            description: "What an agent may do to the Agent Panel's threads through Marley's tools: Off refuses them, reads too; Ask First asks before an agent's session first posts into a thread; Ask Every asks each time; Allow never asks. Answering a thread's permission asks every time unless this is Allow.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.agent_control.threads"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.agent_control.as_ref())
                        .and_then(|agent_control| agent_control.threads.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .agent_control
                        .get_or_insert_default()
                        .threads = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: what an agent may do with Zed's palette actions (#707).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Actions",
            description: "What an agent may do with Zed's palette actions through Marley's tools: Off refuses them; Ask First asks before an agent's session first runs one; Ask Every asks each time; Allow never asks. Only Marley's safe actions and those named in marley.agent_control.actions_allowed run at all.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.agent_control.actions"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.agent_control.as_ref())
                        .and_then(|agent_control| agent_control.actions.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .agent_control
                        .get_or_insert_default()
                        .actions = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
    ]
}

// Marley: the switch of Marley's own agent, off by default (#683), and what it runs on (#687).
fn assistant_section() -> [SettingsPageItem; 3] {
    [
        SettingsPageItem::SectionHeader("Marley Agent"),
        SettingsPageItem::SettingItem(SettingItem {
            title: "Marley Agent",
            description: "Marley's own agent, told to explain and configure Marley with Marley's tools and kept from editing files. Each settings change it proposes waits for your Apply.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.assistant.enabled"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.assistant.as_ref())
                        .and_then(|assistant| assistant.enabled.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .assistant
                        .get_or_insert_default()
                        .enabled = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: "Agent",
            description: "What Marley's own agent runs on, and Rusty's entry while Rusty is on. Auto takes the first found at start: Claude Code signed in, else Codex signed in, else Zed's agent with its default model set up. Claude Code and Codex each run on your own login as an entry in the Agent Panel, Codex in its read-only mode. Zed adds a profile to Zed's own agent, with no file tools.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.assistant.agent"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.assistant.as_ref())
                        .and_then(|assistant| assistant.agent.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .assistant
                        .get_or_insert_default()
                        .agent = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
    ]
}

// Marley: the ntfy server Claude Code's events are pushed to, for the phone (#535).
fn push_section() -> [SettingsPageItem; 4] {
    [
        SettingsPageItem::SectionHeader("Push"),
        SettingsPageItem::SettingItem(SettingItem {
            title: "Push Server",
            description: "An ntfy server on this machine, such as http://127.0.0.1:8090. When Claude Code in a terminal you are not looking at needs input, finishes or fails, Marley pushes one line to it, which the ntfy app shows on your phone. Empty pushes nothing.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.push.url"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.push.as_ref())
                        .and_then(|push| push.url.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .push
                        .get_or_insert_default()
                        .url = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: "Push Topic",
            description: "The ntfy topic the phone subscribes to.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.push.topic"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.push.as_ref())
                        .and_then(|push| push.topic.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .push
                        .get_or_insert_default()
                        .topic = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: "Push Token File",
            description: "A file holding the ntfy access token Marley sends, readable by you alone. Empty sends none.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.push.token_file"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.push.as_ref())
                        .and_then(|push| push.token_file.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .push
                        .get_or_insert_default()
                        .token_file = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
    ]
}

// Marley: the System One layer's switch, provider and budget, and a way to its System One calls
// view
// (#565). The project lists, and each use's mode past the check's, the stop kind's (#566), the
// find tools' (#567), the stall kind's (#569), the click consequence's (#571), the inbox's
// (#568) and the question route's (#570), live in settings.json.
fn system_one_section() -> [SettingsPageItem; 18] {
    [
        SettingsPageItem::SectionHeader("System One"),
        SettingsPageItem::SettingItem(SettingItem {
            title: "System One",
            description: "Ask a System One model typed questions about the states Marley builds from what it knows. Off, Marley makes no request, reads no key and writes no file.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.system_one.enabled"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.system_one.as_ref())
                        .and_then(|system_one| system_one.enabled.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .system_one
                        .get_or_insert_default()
                        .enabled = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: "Provider",
            description: "TypeSafe's API; another server that speaks the same request, at the endpoint below; each use's own rules, with no request; answers recorded in system_one/replay.jsonl under Marley's data directory; or Jev through Cloudflare Workers AI, which states it keeps no data, in the Cloudflare account below. provider_by_project in settings.json picks one per project.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.system_one.provider"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.system_one.as_ref())
                        .and_then(|system_one| system_one.provider.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .system_one
                        .get_or_insert_default()
                        .provider = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: "Endpoint",
            description: "The compatible provider's URL: https, or http on this machine.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.system_one.endpoint"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.system_one.as_ref())
                        .and_then(|system_one| system_one.endpoint.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .system_one
                        .get_or_insert_default()
                        .endpoint = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: #548, the account the cloudflare provider runs Jev in.
        SettingsPageItem::SettingItem(SettingItem {
            title: "Cloudflare Account ID",
            description: "The Cloudflare account the cloudflare provider runs Jev in. Its API token comes from MARLEY_CLOUDFLARE_API_TOKEN, or Set Cloudflare Token in System One calls.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.system_one.cloudflare_account_id"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.system_one.as_ref())
                        .and_then(|system_one| system_one.cloudflare_account_id.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .system_one
                        .get_or_insert_default()
                        .cloudflare_account_id = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: "Model",
            description: "The model asked, pinned to a version, such as jev-1.13.0.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.system_one.model"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.system_one.as_ref())
                        .and_then(|system_one| system_one.model.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .system_one
                        .get_or_insert_default()
                        .model = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: "Daily Budget",
            description: "The most the layer spends in a day, in cents. Once it is spent, calls wait for the next day.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.system_one.daily_budget_cents"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.system_one.as_ref())
                        .and_then(|system_one| system_one.daily_budget_cents.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .system_one
                        .get_or_insert_default()
                        .daily_budget_cents = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: "Check",
            description: "The mode of the check (marley: system one check), which asks whether the last command of the terminal you used last failed. It runs only when you run it.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.system_one.uses.check"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.system_one.as_ref())
                        .and_then(|system_one| system_one.uses.as_ref())
                        .and_then(|uses| uses.get("check"))
                },
                write: |settings_content, value, _| {
                    let uses = settings_content
                        .marley
                        .get_or_insert_default()
                        .system_one
                        .get_or_insert_default()
                        .uses
                        .get_or_insert_default();
                    let _before = match value {
                        Some(mode) => uses.insert("check".to_string(), mode),
                        None => uses.remove("check"),
                    };
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: the stop kind's mode (#566).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Stop Kind",
            description: "The mode of the stop kind, which says on an idle Claude Code's rail row what its stop needs: done and checked, done and only claimed, a question for you, a block, or work still going. Shadow logs it in System One calls, Suggest adds it after idle with a question mark, and Act shows it in place of idle.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.system_one.uses.stop_kind"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.system_one.as_ref())
                        .and_then(|system_one| system_one.uses.as_ref())
                        .and_then(|uses| uses.get("stop_kind"))
                },
                write: |settings_content, value, _| {
                    let uses = settings_content
                        .marley
                        .get_or_insert_default()
                        .system_one
                        .get_or_insert_default()
                        .uses
                        .get_or_insert_default();
                    let _before = match value {
                        Some(mode) => uses.insert("stop_kind".to_string(), mode),
                        None => uses.remove("stop_kind"),
                    };
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: the mode of browser_find (#567).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Browser Find",
            description: "The mode of browser_find, an agents' tool that finds the element of a page a query in words names, such as the sign in button: by the query's words first, then by the model for what the words leave open. Off, agents do not see the tool. Shadow answers by the words and logs the model in System One calls, Suggest gives the model's candidates to check, and Act gives the element to act on when the model is sure.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.system_one.uses.browser_find"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.system_one.as_ref())
                        .and_then(|system_one| system_one.uses.as_ref())
                        .and_then(|uses| uses.get("browser_find"))
                },
                write: |settings_content, value, _| {
                    let uses = settings_content
                        .marley
                        .get_or_insert_default()
                        .system_one
                        .get_or_insert_default()
                        .uses
                        .get_or_insert_default();
                    let _before = match value {
                        Some(mode) => uses.insert("browser_find".to_string(), mode),
                        None => uses.remove("browser_find"),
                    };
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: the mode of terminal_find (#567).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Terminal Find",
            description: "The mode of terminal_find, an agents' tool that finds the line of a block's output a query in words names, such as where the server refused the connection: by the query's words first, then by the model for what the words leave open. Off, agents do not see the tool. Shadow answers by the words and logs the model in System One calls, Suggest gives the model's candidates to check, and Act gives the line when the model is sure.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.system_one.uses.terminal_find"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.system_one.as_ref())
                        .and_then(|system_one| system_one.uses.as_ref())
                        .and_then(|uses| uses.get("terminal_find"))
                },
                write: |settings_content, value, _| {
                    let uses = settings_content
                        .marley
                        .get_or_insert_default()
                        .system_one
                        .get_or_insert_default()
                        .uses
                        .get_or_insert_default();
                    let _before = match value {
                        Some(mode) => uses.insert("terminal_find".to_string(), mode),
                        None => uses.remove("terminal_find"),
                    };
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: the mode of the stall kind (#569).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Stall Kind",
            description: "The mode of the stall kind, which flags a working Claude Code's rail row looping? when it repeats one step and stalled? when it has gone quiet with nothing running, from Marley's own facts first and the model for the quiet case. It never stops the agent. Shadow logs in System One calls, Suggest shows the flag, and Act adds a notification.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.system_one.uses.stall_kind"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.system_one.as_ref())
                        .and_then(|system_one| system_one.uses.as_ref())
                        .and_then(|uses| uses.get("stall_kind"))
                },
                write: |settings_content, value, _| {
                    let uses = settings_content
                        .marley
                        .get_or_insert_default()
                        .system_one
                        .get_or_insert_default()
                        .uses
                        .get_or_insert_default();
                    let _before = match value {
                        Some(mode) => uses.insert("stall_kind".to_string(), mode),
                        None => uses.remove("stall_kind"),
                    };
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: the mode of the click consequence (#571).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Click Consequence",
            description: "The mode of the click consequence, which holds an agent's click in a Browser tab that pays, deletes, sends in your name or changes an account until you Allow or Refuse it, from Marley's own rules first and the model for a click the rules leave open. Off makes no pause. Shadow pauses on the rules and logs the model, Suggest adds a notice after an open click the model reads as consequential, and Act pauses on it too.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.system_one.uses.click_consequence"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.system_one.as_ref())
                        .and_then(|system_one| system_one.uses.as_ref())
                        .and_then(|uses| uses.get("click_consequence"))
                },
                write: |settings_content, value, _| {
                    let uses = settings_content
                        .marley
                        .get_or_insert_default()
                        .system_one
                        .get_or_insert_default()
                        .uses
                        .get_or_insert_default();
                    let _before = match value {
                        Some(mode) => uses.insert("click_consequence".to_string(), mode),
                        None => uses.remove("click_consequence"),
                    };
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: the mode of the inbox's risk chips (#568).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Inbox Risk",
            description: "The mode of the inbox's risk chips, which mark what an agent waits on in the rail's inbox (destroys, credentials, rewrites history, sends out, installs, outside project, claims approval) and put what matters most first, from Marley's own rules first and the model for what the rules found nothing on. Nothing here answers a prompt. Off keeps the inbox oldest first with no chips. Shadow shows the rules' chips and order and logs the model, Suggest shows the model's chips with a question mark, and Act lets them order the inbox too.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.system_one.uses.inbox"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.system_one.as_ref())
                        .and_then(|system_one| system_one.uses.as_ref())
                        .and_then(|uses| uses.get("inbox"))
                },
                write: |settings_content, value, _| {
                    let uses = settings_content
                        .marley
                        .get_or_insert_default()
                        .system_one
                        .get_or_insert_default()
                        .uses
                        .get_or_insert_default();
                    let _before = match value {
                        Some(mode) => uses.insert("inbox".to_string(), mode),
                        None => uses.remove("inbox"),
                    };
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: the mode of the question route (#570).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Question Route",
            description: "The mode of the question route, which marks each entry of the rail's inbox with who should answer it: you, a manager agent, the agent itself (could proceed), or unclear, from Marley's own rules first and the model for what the rules leave open. Nothing here answers anything. Off shows no marks. Shadow shows the rules' marks and logs the model, Suggest shows the model's marks with a question mark, and Act lets the marks order the inbox within each level too.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.system_one.uses.question_route"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.system_one.as_ref())
                        .and_then(|system_one| system_one.uses.as_ref())
                        .and_then(|uses| uses.get("question_route"))
                },
                write: |settings_content, value, _| {
                    let uses = settings_content
                        .marley
                        .get_or_insert_default()
                        .system_one
                        .get_or_insert_default()
                        .uses
                        .get_or_insert_default();
                    let _before = match value {
                        Some(mode) => uses.insert("question_route".to_string(), mode),
                        None => uses.remove("question_route"),
                    };
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: the mode of the running error (#572).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Running Error",
            description: "The mode of the running error, which tells a command that keeps running, such as a dev server, when it prints a failure and is still running five seconds later: a red mark on its rail row with the line, a notification when the terminal is not in front, and another when it recovers. The lines' shapes decide first; the model reads only the lines they leave open. Shadow logs the model in System One calls, Suggest marks its failures with a question mark and no notification, and Act treats them as the shapes'. Off watches nothing.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.system_one.uses.running_error"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.system_one.as_ref())
                        .and_then(|system_one| system_one.uses.as_ref())
                        .and_then(|uses| uses.get("running_error"))
                },
                write: |settings_content, value, _| {
                    let uses = settings_content
                        .marley
                        .get_or_insert_default()
                        .system_one
                        .get_or_insert_default()
                        .uses
                        .get_or_insert_default();
                    let _before = match value {
                        Some(mode) => uses.insert("running_error".to_string(), mode),
                        None => uses.remove("running_error"),
                    };
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: the mode of the typed line (#573).
        SettingsPageItem::SettingItem(SettingItem {
            title: "Typed Line",
            description: "The mode of the typed line, which reads a line typed at a shell's prompt that Marley's own rules leave open, a command's name followed by plain words, as a command, a request, a comment, or a command followed by English. It asks only after 250 ms without typing, never for a line holding a secret, and never holds up Enter. Shadow logs the reading in System One calls, Suggest shows it after the line with a question mark, and Act shows it plainly and colours the words a command would take as arguments. Off asks nothing.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.system_one.uses.typed_line"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.system_one.as_ref())
                        .and_then(|system_one| system_one.uses.as_ref())
                        .and_then(|uses| uses.get("typed_line"))
                },
                write: |settings_content, value, _| {
                    let uses = settings_content
                        .marley
                        .get_or_insert_default()
                        .system_one
                        .get_or_insert_default()
                        .uses
                        .get_or_insert_default();
                    let _before = match value {
                        Some(mode) => uses.insert("typed_line".to_string(), mode),
                        None => uses.remove("typed_line"),
                    };
                },
            }),
            metadata: None,
            files: USER,
        }),
        SettingsPageItem::ActionLink(ActionLink {
            title: "System One Calls".into(),
            description: Some("Every call the layer made today with what came back, the day's spend against the budget, and where the key comes from: MARLEY_SYSTEM_ONE_KEY or the keyring, never its value. The project lists (projects, metadata_only_projects) and each use's mode live under marley.system_one in settings.json.".into()),
            button_text: "Open System One Calls".into(),
            on_click: Arc::new(|settings_window, window, cx| {
                let Some(original_window) = settings_window.original_window else {
                    return;
                };
                // By its name: this crate depends on no Marley crate.
                let Some(action) = cx.build_action("marley::OpenSystemOneCalls", None).log_err() else {
                    return;
                };
                original_window
                    .update(cx, |_workspace, original_window, cx| {
                        original_window.dispatch_action(action, cx);
                        original_window.activate_window();
                    })
                    .log_err();
                window.remove_window();
            }),
            files: USER,
        }),
    ]
}

/// Whether `marley.rusty.enabled` is on, as the settings resolve it: the Marley page shows Rusty's
/// items past its switch only then, and the window rebuilds its pages when it changes (#661).
pub(crate) fn rusty_on(cx: &App) -> bool {
    cx.global::<SettingsStore>()
        .merged_settings()
        .marley
        .as_ref()
        .and_then(|marley| marley.rusty.as_ref())
        .and_then(|rusty| rusty.enabled)
        .unwrap_or(false)
}

// Marley: Rusty's switch and connection (#643); the connection's state and Rusty's own settings
// are on the Rusty's Server sub-page, which draws the view `marley_workbench::rusty` registers.
// The header and the switch come first: while Rusty is off the page keeps those two (#661).
fn rusty_section() -> [SettingsPageItem; 6] {
    [
        SettingsPageItem::SectionHeader("Rusty"),
        SettingsPageItem::SettingItem(SettingItem {
            title: "Rusty",
            description: "Connect Marley to Rusty, the local assistant store, through its MCP server, rusty-mcp. Off, Marley starts no rusty-mcp, opens no connection and offers Zed's agents no Rusty tools.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.rusty.enabled"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.rusty.as_ref())
                        .and_then(|rusty| rusty.enabled.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .rusty
                        .get_or_insert_default()
                        .enabled = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: "Connection",
            description: "Embedded starts rusty-mcp on stdio, from MARLEY_RUSTY_MCP or your PATH, and ends it when Rusty turns off or Marley quits. Service connects to Rusty's running service at the Service URL and starts nothing.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.rusty.connection"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.rusty.as_ref())
                        .and_then(|rusty| rusty.connection.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .rusty
                        .get_or_insert_default()
                        .connection = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: "Service URL",
            description: "Where Rusty's service listens, an http URL on this machine, for the Service connection.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.rusty.service_url"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.rusty.as_ref())
                        .and_then(|rusty| rusty.service_url.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .rusty
                        .get_or_insert_default()
                        .service_url = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        // Marley: Rusty's MCP server for Zed's agents (#633), in the Agents section until #643.
        SettingsPageItem::SettingItem(SettingItem {
            title: "Rusty Tools for Agents",
            description: "Whether Zed's agents get Rusty's tools, its brain loop among them, through the context server rusty, while Rusty is on and rusty-mcp is found. A rusty entry of your own in context_servers wins.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("marley.rusty.agent_tools"),
                pick: |settings_content| {
                    settings_content
                        .marley
                        .as_ref()
                        .and_then(|marley| marley.rusty.as_ref())
                        .and_then(|rusty| rusty.agent_tools.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .marley
                        .get_or_insert_default()
                        .rusty
                        .get_or_insert_default()
                        .agent_tools = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        SettingsPageItem::SubPageLink(SubPageLink {
            title: "Rusty's Server".into(),
            r#type: Default::default(),
            description: Some(
                "Whether Marley is connected to rusty-mcp, and Rusty's own settings, read from it and written to it."
                    .into(),
            ),
            search_aliases: &[],
            // A path that names no key, as `agent.skills` does, so `zed::OpenSettingsAt` opens it.
            json_path: Some("marley.rusty.server"),
            in_json: false,
            files: USER,
            render: render_rusty_server_page,
        }),
    ]
}

/// The Rusty's Server sub-page: the view `marley_workbench::rusty` registers as `rusty`.
fn render_rusty_server_page(
    _: &SettingsWindow,
    scroll_handle: &ScrollHandle,
    _: &mut Window,
    cx: &mut Context<SettingsWindow>,
) -> AnyElement {
    let content = MarleyPageViews::get("rusty", cx).map_or_else(
        || {
            Label::new("Rusty's view is not loaded.")
                .color(Color::Muted)
                .into_any_element()
        },
        AnyView::into_any_element,
    );
    v_flex()
        .id("marley-rusty-server-page")
        .track_scroll(scroll_handle)
        .size_full()
        .pt_2p5()
        .pb_16()
        .overflow_y_scroll()
        .child(v_flex().w_full().px_8().gap_2().child(content))
        .into_any_element()
}

// The same two keys as Zed's General page, which shows them too: off unless turned on (#514).
fn privacy_section() -> [SettingsPageItem; 3] {
    [
        SettingsPageItem::SectionHeader("Privacy"),
        SettingsPageItem::SettingItem(SettingItem {
            title: "Telemetry Diagnostics",
            description: "Send debug information like crash reports to Zed's servers.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("telemetry.diagnostics"),
                pick: |settings_content| {
                    settings_content
                        .telemetry
                        .as_ref()
                        .and_then(|telemetry| telemetry.diagnostics.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content
                        .telemetry
                        .get_or_insert_default()
                        .diagnostics = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: "Telemetry Metrics",
            description: "Send anonymized usage data to Zed's servers.",
            field: Box::new(SettingField {
                organization_override: None,
                json_path: Some("telemetry.metrics"),
                pick: |settings_content| {
                    settings_content
                        .telemetry
                        .as_ref()
                        .and_then(|telemetry| telemetry.metrics.as_ref())
                },
                write: |settings_content, value, _| {
                    settings_content.telemetry.get_or_insert_default().metrics = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
    ]
}
