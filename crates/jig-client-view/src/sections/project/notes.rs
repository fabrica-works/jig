//! Scope/search and note cards retain the revision read (pages.md, 6.4).
use crate::phrases::jig::JigPhrase;
use crate::words::Words;
use crate::{JigBindings, JigMeaning, cards};
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, State};

use alloc::boxed::Box;
use jig_domain_client::{
    Domain,
    action::Act,
    contract::{NoteScope, Part},
    pages::project::NoteFilter,
};
pub(super) fn notes(b: &mut Builder, client: &Domain, project: u32, words: &Words, bindings: &mut JigBindings) {
    b.open(Role::Search);
    b.region(crate::regions::JigRegion::NotesSearch.region());
    b.name(&words.say(JigPhrase::NoteSearch, &[]));
    let filter = client
        .project_page()
        .note_filter
        .clone()
        .unwrap_or(NoteFilter { scope: NoteScope::Project(project), description: None });
    b.open(Role::TextField);
    b.name(&words.say(JigPhrase::NoteSearch, &[]));
    if let Some(description) = &filter.description {
        b.value(description.bytes(), client.project_page().note_writes);
    } else {
        b.value(&[], client.project_page().note_writes);
    }
    b.bind(bindings.insert(JigMeaning::NoteSearch { filter: filter.clone() }));
    b.close();
    for scope in [NoteScope::Deployment, NoteScope::Project(project)] {
        scope_choice(b, words, bindings, &filter, scope);
    }
    if let Some(doc) = &client.project_page().document {
        for part in &doc.resources {
            match part {
                Part::Resource { connector, path } => {
                    if let Some(pattern) = jig_domain_client::contract::exact_resource_pattern(path) {
                        scope_choice(
                            b,
                            words,
                            bindings,
                            &filter,
                            NoteScope::Resource { project, connector: *connector, pattern },
                        );
                    }
                }
                Part::Connector { .. }
                | Part::Kind { .. }
                | Part::Panel { .. }
                | Part::Result { .. }
                | Part::Effect { .. }
                | Part::OwnerEffect { .. }
                | Part::Outbox { .. }
                | Part::Hold { .. }
                | Part::Project { .. }
                | Part::Deployment { .. } => {}
            }
        }
    }
    for goal in &client.project_page().goals {
        scope_choice(b, words, bindings, &filter, NoteScope::Goal { project, goal: goal.chip.task });
    }
    b.close();
    b.open(Role::List);
    for note in &client.project_page().notes {
        cards::card(b, client, *note, &cards::Place::Elsewhere, words, bindings);
    }
    b.close();
    crate::pages::cut(b, words, client.project_page().notes_cut);
    crate::pages::work::cursors(
        b,
        words,
        bindings,
        client.project_page().notes_before.as_ref(),
        client.project_page().notes_after.as_ref(),
        jig_domain_client::pages::project::NOTES_WINDOW,
    );
}
fn scope_choice(b: &mut Builder, words: &Words, bindings: &mut JigBindings, filter: &NoteFilter, scope: NoteScope) {
    let (phrase, holes): (JigPhrase, Box<[Hole]>) = match &scope {
        NoteScope::Deployment => (JigPhrase::Deployment, Box::new([])),
        NoteScope::Project(project) => (JigPhrase::ContextProject, Box::new([Hole::Number(u64::from(*project))])),
        NoteScope::Goal { goal, .. } => (JigPhrase::OnlyGoal, Box::new([Hole::Number(*goal)])),
        NoteScope::Resource { project, connector, .. } => (
            JigPhrase::ScopedResourceNotes,
            Box::new([Hole::Number(u64::from(*project)), Hole::Number(u64::from(*connector))]),
        ),
    };
    let label = words.say(phrase, &holes);
    b.open(Role::Button);
    b.name(&label);
    b.text(skein_ui::Run::Plain, &label);
    if let NoteScope::Resource { pattern, .. } = &scope {
        crate::owner_authority::pattern(b, words, pattern);
    }
    if scope == filter.scope {
        b.state(State::Selected);
    }
    b.bind(bindings.insert(JigMeaning::Action(Act::FilterNotes {
        filter: NoteFilter { scope, description: filter.description.clone() },
    })));
    b.close();
}
