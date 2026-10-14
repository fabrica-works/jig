use super::native_hold::{drive, ready, view};
use super::*;
use jig_test_domain_client::connector_values as owner;

fn proposal(part: OwnerPart) -> OwnerProposal {
    OwnerProposal {
        key: ProposalKey { proposer: Proposer::Task(9), number: 2 },
        project: 11,
        reason: DocumentText::complete(text(b"supplied effect")),
        action: OwnerProposalAction::Effect {
            part,
            authority: owner_authority(),
            attempt: 13,
            completion: 3,
            position: 7,
            purpose: 19,
        },
        as_holder: true,
        waiting: OwnerProposalWaiting { holder: OwnerHistoryHolder::Person(7), since: Wall::from_nanos(23) },
        funding: List::with_capacity(0),
        acts: List::with_capacity(0),
    }
}
fn load(root: &mut Domain, proposal: OwnerProposal) -> Vec<Request> {
    let rows =
        drive(root, Event::Went { place: Place::Address(Address::First(first::Address { project: 3, item: 5 })) });
    let named = drive(
        root,
        Event::First(first::Event::Read {
            read: family_read(&rows),
            result: first::ReadResult::Value(family::Document::Item(family::Item {
                project: 3,
                key: 5,
                revision: Revision { value: 1 },
                state: family::ItemState::Ready,
                value: 1,
                task: 9,
                second: None,
                proposals: list([proposal.key]),
                touchable: false,
            })),
        }),
    );
    let read = named
        .iter()
        .find_map(|row| match row {
            Request::Jig(JigOut::Read { read, what: Read::Proposal { proposal: key } }) if *key == proposal.key => {
                Some(*read)
            }
            _ => None,
        })
        .unwrap();
    drive(
        root,
        Event::Jig(jig_domain_client::Event::Read {
            read,
            result: ReadResult::Value(ReadValue::OwnerProposal(Box::new(proposal))),
        }),
    )
}
fn part() -> OwnerPart {
    OwnerPart { connector: jig_test_domain_client::FIRST, kind: OwnerPartKind::Effect, key: OwnerPartKey::Number(17) }
}
fn document(subject: family::Subject) -> owner::ConnectorDocument {
    let path = owner::Path::new(list([Box::<[u8]>::from(*b"literal/slash"), Box::<[u8]>::from([255, 0])]), 16).unwrap();
    owner::ConnectorDocument {
        subject,
        value: Box::new(owner::ClientDocument::Effect {
            proposal: 17,
            task: 9,
            effect: owner::Effect {
                kind: 5,
                resources: Box::new([path.clone()]),
                purpose: 19,
                condition: Some(21),
                target: 23,
                state: 13,
            },
            description: owner::Description {
                kind: 5,
                resources: Box::new([path]),
                purpose: 19,
                condition: true,
                form: owner::Form::Transition,
                recovery: owner::Recovery::Unrecoverable,
                price: Some(31),
                state: 13,
            },
        }),
    }
}

#[test]
fn a_native_effect_slot_reads_its_separate_scoped_description_and_releases_it_on_exit() {
    let mut root = ready();
    let original = proposal(part());
    let rows = load(&mut root, original.clone());
    let subject = family::Subject { scope: family::Scope::Project(11), kind: family::Kind::OwnerEffect(17) };
    let reads: Vec<_> = rows
        .iter()
        .filter_map(|row| match row {
            Request::First(first::Out::Read { read, subject }) => Some((*read, *subject)),
            _ => None,
        })
        .collect();
    assert_eq!(reads.len(), 1);
    assert_eq!(reads[0].1, subject);
    let mut view = view();
    patches(&mut view, &root);
    assert!(
        view.tree().nodes().iter().any(
            |node| node.role == Role::Status && node.name.as_deref() == Some(b"Loading family document".as_slice())
        )
    );
    let supplied = document(subject);
    drive(
        &mut root,
        Event::First(first::Event::Read {
            read: reads[0].0,
            result: first::ReadResult::Value(family::Document::Connector(supplied.clone())),
        }),
    );
    let watch = rows
        .iter()
        .find_map(|row| match row {
            Request::First(first::Out::Open { watch, subject: actual }) if *actual == subject => Some(*watch),
            _ => None,
        })
        .unwrap();
    patches(&mut view, &root);
    named(&view, Role::Region, b"Effect description");
    for text in [b"Effect kind: 5".as_slice(), b"Effect purpose: 19", b"literal/slash", b"\\xFF\\x00"] {
        assert!(contains(&view, text), "missing {}", String::from_utf8_lossy(text));
    }
    assert!(view.tree().nodes().iter().any(|node| node.key == Some(crate::family_key(&subject))));
    let id = root.first().find(subject).unwrap();
    assert_eq!(root.first().object(id).unwrap().value, first::Value::Known(family::Document::Connector(supplied)));
    let retained =
        root.jig().objects().get(root.jig().objects().find(ObjectKey::Proposal(original.key)).unwrap()).unwrap();
    let jig_domain_client::objects::Body::OwnerProposal(retained) = retained.body.as_ref().unwrap() else {
        panic!("whole owner proposal")
    };
    assert_eq!(retained.as_ref(), &original);
    assert!(patches(&mut view, &root).is_empty());
    let left = drive(
        &mut root,
        Event::Went {
            place: Place::Address(Address::Jig(jig_domain_client::address::Address::Inbox { project: None })),
        },
    );
    assert!(left.iter().any(|row| match row {
        Request::First(first::Out::Close { watch: actual }) => *actual == watch,
        _ => false,
    }));
    patches(&mut view, &root);
    assert!(view.tree().nodes().iter().all(|node| node.name.as_deref() != Some(b"Effect description".as_slice())));
    assert!(root.first().find(subject).is_none());
}

#[test]
fn a_path_keyed_native_effect_keeps_whole_proposal_facts_without_inventing_a_read() {
    let mut root = ready();
    let mut part = part();
    part.key = OwnerPartKey::Path(list([Box::<[u8]>::from(*b"number/17"), Box::<[u8]>::from([255, 0])]));
    let original = proposal(part);
    let rows = load(&mut root, original.clone());
    assert!(
        rows.iter()
            .find_map(|row| match row {
                Request::First(first::Out::Read { read, .. }) => Some(*read),
                _ => None,
            })
            .is_none()
    );
    let mut view = view();
    patches(&mut view, &root);
    assert!(contains(&view, b"number/17"));
    assert!(contains(&view, b"\\xFF\\x00"));
    assert!(view.tree().nodes().iter().all(|node| node.name.as_deref() != Some(b"Effect description".as_slice())));
    assert!(view.tree().nodes().iter().any(|node| node.name.as_deref() == Some(b"Cannot show this part".as_slice())));
    let retained =
        root.jig().objects().get(root.jig().objects().find(ObjectKey::Proposal(original.key)).unwrap()).unwrap();
    let jig_domain_client::objects::Body::OwnerProposal(retained) = retained.body.as_ref().unwrap() else {
        panic!("whole owner proposal")
    };
    assert_eq!(retained.as_ref(), &original);
}
