//! Filters and live rows are the domain's facts (pages.md, 6.3).
use crate::{
    Builder, JigBindings, Role, cards,
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};
use jig_domain_client::{
    Domain,
    action::Act,
    contract::{Executor, ObjectKey, Phase, Project},
};
pub(super) fn tasks(b: &mut Builder, client: &Domain, doc: &Project, words: &Words, bindings: &mut JigBindings) {
    let filter = client.project_page().task_filter;
    super::super::work::control(
        b,
        words,
        bindings,
        JigPhrase::AllTasks,
        Act::FilterTasks {
            filter: jig_domain_client::contract::TaskFilter {
                phase: None,
                executor: None,
                goal: None,
                asked_by_me: false,
            },
        },
        filter.phase.is_none() && filter.executor.is_none() && filter.goal.is_none() && !filter.asked_by_me,
    );
    super::super::work::control(
        b,
        words,
        bindings,
        JigPhrase::AskedByMe,
        Act::FilterTasks {
            filter: jig_domain_client::contract::TaskFilter { asked_by_me: !filter.asked_by_me, ..filter },
        },
        filter.asked_by_me,
    );
    for phase in [
        Phase::Waiting,
        Phase::Idle,
        Phase::Due,
        Phase::Preparing,
        Phase::Claimed,
        Phase::Running,
        Phase::BackingOff,
        Phase::Active,
        Phase::Closing,
        Phase::Held,
        Phase::Ended,
        Phase::Done,
        Phase::Failed,
        Phase::Cancelled,
    ] {
        super::super::work::control(
            b,
            words,
            bindings,
            phrase(phase),
            Act::FilterTasks { filter: jig_domain_client::contract::TaskFilter { phase: Some(phase), ..filter } },
            filter.phase == Some(phase),
        );
    }
    for executor in &doc.policy.ceiling.task_kinds {
        let phrase = match executor {
            Executor::Agent { .. } => JigPhrase::Agent,
            Executor::Procedure { .. } => JigPhrase::Procedure,
            Executor::Party { .. } => JigPhrase::Person,
            Executor::Role { .. } => JigPhrase::RoleExecutor,
        };
        super::super::work::control(
            b,
            words,
            bindings,
            phrase,
            Act::FilterTasks {
                filter: jig_domain_client::contract::TaskFilter { executor: Some(*executor), ..filter },
            },
            filter.executor == Some(*executor),
        );
    }
    b.open(Role::List);
    for row in &client.project_page().tasks {
        b.open(Role::Item);
        b.key(super::super::object_key(ObjectKey::Task(row.chip.task)));
        cards::chip::chip(b, client, &row.chip, words, bindings);
        if let Some(id) = client.objects().find(ObjectKey::Task(row.chip.task)) {
            cards::card(b, client, id, &cards::Place::Elsewhere, words, bindings);
        }
        crate::owner_result::requester(b, row.requester, words);
        words.wall(b, row.changed);
        if let Some(goal) = row.goal {
            crate::frame::control(
                b,
                words,
                bindings,
                Role::Button,
                JigPhrase::OnlyGoal,
                &[Hole::Number(goal)],
                Act::FilterTasks { filter: jig_domain_client::contract::TaskFilter { goal: Some(goal), ..filter } },
            );
        }
        b.close();
    }
    b.close();
    if client.project_page().tasks_more {
        words.status(b, JigPhrase::MoreRows, &[]);
    }
}
const fn phrase(phase: Phase) -> JigPhrase {
    match phase {
        Phase::Waiting => JigPhrase::Waiting,
        Phase::Idle => JigPhrase::Idle,
        Phase::Due => JigPhrase::DuePhase,
        Phase::Preparing => JigPhrase::PreparingPhase,
        Phase::Claimed => JigPhrase::ClaimedPhase,
        Phase::Running => JigPhrase::Running,
        Phase::BackingOff => JigPhrase::RetryingPhase,
        Phase::Active => JigPhrase::ActivePhase,
        Phase::Closing => JigPhrase::ClosingPhase,
        Phase::Held => JigPhrase::Held,
        Phase::Ended => JigPhrase::Ended,
        Phase::Done => JigPhrase::Done,
        Phase::Failed => JigPhrase::FailedPhase,
        Phase::Cancelled => JigPhrase::CancelledPhase,
    }
}
