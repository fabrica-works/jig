//! The world holds a decision to the application's own phrasing.
use super::*;
use crate::{Scripted, Test, stories, test::list};
use jig_client::{
    Asker, Key,
    contract::{ActKind, Admission, Ask, DecideOn, Decision, Offered, ReadValue},
    words::{self, DecisionWord, Word},
};
use jig_client_test_kit::check;
use skein_ui::phrases::{Form, PluralClass, Segment};
use skein_ui::{Builder, Role, Run};
use skein_world::domain::Verdict;
const PROJECT: u32 = 1;
const CLIENT: ClientId = ClientId(0);
const KEY: Key = Key { asker: Asker::JIG, number: 1 };
const OVERRIDE: &[u8] = b"Review this refusal with care";
const ENGLISH: &[u8] = b"Dangerous action: review its consequences";
const UI: skein_ui::Limits = skein_ui::Limits {
    nodes: 3,
    depth: 2,
    text_per_node: 64,
    text_bytes: 128,
    drawing_bytes: 0,
    styles_per_node: 0,
    patches: 0,
    bindings: 0,
    slots: 0,
    markdown_depth: 1,
    scene_marks: 0,
    scene_bytes: 0,
};
fn overridden_world() -> World<Test, Scripted> {
    let mut language = words::english();
    let phrase = language
        .phrases
        .iter_mut()
        .find(|phrase| phrase.id == Word::Decision(DecisionWord::Dangerous).id())
        .expect("danger is in the public words");
    phrase.forms = Box::new([Form { class: PluralClass::Other, segments: Box::new([Segment::Text(OVERRIDE.into())]) }]);
    stories::configured(Settings { words: language, ..Settings::default() })
}
fn reject(world: &mut World<Test, Scripted>, text: &[u8]) {
    let mut proposal = stories::proposal();
    proposal.acts = list([Offered { act: ActKind::Reject, admission: Admission::Allowed, effect: None }]);
    let key = proposal.key;
    world.observe(Observation::Delivered {
        client: CLIENT,
        receipt: super::super::Receipt::Document { value: Box::new(ReadValue::Proposal(Box::new(proposal))) },
    });
    let mut builder = Builder::new(&UI);
    builder.open(Role::Region);
    builder.name(b"Proposal");
    builder.text(Run::Plain, text);
    builder.close();
    let tree = builder.finish();
    let node = tree.nodes().last().expect("decision text node").id;
    world.observe(Observation::Decision {
        client: CLIENT,
        key: KEY,
        ask: Box::new(Ask::Decide {
            project: PROJECT,
            on: DecideOn::Proposal {
                proposal: key,
                decision: Decision::Rejected { reason: stories::text(b"No") },
                running: None,
            },
        }),
        scene: Some(check::scene(&tree, node)),
    });
}
#[test]
fn the_world_accepts_a_tree_in_its_overridden_words() {
    let mut world = overridden_world();
    reject(&mut world, OVERRIDE);
    assert_eq!(world.referee.verdict(), Verdict::Passed);
}
#[test]
#[should_panic(expected = "a deciding act did not show its facts")]
fn the_world_refuses_english_when_its_words_are_overridden() {
    let mut world = overridden_world();
    reject(&mut world, ENGLISH);
}
