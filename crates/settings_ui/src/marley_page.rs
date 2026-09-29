// Marley: the Settings window's Marley page (#515). It holds Marley's own settings, and each
// Marley feature with a setting adds its section here.

use std::sync::Arc;

use util::ResultExt as _;

use crate::{ActionLink, SettingField, SettingItem, SettingsPage, SettingsPageItem, USER};

pub(crate) fn marley_page() -> SettingsPage {
    SettingsPage {
        title: "Marley",
        items: layout_section()
            .into_iter()
            .chain(agents_section())
            .chain(terminal_section())
            .chain(push_section())
            .chain(system_one_section())
            .chain(privacy_section())
            .collect(),
    }
}

fn layout_section() -> [SettingsPageItem; 2] {
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
    ]
}

fn agents_section() -> [SettingsPageItem; 11] {
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
    ]
}

// Marley: where a URL clicked in a terminal opens (#503).
fn terminal_section() -> [SettingsPageItem; 3] {
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

// Marley: the System One layer's switch, provider and budget, and a way to its Decisions view
// (#565). The project lists, and each use's mode past the check's, the stop kind's (#566), the
// find tools' (#567), the stall kind's (#569), the click consequence's (#571), the inbox's
// (#568) and the question route's (#570), live in settings.json.
fn system_one_section() -> [SettingsPageItem; 15] {
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
            description: "TypeSafe's API; another server that speaks the same request, at the endpoint below; each use's own rules, with no request; or answers recorded in system_one/replay.jsonl under Marley's data directory.",
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
            description: "The mode of the stop kind, which says on an idle Claude Code's rail row what its stop needs: done and checked, done and only claimed, a question for you, a block, or work still going. Shadow logs it in Decisions, Suggest adds it after idle with a question mark, and Act shows it in place of idle.",
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
            description: "The mode of browser_find, an agents' tool that finds the element of a page a query in words names, such as the sign in button: by the query's words first, then by the model for what the words leave open. Off, agents do not see the tool. Shadow answers by the words and logs the model in Decisions, Suggest gives the model's candidates to check, and Act gives the element to act on when the model is sure.",
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
            description: "The mode of terminal_find, an agents' tool that finds the line of a block's output a query in words names, such as where the server refused the connection: by the query's words first, then by the model for what the words leave open. Off, agents do not see the tool. Shadow answers by the words and logs the model in Decisions, Suggest gives the model's candidates to check, and Act gives the line when the model is sure.",
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
            description: "The mode of the stall kind, which flags a working Claude Code's rail row looping? when it repeats one step and stalled? when it has gone quiet with nothing running, from Marley's own facts first and the model for the quiet case. It never stops the agent. Shadow logs in Decisions, Suggest shows the flag, and Act adds a notification.",
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
        SettingsPageItem::ActionLink(ActionLink {
            title: "Decisions".into(),
            description: Some("Every call the layer made today with what came back, the day's spend against the budget, and where the key comes from: MARLEY_SYSTEM_ONE_KEY or the keyring, never its value. The project lists (projects, metadata_only_projects) and each use's mode live under marley.system_one in settings.json.".into()),
            button_text: "Open Decisions".into(),
            on_click: Arc::new(|settings_window, window, cx| {
                let Some(original_window) = settings_window.original_window else {
                    return;
                };
                // By its name: this crate depends on no Marley crate.
                let Some(action) = cx.build_action("marley::OpenDecisions", None).log_err() else {
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
