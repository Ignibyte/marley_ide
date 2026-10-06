//! A line or a URL captured into Rusty's brain, and an Obsidian vault imported (#663).
//!
//! `brain_capture` appends a line to today's daily note or the inbox as a timeline entry and
//! answers where it went; `source_capture` has Rusty fetch a URL and keep it as a `source` page,
//! recording a failed fetch on the page rather than refusing; `brain_import_plan` says what an
//! import of a vault folder would do, and `brain_import` does it. The last three can take longer
//! than Marley's usual deadline, so each carries its own.

use std::time::Duration;

use serde::Deserialize;
use serde_json::{Value, json};

use crate::bookmarks::Bookmark;

/// Rusty's tool that appends a line to the daily note or the inbox.
pub const BRAIN_CAPTURE: &str = "brain_capture";
/// Rusty's tool that keeps a URL as a source page.
pub const SOURCE_CAPTURE: &str = "source_capture";
/// Rusty's tool that says what importing a vault would do.
pub const BRAIN_IMPORT_PLAN: &str = "brain_import_plan";
/// Rusty's tool that imports a vault.
pub const BRAIN_IMPORT: &str = "brain_import";

/// How long a URL's capture may take: Rusty's fetch stops at 20 s, then it reads the page, indexes
/// it and commits.
pub const CAPTURE_URL_DEADLINE: Duration = Duration::from_secs(45);
/// How long the import's plan may take: Rusty reads every file of the vault.
pub const IMPORT_PLAN_DEADLINE: Duration = Duration::from_secs(60);
/// How long an import may take: Rusty copies, rewrites links and rebuilds its index.
pub const IMPORT_DEADLINE: Duration = Duration::from_secs(600);

/// Where a line goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureTarget {
    /// Today's daily note.
    Daily,
    /// The inbox page.
    Inbox,
}

impl CaptureTarget {
    /// The target as Rusty takes it.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Daily => "daily",
            Self::Inbox => "inbox",
        }
    }

    /// The capture form's title.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Daily => "Capture to Today's Note",
            Self::Inbox => "Capture to the Inbox",
        }
    }

    /// `brain_capture`'s arguments for `text`, sent as typed: Rusty trims it and refuses an empty
    /// line in its own words.
    #[must_use]
    pub fn arguments(self, text: &str) -> Value {
        json!({ "text": text, "target": self.as_str() })
    }
}

/// `brain_capture`'s answer.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CaptureReceipt {
    /// The page the line went to.
    pub slug: String,
    /// Whether Rusty made the page first.
    #[serde(default)]
    pub created_page: bool,
}

/// `brain_capture`'s answer.
///
/// # Errors
///
/// When the answer is not a receipt.
pub fn receipt_from_answer(text: &str) -> Result<CaptureReceipt, serde_json::Error> {
    serde_json::from_str(text)
}

/// `source_capture`'s arguments.
#[must_use]
pub fn source_arguments(url: &str) -> Value {
    json!({ "url": url.trim() })
}

/// The source page `source_capture` answers with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePage {
    /// The page's slug, under `sources/`.
    pub slug: String,
    /// Rusty's error, when the fetch failed and the page records the failure.
    pub failed: Option<String>,
}

/// `source_capture`'s answer: a page, its frontmatter's `status` and `error` flattened into
/// `frontmatter` as Rusty serves them.
///
/// # Errors
///
/// When the answer is not a page with a slug.
pub fn source_page_from_answer(text: &str) -> Result<SourcePage, String> {
    let page: Value = serde_json::from_str(text).map_err(|error| error.to_string())?;
    let slug = page
        .get("slug")
        .and_then(Value::as_str)
        .filter(|slug| !slug.is_empty())
        .ok_or_else(|| "the page has no slug".to_string())?;
    let frontmatter = page.get("frontmatter");
    let field = |key: &str| {
        frontmatter
            .and_then(|frontmatter| frontmatter.get(key))
            .and_then(Value::as_str)
    };
    let failed = (field("status") == Some("failed"))
        .then(|| field("error").unwrap_or("the fetch failed").to_string());
    Ok(SourcePage {
        slug: slug.to_string(),
        failed,
    })
}

/// `brain_import_plan`'s and `brain_import`'s arguments.
#[must_use]
pub fn import_arguments(path: &str) -> Value {
    json!({ "path": path })
}

