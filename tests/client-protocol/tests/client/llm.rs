//! A party's LLM choice through the client: `me` names the models it may choose, a Create names
//! one, and the task's document shows it (client/domain/contract.md, 3.2 and 5.1).

use super::{CREATE, ME, PROJECT_NUMBER, PROOF_WAIT, Story, fixtures};
use jig_client::{Answered, Ask, Asker, Handle, Latest, Outcome, Watchable, contract};
use skein_lib::List;

const CHOSEN: Handle = Handle { asker: Asker::JIG, number: 6 };

fn text(bytes: &[u8]) -> contract::Text {
    contract::Text::new(bytes.into(), 32).expect("a short name")
}
fn named(model: &[u8], effort: &[u8]) -> contract::ModelEffort {
    contract::ModelEffort { model: text(model), effort: Some(text(effort)) }
}
/// A model's name and its efforts' names.
type Named = (Vec<u8>, Vec<Vec<u8>>);

/// Each model `me` names in the project, with its efforts.
fn me_models(story: &Story) -> Option<Vec<Named>> {
    let Some(Latest::Jig(contract::Delivery::Me(me))) = &story.client.document(ME)?.latest else { return None };
    let project = me.projects.iter().find(|row| row.project == PROJECT_NUMBER)?;
    Some(
        project
            .models
            .iter()
            .map(|model| {
                (model.name.bytes().to_vec(), model.efforts.iter().map(|value| value.bytes().to_vec()).collect())
            })
            .collect(),
    )
}
/// The chosen task's delivered document.
fn chosen(story: &Story) -> Option<&contract::Task> {
    let Some(Latest::Jig(contract::Delivery::Task(task))) = &story.client.document(CHOSEN)?.latest else {
        return None;
    };
    Some(task)
}

#[test]
fn me_names_the_models_a_create_chooses_from_and_the_task_shows_the_choice() {
    let mut story = Story::live_batch(fixtures::SEED, true);
    story.watch(ME, Watchable::Jig(contract::Subject::Me));
    assert!(story.run_until(|s| me_models(s).is_some(), PROOF_WAIT));
    assert_eq!(me_models(&story), Some(vec![(b"scripted".to_vec(), vec![b"low".to_vec(), b"high".to_vec()])]));
    let mut subagents = List::with_capacity(1);
    subagents.push(named(b"scripted", b"low")).expect("one sub-agent");
    let choice = contract::LlmChoice { main: named(b"scripted", b"high"), subagents };
    let mut tasks = fixtures::create_tasks();
    tasks.get_mut(0).expect("first root").llm = Some(choice.clone());
    story.send(CREATE, Ask::Jig(Box::new(contract::Ask::Create { project: PROJECT_NUMBER, tasks })));
    assert!(story.run_until(|s| !s.answered(CREATE).is_empty(), PROOF_WAIT));
    let answered = story.answered(CREATE);
    let [
        Outcome::Answered(Answered::Jig(contract::Answer::Done(contract::Outcome::Owner(
            contract::OwnerOutcome::Created { first: task, .. },
        )))),
    ] = answered.as_slice()
    else {
        panic!("created: {answered:?}");
    };
    let task = *task;
    story.watch(CHOSEN, Watchable::Jig(contract::Subject::Task { task }));
    assert!(story.run_until(|s| chosen(s).is_some(), PROOF_WAIT));
    assert_eq!(chosen(&story).map(|task| task.llm.clone()), Some(Some(choice)));
}
