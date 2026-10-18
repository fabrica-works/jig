//! Jig addresses as typed places, never platform address text.
//! Navigation is owned by the client; see client/domain/client.md, 5.
/// The section of a project at an address.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProjectSection {
    /// Project overview.
    Overview,
    /// Goal board.
    Goals,
    /// Project tasks.
    Tasks,
    /// Project notes.
    Notes,
    /// Policy, resources, roles, spend and recurring settings.
    Settings,
    /// Adopted resources.
    Resources,
    /// Project policy.
    Policy,
    /// Party roles and services.
    Roles,
    /// Period spend.
    Spend,
    /// Recurring tasks.
    Recurring,
}

/// The section of a task at an address.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TaskSection {
    /// Task overview.
    Overview,
    /// Conversation or executor activity.
    Activity,
    /// Live and ended delegates.
    Plan,
    /// Task history.
    History,
    /// Budget and authority.
    Budget,
    /// Connectors with something to say about the task.
    Panels,
}

/// A page owned by jig.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Address {
    /// The inbox, optionally filtered.
    Inbox { project: Option<u32> },
    /// Conversations, optionally filtered.
    Conversations { project: Option<u32> },
    /// One task section.
    Task { task: u64, section: TaskSection },
    /// One project section.
    Project { project: u32, section: ProjectSection },
    /// Deployment capacity.
    Deployment,
}

/// Whether the root address belongs to jig.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Place {
    /// A jig page.
    Jig(Address),
    /// An application page.
    Elsewhere,
    /// No page at this place.
    Missing,
}

impl Address {
    /// Every task chip links by its never-reused task number.
    #[must_use]
    pub const fn task(task: u64) -> Self {
        Self::Task { task, section: TaskSection::Overview }
    }
}

use alloc::boxed::Box;
use skein_lib::{Decimal, Writer};

/// The application's path segments (protocol/browser.md, 9).
/// Each name is one nonempty segment of lowercase letters, digits and hyphens.
#[derive(Debug)]
pub struct Names<'a> {
    pub inbox: &'a [u8],
    pub conversations: &'a [u8],
    pub tasks: &'a [u8],
    pub projects: &'a [u8],
    pub deployment: &'a [u8],
    /// Activity, plan, history, budget and panels, in that order.
    pub task_sections: [&'a [u8]; 5],
    /// Goals through recurring, in `ProjectSection` order.
    pub project_sections: [&'a [u8]; 9],
}

/// A path's prefix and segment names (protocol/browser.md, 9).
#[derive(Debug)]
pub struct Codec<'a> {
    /// Empty at the root; otherwise starts with `/` and has no trailing `/`.
    pub prefix: &'a [u8],
    pub names: Names<'a>,
}

/// The reference UI's paths, at the root.
pub const REFERENCE: Codec<'static> = Codec {
    prefix: b"",
    names: Names {
        inbox: b"inbox",
        conversations: b"conversations",
        tasks: b"tasks",
        projects: b"projects",
        deployment: b"deployment",
        task_sections: [b"activity", b"plan", b"history", b"budget", b"panels"],
        project_sections: [
            b"goals",
            b"tasks",
            b"notes",
            b"settings",
            b"resources",
            b"policy",
            b"roles",
            b"spend",
            b"recurring",
        ],
    },
};

const TASK_SECTIONS: [TaskSection; 6] = [
    TaskSection::Overview,
    TaskSection::Activity,
    TaskSection::Plan,
    TaskSection::History,
    TaskSection::Budget,
    TaskSection::Panels,
];
const PROJECT_SECTIONS: [ProjectSection; 10] = [
    ProjectSection::Overview,
    ProjectSection::Goals,
    ProjectSection::Tasks,
    ProjectSection::Notes,
    ProjectSection::Settings,
    ProjectSection::Resources,
    ProjectSection::Policy,
    ProjectSection::Roles,
    ProjectSection::Spend,
    ProjectSection::Recurring,
];

