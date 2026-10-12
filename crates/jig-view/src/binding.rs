//! Owner tokens carried unchanged by the tree (client/domain/view.md, section 5).

use crate::Owner;
use alloc::boxed::Box;
use skein_lib::Token;

/// A meaning kept by its owner's view for the last built tree.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Binding {
    pub owner: Owner,
    pub token: Token,
}

/// The shell's act, with a field's whole value owned by the event.
#[derive(PartialEq, Eq, Debug)]
pub enum Act {
    /// Press a button or follow a link.
    Press,
    /// Replace a field's typed value.
    Input { value: Box<[u8]> },
    /// Submit a form or composer.
    Submit,
    /// Toggle a choice or check.
    Toggle,
}

/// An act resolved to the owner that decodes it into a typed domain action.
#[derive(PartialEq, Eq, Debug)]
pub struct BoundAct {
    pub binding: Binding,
    pub act: Act,
}

/// A field's ASCII digits as a count; empty, non-digits and overflow are absent.
#[must_use]
pub fn number(bytes: &[u8]) -> Option<u64> {
    if bytes.is_empty() {
        return None;
    }
    let mut value = 0_u64;
    for byte in bytes {
        if !byte.is_ascii_digit() {
            return None;
        }
        let digit = byte.checked_sub(b'0')?;
        value = value.checked_mul(10)?.checked_add(u64::from(digit))?;
    }
    Some(value)
}

