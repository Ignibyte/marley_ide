//! Whether an agent's click has a consequence the user should allow first (#571): it pays,
//! deletes, sends in the user's name or changes an account.
//!
//! Code decides first, from the element, its form and the page's URL: a name in a consequential
//! list, a form or a link whose target says so, or a plain element. What neither settles, an
//! open name such as "Continue" among text about a charge, is `Open`, which the System One layer
//! may read; a reading can add a pause and never remove one. Pure.

/// What code knows of an element an agent is about to click.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClickFacts {
    /// Its accessibility role, such as `button` or `link`.
    pub role: String,
    /// Its accessible name, such as `Place order`.
    pub name: String,
    /// Its tag, lowercased.
    pub tag: String,
    /// An input's `type`, lowercased.
    pub input_type: Option<String>,
    /// Whether it sits in a form, or submits one.
    pub in_form: bool,
    /// Where its form posts: the button's `formaction` or the form's `action`, resolved.
    pub form_action: String,
    /// How its form posts, lowercased: `get`, `post`, or a framework's `delete` from its
    /// `_method` field.
    pub form_method: String,
    /// Whether its form holds a text area.
    pub form_has_text_area: bool,
    /// The names of its form's fields, lowercased.
    pub form_fields: Vec<String>,
    /// A link's target, resolved against the page.
    pub link: Option<String>,
    /// The text of the form, dialog or section around it.
    pub context: String,
    /// The page's URL.
    pub page_url: String,
}

/// What a click does, as code reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Class {
    /// It spends money.
    Pays,
    /// It deletes or removes something for good.
    Deletes,
    /// It sends or publishes something in the user's name.
    Sends,
    /// It changes the user's account or who can use it.
    ChangesAccount,
    /// Code cannot tell, and the page speaks of something consequential: the layer may read it.
    Open,
    /// It has no consequence to allow first.
    Plain,
}

impl Class {
    /// Whether the click waits for the user on code's word alone.
    #[must_use]
    pub const fn pauses(self) -> bool {
        matches!(
            self,
            Self::Pays | Self::Deletes | Self::Sends | Self::ChangesAccount
        )
    }

    /// The class in words, for the card and the log: `pays`, `deletes`, …
    #[must_use]
    pub const fn words(self) -> &'static str {
        match self {
            Self::Pays => "pays",
            Self::Deletes => "deletes",
            Self::Sends => "sends in your name",
            Self::ChangesAccount => "changes your account",
            Self::Open => "may have a consequence",
            Self::Plain => "has no consequence",
        }
    }

    /// The layer's noul that asks about the class; `Open` and `Plain` have none.
    #[must_use]
    pub const fn noul(self) -> Option<&'static str> {
        match self {
            Self::Pays => Some("pays"),
            Self::Deletes => Some("deletes"),
            Self::Sends => Some("sends"),
            Self::ChangesAccount => Some("changes_account"),
            Self::Open | Self::Plain => None,
        }
    }

    /// The class a layer's noul names.
    #[must_use]
    pub fn from_noul(key: &str) -> Option<Self> {
        match key {
            "pays" => Some(Self::Pays),
            "deletes" => Some(Self::Deletes),
            "sends" => Some(Self::Sends),
            "changes_account" => Some(Self::ChangesAccount),
            _ => None,
        }
    }
}

/// What code decided and why, in words: `its name`, `its form posts to /checkout`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    /// The class.
    pub class: Class,
    /// What the class rests on.
    pub because: String,
}

/// Roles whose click chooses or types rather than commits.
const PLAIN_ROLES: [&str; 13] = [
    "textbox",
    "searchbox",
    "combobox",
    "checkbox",
    "radio",
    "switch",
    "tab",
    "option",
    "slider",
    "spinbutton",
    "menuitemcheckbox",
    "menuitemradio",
    "treeitem",
];

/// Names that spend money.
const PAYS: [&str; 17] = [
    "pay",
    "pay now",
    "buy",
    "buy now",
    "purchase",
    "checkout",
    "check out",
    "place order",
    "order now",
    "subscribe",
    "upgrade",
    "renew",
    "confirm payment",
    "charge",
    "donate",
    "tip",
    "complete purchase",
];

/// Names that delete or remove.
const DELETES: [&str; 13] = [
    "delete",
    "remove",
    "destroy",
    "erase",
    "discard",
    "drop",
    "purge",
    "deactivate",
    "close account",
    "cancel subscription",
    "unsubscribe",
    "revoke",
    "wipe",
];

/// Names that send or publish in the user's name.
const SENDS: [&str; 9] = [
    "send", "post", "publish", "reply", "tweet", "share", "email", "invite", "comment",
];

/// Names that change an account or who can use it.
const CHANGES_ACCOUNT: [&str; 18] = [
    "change password",
    "change email",
    "update email",
    "enable two factor",
    "disable two factor",
    "add key",
    "create token",
    "generate token",
    "api key",
    "transfer ownership",
    "make admin",
    "sign out everywhere",
    "log out all",
    "link account",
    "connect account",
    "grant",
    "authorize",
    "reset password",
];

/// Names that move about, choose or look, and commit nothing.
const PLAIN_NAMES: [&str; 22] = [
    "cancel",
    "close",
    "back",
    "next",
    "previous",
    "skip",
    "learn more",
    "menu",
    "dismiss",
    "not now",
    "later",
    "no thanks",
    "show more",
    "show less",
    "expand",
    "collapse",
    "more",
    "help",
    "search",
    "filter",
    "sort",
    "details",
];

