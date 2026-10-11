//! Project documents, board and scoped notes belong to the page generation.
//! Cards remain in the shared object set; connector values stay in root slots.
//! Goal creation has one page confirmation and one keyed intent. See
//! client/domain/pages.md, 6, and client/domain/root.md, 6.
//!
//! | Page | Event | Next | Emits |
//! |---|---|---|---|
//! | loading | project document | current facts | section parts |
//! | board | goals | ordered board | shared task names |
//! | notes | scope/search | reading | scoped notes read |
//! | any | more | reading | opaque page cursor |
//! | any | leave | empty context | parts leave through routing |
use crate::{
    Domain, Request,
    action::End,
    address::{Address, ProjectSection},
    contract::{self, Cursor, GoalLine, NoteScope, Part, Project, Read, TaskFilter, TaskLine, Text},
    objects::{Object, Source},
    watches::Owner,
};
use alloc::boxed::Box;
use skein_lib::{Env, Id, List, Queue, Token};
pub mod goals;
pub mod notes;
pub mod owner_form;
mod owner_goals;
/// Selected scope and description search, passed to the notes owner unchanged.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NoteFilter {
    pub scope: NoteScope,
    pub description: Option<Text>,
}
/// Only current project context; pending goal requests survive leaving the page.
#[derive(Debug)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent source loss flags for three read windows and live continuation flags"
)]
pub struct ProjectPage {
    pub document: Option<Box<Project>>,
    /// Retained historical funding rows, independent of live project spend.
    pub owner_spend: Option<Box<contract::OwnerSpendPage>>,
    /// Whole owner facts, with no fabricated earlier document.
    pub owner: Option<Box<contract::OwnerProject>>,
    pub goals: List<GoalLine>,
    /// The current read window, which may include both live and ended goals.
    pub ended: List<GoalLine>,
    /// Keyed display merge; the board's current row wins over a read row.
    pub listed_goals: List<GoalLine>,
    pub goals_more: bool,
    pub ended_cut: bool,
    pub ended_before: Option<Cursor>,
    pub ended_after: Option<Cursor>,
    pub tasks: List<TaskLine>,
    pub tasks_more: bool,
    pub task_filter: TaskFilter,
    pub notes: List<Id<Object>>,
    pub note_filter: Option<NoteFilter>,
    /// Domain writes of the selected note scope and description.
    pub note_writes: u64,
    pub notes_cut: bool,
    pub notes_before: Option<Cursor>,
    pub notes_after: Option<Cursor>,
    pub spend: List<contract::SpendPeriod>,
    pub spend_cut: bool,
    pub spend_before: Option<Cursor>,
    pub spend_after: Option<Cursor>,
    pub form: goals::FormState,
    pub settings: owner_form::Form,
}
impl ProjectPage {
    pub(crate) fn new(rows: u32) -> Self {
        Self {
            document: None,
            owner_spend: None,
            owner: None,
            goals: List::with_capacity(rows),
            ended: List::with_capacity(rows),
            listed_goals: List::with_capacity(rows.checked_mul(2).expect("goal source rows fit")),
            goals_more: false,
            ended_cut: false,
            ended_before: None,
            ended_after: None,
            tasks: List::with_capacity(rows),
            tasks_more: false,
            task_filter: TaskFilter { phase: None, executor: None, goal: None, asked_by_me: false },
            notes: List::with_capacity(rows),
            note_filter: None,
            note_writes: 0,
            notes_cut: false,
            notes_before: None,
            notes_after: None,
            spend: List::with_capacity(rows),
            spend_cut: false,
            spend_before: None,
            spend_after: None,
            form: goals::FormState::new(),
            settings: owner_form::Form::new(),
        }
    }
    pub(crate) fn clear(&mut self) {
        self.document = None;
        self.owner_spend = None;
        self.owner = None;
        self.goals.clear();
        self.ended.clear();
        self.listed_goals.clear();
        self.goals_more = false;
        self.ended_cut = false;
        self.ended_before = None;
        self.ended_after = None;
        self.tasks.clear();
        self.tasks_more = false;
        self.task_filter = TaskFilter { phase: None, executor: None, goal: None, asked_by_me: false };
        self.notes.clear();
        self.note_filter = None;
        self.note_writes = self.note_writes.checked_add(1).expect("note writes remain");
        self.notes_cut = false;
        self.notes_before = None;
        self.notes_after = None;
        self.spend.clear();
        self.spend_cut = false;
        self.spend_before = None;
        self.spend_after = None;
        self.form.confirmation = None;
        self.settings.review = None;
        self.settings.service_role = None;
        self.settings.pool_period = None;
        self.settings.before = None;
        self.settings.refused = None;
    }
}
/// Number of the currently shown project, never inferred from another page.
#[must_use]
pub(crate) const fn number(domain: &Domain) -> Option<u32> {
    match domain.address {
        Address::Project { project, .. } => Some(project),
        Address::Inbox { .. } | Address::Conversations { .. } | Address::Task { .. } | Address::Deployment => None,
    }
}
/// Begins exactly the selected project's sources.
pub(crate) fn enter(domain: &mut Domain, env: &Env<crate::Limits>, out: &mut Queue<Request>) {
    let (project, section) = match domain.address {
        Address::Project { project, section } => (project, section),
        Address::Inbox { .. } | Address::Conversations { .. } | Address::Task { .. } | Address::Deployment => return,
    };
    match section {
        ProjectSection::Overview => {
            open(domain, env, contract::Subject::Goals { project }, out);
            open(domain, env, contract::Subject::Inbox { window: env.limits.rows }, out);
            domain.inbox.filter = crate::pages::inbox::Filter { project: Some(project), kind: None, goal: None };
        }
        ProjectSection::Goals => {
            open(domain, env, contract::Subject::Goals { project }, out);
            ask(domain, 20, Read::Goals { project, after: None }, out);
        }
        ProjectSection::Tasks => tasks(domain, env, domain.project_page.task_filter, out),
        ProjectSection::Notes => {
            notes::filter(domain, NoteFilter { scope: NoteScope::Project(project), description: None }, out);
        }
        ProjectSection::Spend => ask(domain, 23, Read::Spend { project, after: None }, out),
        ProjectSection::Settings
        | ProjectSection::Resources
        | ProjectSection::Policy
        | ProjectSection::Roles
        | ProjectSection::Recurring => {}
    }
    ask(domain, 0, Read::Project { project }, out);
}
fn open(domain: &mut Domain, env: &Env<crate::Limits>, subject: contract::Subject, out: &mut Queue<Request>) {
    if domain.open(env, subject, Owner::Page { generation: domain.generation }, out).is_none() {
        crate::pages::routing::refuse(domain, contract::Limit::Watches);
    }
}
pub(crate) fn ask(domain: &mut Domain, number: u16, what: Read, out: &mut Queue<Request>) {
    if domain.reads.pending(&what) {
        return;
    }
    if domain
        .reads
        .ask(domain.generation, Source::Window { generation: domain.generation, number }, what, out)
        .is_none()
    {
        crate::pages::routing::refuse(domain, contract::Limit::Reads);
    }
}
/// Replaces the project document and asks only this section's part references.
pub(crate) fn document(
    domain: &mut Domain,
    env: &Env<crate::Limits>,
    mut document: Box<Project>,
    out: &mut Queue<Request>,
) {
    if number(domain) != Some(document.number) {
        return;
    }
    order_spend(&mut document.spend_lines);
    domain.project_page.owner = None;
    domain.project_page.document = Some(document);
    goals::sync(domain);
    let section = match domain.address {
        Address::Project { section, .. } => section,
        Address::Inbox { .. } | Address::Conversations { .. } | Address::Task { .. } | Address::Deployment => return,
    };
    let doc = domain.project_page.document.as_ref().expect("stored above");
    let parts = parts(doc, section, env.limits.parts.saturating_add(1));
    show(domain, env, parts, out);
    domain.changed();
}
/// Resources, requirements and connector panels are facts, never connector values.
#[must_use]
pub fn parts(doc: &Project, section: ProjectSection, limit: u32) -> List<Part> {
    let mut out = List::with_capacity(limit);
    match section {
        ProjectSection::Overview => append(&mut out, &doc.parts),
        ProjectSection::Resources => append(&mut out, &doc.resources),
        ProjectSection::Policy => append(&mut out, &doc.policy.requirements),
        ProjectSection::Settings => {
            append(&mut out, &doc.parts);
            append(&mut out, &doc.resources);
            append(&mut out, &doc.policy.requirements);
        }
        ProjectSection::Goals
        | ProjectSection::Tasks
        | ProjectSection::Notes
        | ProjectSection::Roles
        | ProjectSection::Spend
        | ProjectSection::Recurring => {}
    }
    out
}
fn append(out: &mut List<Part>, parts: &List<Part>) {
    for part in parts {
        if !out.as_slice().contains(part) && out.room() > 0 {
            out.push(part.clone()).expect("part fits");
        }
    }
}
pub(crate) fn show(domain: &mut Domain, env: &Env<crate::Limits>, parts: List<Part>, out: &mut Queue<Request>) {
    let mut count = 0_u32;
    for part in parts.into_boxed() {
        if count == env.limits.parts {
            crate::pages::routing::refuse(domain, contract::Limit::Objects);
            break;
        }
        let scope = match domain.address {
            Address::Project { project, .. } => contract::PartScope::Project(project),
            Address::Inbox { .. } | Address::Conversations { .. } | Address::Task { .. } | Address::Deployment => {
                contract::PartScope::Deployment
            }
        };
        let scope = crate::application::owner_scope(&part, scope);
        out.push(Request::ShowPart { part, generation: domain.generation, scope });
        count = count.saturating_add(1);
    }
}
/// Replaces the live task watch so every filter is authoritative at its source.
pub(crate) fn tasks(domain: &mut Domain, env: &Env<crate::Limits>, filter: TaskFilter, out: &mut Queue<Request>) {
    let Some(project) = number(domain) else {
        return;
    };
    let mut old = List::with_capacity(env.limits.watches);
    for (id, ()) in &domain.watch_ids {
        let watch = domain.watches.get(*id).expect("registered watch");
        match watch.subject {
            contract::Subject::Tasks { .. } => {
                if watch.owner == (Owner::Page { generation: domain.generation }) {
                    old.push(*id).expect("watch IDs fit");
                }
            }
            contract::Subject::Frame
            | contract::Subject::Task { .. }
            | contract::Subject::Tree { .. }
            | contract::Subject::Chips { .. }
            | contract::Subject::Inbox { .. }
            | contract::Subject::Goals { .. }
            | contract::Subject::Conversations
            | contract::Subject::Run { .. }
            | contract::Subject::Deployment => {}
        }
    }
    for id in old.into_boxed() {
        domain.close_one(id, out);
    }
    domain.project_page.tasks_more = false;
    domain.project_page.task_filter = filter;
    domain.project_page.tasks.clear();
    open(domain, env, contract::Subject::Tasks { project, filter, window: env.limits.rows }, out);
    domain.changed();
}
/// Opaque windows exposed by the board, notes and spend views.
pub const GOALS_WINDOW: u64 = 20;
/// Scoped notes window.
pub const NOTES_WINDOW: u64 = 21;
/// Past spend periods window.
pub const SPEND_WINDOW: u64 = 22;
pub(crate) fn more(domain: &mut Domain, window: Token, end: End, out: &mut Queue<Request>) {
    let Some(project) = number(domain) else {
        return;
    };
    if window == Token::new(GOALS_WINDOW) {
        let cursor = match end {
            End::Before => domain.project_page.ended_before.clone(),
            End::After => domain.project_page.ended_after.clone(),
        };
        if let Some(cursor) = cursor {
            ask(domain, 20, Read::Goals { project, after: Some(cursor) }, out);
        }
    } else if window == Token::new(NOTES_WINDOW) {
        let cursor = match end {
            End::Before => domain.project_page.notes_before.clone(),
            End::After => domain.project_page.notes_after.clone(),
        };
        if let Some(cursor) = cursor
            && let Some(filter) = domain.project_page.note_filter.clone()
        {
            ask(
                domain,
                3,
                Read::Notes { scope: filter.scope, description: filter.description, after: Some(cursor) },
                out,
            );
        }
    } else if window == Token::new(SPEND_WINDOW) {
        let cursor = match end {
            End::Before => domain.project_page.spend_before.clone(),
            End::After => domain.project_page.spend_after.clone(),
        };
        if let Some(cursor) = cursor {
            ask(domain, 23, Read::Spend { project, after: Some(cursor) }, out);
        }
    }
}
/// Whole documents, nested bounded rows, note handles, cursors and pending goal words.
#[must_use]
pub fn worst_case(limits: &crate::Limits) -> Option<u64> {
    limits.rows.checked_mul(2)?;
    u64::from(limits.rows)
        .checked_mul(limits.document_bytes)?
        .checked_mul(6)?
        .checked_add(limits.document_bytes.checked_mul(4)?)?
        .checked_add(List::<u16>::worst_case(limits.rows)?)?
        .checked_add(List::<GoalLine>::worst_case(limits.rows)?.checked_mul(4)?)?
        .checked_add(List::<TaskLine>::worst_case(limits.rows)?)?
        .checked_add(List::<Id<Object>>::worst_case(limits.rows)?)?
        .checked_add(List::<contract::SpendPeriod>::worst_case(limits.rows)?)?
        .checked_add(u64::from(limits.text).checked_mul(10)?)?
        .checked_add(limits.document_bytes)
}

