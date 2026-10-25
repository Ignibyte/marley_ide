//! A package-manager command written for this box's family (#918): `dpkg -l | grep libfoo` on an
//! Arch box reads as `pacman -Qs libfoo`.
//!
//! [`Family::from_os_release`] reads the family from `/etc/os-release`'s text (`ID`, then each word
//! of `ID_LIKE`). [`translate`] reads a command of another family's package manager (install,
//! remove, search, list installed with an optional `| grep X`) and writes this family's. Anything
//! else gives none.

/// The package families whose managers the hint knows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    /// pacman.
    Arch,
    /// apt and dpkg.
    Debian,
    /// dnf, yum and rpm.
    Fedora,
    /// zypper.
    OpenSuse,
    /// apk.
    Alpine,
    /// Homebrew.
    MacOs,
}

impl Family {
    /// The family `/etc/os-release`'s `content` names in `ID`, else in `ID_LIKE`.
    #[must_use]
    pub fn from_os_release(content: &str) -> Option<Self> {
        let value = |key: &str| {
            content.lines().find_map(|line| {
                let value = line.strip_prefix(key)?.strip_prefix('=')?;
                Some(value.trim().trim_matches('"').to_ascii_lowercase())
            })
        };
        let id = value("ID").and_then(|id| Self::of_word(&id));
        id.or_else(|| {
            let like = value("ID_LIKE")?;
            like.split_whitespace().find_map(Self::of_word)
        })
    }

    /// The family's name as the hint says it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Arch => "Arch",
            Self::Debian => "Debian",
            Self::Fedora => "Fedora",
            Self::OpenSuse => "openSUSE",
            Self::Alpine => "Alpine",
            Self::MacOs => "macOS",
        }
    }

    fn of_word(word: &str) -> Option<Self> {
        match word {
            "arch" | "archarm" | "manjaro" | "endeavouros" | "cachyos" => Some(Self::Arch),
            "debian" | "ubuntu" | "linuxmint" | "pop" | "raspbian" => Some(Self::Debian),
            "fedora" | "rhel" | "centos" | "rocky" | "almalinux" => Some(Self::Fedora),
            "suse" | "sles" | "opensuse" | "opensuse-leap" | "opensuse-tumbleweed" => {
                Some(Self::OpenSuse)
            }
            "alpine" => Some(Self::Alpine),
            _ => None,
        }
    }

    fn of_program(program: &str) -> Option<Self> {
        match program {
            "pacman" => Some(Self::Arch),
            "apt" | "apt-get" | "dpkg" => Some(Self::Debian),
            "dnf" | "yum" | "rpm" => Some(Self::Fedora),
            "zypper" => Some(Self::OpenSuse),
            "apk" => Some(Self::Alpine),
            "brew" => Some(Self::MacOs),
            _ => None,
        }
    }
}

/// What a package-manager command asks.
#[derive(Debug, PartialEq, Eq)]
enum Verb {
    Install(Vec<String>),
    Remove(Vec<String>),
    Search(String),
    ListInstalled(Option<String>),
}

/// `command` written for `family`'s package manager, when it is another family's install, remove,
/// search or list of installed packages.
#[must_use]
pub fn translate(command: &str, family: Family) -> Option<String> {
    let (from, verb) = read(command)?;
    (from != family).then(|| write(&verb, family))
}

