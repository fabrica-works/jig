use jig_test_client_domain::{
    connector_values::*,
    family::{Kind, Scope, Subject},
};
use jig_test_client_kit::{Bounds, connector_values::*};
use skein_lib::{List, Wall};
use std::mem::size_of;
fn list<T>(capacity: u32, values: impl IntoIterator<Item = T>) -> List<T> {
    let mut result = List::with_capacity(capacity);
    for value in values {
        result.push(value).ok().expect("fixture capacity");
    }
    result
}
fn path() -> Path {
    Path::new(list(3, [b"literal/slash".as_slice().into(), [0xff, 0x00].as_slice().into()]), 16)
        .expect("admitted opaque fixture path")
}
fn effect() -> Effect {
    Effect {
        kind: u16::MAX,
        resources: Box::new([path()]),
        purpose: u64::MAX,
        condition: Some(u64::MAX),
        target: u64::MAX,
        state: u64::MAX,
    }
}
fn description() -> Description {
    Description {
        kind: u16::MAX,
        resources: Box::new([path()]),
        purpose: u64::MAX,
        condition: true,
        form: Form::Transition,
        recovery: Recovery::Unrecoverable,
        price: Some(u64::MAX),
        state: u64::MAX,
    }
}
fn entry() -> OutboxEntry {
    OutboxEntry {
        number: u64::MAX,
        task: u64::MAX,
        key: Key {
            attempt: u64::MAX,
            completion: u32::MAX,
            position: u32::MAX,
            deployment: [u8::MAX; 16],
            task: u64::MAX,
            purpose: u64::MAX,
        },
        effect: effect(),
        attempt: Some(Attempt {
            number: u32::MAX,
            sent: Wall::from_nanos(u64::MAX),
            deadline: Wall::from_nanos(u64::MAX),
        }),
        phase: EffectPhase::Held,
    }
}
fn resource() -> ClientResource {
    ClientResource {
        path: path(),
        role: ResourceRole::Unavailable,
        hold: Hold::Pooled { mode: HoldMode::Wait },
        fact: Some(Fact { state: Some(u64::MAX), observed: Wall::from_nanos(u64::MAX), pending: true }),
    }
}
fn procedure() -> ProcedureState {
    ProcedureState {
        task: u64::MAX,
        number: u16::MAX,
        resource: path(),
        index: u16::MAX,
        steps: u16::MAX,
        awaiting: true,
        deadline: Some(Wall::from_nanos(u64::MAX)),
    }
}
fn carrier(value: ClientDocument) -> ConnectorDocument {
    ConnectorDocument {
        subject: Subject { scope: Scope::Project(u32::MAX), kind: Kind::Outbox(u64::MAX) },
        value: Box::new(value),
    }
}
fn bounds() -> Bounds {
    Bounds { rows: 3, paths: 2, segments: 3, segment_bytes: 16, bytes: u64::MAX }
}
fn variants() -> [ClientDocument; 8] {
    [
        ClientDocument::Object { object: u64::MAX, now: ApplicationObject { revision: u64::MAX, value: u64::MAX } },
        ClientDocument::Panel {
            task: u64::MAX,
            procedure: Some(procedure()),
            result: Some(u16::MAX),
            resources: list(3, [resource()]),
            effects: list(3, [entry()]),
        },
        ClientDocument::Result { task: u64::MAX, code: u16::MAX },
        ClientDocument::Effect { proposal: u64::MAX, task: u64::MAX, effect: effect(), description: description() },
        ClientDocument::Outbox { entry: entry(), description: description() },
        ClientDocument::Hold { task: u64::MAX, resources: list(3, [resource()]), effects: list(3, [entry()]) },
        ClientDocument::Project { project: u32::MAX, resources: list(3, [resource()]) },
        ClientDocument::Deployment {
            outbox: list(3, [entry()]),
            subject_limit: u32::MAX,
            staged_limit: u32::MAX,
            request_budget: DiagnosticAvailability::NotKept,
            system_last_response: DiagnosticAvailability::NotKept,
        },
    ]
}
#[test]
fn connector_values_keep_all_eight_whole_variants_and_only_the_actual_object_revision() {
    for (index, original) in variants().into_iter().enumerate() {
        let document = carrier(original.clone());
        assert_eq!(document.value.as_ref(), &original);
        assert_eq!(document.clone(), document);
        assert_eq!(document.revision(), if index == 0 { Some(u64::MAX) } else { None });
        assert!(document_fits(&document, &bounds()));
        assert!(document_bytes(&document).unwrap() <= document_worst_case(&bounds()).unwrap());
    }
}
#[test]
fn connector_values_whole_allocation_admission_has_exact_minus_one_at_and_plus_one_boundaries() {
    for original in variants() {
        let document = carrier(original);
        let price = document_bytes(&document).unwrap();
        let mut bound = bounds();
        for (bytes, admitted) in [(price - 1, false), (price, true), (price + 1, true)] {
            bound.bytes = bytes;
            assert_eq!(document_fits(&document, &bound), admitted);
        }
    }
}
#[test]
fn connector_values_price_unused_list_capacity_every_box_and_each_literal_path_byte() {
    let document = carrier(ClientDocument::Panel {
        task: 0,
        procedure: None,
        result: None,
        resources: list(3, []),
        effects: list(2, []),
    });
    assert_eq!(
        document_heap(&document),
        Some(
            size_of::<ClientDocument>() as u64
                + List::<ClientResource>::worst_case(3).unwrap()
                + List::<OutboxEntry>::worst_case(2).unwrap()
        )
    );
    let path_price = 2 * size_of::<Box<[u8]>>() as u64 + 13 + 2;
    let value = carrier(ClientDocument::Effect { proposal: 9, task: 8, effect: effect(), description: description() });
    assert_eq!(
        document_heap(&value),
        Some(size_of::<ClientDocument>() as u64 + 2 * (size_of::<Path>() as u64 + path_price))
    );
    let cloned = document.clone();
    let ClientDocument::Panel { resources, effects, .. } = cloned.value.as_ref() else { panic!("panel remains whole") };
    assert_eq!((resources.capacity(), effects.capacity()), (3, 2));
    let mut bound = bounds();
    bound.rows = 2;
    assert!(!document_fits(&document, &bound), "unused rows are still allocated");
}
#[test]
fn connector_values_each_structural_capacity_checks_its_own_boundary() {
    let document =
        carrier(ClientDocument::Effect { proposal: 3, task: 4, effect: effect(), description: description() });
    for (segment_bytes, admitted) in [(12, false), (13, true), (14, true)] {
        let mut bound = bounds();
        bound.segment_bytes = segment_bytes;
        assert_eq!(document_fits(&document, &bound), admitted);
    }
    for (segments, admitted) in [(1, false), (2, true), (3, true)] {
        let mut bound = bounds();
        bound.segments = segments;
        assert_eq!(document_fits(&document, &bound), admitted);
    }
    for (paths, admitted) in [(0, false), (1, true), (2, true)] {
        let mut bound = bounds();
        bound.paths = paths;
        assert_eq!(document_fits(&document, &bound), admitted);
    }
    let document = carrier(ClientDocument::Deployment {
        outbox: list(2, []),
        subject_limit: 0,
        staged_limit: 0,
        request_budget: DiagnosticAvailability::NotKept,
        system_last_response: DiagnosticAvailability::NotKept,
    });
    for (rows, admitted) in [(1, false), (2, true), (3, true)] {
        let mut bound = bounds();
        bound.rows = rows;
        assert_eq!(document_fits(&document, &bound), admitted);
    }
}
#[test]
fn connector_values_paths_reject_actual_empty_names_and_preserve_opaque_admitted_segments() {
    assert!(Path::new(list(0, []), 16).is_none());
    assert!(Path::new(list(1, [Box::<[u8]>::from([])]), 16).is_none());
    let path = path();
    assert_eq!(path.segments()[0].as_ref(), b"literal/slash");
    assert_eq!(path.segments()[1].as_ref(), [0xff, 0]);
    assert!(path.under(&path.clone()));
    let different = Path::new(list(3, [b"literal".as_slice().into(), b"slash".as_slice().into()]), 16).unwrap();
    assert_ne!(path, different);
    assert!(!path.under(&different));
}
#[test]
fn connector_values_scoped_outbox_subjects_and_raw_effect_references_remain_independent() {
    let mut document = carrier(ClientDocument::Outbox { entry: entry(), description: description() });
    for scope in [Scope::Project(0), Scope::Project(3), Scope::Project(4), Scope::Deployment] {
        document.subject.scope = scope;
        assert_eq!(document.clone().subject.scope, scope);
        let ClientDocument::Outbox { entry, .. } = document.value.as_ref() else { panic!("outbox") };
        assert_eq!(entry.number, u64::MAX);
    }
    let original_subject = document.subject;
    *document.value = ClientDocument::Effect { proposal: 87, task: 42, effect: effect(), description: description() };
    assert_eq!(document.subject, original_subject, "supplier never invents a jig maker from an owner reference");
}
#[test]
fn connector_values_every_terminal_retains_the_complete_value_and_exact_refusal() {
    use application_core::{Lack, Refusal};
    for value in [
        ApplicationOutcome::Changed {
            object: u64::MAX,
            now: ApplicationObject { revision: u64::MAX, value: u64::MAX },
        },
        ApplicationOutcome::Moved { object: 2, now: ApplicationObject { revision: 3, value: 4 } },
        ApplicationOutcome::Role { held: u32::MAX },
        ApplicationOutcome::Unknown,
        ApplicationOutcome::Exhausted,
        ApplicationOutcome::CoreRefused {
            why: Refusal::Authority { lacking: Lack::Budget { needs: u64::MAX, left: u64::MAX - 1 }, proposable: None },
        },
    ] {
        let answer = ConnectorAnswer { value };
        assert_eq!(answer.value, value);
        let copied = answer;
        assert_eq!(copied, answer);
        let mut bound = bounds();
        for (bytes, admitted) in
            [(answer_bytes(&answer) - 1, false), (answer_bytes(&answer), true), (answer_bytes(&answer) + 1, true)]
        {
            bound.bytes = bytes;
            assert_eq!(answer_fits(&answer, &bound), admitted);
        }
    }
}
#[test]
fn connector_values_the_full_panel_attains_the_priced_structural_worst_case() {
    let bound = Bounds { rows: 2, paths: 2, segments: 2, segment_bytes: 12, bytes: u64::MAX };
    let full_path = || Path::new(list(2, [Box::<[u8]>::from([0xff; 12]), Box::<[u8]>::from([0xff; 12])]), 12).unwrap();
    let mut outbox = entry();
    outbox.effect.resources = Box::new([full_path(), full_path()]);
    let mut resource = resource();
    resource.path = full_path();
    let mut procedure = procedure();
    procedure.resource = full_path();
    let value = carrier(ClientDocument::Panel {
        task: 0,
        procedure: Some(procedure),
        result: None,
        resources: list(2, [resource.clone(), resource]),
        effects: list(2, [outbox.clone(), outbox]),
    });
    assert!(document_fits(&value, &bound));
    assert_eq!(document_bytes(&value), document_worst_case(&bound));
    let mut overflow = bound;
    overflow.rows = u32::MAX;
    overflow.paths = u32::MAX;
    overflow.segments = u32::MAX;
    overflow.segment_bytes = u32::MAX;
    assert!(document_worst_case(&overflow).is_none());
}

mod deployment_diagnostics;