/// What importing a vault would do, as `brain_import_plan` answers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct ImportPlan {
    /// The vault folder.
    pub source: String,
    /// Its name.
    pub name: String,
    /// The slugs that come in.
    pub pages: Vec<String>,
    /// The folders they and the attachments live in.
    pub folders: Vec<String>,
    /// The attachments that come in.
    pub attachments: Vec<String>,
    /// What is already in the brain: skipped, never overwritten.
    pub collisions: Vec<String>,
    /// The incoming pages' tags.
    pub tags: Vec<String>,
    /// Links in the incoming pages that resolve to nothing, as `slug: [[target]]`.
    pub unresolved_links: Vec<String>,
    /// The vault's bookmarks that come across.
    pub bookmarks: Vec<Bookmark>,
    /// The ones that do not, and why.
    pub bookmarks_skipped: Vec<String>,
}

impl ImportPlan {
    /// Whether the import brings in anything: Import waits for a page or an attachment.
    #[must_use]
    pub const fn brings_anything(&self) -> bool {
        !self.pages.is_empty() || !self.attachments.is_empty()
    }

    /// The plan in one sentence, as Rusty's app words it.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "Will bring in {} in {}, {}, {} and {}; {} skipped; {} unresolved. The vault is read \
             and never written.",
            count(self.pages.len(), "page"),
            count(self.folders.len(), "folder"),
            count(self.attachments.len(), "attachment"),
            count(self.tags.len(), "tag"),
            count(self.bookmarks.len(), "bookmark"),
            count(self.collisions.len(), "collision"),
            count(self.unresolved_links.len(), "link"),
        )
    }

    /// The plan's lists, each under its heading, `none` for an empty one.
    #[must_use]
    pub fn details(&self) -> Vec<(&'static str, Vec<String>)> {
        let tags = self.tags.iter().map(|tag| format!("#{tag}")).collect();
        let bookmarks = self.bookmarks.iter().map(bookmark_line).collect();
        vec![
            ("Pages", self.pages.clone()),
            ("Attachments", self.attachments.clone()),
            (
                "Collisions (already in the brain, left as they are)",
                self.collisions.clone(),
            ),
            ("Unresolved links", self.unresolved_links.clone()),
            ("Tags", tags),
            ("Bookmarks", bookmarks),
            ("Bookmarks not carried", self.bookmarks_skipped.clone()),
        ]
    }
}

/// `brain_import_plan`'s answer.
///
/// # Errors
///
/// When the answer is not a plan.
pub fn plan_from_answer(text: &str) -> Result<ImportPlan, serde_json::Error> {
    serde_json::from_str(text)
}

/// What an import did, as `brain_import` answers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct ImportReport {
    /// The plan it carried out.
    pub plan: ImportPlan,
    /// The pages it brought in.
    pub imported_pages: usize,
    /// The attachments it brought in.
    pub imported_attachments: usize,
    /// The bare-name links it rewrote to vault paths.
    pub links_rewritten: usize,
    /// The vault's bookmarks Rusty did not have yet.
    pub bookmarks_added: usize,
    /// The report page under `inbox/`.
    pub report_slug: String,
}

impl ImportReport {
    /// The report in one sentence, as Rusty's app words it.
    #[must_use]
    pub fn summary(&self) -> String {
        let report = if self.report_slug.is_empty() {
            String::new()
        } else {
            format!(" The report is {}.", self.report_slug)
        };
        format!(
            "Imported {} and {}; {} rewritten to vault paths; {} added.{report}",
            count(self.imported_pages, "page"),
            count(self.imported_attachments, "attachment"),
            count(self.links_rewritten, "link"),
            count(self.bookmarks_added, "bookmark"),
        )
    }
}

/// `brain_import`'s answer.
///
/// # Errors
///
/// When the answer is not a report.
pub fn report_from_answer(text: &str) -> Result<ImportReport, serde_json::Error> {
    serde_json::from_str(text)
}

/// "1 page", "2 pages".
fn count(number: usize, noun: &str) -> String {
    if number == 1 {
        format!("1 {noun}")
    } else {
        format!("{number} {noun}s")
    }
}

/// A bookmark in the plan's list: its kind and title, then its query or its path and heading.
fn bookmark_line(bookmark: &Bookmark) -> String {
    let target = if !bookmark.query.is_empty() {
        bookmark.query.clone()
    } else if bookmark.heading.is_empty() {
        bookmark.path.clone()
    } else {
        format!("{}#{}", bookmark.path, bookmark.heading)
    };
    if target.is_empty() {
        format!("{}: {}", bookmark.kind, bookmark.title)
    } else {
        format!("{}: {} ({target})", bookmark.kind, bookmark.title)
    }
}