fn merge_goals(page: &mut ProjectPage) {
    page.listed_goals.clear();
    for rows in [&page.goals, &page.ended] {
        for row in rows {
            let mut known = false;
            for current in &page.listed_goals {
                if current.chip.task == row.chip.task {
                    known = true;
                    break;
                }
            }
            if !known {
                page.listed_goals.push(row.clone()).expect("two bounded goal sources fit");
            }
        }
    }
    goals::order(&mut page.listed_goals);
}
pub(crate) fn board(domain: &mut Domain, goals: contract::Goals) {
    if number(domain) != Some(goals.project) {
        return;
    }
    domain.project_page.goals_more = goals.more;
    domain.project_page.goals.clear();
    for row in goals.rows.into_boxed() {
        if domain.project_page.goals.room() == 0 {
            break;
        }
        domain.project_page.goals.push(row).expect("goal row fits");
    }
    goals::order(&mut domain.project_page.goals);
    merge_goals(&mut domain.project_page);
    domain.changed();
}
pub(crate) fn ended(
    domain: &mut Domain,
    env: &Env<crate::Limits>,
    source: Source,
    page: contract::GoalPage,
    out: &mut Queue<Request>,
) {
    domain.project_page.ended_cut = page.cut;
    domain.project_page.ended_before = page.before;
    domain.project_page.ended_after = page.after;
    domain.project_page.ended.clear();
    for row in page.rows.into_boxed() {
        if domain.project_page.ended.room() == 0 {
            break;
        }
        domain.project_page.ended.push(row).expect("ended row fits");
    }
    goals::order(&mut domain.project_page.ended);
    merge_goals(&mut domain.project_page);
    let mut keys = List::with_capacity(env.limits.rows);
    for goal in &domain.project_page.ended {
        keys.push(contract::ObjectKey::Task(goal.chip.task)).expect("keys fit row window");
    }
    crate::pages::routing::names(domain, env, source, keys, out);
    for goal in &domain.project_page.ended {
        let mut watched = false;
        for current in &domain.project_page.goals {
            if current.chip.task == goal.chip.task {
                watched = true;
                break;
            }
        }
        if watched {
            continue;
        }
        if !domain.objects.put(contract::ObjectKey::Task(goal.chip.task), crate::objects::Body::Chip(goal.chip.clone()))
        {
            break;
        }
    }
    domain.changed();
}
pub(crate) fn task_rows(
    domain: &mut Domain,
    env: &Env<crate::Limits>,
    source: Source,
    tasks: contract::Tasks,
    out: &mut Queue<Request>,
) {
    if number(domain) != Some(tasks.project) || tasks.filter != domain.project_page.task_filter {
        return;
    }
    domain.project_page.tasks_more = tasks.more;
    domain.project_page.tasks.clear();
    for row in tasks.rows.into_boxed() {
        if domain.project_page.tasks.room() == 0 {
            break;
        }
        if matches_filter(&row, tasks.filter) {
            domain.project_page.tasks.push(row).expect("task row fits");
        }
    }
    order_tasks(&mut domain.project_page.tasks);
    let mut keys = List::with_capacity(env.limits.rows);
    for task in &domain.project_page.tasks {
        keys.push(contract::ObjectKey::Task(task.chip.task)).expect("keys fit rows");
    }
    crate::pages::routing::names(domain, env, source, keys, out);
    for task in &domain.project_page.tasks {
        if !domain.objects.put(contract::ObjectKey::Task(task.chip.task), crate::objects::Body::Chip(task.chip.clone()))
        {
            break;
        }
    }
    domain.changed();
}
fn matches_filter(row: &TaskLine, filter: TaskFilter) -> bool {
    crate::pages::conversations::live(row.chip.phase)
        && match filter.phase {
            Some(phase) => phase == row.chip.phase,
            None => true,
        }
        && match filter.executor {
            Some(executor) => executor == row.chip.executor,
            None => true,
        }
        && match filter.goal {
            Some(goal) => row.goal == Some(goal),
            None => true,
        }
        && (!filter.asked_by_me || row.asked_by_me)
}
fn order_tasks(rows: &mut List<TaskLine>) {
    for first in 0..rows.len() {
        for second in first.checked_add(1).expect("row index fits")..rows.len() {
            let a = rows.get(first).expect("row exists");
            let b = rows.get(second).expect("row exists");
            if b.changed > a.changed || (b.changed == a.changed && b.chip.task < a.chip.task) {
                let b = b.clone();
                let a = core::mem::replace(rows.get_mut(first).expect("row exists"), b);
                *rows.get_mut(second).expect("row exists") = a;
            }
        }
    }
}
pub(crate) fn spend(domain: &mut Domain, page: contract::SpendPage) {
    domain.project_page.spend_cut = page.cut;
    domain.project_page.spend_before = page.before;
    domain.project_page.spend_after = page.after;
    domain.project_page.spend.clear();
    for mut row in page.rows.into_boxed() {
        order_spend(&mut row.tasks);
        if domain.project_page.spend.room() == 0 {
            break;
        }
        domain.project_page.spend.push(row).expect("period row fits");
    }
    domain.changed();
}

