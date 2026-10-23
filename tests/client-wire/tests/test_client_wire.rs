//! The committed client codec and bytes against its schema (skein's codec.md, 6).
use skein_codegen::{generate, parse};
use std::{fs, path::Path};

#[test]
fn test_client_wire_schema_matches_generated_code_and_goldens() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testing/jig-client-test-protocol-wire");
    let source = fs::read_to_string(root.join("schema/test-client-v2.schema")).expect("client schema");
    let output = generate(&parse(&source).expect("schema valid"));
    let code = root.join("src/generated/v2.rs");
    let goldens = root.join("golden/v2");
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    let again = "run SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-client-wire-world --test test_client_wire";
    if write {
        fs::write(&code, &output.rust).expect("write generated code");
        fs::create_dir_all(&goldens).expect("create golden directory");
    } else {
        assert_eq!(fs::read_to_string(&code).expect("generated code committed"), output.rust, "{again}");
    }
    for golden in output.goldens {
        let path = goldens.join(golden.name);
        if write {
            fs::write(&path, golden.bytes).expect("write golden bytes");
        } else {
            assert_eq!(fs::read(&path).expect("golden bytes committed"), golden.bytes, "{again}");
        }
    }
}

mod fixtures;
use jig_client_test_domain::family;

#[test]
fn a_scripted_item_round_trips_with_every_field_and_state() {
    for state in [family::ItemState::Ready, family::ItemState::Held, family::ItemState::Ended] {
        fixtures::assert_document(family::Document::Item(fixtures::item(state)));
    }
}
#[test]
fn a_scripted_part_round_trips_without_parameters() {
    fixtures::assert_document(family::Document::Part(fixtures::part(None)));
}
#[test]
fn a_scripted_part_round_trips_with_parameters() {
    let parameters =
        family::Parameters { item: fixtures::ITEM, revision: fixtures::REVISION, value: 0, amendable: false };
    fixtures::assert_document(family::Document::Part(fixtures::part(Some(parameters))));
}

#[test]
fn a_connector_document_round_trips_with_its_supplied_subject() {
    use jig_client_test_domain::connector_values::{ApplicationObject, ClientDocument, ConnectorDocument};
    let value = ConnectorDocument {
        subject: family::Subject {
            scope: family::Scope::Project(fixtures::PROJECT),
            kind: family::Kind::Item(fixtures::ITEM),
        },
        value: Box::new(ClientDocument::Object {
            object: fixtures::ITEM,
            now: ApplicationObject { revision: fixtures::REVISION.value, value: 42 },
        }),
    };
    fixtures::assert_document(family::Document::Connector(value));
}

#[test]
fn a_scripted_touched_answer_round_trips() {
    fixtures::assert_answer(family::Answer::Touched { item: fixtures::ITEM, revision: fixtures::REVISION });
}
#[test]
fn a_scripted_moved_answer_round_trips() {
    fixtures::assert_answer(family::Answer::Moved { item: fixtures::ITEM, revision: fixtures::REVISION });
}
#[test]
fn a_scripted_gone_answer_round_trips() {
    fixtures::assert_answer(family::Answer::Gone);
}
#[test]
fn a_connector_answer_round_trips() {
    use jig_client_test_domain::connector_values::{ApplicationObject, ApplicationOutcome, ConnectorAnswer};
    fixtures::assert_answer(family::Answer::Connector(ConnectorAnswer {
        value: ApplicationOutcome::Moved {
            object: fixtures::ITEM,
            now: ApplicationObject { revision: fixtures::REVISION.value, value: 42 },
        },
    }));
}
#[test]
fn an_entrance_refusal_has_no_application_wire_record() {
    use jig_client_domain::contract::{EntranceRefusal, Refusal};
    let refusal = EntranceRefusal::new(Refusal::Unknown).expect("fixed entrance refusal");
    assert_eq!(
        family::wire::encode_answer(&family::Answer::Refused(refusal)),
        Err(family::wire::Invalid::EntranceRefusal)
    );
}

#[test]
fn a_scripted_touch_round_trips_without_parameters() {
    fixtures::assert_touch(None);
}
#[test]
fn a_scripted_touch_round_trips_with_parameters() {
    fixtures::assert_touch(Some(0));
}
#[test]
fn every_project_subject_round_trips_through_reads_and_opens() {
    for kind in fixtures::SUBJECT_KINDS {
        fixtures::assert_subject(family::Subject { scope: family::Scope::Project(fixtures::PROJECT), kind });
    }
}
#[test]
fn every_deployment_subject_round_trips_through_reads_and_opens() {
    for kind in fixtures::SUBJECT_KINDS {
        fixtures::assert_subject(family::Subject { scope: family::Scope::Deployment, kind });
    }
}
