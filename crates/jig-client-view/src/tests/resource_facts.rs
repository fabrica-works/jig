#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary tests inspect whole public resource slots and literal trees"
)]
use super::cards::{CLIENT, contains, shared_words};
use crate::JigBindings;
use jig_client::contract::*;
use skein_lib::List;
use skein_ui::{Builder, Role};
fn list<T>(values: impl IntoIterator<Item = T>) -> List<T> {
    let values: Vec<_> = values.into_iter().collect();
    let mut out = List::with_capacity(u32::try_from(values.len()).unwrap());
    for value in values {
        out.push(value).ok().unwrap();
    }
    out
}
#[test]
fn segmented_names_and_exact_open_patterns_keep_raw_bindings_and_separate_literal_nodes() {
    let words = shared_words();
    let limits = crate::limits::of_words(&CLIENT, words).unwrap();
    let mut b = Builder::new(&limits);
    let mut bindings = JigBindings::new(limits.bindings, CLIENT.text);
    let path = list([Box::<[u8]>::from(*b"a/b\0"), Box::<[u8]>::from([255, 0, 128])]);
    let resource = Part::Resource { connector: u16::MAX, path: path.clone() };
    let exact = OwnerPattern {
        segments: list([path.get(0).unwrap().clone()]),
        last: OwnerPatternEnd::Exact(path.get(1).unwrap().clone()),
    };
    let open =
        OwnerPattern { segments: exact.segments.clone(), last: OwnerPatternEnd::Open(path.get(1).unwrap().clone()) };
    assert_ne!(exact, open);
    let authority = Authority {
        budget: 0,
        grants: list([
            Grant { connector: u16::MAX, kind: 1, pattern: exact },
            Grant { connector: u16::MAX, kind: 2, pattern: open.clone() },
        ]),
        task_kinds: list([]),
        depth: 0,
        tasks: 0,
        deadline: None,
        tools: list([]),
        note_scopes: list([NoteScope::Resource { project: u32::MAX, connector: u16::MAX, pattern: open }]),
    };
    b.open(Role::Page);
    crate::cards::common::authority(&mut b, &authority, words, &mut bindings);
    crate::cards::common::part(&mut b, &resource, words, &mut bindings);
    words.refusal(
        &mut b,
        &Refusal::Moved {
            now: Revision { value: 1 },
            what: Moved::Hold {
                reason: Some(CurrentHold::Document(Box::new(HoldReason::Resource {
                    resource: resource.clone(),
                    holder: Actor::Task(u64::MAX),
                }))),
            },
        },
        CLIENT.text,
    );
    b.close();
    let slots = b.slots(0);
    assert_eq!(slots.len(), 3, "only two supplied kinds and the actual resource get slots");
    assert_eq!(
        bindings.part(slots.first().unwrap().reference.token),
        Some(&Part::Kind { connector: u16::MAX, kind: 1 })
    );
    assert_eq!(
        bindings.part(slots.get(1).unwrap().reference.token),
        Some(&Part::Kind { connector: u16::MAX, kind: 2 })
    );
    assert_eq!(bindings.part(slots.get(2).unwrap().reference.token), Some(&resource));
    let tree = b.finish();
    assert!(contains(&tree, b"a/b\0"));
    assert!(contains(&tree, b"\\xFF\\x00\\x80"));
    assert!(contains(&tree, b"Exact"));
    assert!(contains(&tree, b"Open"));
    assert!(tree.len() <= limits.nodes);
    assert_eq!(resource, Part::Resource { connector: u16::MAX, path });
}
