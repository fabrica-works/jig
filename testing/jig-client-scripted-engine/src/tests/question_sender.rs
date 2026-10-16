use super::{script, task, text};
use crate::{Engine, Invalid};
use jig_domain_client::contract::*;
use skein_lib::{List, Wall};
#[test]
fn question_sender_kit_rejects_malformed_original_packet_before_engine_state_exists() {
    let mut supplied = Question {
        task: task().chip,
        from: Some(OwnerRequester::Person(u64::MAX)),
        message: u64::MAX,
        words: text(b"whole original words"),
        since: Wall::from_nanos(u64::MAX),
        acts: List::with_capacity(2),
        ending: None,
    };
    let key = ObjectKey::Question { task: supplied.task.task, message: supplied.message };
    let mut setup = script();
    setup.truth.object_projects.insert(key, 1);
    setup.truth.objects.insert(key, ReadValue::Question(Box::new(supplied.clone())));
    assert!(Engine::new(setup.clone(), 1).is_ok());
    supplied.acts = List::with_capacity(33);
    setup.truth.objects.insert(key, ReadValue::Question(Box::new(supplied.clone())));
    assert_eq!(Engine::new(setup.clone(), 1).err(), Some(Invalid::Payload));
    assert_eq!(setup.truth.objects.get(&key), Some(&ReadValue::Question(Box::new(supplied))));
}