/// Print an address under the application's prefix.
#[must_use]
pub fn print(codec: &Codec<'_>, address: Address) -> Box<[u8]> {
    let names = &codec.names;
    let (name, number, section) = match address {
        Address::Inbox { project } => match project {
            Some(project) => (names.inbox, Some(u64::from(project)), b"".as_slice()),
            None => (names.inbox, None, b"".as_slice()),
        },
        Address::Conversations { project } => match project {
            Some(project) => (names.conversations, Some(u64::from(project)), b"".as_slice()),
            None => (names.conversations, None, b"".as_slice()),
        },
        Address::Task { task, section } => {
            let index = match section {
                TaskSection::Overview => None,
                TaskSection::Activity => Some(0),
                TaskSection::Plan => Some(1),
                TaskSection::History => Some(2),
                TaskSection::Budget => Some(3),
                TaskSection::Panels => Some(4),
            };
            (names.tasks, Some(task), section_name(index, &names.task_sections))
        }
        Address::Project { project, section } => {
            let index = match section {
                ProjectSection::Overview => None,
                ProjectSection::Goals => Some(0),
                ProjectSection::Tasks => Some(1),
                ProjectSection::Notes => Some(2),
                ProjectSection::Settings => Some(3),
                ProjectSection::Resources => Some(4),
                ProjectSection::Policy => Some(5),
                ProjectSection::Roles => Some(6),
                ProjectSection::Spend => Some(7),
                ProjectSection::Recurring => Some(8),
            };
            (names.projects, Some(u64::from(project)), section_name(index, &names.project_sections))
        }
        Address::Deployment => (names.deployment, None, b"".as_slice()),
    };
    let digits = Decimal::of(number.unwrap_or(0));
    let size = codec.prefix.len().checked_add(name.len()).expect("codec path fits usize");
    let mut size = size.checked_add(1).expect("codec path fits usize");
    if number.is_some() {
        size = size.checked_add(digits.as_bytes().len()).expect("codec path fits usize");
        size = size.checked_add(1).expect("codec path fits usize");
    }
    if !section.is_empty() {
        size = size.checked_add(section.len()).expect("codec path fits usize");
        size = size.checked_add(1).expect("codec path fits usize");
    }
    let mut path = Writer::new(size);
    path.put(codec.prefix).expect("prefix fits");
    path.put(b"/").expect("separator fits");
    path.put(name).expect("name fits");
    if number.is_some() {
        path.put(b"/").expect("separator fits");
        path.put(digits.as_bytes()).expect("number fits");
    }
    if !section.is_empty() {
        path.put(b"/").expect("separator fits");
        path.put(section).expect("section fits");
    }
    path.finish()
}

/// Parse a path, ignoring its query and one trailing slash.
#[must_use]
pub fn parse(codec: &Codec<'_>, path: &[u8]) -> Place {
    let query = skein_lib::bytes::find(path, b"?").unwrap_or(path.len());
    let path = path.get(..query).expect("query in path");
    let Some(path) = path.strip_prefix(codec.prefix) else {
        return Place::Elsewhere;
    };
    if !path.is_empty() && !path.starts_with(b"/") {
        return Place::Elsewhere;
    }
    let path = path.strip_suffix(b"/").unwrap_or(path);
    if path.is_empty() {
        return Place::Jig(Address::Inbox { project: None });
    }
    if path.ends_with(b"/") {
        return Place::Missing;
    }
    let Some(path) = path.strip_prefix(b"/") else {
        return Place::Missing;
    };
    let (name, rest) = segment(path);
    let (number, tail) = segment(rest);
    let names = &codec.names;
    let address = if name == names.inbox || name == names.conversations {
        if !tail.is_empty() {
            return Place::Missing;
        }
        let project = if number.is_empty() {
            None
        } else {
            let Some(project) = project_decimal(number) else {
                return Place::Missing;
            };
            Some(project)
        };
        if name == names.inbox { Address::Inbox { project } } else { Address::Conversations { project } }
    } else if name == names.tasks {
        let Some(task) = decimal(number) else {
            return Place::Missing;
        };
        let Some(section) = section_index(tail, &names.task_sections) else {
            return Place::Missing;
        };
        Address::Task { task, section: *TASK_SECTIONS.get(section).expect("task section") }
    } else if name == names.projects {
        let Some(project) = project_decimal(number) else {
            return Place::Missing;
        };
        let Some(section) = section_index(tail, &names.project_sections) else {
            return Place::Missing;
        };
        Address::Project { project, section: *PROJECT_SECTIONS.get(section).expect("project section") }
    } else if name == names.deployment && rest.is_empty() {
        Address::Deployment
    } else {
        return Place::Missing;
    };
    Place::Jig(address)
}

fn segment(path: &[u8]) -> (&[u8], &[u8]) {
    let split = skein_lib::bytes::find(path, b"/").unwrap_or(path.len());
    let first = path.get(..split).expect("separator in path");
    let rest = path.get(split..).expect("separator in path").strip_prefix(b"/").unwrap_or(b"");
    (first, rest)
}

fn decimal(bytes: &[u8]) -> Option<u64> {
    if bytes.is_empty() || bytes.len() > 20 || (bytes.len() > 1 && bytes.starts_with(b"0")) {
        return None;
    }
    let mut value = 0_u64;
    for byte in bytes {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
    }
    Some(value)
}

fn section_index(section: &[u8], names: &[&[u8]]) -> Option<usize> {
    if section.is_empty() {
        return Some(0);
    }
    for (index, name) in names.iter().enumerate() {
        if section == *name {
            return index.checked_add(1);
        }
    }
    None
}

fn project_decimal(bytes: &[u8]) -> Option<u32> {
    let number = decimal(bytes)?;
    u32::try_from(number).ok()
}

fn section_name<'a>(index: Option<usize>, names: &[&'a [u8]]) -> &'a [u8] {
    match index {
        Some(index) => names.get(index).expect("section name"),
        None => b"",
    }
}
