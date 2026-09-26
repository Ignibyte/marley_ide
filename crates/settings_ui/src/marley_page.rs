// Marley: the Settings window's Marley page (#515). It holds Marley's own settings, and each
// Marley feature with a setting adds its section here.

use crate::{SettingField, SettingItem, SettingsPage, SettingsPageItem, USER};

pub(crate) fn marley_page() -> SettingsPage {
    SettingsPage {
        title: "Marley",
        items: layout_section()
            .into_iter()
            .chain(agents_section())
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

fn agents_section() -> [SettingsPageItem; 2] {
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
