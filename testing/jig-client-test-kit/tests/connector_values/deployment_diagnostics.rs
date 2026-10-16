//! The receiving value retains the owning diagnostic availability and allocation.
use super::*;

#[test]
fn deployment_receiving_keeps_unkept_facts_scope_full_width_capacities_and_reserved_outbox_whole() {
    let original = ClientDocument::Deployment {
        outbox: list(3, [entry()]),
        subject_limit: u32::MAX,
        staged_limit: u32::MAX - 1,
        request_budget: DiagnosticAvailability::NotKept,
        system_last_response: DiagnosticAvailability::NotKept,
    };
    let mut value = carrier(original.clone());
    value.subject = Subject { scope: Scope::Deployment, kind: Kind::Deployment };
    assert!(value.matches_subject());
    assert_eq!(value.clone().value.as_ref(), &original);
    assert_eq!(value.revision(), None);
    for scope in [Scope::Project(0), Scope::Project(u32::MAX)] {
        value.subject.scope = scope;
        assert!(!value.matches_subject());
    }
    value.subject.scope = Scope::Deployment;
    value.subject.kind = Kind::Outbox(u64::MAX);
    assert!(!value.matches_subject());
    let price = document_bytes(&value).unwrap();
    let mut bound = bounds();
    for (bytes, admitted) in [(price - 1, false), (price, true), (price + 1, true)] {
        bound.bytes = bytes;
        assert_eq!(document_fits(&value, &bound), admitted);
    }
    let empty = carrier(ClientDocument::Deployment {
        outbox: list(3, []),
        subject_limit: 0,
        staged_limit: 0,
        request_budget: DiagnosticAvailability::NotKept,
        system_last_response: DiagnosticAvailability::NotKept,
    });
    assert_eq!(
        document_heap(&empty),
        Some(size_of::<ClientDocument>() as u64 + List::<OutboxEntry>::worst_case(3).unwrap())
    );
    bound = bounds();
    bound.rows = 2;
    assert!(!document_fits(&empty, &bound), "unused outbox capacity is still owned");
}
