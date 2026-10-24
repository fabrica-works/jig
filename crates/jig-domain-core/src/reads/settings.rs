//! Dense revision reads (engine.md, 17.7; client/domain/contract.md, section 6).
use super::{
    Answer, Range, RevisionAnswer, RevisionEdit, RevisionItemHead, RevisionRow, RevisionsPage, Row, Scope,
    SettingDocument, Why, terminal,
};
use crate::{Core, CoreRecord, Limits, Record, Requests, settings};
use alloc::boxed::Box;
use jig_domain_people as people;
use skein_lib::{Env, List, Map, ReplyTo};

#[must_use]
pub fn oldest(core: &Core, limits: &Limits) -> u32 {
    core.current.oldest().max(core.current.newest().saturating_sub(limits.settings.history)).max(1)
}
/// The M4 held-charter selection belongs at this seam.
#[must_use]
pub fn may_read(current: &settings::Current, revision: u32) -> bool {
    revision == current.newest()
}

pub(super) fn setting(core: &Core, reply: ReplyTo, party: u64, setting: settings::Setting, revision: u32) -> Requests {
    if core.people.role(party, people::DEPLOYMENT).is_none() || !may_read(&core.current, revision) {
        return terminal(reply, Answer::None(Why::Unknown));
    }
    let answer = match core.current.value(setting) {
        Some(value) => Answer::Setting(Box::new(SettingDocument { setting, revision, value: value.clone() })),
        None => Answer::None(Why::Unknown),
    };
    terminal(reply, answer)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Selection {
    Unknown,
    Forgotten { oldest: u32 },
    Load { first: u32, last: u32 },
}
#[must_use]
pub fn select_revision(newest: u32, oldest: u32, number: u32) -> Selection {
    if number == 0 || number > newest {
        Selection::Unknown
    } else if number < oldest {
        Selection::Forgotten { oldest }
    } else {
        Selection::Load { first: number, last: number }
    }
}
#[must_use]
pub fn page_range(newest: u32, oldest: u32, before: Option<u32>, rows: u32) -> Option<(u32, u32)> {
    let last = match before {
        Some(before) => newest.min(before.saturating_sub(1)),
        None => newest,
    };
    if last < oldest || rows == 0 {
        return None;
    }
    Some((oldest.max(last.saturating_sub(rows.saturating_sub(1))), last))
}

pub(super) fn revision(
    core: &mut Core,
    env: &Env<Limits>,
    reply: ReplyTo,
    sign_in: u64,
    party: u64,
    number: u32,
) -> Requests {
    if core.people.role(party, people::DEPLOYMENT).is_none() {
        return terminal(reply, Answer::None(Why::Unknown));
    }
    match select_revision(core.current.newest(), oldest(core, &env.limits), number) {
        Selection::Unknown => terminal(reply, Answer::Revision(Box::new(RevisionAnswer::Unknown))),
        Selection::Forgotten { oldest } => {
            terminal(reply, Answer::Revision(Box::new(RevisionAnswer::Forgotten { oldest })))
        }
        Selection::Load { first, last } => load(core, env, reply, sign_in, party, first, last, true),
    }
}
pub(super) fn revisions(
    core: &mut Core,
    env: &Env<Limits>,
    reply: ReplyTo,
    sign_in: u64,
    party: u64,
    before: Option<u32>,
) -> Requests {
    if core.people.role(party, people::DEPLOYMENT).is_none() {
        return terminal(reply, Answer::None(Why::Unknown));
    }
    let oldest = oldest(core, &env.limits);
    match page_range(core.current.newest(), oldest, before, env.limits.documents.rows) {
        Some((first, last)) => load(core, env, reply, sign_in, party, first, last, false),
        None => terminal(
            reply,
            Answer::Revisions(Box::new(RevisionsPage {
                rows: List::with_capacity(0),
                next: None,
                forgotten: oldest > 1,
            })),
        ),
    }
}
#[expect(clippy::too_many_arguments, reason = "one bounded settings load admission")]
fn load(
    core: &mut Core,
    env: &Env<Limits>,
    reply: ReplyTo,
    sign_in: u64,
    party: u64,
    first: u32,
    last: u32,
    whole: bool,
) -> Requests {
    let rows = last.checked_sub(first).expect("ordered revision range").checked_add(1).expect("nonzero revision range");
    let Some(bytes) = load_bytes(&env.limits, rows) else {
        return terminal(reply, Answer::None(Why::Busy));
    };
    let range = Range {
        party,
        scope: Scope::SettingsHistory { first, last, newest: core.current.newest(), whole },
        after: None,
        backwards: false,
        rows,
        bytes,
    };
    super::load_settings(core, env, reply, sign_in, range)
}
/// A selected revision is decoded whole before its values are admitted separately.
#[must_use]
pub fn load_bytes(limits: &Limits, rows: u32) -> Option<u64> {
    settings::revision_worst_case(limits)?
        .checked_add(u64::try_from(size_of::<Row>()).ok()?)?
        .checked_add(u64::try_from(size_of::<Record>()).ok()?)?
        .checked_mul(u64::from(rows))
}

/// Validate one decoded history record before it crosses a load boundary.
/// Its set values occupy at most one whole bounded settings set.
#[must_use]
pub fn valid_revision(revision: &settings::Revision, limits: &Limits) -> bool {
    if !valid_names(&revision.models) {
        return false;
    }
    let Some(capacity) = settings::capacity(limits) else { return false };
    let Some(capacity) = capacity.checked_mul(2) else { return false };
    let Ok(count) = u32::try_from(revision.items.len()) else { return false };
    if count > capacity {
        return false;
    }
    let mut seen = Map::with_capacity(count);
    let mut charters = 0_u32;
    let mut templates = 0_u32;
    let mut policies = 0_u32;
    for item in &revision.items {
        if seen.insert(item.setting, ()).ok() != Some(None) {
            return false;
        }
        match &item.edit {
            settings::Edit::Set(value) => {
                if settings::check::shape(item.setting, value, limits).is_err()
                    || !valid_indices(value, &revision.models)
                {
                    return false;
                }
                match value {
                    settings::Value::Charter(value) => {
                        charters = charters.saturating_add(1);
                        if charters > limits.tasks.charters
                            || value.policy.alternatives.len()
                                > usize::try_from(limits.settings.alternatives).expect("alternative count")
                        {
                            return false;
                        }
                    }
                    settings::Value::Template(_) => {
                        templates = templates.saturating_add(1);
                        if templates > limits.templates {
                            return false;
                        }
                    }
                    settings::Value::Policy(_) => {
                        policies = policies.saturating_add(1);
                        if policies > limits.authority.projects {
                            return false;
                        }
                    }
                    settings::Value::Rules(_) => {}
                }
            }
            settings::Edit::Removed | settings::Edit::Reset => {}
        }
    }
    true
}

fn valid_names(models: &[settings::ModelName]) -> bool {
    if models.len() > usize::try_from(jig_domain_tasks::MODELS).expect("model bound") {
        return false;
    }
    for model in models {
        if model.name.len() > usize::try_from(crate::MODEL_NAME_BYTES).expect("model name bound")
            || model.efforts.len() > usize::try_from(crate::EFFORTS).expect("effort bound")
        {
            return false;
        }
        for effort in &model.efforts {
            if effort.len() > usize::try_from(crate::EFFORT_BYTES).expect("effort name bound") {
                return false;
            }
        }
    }
    true
}
fn valid_mask(value: jig_domain_authority::Models, models: &[settings::ModelName]) -> bool {
    value.is_all() || value.bits().checked_shr(u32::try_from(models.len()).expect("model count")).unwrap_or(0) == 0
}
fn valid_choice(value: jig_domain_tasks::ModelEffort, models: &[settings::ModelName]) -> bool {
    let Some(model) = models.get(usize::from(value.model)) else { return false };
    match value.effort {
        Some(effort) => usize::from(effort) < model.efforts.len(),
        None => model.efforts.is_empty(),
    }
}
fn valid_indices(value: &settings::Value, models: &[settings::ModelName]) -> bool {
    match value {
        settings::Value::Template(template) => {
            let Some(choice) = &template.llm else { return true };
            if choice.subagents.len() > usize::try_from(jig_domain_tasks::MODELS).expect("model bound")
                || !valid_choice(choice.main, models)
            {
                return false;
            }
            for chosen in &choice.subagents {
                if !valid_choice(*chosen, models) {
                    return false;
                }
            }
            true
        }
        settings::Value::Policy(policy) => {
            if !valid_mask(policy.policy.ceiling.models, models) {
                return false;
            }
            for role in &policy.policy.roles {
                if !valid_mask(role.authority.models, models) {
                    return false;
                }
            }
            true
        }
        settings::Value::Rules(rules) => valid_mask(rules.ceiling.models, models),
        settings::Value::Charter(_) => true,
    }
}

#[must_use]
pub fn record_revision(value: &Record) -> Option<&settings::Revision> {
    match value {
        Record::Core(CoreRecord::Settings(settings::Record::Revision { revision, .. })) => Some(revision),
        Record::Core(
            CoreRecord::Settings(
                settings::Record::Default { .. }
                | settings::Record::Changed { .. }
                | settings::Record::Charter { .. }
                | settings::Record::Proposal { .. }
                | settings::Record::Extent { .. },
            )
            | CoreRecord::Deployment(_)
            | CoreRecord::RunProof(_)
            | CoreRecord::Call(_)
            | CoreRecord::Terminal(_)
            | CoreRecord::Turn(_)
            | CoreRecord::Projection(_)
            | CoreRecord::EscalationDecision(_)
            | CoreRecord::ProposalDecision(_),
        )
        | Record::People(_)
        | Record::Tasks(_)
        | Record::Notes(_)
        | Record::Accounts(_) => None,
    }
}

fn selected(scope: &Scope) -> (u32, u32, u32, bool) {
    match scope {
        Scope::SettingsHistory { first, last, newest, whole } => (*first, *last, *newest, *whole),
        Scope::Tasks { .. }
        | Scope::Task { .. }
        | Scope::Proposal { .. }
        | Scope::Escalation { .. }
        | Scope::Question { .. }
        | Scope::PersonTask { .. }
        | Scope::Result { .. }
        | Scope::Reply { .. }
        | Scope::Turn { .. }
        | Scope::EndedDelegates { .. }
        | Scope::History { .. }
        | Scope::Activity { .. }
        | Scope::Inbox { .. }
        | Scope::Notes { .. }
        | Scope::ScopedNotes { .. }
        | Scope::Spend { .. } => unreachable!("settings range dispatched by its scope"),
    }
}

pub(super) fn from_store(
    core: &Core,
    limits: &Limits,
    range: &Range,
    raw: Box<[Row]>,
    next: Option<super::Position>,
) -> Answer {
    let (first, last, newest, whole) = selected(&range.scope);
    if next.is_some() || raw.len() > usize::try_from(range.rows).expect("row cap") {
        return Answer::None(Why::Busy);
    }
    let oldest = oldest(core, limits);
    if whole && first < oldest {
        return Answer::Revision(Box::new(RevisionAnswer::Forgotten { oldest }));
    }
    let mut found = Map::with_capacity(range.rows);
    for row in raw {
        if row.owner.is_some()
            || !row.ancestors.is_empty()
            || row.related.is_some()
            || !row.linked.is_empty()
            || row.inbox.is_some()
            || row.unjoined.is_some()
            || row.ended_read.is_some()
        {
            return Answer::None(Why::Unknown);
        }
        if !super::admits(range.party, &range.scope, &row.value.key()) {
            return Answer::None(Why::Unknown);
        }
        let revision = match row.value {
            Record::Core(CoreRecord::Settings(settings::Record::Revision { revision, .. })) => revision,
            Record::Core(
                CoreRecord::Settings(
                    settings::Record::Default { .. }
                    | settings::Record::Changed { .. }
                    | settings::Record::Charter { .. }
                    | settings::Record::Proposal { .. }
                    | settings::Record::Extent { .. },
                )
                | CoreRecord::Deployment(_)
                | CoreRecord::RunProof(_)
                | CoreRecord::Call(_)
                | CoreRecord::Terminal(_)
                | CoreRecord::Turn(_)
                | CoreRecord::Projection(_)
                | CoreRecord::EscalationDecision(_)
                | CoreRecord::ProposalDecision(_),
            )
            | Record::People(_)
            | Record::Tasks(_)
            | Record::Notes(_)
            | Record::Accounts(_) => return Answer::None(Why::Unknown),
        };
        if !valid_revision(&revision, limits) {
            return Answer::None(Why::Busy);
        }
        let previous = found.insert(revision.head.number, revision).expect("bounded loaded revisions");
        if previous.is_some() {
            return Answer::None(Why::Unknown);
        }
    }
    if whole {
        return Answer::Revision(Box::new(match found.remove(&first) {
            Some(mut revision) => {
                if first == newest && newest == core.current.newest() {
                    revision.head = core.current.newest_head().clone();
                }
                RevisionAnswer::Revision { revision, value_bytes: limits.load_bytes }
            }
            None => RevisionAnswer::Forgotten { oldest },
        }));
    }
    let mut rows = List::with_capacity(range.rows);
    for number in (first..=last).rev() {
        if number < oldest {
            continue;
        }
        let Some(revision) = found.remove(&number) else { continue };
        let total = u32::try_from(revision.items.len()).expect("checked item count");
        let mut items = List::with_capacity(total.min(limits.documents.lines));
        for item in revision.items.iter().take(usize::try_from(limits.documents.lines).expect("line cap")) {
            let edit = match item.edit {
                settings::Edit::Set(_) => RevisionEdit::Set,
                settings::Edit::Removed => RevisionEdit::Removed,
                settings::Edit::Reset => RevisionEdit::Reset,
            };
            items.push(RevisionItemHead { setting: item.setting, edit }).expect("bounded item preview");
        }
        rows.push(RevisionRow { head: revision.head, more: total.saturating_sub(items.len()), items })
            .expect("bounded revision page");
    }
    let next = if first > oldest { Some(first) } else { None };
    Answer::Revisions(Box::new(RevisionsPage { rows, next, forgotten: next.is_none() && oldest > 1 }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stored_catalogue_names_stay_within_their_priced_bounds() {
        let model = settings::ModelName {
            name: vec![b'm'; usize::try_from(crate::MODEL_NAME_BYTES).unwrap()].into_boxed_slice(),
            efforts: vec![
                vec![b'e'; usize::try_from(crate::EFFORT_BYTES).unwrap()].into_boxed_slice();
                usize::try_from(crate::EFFORTS).unwrap()
            ]
            .into_boxed_slice(),
        };
        let models = vec![model.clone(); usize::try_from(jig_domain_tasks::MODELS).unwrap()];
        assert!(valid_names(&models));
        let models = vec![model.clone(); usize::try_from(jig_domain_tasks::MODELS).unwrap() + 1];
        assert!(!valid_names(&models));
        let mut too_long = model.clone();
        too_long.name = vec![b'm'; usize::try_from(crate::MODEL_NAME_BYTES).unwrap() + 1].into_boxed_slice();
        assert!(!valid_names(&[too_long]));
        let mut too_many = model.clone();
        too_many.efforts =
            vec![Box::from(b"e".as_slice()); usize::try_from(crate::EFFORTS).unwrap() + 1].into_boxed_slice();
        assert!(!valid_names(&[too_many]));
        let mut too_long = model;
        too_long.efforts[0] = vec![b'e'; usize::try_from(crate::EFFORT_BYTES).unwrap() + 1].into_boxed_slice();
        assert!(!valid_names(&[too_long]));
    }
    #[test]
    fn the_first_revisions_last_page_has_no_forgotten_predecessors() {
        let (core, limits) = crate::tests::core();
        let revision =
            settings::Revision { models: Box::new([]), head: core.current.newest_head().clone(), items: Box::new([]) };
        let range = Range {
            party: 1,
            scope: Scope::SettingsHistory { first: 1, last: 1, newest: 1, whole: false },
            after: None,
            backwards: false,
            rows: 1,
            bytes: load_bytes(&limits, 1).unwrap(),
        };
        let row = Row {
            value: Record::Core(CoreRecord::Settings(settings::Record::Revision { history: false, revision })),
            owner: None,
            ancestors: Box::new([]),
            related: None,
            linked: Box::new([]),
            inbox: None,
            unjoined: None,
            ended_read: None,
        };
        let Answer::Revisions(page) = from_store(&core, &limits, &range, Box::new([row]), None) else { panic!("page") };
        assert_eq!(page.rows.len(), 1);
        assert_eq!(page.next, None);
        assert!(!page.forgotten);
    }
    #[test]
    fn a_revision_archived_after_admission_keeps_its_original_head() {
        let (mut core, limits) = crate::tests::core();
        let head = core.current.newest_head().clone();
        core.current.head.number = 2;
        core.current.head.against = 1;
        let revision = settings::Revision { models: Box::new([]), head: head.clone(), items: Box::new([]) };
        let range = Range {
            party: 1,
            scope: Scope::SettingsHistory { first: 1, last: 1, newest: 1, whole: true },
            after: None,
            backwards: false,
            rows: 1,
            bytes: load_bytes(&limits, 1).unwrap(),
        };
        let row = Row {
            value: Record::Core(CoreRecord::Settings(settings::Record::Revision { history: true, revision })),
            owner: None,
            ancestors: Box::new([]),
            related: None,
            linked: Box::new([]),
            inbox: None,
            unjoined: None,
            ended_read: None,
        };
        let Answer::Revision(answer) = from_store(&core, &limits, &range, Box::new([row]), None) else {
            panic!("revision")
        };
        let RevisionAnswer::Revision { revision, .. } = *answer else { panic!("kept revision") };
        assert_eq!(revision.head, head);
    }
    #[test]
    fn a_lowered_history_bound_hides_rows_the_commit_could_not_erase() {
        let (mut core, mut limits) = crate::tests::core();
        core.current.head.number = 6;
        core.current.oldest = 2;
        limits.settings.history = 2;
        assert_eq!(oldest(&core, &limits), 4);
        core.current.oldest = 5;
        assert_eq!(oldest(&core, &limits), 5);
        core.current.head.number = 1;
        core.current.oldest = 0;
        assert_eq!(oldest(&core, &limits), 1);
    }
    #[test]
    fn revision_selection_names_unknown_forgotten_and_kept_ranges() {
        const NEWEST: u32 = 6;
        const OLDEST: u32 = 4;
        for (number, expected) in [
            (0, Selection::Unknown),
            (1, Selection::Forgotten { oldest: OLDEST }),
            (3, Selection::Forgotten { oldest: OLDEST }),
            (4, Selection::Load { first: 4, last: 4 }),
            (5, Selection::Load { first: 5, last: 5 }),
            (6, Selection::Load { first: 6, last: 6 }),
            (7, Selection::Unknown),
        ] {
            assert_eq!(select_revision(NEWEST, OLDEST, number), expected, "{number}");
        }
    }
    #[test]
    fn dense_pages_select_only_the_requested_predecessors() {
        for (newest, oldest, before, rows, expected) in [
            (6, 4, None, 2, Some((5, 6))),
            (6, 4, Some(5), 2, Some((4, 4))),
            (6, 1, Some(5), 2, Some((3, 4))),
            (6, 1, Some(3), 2, Some((1, 2))),
            (6, 4, None, 8, Some((4, 6))),
            (6, 4, Some(4), 2, None),
            (6, 4, Some(0), 2, None),
            (6, 4, Some(u32::MAX), 2, Some((5, 6))),
            (u32::MAX, 1, None, 2, Some((u32::MAX - 1, u32::MAX))),
        ] {
            assert_eq!(page_range(newest, oldest, before, rows), expected);
        }
    }
    #[test]
    fn loads_admit_only_the_selected_history_and_held_keys() {
        use crate::{CoreKey, Key};
        let scope = Scope::SettingsHistory { first: 4, last: 6, newest: 6, whole: false };
        for (key, expected) in [
            (settings::Key::History(3), false),
            (settings::Key::History(4), true),
            (settings::Key::History(5), true),
            (settings::Key::History(6), true),
            (settings::Key::Held(5), false),
            (settings::Key::Held(6), true),
            (settings::Key::Held(7), false),
            (settings::Key::Extent, false),
        ] {
            assert_eq!(super::super::admits(1, &scope, &Key::Core(CoreKey::Settings(key))), expected);
        }
    }
}