/// The family whose manager `command` runs, and what it asks.
fn read(command: &str) -> Option<(Family, Verb)> {
    let mut parts = command.split('|');
    let words: Vec<&str> = parts
        .next()?
        .split_whitespace()
        .filter(|word| !matches!(*word, "sudo" | "-y" | "--yes" | "-q" | "--noconfirm"))
        .collect();
    // A `| grep X` filter is the list's.
    let filter = parts.next().and_then(|part| {
        let mut words = part.split_whitespace();
        (words.next() == Some("grep")).then(|| words.last().map(str::to_string))?
    });
    let (program, rest) = words.split_first()?;
    let family = Family::of_program(program)?;
    let names = |from: usize| -> Vec<String> {
        rest.iter()
            .skip(from)
            .filter(|word| !word.starts_with('-'))
            .map(|word| (*word).to_string())
            .collect()
    };
    let first = |from: usize| names(from).into_iter().next();
    let verb = match (*program, rest.first().copied()) {
        ("apt" | "apt-get" | "dnf" | "yum" | "zypper" | "brew", Some("install" | "in"))
        | ("apk", Some("add"))
        | ("pacman", Some("-S")) => Verb::Install(names(1)),
        (
            "apt" | "apt-get" | "dnf" | "yum" | "zypper" | "brew",
            Some("remove" | "purge" | "erase" | "rm" | "uninstall"),
        )
        | ("apk", Some("del"))
        | ("pacman", Some("-R" | "-Rs" | "-Rns")) => Verb::Remove(names(1)),
        // zypper lists installed packages with a search's `-i`.
        ("zypper", Some("search" | "se"))
        | ("dpkg", Some("-l"))
        | ("pacman", Some("-Qs" | "-Q"))
            if *program != "zypper" || rest.contains(&"-i") =>
        {
            Verb::ListInstalled(first(1).or(filter))
        }
        ("apt" | "apt-get" | "dnf" | "yum" | "zypper" | "brew" | "apk", Some("search" | "se"))
        | ("pacman", Some("-Ss")) => Verb::Search(first(1)?),
        ("apt" | "dnf" | "yum" | "brew", Some("list"))
        | ("rpm", Some("-qa"))
        | ("apk", Some("info"))
            if !matches!(*program, "apt" | "dnf" | "yum")
                || rest
                    .iter()
                    .any(|word| matches!(*word, "--installed" | "installed")) =>
        {
            Verb::ListInstalled(filter)
        }
        _ => return None,
    };
    match &verb {
        Verb::Install(names) | Verb::Remove(names) if names.is_empty() => None,
        _ => Some((family, verb)),
    }
}

/// `verb` as `family`'s manager writes it; install and remove with `sudo` but on a Mac.
fn write(verb: &Verb, family: Family) -> String {
    let sudo = if family == Family::MacOs { "" } else { "sudo " };
    match verb {
        Verb::Install(names) => {
            let names = names.join(" ");
            let program = match family {
                Family::Arch => "pacman -S",
                Family::Debian => "apt install",
                Family::Fedora => "dnf install",
                Family::OpenSuse => "zypper install",
                Family::Alpine => "apk add",
                Family::MacOs => "brew install",
            };
            format!("{sudo}{program} {names}")
        }
        Verb::Remove(names) => {
            let names = names.join(" ");
            let program = match family {
                Family::Arch => "pacman -R",
                Family::Debian => "apt remove",
                Family::Fedora => "dnf remove",
                Family::OpenSuse => "zypper remove",
                Family::Alpine => "apk del",
                Family::MacOs => "brew uninstall",
            };
            format!("{sudo}{program} {names}")
        }
        Verb::Search(name) => {
            let program = match family {
                Family::Arch => "pacman -Ss",
                Family::Debian => "apt search",
                Family::Fedora => "dnf search",
                Family::OpenSuse => "zypper search",
                Family::Alpine => "apk search",
                Family::MacOs => "brew search",
            };
            format!("{program} {name}")
        }
        Verb::ListInstalled(Some(name)) => match family {
            Family::Arch => format!("pacman -Qs {name}"),
            Family::Debian => format!("apt list --installed | grep {name}"),
            Family::Fedora => format!("dnf list installed | grep {name}"),
            Family::OpenSuse => format!("zypper search -i {name}"),
            Family::Alpine => format!("apk info | grep {name}"),
            Family::MacOs => format!("brew list | grep {name}"),
        },
        Verb::ListInstalled(None) => match family {
            Family::Arch => "pacman -Q",
            Family::Debian => "apt list --installed",
            Family::Fedora => "dnf list installed",
            Family::OpenSuse => "zypper search -i",
            Family::Alpine => "apk info",
            Family::MacOs => "brew list",
        }
        .to_string(),
    }
}
