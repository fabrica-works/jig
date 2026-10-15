#![expect(
    clippy::wildcard_enum_match_arm,
    clippy::disallowed_methods,
    reason = "ordinary accessibility assertions select semantic role categories"
)]
use crate::{JigStyle, Owner, Role, State, Tree};

/// Run on the same trees as every fact/state/action scenario. The shell can
/// discard styles and still read names, roles, statuses and semantic states.
pub(super) fn check(tree: &Tree) {
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.binding.is_some() || node.link.is_some() || node.external.is_some() {
            assert!(
                [
                    Role::Button,
                    Role::Link,
                    Role::TextField,
                    Role::TextArea,
                    Role::Choice,
                    Role::Check,
                    Role::MenuItem,
                    Role::Tab
                ]
                .contains(&node.role)
            );
            assert!(node.name.as_ref().is_some_and(|name| !name.is_empty()), "action at {index} has a name");
            assert!(!node.states.as_slice().contains(&State::Disabled), "disabled nodes have no action");
        }
        match node.role {
            Role::Region | Role::Dialog | Role::Status | Role::ChoiceGroup => {
                assert!(node.name.as_ref().is_some_and(|name| !name.is_empty()), "named {:?} at {index}", node.role);
            }
            _ => {}
        }
        if let Some(key) = node.key {
            assert_eq!(node.role, Role::Item);
            let card = tree.nodes().get(index.checked_add(1).unwrap()).unwrap();
            let role = if key.owner == Owner::Jig.opaque() && key.kind == 28 { Role::Log } else { Role::Region };
            assert_eq!(card.role, role);
            assert!(card.name.as_ref().is_some_and(|name| !name.is_empty()));
        }
        if node.styles.as_slice().contains(&JigStyle::Danger.token()) {
            assert!(
                tree.nodes().iter().any(|status| {
                    status.role == Role::Status
                        && status.name.as_deref() == Some(b"Dangerous action: review its consequences".as_slice())
                }),
                "danger also has explicit status words"
            );
        }
    }
}
