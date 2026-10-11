//! One task page, whose header names the shared object and whose plan keeps only
//! bounded tree context. Executor facts decide which sections have content. Parts
//! are references handed to the root with the page generation, never their values.
//! History and ended subtrees are read on demand; live plans are watched. Agent
//! activity is the conversation owner's state, never copied here. See client/domain/pages.md, 5.
//!
//! | Page | Event | Next | Emits |
//! |---|---|---|---|
//! | loading | task document | header known | section parts and executor activity |
//! | plan | tree delivery | bounded chips and folds | shared object reads |
//! | history | more | pending read | history cursor |
//! | plan | open ended subtree | pending read | ended-delegate cursor |
//! | any | leave | empty | handled by generation routing |
use crate::{
    Domain, Request,
    address::{Address, TaskSection},
    contract::{self, Executor, Limit, ObjectKey, Part, Phase, PhaseCounts, Read, Task, Tree, TreeNode},
    objects::{Body, Object, Source},
    watches::Owner,
};
use alloc::boxed::Box;
use skein_lib::{Env, Id, List, Map, Queue, Token};
/// Which plan rows a person has asked to see.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlanFilter {
    /// Every phase.
    All,
    /// Rows held, or with held work below them.
    Held,
    /// Rows whose work has not ended.
    Live,
}
/// Sections whose current task document has something to show.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Sections {
    pub offered: List<TaskSection>,
}
/// Bounded plan context, preserving counts of every held task beyond the window.
#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "fold choices change only through admitted page actions")]
pub struct Plan {
    /// Whole supplied owner tree, independent of older node fixtures.
    pub owner: Option<Box<contract::OwnerTree>>,
    pub nodes: List<TreeNode>,
    pub folded: PhaseCounts,
    pub held_below: u32,
    pub filter: PlanFilter,
    folds: Map<u64, bool>,
}
/// The page's header handle and section-specific read choices.
#[derive(Debug)]
pub struct TaskPage {
    pub header: Option<Id<Object>>,
    /// Whole owner ended page under the selected original read.
    pub owner_ended: Option<Box<contract::OwnerEndedPage>>,
    pub plan: Plan,
    pub subtree_history: bool,
    /// Loss reported by the current ended-delegate read window.
    pub ended_cut: bool,
    pub ended_after: Option<contract::Cursor>,
    pub ended_before: Option<contract::Cursor>,
    pub ended_task: Option<u64>,
}
impl TaskPage {
    pub(crate) fn new(rows: u32) -> Self {
        Self {
            header: None,
            owner_ended: None,
            plan: Plan {
                owner: None,
                nodes: List::with_capacity(rows),
                folded: counts(),
                held_below: 0,
                filter: PlanFilter::All,
                folds: Map::with_capacity(rows.checked_add(1).expect("root fold fits")),
            },
            subtree_history: false,
            ended_cut: false,
            ended_after: None,
            ended_before: None,
            ended_task: None,
        }
    }
    pub(crate) fn clear(&mut self) {
        self.header = None;
        self.owner_ended = None;
        self.plan.owner = None;
        self.plan.nodes.clear();
        self.plan.folded = counts();
        self.plan.held_below = 0;
        self.plan.filter = PlanFilter::All;
        let mut keys = List::with_capacity(self.plan.folds.len());
        for (task, _) in &self.plan.folds {
            keys.push(*task).expect("fold key fits");
        }
        for task in &keys {
            let _open = self.plan.folds.remove(task).expect("collected fold exists");
        }
        self.subtree_history = false;
        self.ended_cut = false;
        self.ended_after = None;
        self.ended_before = None;
        self.ended_task = None;
    }
}
fn counts() -> PhaseCounts {
    PhaseCounts { ended: 0, waiting: 0, running: 0, held: 0, closing: 0, done: 0, failed: 0, cancelled: 0 }
}
/// Whether a section has content for this executor and its current facts.
#[must_use]
pub fn sections(task: &Task) -> Sections {
    let (impact, panels) = match &task.context {
        Some(context) => (
            context.impact.tasks.waiting > 0
                || context.impact.tasks.running > 0
                || context.impact.tasks.held > 0
                || context.impact.tasks.closing > 0
                || context.impact.tasks.done > 0
                || context.impact.tasks.failed > 0
                || context.impact.tasks.cancelled > 0
                || context.impact.tasks.ended > 0,
            !context.parts.is_empty(),
        ),
        None => (false, false),
    };
    let plan = !task.delegates.is_empty() || impact;
    let panels = panels
        || !task.resources.is_empty()
        || match task.chip.executor {
            Executor::Procedure { .. } => true,
            Executor::Agent { .. } | Executor::Party { .. } | Executor::Role { .. } => false,
        };
    let budget = match task.chip.executor {
        Executor::Agent { .. } | Executor::Procedure { .. } => true,
        Executor::Party { .. } | Executor::Role { .. } => {
            task.budget.numbers.budget > 0
                || task.budget.numbers.spent > 0
                || task.budget.numbers.spent_below > 0
                || task.budget.numbers.reserved > 0
        }
    };
    // The mandatory result contract supplies overview content even with an empty spec.
    let overview = true;
    let mut offered = List::with_capacity(6);
    for section in [
        TaskSection::Overview,
        TaskSection::Activity,
        TaskSection::Plan,
        TaskSection::History,
        TaskSection::Budget,
        TaskSection::Panels,
    ] {
        let present = match section {
            TaskSection::Overview => overview,
            TaskSection::Activity | TaskSection::History => true,
            TaskSection::Plan => plan,
            TaskSection::Budget => budget,
            TaskSection::Panels => panels,
        };
        if present {
            offered.push(section).expect("at most six sections");
        }
    }
    Sections { offered }
}
/// Whether a section should be offered, without interpreting acts or authority.
#[must_use]
pub fn offered(sections: &Sections, section: TaskSection) -> bool {
    sections.offered.as_slice().contains(&section)
}
/// Retains a whole bounded tree, folding additional rows by phase and preserving
/// the owner's total held count so a held descendant cannot disappear.
pub(crate) fn tree(domain: &mut Domain, tree: Tree) {
    let plan = &mut domain.task_page.plan;
    plan.owner = None;
    plan.nodes.clear();
    plan.folded = tree.folded;
    plan.held_below = tree.held_below;
    for node in tree.nodes.into_boxed() {
        if plan.nodes.room() == 0 {
            add(&mut plan.folded, node.chip.phase);
            continue;
        }
        plan.nodes.push(node).expect("a row fits its window");
    }
    let mut removed = List::with_capacity(plan.folds.len());
    for (task, _) in &plan.folds {
        let mut retained = false;
        for node in &plan.nodes {
            if node.chip.task == *task {
                retained = true;
            }
        }
        if !retained {
            removed.push(*task).expect("fold key fits");
        }
    }
    for task in &removed {
        let _removed = plan.folds.remove(task);
    }
    domain.changed();
}
/// Moves the complete owner envelope; local choices do not alter source rows.
pub(crate) fn owner_tree(domain: &mut Domain, tree: Box<contract::OwnerTree>) {
    let plan = &mut domain.task_page.plan;
    plan.nodes.clear();
    plan.folded = counts();
    plan.held_below = 0;
    let mut removed = List::with_capacity(plan.folds.len());
    for (task, _) in &plan.folds {
        let mut retained = tree.root.task == *task;
        for row in &tree.rows {
            retained |= row.chip.task == *task;
        }
        if !retained {
            removed.push(*task).expect("fold keys fit");
        }
    }
    for task in &removed {
        let _open = plan.folds.remove(task);
    }
    plan.owner = Some(tree);
    domain.changed();
}
fn add(counts: &mut PhaseCounts, phase: Phase) {
    let value = match phase {
        Phase::Waiting | Phase::Idle => &mut counts.waiting,
        Phase::Due | Phase::Preparing | Phase::Claimed | Phase::Running | Phase::BackingOff | Phase::Active => {
            &mut counts.running
        }
        Phase::Held => &mut counts.held,
        Phase::Closing => &mut counts.closing,
        Phase::Ended => &mut counts.ended,
        Phase::Done => &mut counts.done,
        Phase::Failed => &mut counts.failed,
        Phase::Cancelled => &mut counts.cancelled,
    };
    *value = value.checked_add(1).expect("a bounded tree count fits u32");
}
impl Plan {
    /// Existing local fold choice, defaulting to the supplied chip phase.
    #[must_use]
    pub fn owner_expanded(&self, chip: &contract::Chip) -> bool {
        match self.folds.get(&chip.task) {
            Some(open) => *open,
            None => crate::pages::conversations::live(chip.phase),
        }
    }
    /// Source folds carry held work even when their parent is locally filtered.
    #[must_use]
    pub fn owner_visible(&self, row: &contract::OwnerTreeRow) -> bool {
        match self.filter {
            PlanFilter::All => true,
            PlanFilter::Live => crate::pages::conversations::live(row.chip.phase),
            PlanFilter::Held => {
                let mut held = row.chip.phase == Phase::Held;
                if let Some(owner) = &self.owner {
                    for fold in &owner.folded {
                        held |= fold.parent == row.chip.task && fold.held_below > 0;
                    }
                }
                held
            }
        }
    }
    /// Live subtrees begin open; ended subtrees begin folded to their line.
    #[must_use]
    pub fn expanded(&self, node: &TreeNode) -> bool {
        match self.folds.get(&node.chip.task) {
            Some(open) => *open,
            None => crate::pages::conversations::live(node.chip.phase),
        }
    }
    /// Filtering preserves every retained row so changing the filter loses no facts.
    #[must_use]
    pub fn visible(&self, node: &TreeNode) -> bool {
        match self.filter {
            PlanFilter::All => true,
            PlanFilter::Held => node.chip.phase == Phase::Held || node.held_below > 0,
            PlanFilter::Live => crate::pages::conversations::live(node.chip.phase),
        }
    }
}
/// Chooses only the part references named by the selected section.
#[must_use]
pub fn parts(task: &Task, section: TaskSection, limit: u32) -> List<Part> {
    let mut parts = List::with_capacity(limit);
    match section {
        TaskSection::Overview => {
            for resource in &task.resources {
                if let Some(part) = resource.part() {
                    push(&mut parts, part.clone());
                }
            }
            result(&mut parts, task);
        }
        TaskSection::Activity => match task.chip.executor {
            Executor::Procedure { connector, .. } => push(&mut parts, Part::Panel { connector, task: task.chip.task }),
            Executor::Agent { .. } | Executor::Party { .. } | Executor::Role { .. } => {}
        },
        TaskSection::Budget => {
            for grant in &task.authority.authority.grants {
                push(&mut parts, Part::Kind { connector: grant.connector, kind: grant.kind });
            }
        }
        TaskSection::Panels => {
            if let Some(context) = &task.context {
                if let Some(closing) = &context.closing
                    && let Some(connectors) = &closing.connectors
                {
                    for connector in connectors {
                        push(&mut parts, Part::Panel { connector: *connector, task: task.chip.task });
                    }
                }
                append(&mut parts, &context.parts);
            }
            for resource in &task.resources {
                if let Some(part) = resource.part() {
                    push(&mut parts, part.clone());
                }
            }
            match task.chip.executor {
                Executor::Procedure { connector, .. } => {
                    push(&mut parts, Part::Panel { connector, task: task.chip.task });
                }
                Executor::Agent { .. } | Executor::Party { .. } | Executor::Role { .. } => {}
            }
        }
        TaskSection::Plan | TaskSection::History => {}
    }
    parts
}
fn append(out: &mut List<Part>, values: &List<Part>) {
    for part in values {
        push(out, part.clone());
    }
}
fn push(out: &mut List<Part>, part: Part) {
    for kept in &*out {
        if *kept == part {
            return;
        }
    }
    if out.room() > 0 {
        out.push(part).expect("part fits its bounded list");
    }
}
fn result(out: &mut List<Part>, task: &Task) {
    if let Some(result) = &task.result {
        match result {
            contract::TaskResult::Ending(_) => {}
            contract::TaskResult::Document(result) => match &result.value {
                Some(contract::ResultValue::Connector { part, .. }) => push(out, part.clone()),
                Some(
                    contract::ResultValue::Report(_)
                    | contract::ResultValue::Verdict { .. }
                    | contract::ResultValue::OwnerVerdict { .. }
                    | contract::ResultValue::Change { .. }
                    | contract::ResultValue::Choice { .. }
                    | contract::ResultValue::Approved { .. }
                    | contract::ResultValue::Failure(_)
                    | contract::ResultValue::Words(_),
                )
                | None => {}
            },
        }
    }
}
/// After the task document reveals its executor, opens only that activity owner.
pub(crate) fn document(domain: &mut Domain, env: &Env<crate::Limits>, out: &mut Queue<Request>) {
    let (number, section) = match domain.address {
        Address::Task { task, section } => (task, section),
        Address::Inbox { .. } | Address::Conversations { .. } | Address::Project { .. } | Address::Deployment => return,
    };
    let Some(id) = domain.objects.find(ObjectKey::Task(number)) else {
        return;
    };
    domain.task_page.header = Some(id);
    let task = match &domain.objects.get(id).expect("named task exists").body {
        Some(Body::Task(task)) => task,
        Some(
            Body::Historical(_)
            | Body::Chip(_)
            | Body::Proposal(_)
            | Body::OwnerProposal(_)
            | Body::Escalation(_)
            | Body::OwnerEscalation(_)
            | Body::Question(_)
            | Body::PersonTask(_)
            | Body::OwnerPersonTask(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => return,
    };
    let project = task.project;
    let executor = task.chip.executor;
    let phase = task.chip.phase;
    let values = parts(task, section, env.limits.parts.saturating_add(1));
    let mut emitted = 0_u32;
    for part in values.into_boxed() {
        if emitted == env.limits.parts {
            crate::pages::routing::refuse(domain, Limit::Objects);
            break;
        }
        emitted = emitted.saturating_add(1);
        let scope = crate::application::owner_scope(&part, contract::PartScope::Project(project));
        out.push(Request::ShowPart { part, generation: domain.generation, scope });
    }
    if section == TaskSection::Plan {
        if crate::pages::conversations::live(phase) {
            open_once(domain, env, contract::Subject::Tree { task: number, window: env.limits.rows }, out);
        } else {
            domain.task_page.ended_task = Some(number);
            read(domain, 16, Read::Ended { task: number, after: None }, out);
        }
        return;
    }
    if section != TaskSection::Activity {
        return;
    }
    match executor {
        Executor::Agent { .. } => {
            let subject = contract::Subject::Run { task: number, turn: None, offset: 0 };
            if open_once(domain, env, subject, out) {
                read(domain, 1, Read::Activity { task: number, before: None }, out);
            }
        }
        Executor::Procedure { .. } => {}
        Executor::Party { .. } | Executor::Role { .. } => {
            let key = ObjectKey::PersonTask(number);
            let source = Source::Window { generation: domain.generation, number: 15 };
            if domain.objects.name(key, source).is_none() {
                crate::pages::routing::refuse(domain, Limit::Objects);
                return;
            }
            let object = domain.objects.find(key).expect("person task named above");
            if domain.objects.get(object).expect("named person task exists").body.is_none() {
                read(domain, 15, Read::PersonTask { task: number }, out);
            }
        }
    }
}
fn open_once(
    domain: &mut Domain,
    env: &Env<crate::Limits>,
    subject: contract::Subject,
    out: &mut Queue<Request>,
) -> bool {
    for (id, ()) in &domain.watch_ids {
        let watch = domain.watches.get(*id).expect("registered watch exists");
        if watch.owner == (Owner::Page { generation: domain.generation }) && watch.subject == subject {
            return false;
        }
    }
    if domain.open(env, subject, Owner::Page { generation: domain.generation }, out).is_none() {
        crate::pages::routing::refuse(domain, Limit::Watches);
    }
    true
}

fn read(domain: &mut Domain, number: u16, what: Read, out: &mut Queue<Request>) {
    if domain.reads.pending(&what) {
        return;
    }
    if domain
        .reads
        .ask(domain.generation, Source::Window { generation: domain.generation, number }, what, out)
        .is_none()
    {
        crate::pages::routing::refuse(domain, Limit::Reads);
    }
}
/// Fold presentation belongs to this page and never changes committed work.
pub(crate) fn fold(domain: &mut Domain, object: Token, open: bool) {
    match domain.address {
        Address::Task { section: TaskSection::Plan, .. } => {}
        Address::Task { .. }
        | Address::Inbox { .. }
        | Address::Conversations { .. }
        | Address::Project { .. }
        | Address::Deployment => return,
    }
    let Some(object) = domain.objects.get(Id::from_token(object)) else {
        return;
    };
    let task = match object.key {
        ObjectKey::Task(task) => task,
        ObjectKey::Proposal(_)
        | ObjectKey::Escalation(_)
        | ObjectKey::Question { .. }
        | ObjectKey::PersonTask(_)
        | ObjectKey::Result(_)
        | ObjectKey::Note { .. } => return,
    };
    let plan = &mut domain.task_page.plan;
    if let Some(owner) = &plan.owner {
        let mut retained = owner.root.task == task;
        for row in &owner.rows {
            retained |= row.chip.task == task;
        }
        if retained && plan.folds.insert(task, open).is_ok() {
            domain.changed();
        }
        return;
    }
    for node in &plan.nodes {
        if node.chip.task == task {
            if plan.folds.insert(task, open).is_ok() {
                domain.changed();
            }
            return;
        }
    }
}
/// Selects a plan filter and reopens its live source without inventing history.
pub(crate) fn filter(domain: &mut Domain, filter: PlanFilter) {
    domain.task_page.plan.filter = filter;
    domain.changed();
}
/// Chooses task-only or subtree history and starts its first page.
pub(crate) fn history(domain: &mut Domain, subtree: bool, out: &mut Queue<Request>) {
    let task = match domain.address {
        Address::Task { task, .. } => task,
        Address::Inbox { .. } | Address::Conversations { .. } | Address::Project { .. } | Address::Deployment => return,
    };
    domain.task_page.subtree_history = subtree;
    domain.history_rows.rows.clear();
    domain.history_rows.cut = false;
    read(domain, 2, Read::History { task, subtree, after: None }, out);
    domain.changed();
}
/// Opens one ended subtree, whose results name shared cards.
pub(crate) fn open_ended(domain: &mut Domain, task: u64, out: &mut Queue<Request>) {
    match domain.address {
        Address::Task { section: TaskSection::Plan, .. } => {}
        Address::Task { .. }
        | Address::Inbox { .. }
        | Address::Conversations { .. }
        | Address::Project { .. }
        | Address::Deployment => return,
    }
    let mut found = false;
    if let Some(owner) = &domain.task_page.plan.owner {
        found = owner.root.task == task;
        for row in &owner.rows {
            found |= row.chip.task == task;
        }
    }
    for node in &domain.task_page.plan.nodes {
        if node.chip.task == task {
            found = true;
        }
    }
    if !found {
        return;
    }
    domain.task_page.owner_ended = None;
    domain.task_page.ended_task = Some(task);
    domain.task_page.ended_cut = false;
    domain.task_page.ended_after = None;
    domain.task_page.ended_before = None;
    read(domain, 16, Read::Ended { task, after: None }, out);
}
/// The history window's binding number.
pub const HISTORY_WINDOW: u64 = 12;
/// Ended delegates in the opened subtree.
pub const ENDED_WINDOW: u64 = 13;
pub(crate) fn more(domain: &mut Domain, window: Token, end: crate::action::End, out: &mut Queue<Request>) {
    if window == Token::new(HISTORY_WINDOW) {
        let edge = match end {
            crate::action::End::Before => &domain.history_rows.edges.before,
            crate::action::End::After => &domain.history_rows.edges.after,
        };
        let cursor = match edge {
            crate::windows::Edge::More(cursor) => Some(cursor.clone()),
            crate::windows::Edge::End => None,
        };
        if let Some(cursor) = cursor {
            let task = match domain.address {
                Address::Task { task, .. } => task,
                Address::Inbox { .. }
                | Address::Conversations { .. }
                | Address::Project { .. }
                | Address::Deployment => return,
            };
            read(
                domain,
                2,
                Read::History { task, subtree: domain.task_page.subtree_history, after: Some(cursor) },
                out,
            );
        }
    } else if window == Token::new(ENDED_WINDOW) {
        let cursor = match end {
            crate::action::End::Before => domain.task_page.ended_before.clone(),
            crate::action::End::After => domain.task_page.ended_after.clone(),
        };
        if let Some(task) = domain.task_page.ended_task
            && let Some(cursor) = cursor
        {
            read(domain, 16, Read::Ended { task, after: Some(cursor) }, out);
        }
    }
}
/// Plan context, nested chip payloads and opaque subtree cursors.
#[must_use]
pub fn worst_case(limits: &crate::Limits) -> Option<u64> {
    List::<TreeNode>::worst_case(limits.rows)?
        .checked_add(Map::<u64, bool>::worst_case(limits.rows.checked_add(1)?)?)?
        .checked_add(List::<u64>::worst_case(limits.rows.checked_add(1)?)?)?
        .checked_add(u64::from(limits.rows).checked_mul(limits.document_bytes)?.checked_mul(2)?)?
        .checked_add(u64::from(limits.text).checked_mul(2)?)?
        .checked_add(limits.document_bytes)?
        .checked_add(limits.document_bytes)
}

pub(crate) fn ended(domain: &mut Domain, before: Option<contract::Cursor>, after: Option<contract::Cursor>) {
    domain.task_page.ended_before = before;
    domain.task_page.ended_after = after;
}

/// Install one admitted owner page without naming synthetic Result objects.
pub(crate) fn owner_ended(domain: &mut Domain, page: Box<contract::OwnerEndedPage>) {
    domain.task_page.ended_cut = page.cut;
    ended(domain, page.before.clone(), page.after.clone());
    domain.task_page.owner_ended = Some(page);
    domain.changed();
}