/// A typed meaning belonging to jig's last built tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JigMeaning {
    /// One typed Project settings field, preserving other whole channels.
    ProjectSetting { ask: Box<jig_domain_client::contract::Ask>, field: crate::owner_project::Field },
    /// A new service name, after an explicit supplied role choice.
    ProjectServiceName { project: u32, role: jig_domain_client::contract::HeldRole },
    /// An explicit budget for one supplied party in the configured target.
    ProjectPoolBudget { project: u32, party: u64 },
    /// Pressing, submitting or toggling asks this already typed action.
    Action(jig_domain_client::action::Act),
    /// Whole text edits a domain field; submit names that same field.
    Field(jig_domain_client::fields::Purpose),
    /// Digits are kept in the view and submitting goes to that task.
    TaskNumber,
    /// Digits edit the confirmation's priority without a hidden submit.
    Priority,
    /// Source values and selected replacements kept separately for one named field.
    Amendment {
        before: Box<jig_domain_client::contract::Amendment>,
        change: Box<jig_domain_client::contract::Amendment>,
        field: crate::confirmation::amendment::Field,
    },
    /// A typed project goal form field.
    GoalField { form: jig_domain_client::pages::project::goals::GoalForm, field: crate::pages::project::goals::Field },
    /// Description search changes the current note scope only.
    NoteSearch { filter: jig_domain_client::pages::project::NoteFilter },
    /// A connector-owned part referred to by a slot, never an actionable node.
    Part(jig_domain_client::contract::Part),
    /// A whole exact owner reference, with literal path segment boundaries.
    OwnerPart(jig_domain_client::contract::OwnerPart),
}
#[derive(Debug)]
struct Entry {
    token: Token,
    meaning: JigMeaning,
}
/// Bounded meanings for one tree; rebuilding retires every old token.
/// Empty + begin -> building; insert -> issued; decode -> typed action or drop.
/// The next token never resets, so a token from any earlier tree stays stale.
#[derive(Debug)]
pub struct JigBindings {
    entries: skein_lib::List<Entry>,
    next: u64,
    text_limit: u32,
    task: Option<u64>,
}
impl JigBindings {
    /// Allocate the most meanings that a tree may carry.
    #[must_use]
    pub fn new(capacity: u32, text_limit: u32) -> Self {
        Self { entries: skein_lib::List::with_capacity(capacity), next: 0, text_limit, task: None }
    }
    /// Retire the prior tree's meanings before building its replacement.
    pub fn begin(&mut self) {
        self.entries.clear();
    }
    /// Issue one token; exceeding the derived capacity is a view bug.
    pub fn insert(&mut self, meaning: JigMeaning) -> Binding {
        self.next = self.next.checked_add(1).expect("view token space remains");
        let token = Token::new(self.next);
        self.entries.push(Entry { token, meaning }).expect("bindings fit their derived capacity");
        Binding { owner: Owner::Jig, token }
    }
    /// Resolve a part slot's token for its connector's view root.
    #[must_use]
    pub fn part(&self, token: Token) -> Option<&jig_domain_client::contract::Part> {
        for entry in &self.entries {
            if entry.token == token {
                match &entry.meaning {
                    JigMeaning::Part(part) => return Some(part),
                    JigMeaning::OwnerPart(_)
                    | JigMeaning::Action(_)
                    | JigMeaning::Field(_)
                    | JigMeaning::TaskNumber
                    | JigMeaning::Priority
                    | JigMeaning::Amendment { .. }
                    | JigMeaning::ProjectSetting { .. }
                    | JigMeaning::ProjectServiceName { .. }
                    | JigMeaning::ProjectPoolBudget { .. }
                    | JigMeaning::GoalField { .. }
                    | JigMeaning::NoteSearch { .. } => return None,
                }
            }
        }
        None
    }
    /// Resolve an exact reference slot's raw key; scope is supplied by its producer.
    #[must_use]
    pub fn owner_part(&self, token: Token) -> Option<&jig_domain_client::contract::OwnerPart> {
        for entry in &self.entries {
            if entry.token == token {
                match &entry.meaning {
                    JigMeaning::OwnerPart(part) => return Some(part),
                    JigMeaning::Part(_)
                    | JigMeaning::Action(_)
                    | JigMeaning::Field(_)
                    | JigMeaning::TaskNumber
                    | JigMeaning::Priority
                    | JigMeaning::Amendment { .. }
                    | JigMeaning::ProjectSetting { .. }
                    | JigMeaning::ProjectServiceName { .. }
                    | JigMeaning::ProjectPoolBudget { .. }
                    | JigMeaning::GoalField { .. }
                    | JigMeaning::NoteSearch { .. } => return None,
                }
            }
        }
        None
    }
    /// A link's typed address for the platform's address translation.
    #[must_use]
    pub fn address(&self, token: Token) -> Option<jig_domain_client::address::Address> {
        for entry in &self.entries {
            if entry.token == token {
                match &entry.meaning {
                    JigMeaning::Action(action) => match action {
                        jig_domain_client::action::Act::Go { address } => return Some(*address),
                        jig_domain_client::action::Act::OpenChat { task } => {
                            return Some(jig_domain_client::address::Address::Task {
                                task: *task,
                                section: jig_domain_client::address::TaskSection::Activity,
                            });
                        }
                        jig_domain_client::action::Act::Edit { .. }
                        | jig_domain_client::action::Act::Submit { .. }
                        | jig_domain_client::action::Act::Intend { .. }
                        | jig_domain_client::action::Act::Choose { .. }
                        | jig_domain_client::action::Act::CannotDo
                        | jig_domain_client::action::Act::Again { .. }
                        | jig_domain_client::action::Act::Dispose { .. }
                        | jig_domain_client::action::Act::Change { .. }
                        | jig_domain_client::action::Act::WriteRelease { .. }
                        | jig_domain_client::action::Act::Priority { .. }
                        | jig_domain_client::action::Act::ChatAbout { .. }
                        | jig_domain_client::action::Act::LookAgain { .. }
                        | jig_domain_client::action::Act::Confirm
                        | jig_domain_client::action::Act::Dismiss
                        | jig_domain_client::action::Act::Fold { .. }
                        | jig_domain_client::action::Act::FoldCall { .. }
                        | jig_domain_client::action::Act::FoldCurrentCall { .. }
                        | jig_domain_client::action::Act::Filter { .. }
                        | jig_domain_client::action::Act::More { .. }
                        | jig_domain_client::action::Act::FilterInbox { .. }
                        | jig_domain_client::action::Act::OpenUpdates
                        | jig_domain_client::action::Act::StartChat { .. }
                        | jig_domain_client::action::Act::ProjectSettings { .. }
                        | jig_domain_client::action::Act::ProjectServiceRole { .. }
                        | jig_domain_client::action::Act::GoalForm { .. }
                        | jig_domain_client::action::Act::FilterTasks { .. }
                        | jig_domain_client::action::Act::FilterNotes { .. }
                        | jig_domain_client::action::Act::FilterPlan { .. }
                        | jig_domain_client::action::Act::HistoryScope { .. }
                        | jig_domain_client::action::Act::OpenEnded { .. }
                        | jig_domain_client::action::Act::SignOut
                        | jig_domain_client::action::Act::SignIn { .. } => return None,
                    },
                    JigMeaning::Field(_)
                    | JigMeaning::TaskNumber
                    | JigMeaning::Part(_)
                    | JigMeaning::OwnerPart(_)
                    | JigMeaning::Priority
                    | JigMeaning::Amendment { .. }
                    | JigMeaning::ProjectSetting { .. }
                    | JigMeaning::ProjectServiceName { .. }
                    | JigMeaning::ProjectPoolBudget { .. }
                    | JigMeaning::GoalField { .. }
                    | JigMeaning::NoteSearch { .. } => return None,
                }
            }
        }
        None
    }
}
/// Decode an act only through a token issued for the last tree.
/// Numeric input is retained here until submit; no navigation happens while typing.
#[must_use]
pub fn decode(view: &mut JigBindings, token: Token, act: Act) -> Option<jig_domain_client::action::Act> {
    for entry in &view.entries {
        if entry.token == token {
            return match &entry.meaning {
                JigMeaning::Part(_) | JigMeaning::OwnerPart(_) => None,
                JigMeaning::ProjectPoolBudget { project, party } => match act {
                    Act::Input { value } => Some(jig_domain_client::pages::project::owner_form::action(
                        jig_domain_client::contract::Ask::SetPool {
                            project: *project,
                            party: *party,
                            budget: number(&value)?,
                        },
                    )),
                    Act::Press | Act::Submit | Act::Toggle => None,
                },
                JigMeaning::ProjectServiceName { project, role } => match act {
                    Act::Input { value } => Some(jig_domain_client::pages::project::owner_form::action(
                        jig_domain_client::contract::Ask::MakeService {
                            project: *project,
                            role: *role,
                            name: jig_domain_client::contract::Text::new(value, view.text_limit).ok()?,
                        },
                    )),
                    Act::Press | Act::Submit | Act::Toggle => None,
                },
                JigMeaning::ProjectSetting { ask, field } => match act {
                    Act::Input { value } => crate::owner_project::edit(ask, *field, value, view.text_limit),
                    Act::Press | Act::Submit | Act::Toggle => None,
                },
                JigMeaning::GoalField { form, field } => match act {
                    Act::Input { value } => crate::pages::project::goals::edit(*form, *field, &value),
                    Act::Press | Act::Submit | Act::Toggle => None,
                },
                JigMeaning::NoteSearch { filter } => match act {
                    Act::Input { value } => Some(jig_domain_client::action::Act::FilterNotes {
                        filter: jig_domain_client::pages::project::NoteFilter {
                            scope: filter.scope.clone(),
                            description: if value.is_empty() {
                                None
                            } else {
                                Some(jig_domain_client::contract::Text::new(value, view.text_limit).ok()?)
                            },
                        },
                    }),
                    Act::Press | Act::Submit | Act::Toggle => None,
                },
                JigMeaning::Priority => match act {
                    Act::Input { value } => Some(jig_domain_client::action::Act::Priority {
                        priority: u32::try_from(number(&value)?).ok()?,
                    }),
                    Act::Press | Act::Submit | Act::Toggle => None,
                },
                JigMeaning::Amendment { before, change, field } => match act {
                    Act::Input { value } => {
                        crate::confirmation::amendment::edit(before, change, *field, value, view.text_limit)
                    }
                    Act::Press | Act::Submit | Act::Toggle => None,
                },
                JigMeaning::Action(action) => match act {
                    Act::Press | Act::Submit | Act::Toggle => Some(action.clone()),
                    Act::Input { .. } => None,
                },
                JigMeaning::Field(field) => match act {
                    Act::Input { value } => {
                        let text = jig_domain_client::contract::Text::new(value, view.text_limit).ok()?;
                        Some(jig_domain_client::action::Act::Edit { field: *field, text })
                    }
                    Act::Submit => Some(jig_domain_client::action::Act::Submit { field: *field }),
                    Act::Press | Act::Toggle => None,
                },
                JigMeaning::TaskNumber => match act {
                    Act::Input { value } => {
                        view.task = number(&value);
                        None
                    }
                    Act::Submit => Some(jig_domain_client::action::Act::Go {
                        address: jig_domain_client::address::Address::task(view.task?),
                    }),
                    Act::Press | Act::Toggle => None,
                },
            };
        }
    }
    None
}

/// Owned heap for the last tree's jig meanings, separately from tree storage.
/// Amendment controls own both source values and selected replacements, each
/// under the bounded document/text envelope. Action text and part paths also fit.
#[must_use]
pub fn jig_bindings_worst_case(capacity: u32, client: &jig_domain_client::Limits) -> Option<u64> {
    let payload = client.document_bytes.checked_mul(4)?.checked_add(u64::from(client.text).checked_mul(8)?)?;
    skein_lib::List::<Entry>::worst_case(capacity)?.checked_add(u64::from(capacity).checked_mul(payload)?)
}