fn order_spend(rows: &mut List<contract::TaskSpend>) {
    for first in 0..rows.len() {
        for second in first.checked_add(1).expect("row index fits")..rows.len() {
            let a = rows.get(first).expect("row exists");
            let b = rows.get(second).expect("row exists");
            if b.spent > a.spent || (b.spent == a.spent && b.chip.task < a.chip.task) {
                let b = b.clone();
                let a = core::mem::replace(rows.get_mut(first).expect("row exists"), b);
                *rows.get_mut(second).expect("row exists") = a;
            }
        }
    }
}

pub(crate) fn live(
    domain: &mut Domain,
    env: &Env<crate::Limits>,
    source: Source,
    goals: contract::Goals,
    out: &mut Queue<Request>,
) {
    board(domain, goals);
    let mut keys = List::with_capacity(env.limits.rows);
    for row in &domain.project_page.goals {
        match keys.push(contract::ObjectKey::Task(row.chip.task)) {
            Ok(()) => {}
            Err(_) => break,
        }
    }
    crate::pages::routing::names(domain, env, source, keys, out);
    for row in &domain.project_page.goals {
        let key = contract::ObjectKey::Task(row.chip.task);
        let stored = domain.objects.put(key, crate::objects::Body::Chip(row.chip.clone()));
        if !stored {
            break;
        }
    }
    domain.changed();
}

/// Install the complete historical source page in supplied order.
pub(crate) fn owner_spend(domain: &mut Domain, page: Box<contract::OwnerSpendPage>) {
    domain.project_page.spend_cut = page.cut;
    domain.project_page.spend_before.clone_from(&page.before);
    domain.project_page.spend_after.clone_from(&page.after);
    domain.project_page.owner_spend = Some(page);
    domain.changed();
}

/// Receive only the original project's whole owner value; order stays supplied.
pub(crate) fn owner_document(domain: &mut Domain, value: Box<contract::OwnerProject>) {
    if number(domain) != Some(value.number) {
        return;
    }
    domain.project_page.document = None;
    domain.project_page.owner = Some(value);
    domain.changed();
}
