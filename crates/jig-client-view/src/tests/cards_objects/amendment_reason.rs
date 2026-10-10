//! Rendered reason input preserves presence and opaque bytes in one amendment.
use super::*;

#[test]
fn rendered_amendment_reason_preserves_empty_opaque_bytes_and_only_selected_instructions() {
    for bytes in [b"".as_slice(), [0xff; 128].as_slice()] {
        let (mut domain, id) = loaded(ObjectKey::Task(8), ReadValue::Task(Box::new(task_doc(&[ActKind::Amend]))));
        drive(&mut domain, Event::Act { act: Action::Intend { object: id.token(), act: ActKind::Amend } });
        let words = words();
        let mut bindings = JigBindings::new(limits().bindings, CLIENT.text);
        let tree = dialog(&domain, &words, &mut bindings);
        let reason = tree.nodes().iter().find(|node| node.name.as_deref() == Some(b"Your reason")).unwrap();
        assert_eq!(reason.role, Role::TextArea);
        assert!(reason.binding.is_some());
        assert!(!domain.confirmation().unwrap().reason_supplied);
        drive(&mut domain, Event::Act { act: input(&tree, b"New instructions", b"", &mut bindings) });
        assert_eq!(
            domain.confirmation().unwrap().missing,
            Some(jig_client::confirm::Missing::Words { field: Purpose::Reason })
        );
        assert!(drive(&mut domain, Event::Act { act: Action::Confirm }).is_empty());
        drive(&mut domain, Event::Act { act: input(&tree, b"Your reason", bytes, &mut bindings) });
        let selected = domain.confirmation().unwrap().choices.change.clone().unwrap();
        assert_eq!(
            selected,
            Amendment {
                addressed: None,
                spec: None,
                instructions: Some(text(b"")),
                wake: None,
                dependencies: None,
                authority: None
            }
        );
        let tree = dialog(&domain, &words, &mut bindings);
        let reason = tree.nodes().iter().find(|node| node.name.as_deref() == Some(b"Your reason")).unwrap();
        assert_eq!(reason.value.as_ref().unwrap().text.as_ref(), bytes);
        let text_bytes: u32 = tree.nodes().iter().map(|node| node.text_bytes()).sum();
        assert!(text_bytes <= limits().text_bytes);
        assert!(
            tree.nodes().iter().filter(|node| node.binding.is_some()).count()
                <= usize::try_from(limits().bindings).unwrap()
        );
        let out = drive(&mut domain, Event::Act { act: press(&tree, b"Confirm", &mut bindings) });
        let sent = out
            .iter()
            .find_map(|request| match request {
                Request::Send { ask, .. } => Some(&**ask),
                _ => None,
            })
            .unwrap();
        assert_eq!(
            sent,
            &Ask::Amend {
                project: 4,
                task: 8,
                revision: Revision { value: 6 },
                change: selected,
                release: None,
                reason: text(bytes)
            }
        );
    }
}

#[test]
fn reason_checked_binding_rejects_over_limit_without_establishing_presence() {
    let (mut domain, id) = loaded(ObjectKey::Task(8), ReadValue::Task(Box::new(task_doc(&[ActKind::Amend]))));
    drive(&mut domain, Event::Act { act: Action::Intend { object: id.token(), act: ActKind::Amend } });
    let mut bindings = JigBindings::new(limits().bindings, CLIENT.text);
    let tree = dialog(&domain, &words(), &mut bindings);
    let token =
        tree.nodes().iter().find(|node| node.name.as_deref() == Some(b"Your reason")).unwrap().binding.unwrap().token;
    assert_eq!(decode(&mut bindings, token, Act::Input { value: Box::from([0xff; 129]) }), None);
    assert!(!domain.confirmation().unwrap().reason_supplied);
    assert!(domain.fields().get(Purpose::Reason).is_none());
    assert!(tree.len() <= crate::confirmation_nodes(&CLIENT).unwrap().checked_add(1).unwrap());
}