/// Names that commit to whatever the page is about.
const OPEN_NAMES: [&str; 15] = [
    "continue", "confirm", "ok", "okay", "yes", "submit", "done", "proceed", "apply", "save",
    "accept", "agree", "finish", "complete", "go",
];

/// Words in a page's path, a form's action or a link's target, by the class they make.
const PATHS: [(Class, &[&str]); 4] = [
    (
        Class::Pays,
        &[
            "checkout",
            "billing",
            "payment",
            "payments",
            "pay",
            "subscribe",
            "purchase",
        ],
    ),
    (Class::Deletes, &["delete", "remove", "destroy"]),
    (Class::Sends, &["compose", "send", "publish", "post"]),
    (
        Class::ChangesAccount,
        &[
            "password",
            "security",
            "tokens",
            "keys",
            "members",
            "oauth",
            "authorize",
        ],
    ),
];

/// Words of the text around an element that make an open name worth reading.
const CONTEXT_WORDS: [&str; 16] = [
    "charge",
    "charged",
    "payment",
    "billing",
    "card",
    "permanently",
    "cannot be undone",
    "can t be undone",
    "irreversible",
    "will be sent",
    "recipients",
    "password",
    "two factor",
    "delete",
    "subscription",
    "account",
];

/// The currency signs that make the text around an element speak of money.
const CURRENCY: [char; 4] = ['$', '€', '£', '¥'];

/// Fields of a form that make its submission a message.
const MESSAGE_FIELDS: [&str; 6] = ["message", "comment", "body", "subject", "to", "reply"];

/// Decides what a click on the element `facts` describe does.
///
/// First match wins: a role that only chooses; a consequential name; a plain name; a form that
/// deletes, or a form's or a link's target that says what it does; an open name, or a name in no
/// list, among consequential text; else plain.
#[must_use]
pub fn classify(facts: &ClickFacts) -> Verdict {
    let verdict = |class, because: &str| Verdict {
        class,
        because: because.to_string(),
    };
    let role = facts.role.to_lowercase();
    if PLAIN_ROLES.contains(&role.as_str())
        || matches!(
            facts.input_type.as_deref(),
            Some("checkbox" | "radio" | "text" | "search" | "email" | "number" | "password")
        )
    {
        return verdict(Class::Plain, "it chooses or types");
    }
    let name = words(&facts.name);
    for (class, list) in [
        (Class::Pays, &PAYS[..]),
        (Class::Deletes, &DELETES[..]),
        (Class::ChangesAccount, &CHANGES_ACCOUNT[..]),
        (Class::Sends, &SENDS[..]),
    ] {
        if holds_any(&name, list) {
            return verdict(class, "its name");
        }
    }
    if holds_any(&name, &PLAIN_NAMES) {
        return verdict(Class::Plain, "its name");
    }
    if submits(facts) {
        if facts.form_method == "delete" {
            return verdict(Class::Deletes, "its form deletes");
        }
        let action = path_of(&facts.form_action);
        if let Some(class) = path_class(&action) {
            return verdict(class, &format!("its form posts to {action}"));
        }
        let message = facts.form_has_text_area
            || facts
                .form_fields
                .iter()
                .any(|field| MESSAGE_FIELDS.contains(&field.as_str()));
        if message && facts.form_method != "get" {
            return verdict(Class::Sends, "its form sends a message");
        }
    }
    if let Some(link) = &facts.link {
        let target = path_of(link);
        if let Some(class) = path_class(&target) {
            return verdict(class, &format!("its link goes to {target}"));
        }
    }
    // A link commits only under a name such as "Continue"; any other element under any name
    // left, since its script may do anything.
    let open_name = holds_any(&name, &OPEN_NAMES) || facts.link.is_none();
    if open_name && consequential_context(facts) {
        return verdict(Class::Open, "the page around it");
    }
    verdict(Class::Plain, "nothing about it")
}

/// Whether the element submits its form: a button in it, or a submit or image input.
fn submits(facts: &ClickFacts) -> bool {
    facts.in_form
        && match facts.tag.as_str() {
            "button" => !matches!(facts.input_type.as_deref(), Some("button" | "reset")),
            "input" => matches!(facts.input_type.as_deref(), Some("submit" | "image")),
            _ => facts.role.eq_ignore_ascii_case("button"),
        }
}

/// Whether the page's path or the text around the element speaks of something consequential.
fn consequential_context(facts: &ClickFacts) -> bool {
    path_class(&path_of(&facts.page_url)).is_some()
        || facts.context.contains(CURRENCY)
        || holds_any(&words(&facts.context), &CONTEXT_WORDS)
}

/// The class a path's words make, if any.
fn path_class(path: &str) -> Option<Class> {
    let segments = words(path);
    PATHS
        .iter()
        .find(|(_, list)| holds_any(&segments, list))
        .map(|(class, _)| *class)
}

/// The path of `url`, an absolute URL or a path, without its query or fragment.
fn path_of(url: &str) -> String {
    let without_scheme = url.split_once("://").map_or(url, |(_, rest)| rest);
    let path = if url.contains("://") {
        without_scheme
            .find('/')
            .map_or("/", |start| &without_scheme[start..])
    } else {
        without_scheme
    };
    path.split(['?', '#']).next().unwrap_or(path).to_string()
}

/// `text` lowercased, as words separated by single spaces with a space before and after, so a
/// phrase matches whole words only.
fn words(text: &str) -> String {
    let joined = text
        .to_lowercase()
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    format!(" {joined} ")
}

/// Whether the `words` of a text hold one of `phrases` as whole words.
fn holds_any(words: &str, phrases: &[&str]) -> bool {
    phrases
        .iter()
        .any(|phrase| words.contains(&format!(" {phrase} ")))
}
