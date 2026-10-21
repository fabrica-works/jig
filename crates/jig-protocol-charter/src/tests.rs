use alloc::boxed::Box;
use skein_lib::{Duration, Reader};
use smith_charter::v4 as wire;

use crate::{
    Budget, Charter, Contract, EndpointName, Model, Prices, Reserve, Section, TextRule, Tool, ToolEffect, charter,
    encode,
};

fn source() -> Charter {
    let model = Model {
        prices: Prices { input: 2, cached: 1, cache_write: Some(4), output: 3, unit: 1000 },
        dialect: 4,
        endpoint: 7,
        name: Box::from(&b"small"[..]),
        window: 4096,
        output: 128,
        effort: Some(Box::from(&b"low"[..])),
    };
    Charter {
        mode: smith_domain_run::Mode::Finite,
        instructions: Box::from(&b"help"[..]),
        tools: Box::new([Tool {
            name: Box::from(&b"read"[..]),
            description: Box::from(&b"Read a value"[..]),
            schema: Box::from(&b"{}"[..]),
            effect: ToolEffect::Read,
            timeout: Duration::ZERO,
        }]),
        wait: true,
        agents: false,
        workspace: crate::WorkspaceTools { inspect: false, modify: false, shell: false },
        conventions: None,
        contract: Contract {
            report: Some(TextRule { max: 512, fields: Box::new([]) }),
            failure: None,
            verdicts: Box::new([]),
            change: None,
        },
        budget: Budget {
            turns: 3,
            spend: 9000,
            time: Duration::from_secs(60),
            reserve: Reserve { turns: 1, time: Duration::from_secs(10) },
        },
        model: crate::Llm { model, grant: Some(5) },
        models: Box::new([]),
        waiting: Duration::ZERO,
        resumes: true,
    }
}

#[test]
fn core_charter_encodes_with_the_same_sections_tools_contract_and_model() {
    let source = source();
    let sections = Box::new([Section { title: Box::from(&b"Task"[..]), text: Box::from(&b"Do it"[..]) }]);
    let charter = charter(source, sections, false).expect("bounded charter");
    let names = [EndpointName { number: 7, dialect: 4, name: Box::from(&b"primary"[..]) }];
    let bytes = encode(charter, &names, &wire::CEILINGS).expect("bounded encoding");
    let decoded = wire::Charter::decode(&wire::CEILINGS, &mut Reader::new(&bytes)).expect("Smith codec accepts it");
    assert_eq!(decoded.instructions(), b"help");
    assert_eq!(decoded.brief().get(0).expect("one section").title(), b"Task");
    assert_eq!(decoded.tools().host().get(0).expect("one tool").name(), b"read");
    assert_eq!(decoded.main().endpoint(), b"primary");
    assert_eq!(decoded.budget().spend(), 9000);
    assert_eq!(decoded.budget().reserve().turns(), 1);
    assert_eq!(decoded.budget().reserve().time(), Duration::from_secs(10));
    assert_eq!(decoded.main().window(), 4096);
    assert_eq!(decoded.main().output(), 128);
    assert_eq!(*decoded.main().prices().cache_write(), Some(4));
    assert_eq!(decoded.main().effort().as_deref(), Some(b"low".as_slice()), "the choice's effort reaches smith");
    assert!(!decoded.resume(), "a resume requires a validated transcript");
}

fn call_name() -> Box<[u8]> {
    let name = smith_channel::CallName::new(
        &smith_channel::CEILINGS,
        smith_channel::CallNameParts { activation: 7, completion: 3, position: 2 },
    )
    .expect("a call name within its ceilings");
    let mut writer = skein_lib::Writer::new(16);
    name.encode(&mut writer).expect("sixteen bytes");
    writer.finish()
}

#[test]
fn a_call_key_reads_completion_and_position() {
    assert_eq!(crate::call_key(&call_name()), Some(crate::CallKey { activation: 7, completion: 3, position: 2 }));
}

#[test]
fn a_call_name_with_trailing_bytes_has_no_key() {
    let bytes = call_name();
    let mut writer = skein_lib::Writer::new(17);
    writer.put(&bytes).expect("room for the name");
    writer.put(&[0]).expect("room for a trailing byte");
    assert_eq!(crate::call_key(&writer.finish()), None);
}

fn result_bytes(form: wire::Form, label: Option<Box<[u8]>>, fields: skein_lib::List<wire::Field>) -> Box<[u8]> {
    let value = wire::RunResult::new(
        &wire::CEILINGS,
        wire::RunResultParts {
            form,
            label,
            text: Box::from(b"finished".as_slice()),
            fields,
            items: skein_lib::List::with_capacity(0),
        },
    )
    .expect("bounded result");
    let mut writer = skein_lib::Writer::new(usize::try_from(value.measure()).expect("measured size fits usize"));
    value.encode(&mut writer).expect("measured encoding");
    writer.finish()
}

#[test]
fn a_report_result_reads_its_words() {
    let bytes = result_bytes(wire::Form::Report, None, skein_lib::List::with_capacity(0));
    assert_eq!(crate::result(&bytes), Ok(crate::RunResult::Report { words: Box::from(b"finished".as_slice()) }));
}

#[test]
fn a_verdict_result_reads_its_label_and_fields() {
    let mut fields = skein_lib::List::with_capacity(1);
    fields
        .push(
            wire::Field::new(
                &wire::CEILINGS,
                wire::FieldParts { name: Box::from(b"status".as_slice()), text: Box::from(b"green".as_slice()) },
            )
            .expect("bounded field"),
        )
        .expect("one field");
    let bytes = result_bytes(wire::Form::Verdict, Some(Box::from(b"ready".as_slice())), fields);
    assert_eq!(
        crate::result(&bytes),
        Ok(crate::RunResult::Verdict {
            label: Box::from(b"ready".as_slice()),
            words: Box::from(b"finished".as_slice()),
            fields: Box::new([crate::Field {
                name: Box::from(b"status".as_slice()),
                value: Box::from(b"green".as_slice())
            }]),
        })
    );
}

#[test]
fn a_change_result_does_not_decode() {
    let bytes = result_bytes(wire::Form::Change, None, skein_lib::List::with_capacity(0));
    assert_eq!(crate::result(&bytes), Err(crate::Unreadable::Form));
}

#[test]
fn a_truncated_result_does_not_decode() {
    let bytes = result_bytes(wire::Form::Report, None, skein_lib::List::with_capacity(0));
    let end = bytes.len().checked_sub(1).expect("a result has bytes");
    let short = bytes.get(..end).expect("remove the final byte");
    assert_eq!(crate::result(short), Err(crate::Unreadable::Malformed));
}

#[test]
fn a_failure_result_reads_its_reason() {
    let bytes = result_bytes(wire::Form::Failure, None, skein_lib::List::with_capacity(0));
    assert_eq!(crate::result(&bytes), Ok(crate::RunResult::Failure { reason: Box::from(b"finished".as_slice()) }));
}

#[test]
fn a_result_with_trailing_bytes_does_not_decode() {
    let bytes = result_bytes(wire::Form::Report, None, skein_lib::List::with_capacity(0));
    let mut writer = skein_lib::Writer::new(bytes.len().checked_add(1).expect("one extra byte"));
    writer.put(&bytes).expect("room for the result");
    writer.put(&[0]).expect("room for a trailing byte");
    assert_eq!(crate::result(&writer.finish()), Err(crate::Unreadable::Malformed));
}

#[test]
fn a_verdict_without_a_label_does_not_decode() {
    let bytes = result_bytes(wire::Form::Verdict, None, skein_lib::List::with_capacity(0));
    assert_eq!(crate::result(&bytes), Err(crate::Unreadable::Form));
}

#[test]
fn a_report_with_a_label_does_not_decode() {
    let bytes =
        result_bytes(wire::Form::Report, Some(Box::from(b"ready".as_slice())), skein_lib::List::with_capacity(0));
    assert_eq!(crate::result(&bytes), Err(crate::Unreadable::Form));
}

#[test]
fn a_verdict_with_items_does_not_decode() {
    let mut items = skein_lib::List::with_capacity(1);
    items
        .push(
            wire::Item::new(
                &wire::CEILINGS,
                wire::ItemParts { kind: Box::from(b"follow".as_slice()), fields: skein_lib::List::with_capacity(0) },
            )
            .expect("bounded item"),
        )
        .expect("one item");
    let value = wire::RunResult::new(
        &wire::CEILINGS,
        wire::RunResultParts {
            form: wire::Form::Verdict,
            label: Some(Box::from(b"ready".as_slice())),
            text: Box::new([]),
            fields: skein_lib::List::with_capacity(0),
            items,
        },
    )
    .expect("bounded result");
    let mut writer = skein_lib::Writer::new(usize::try_from(value.measure()).expect("measured size fits usize"));
    value.encode(&mut writer).expect("measured encoding");
    assert_eq!(crate::result(&writer.finish()), Err(crate::Unreadable::Form));
}

#[test]
fn a_conversations_attempts_share_one_identity() {
    let deployment = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let expected = [0x06, 0x8b, 0xfc, 0xde, 0x3e, 0x7d, 0xde, 0x1c, 0x53, 0xe8, 0x5c, 0x10, 0xf4, 0x7d, 0xef, 0x5d];
    for _attempt in [3_u64, 4, 9] {
        assert_eq!(crate::run_identity(&deployment, 7, 3), expected);
    }
}

#[test]
fn two_conversations_of_a_task_have_two_identities() {
    let deployment = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    assert_eq!(
        crate::run_identity(&deployment, 7, 3),
        [0x06, 0x8b, 0xfc, 0xde, 0x3e, 0x7d, 0xde, 0x1c, 0x53, 0xe8, 0x5c, 0x10, 0xf4, 0x7d, 0xef, 0x5d]
    );
    assert_eq!(
        crate::run_identity(&deployment, 7, 4),
        [0x92, 0xf3, 0x7d, 0x0f, 0x30, 0xe6, 0xf6, 0x6f, 0x8f, 0xee, 0x8a, 0x01, 0x68, 0x18, 0x8f, 0x3a]
    );
}

#[test]
fn selected_source_grants_are_independent_of_endpoint_resolution() {
    let names = [EndpointName { number: 7, dialect: 4, name: b"primary".as_slice().into() }];
    for main in [Some(5), Some(19), None] {
        let mut source = source();
        source.model.grant = main;
        let mut other = source.model.clone();
        other.grant = Some(23);
        source.models = Box::new([source.model.clone(), other]);
        let translated = charter(source, Box::new([]), true).expect("runtime model bindings");
        let bytes = encode(translated, &names, &wire::CEILINGS).expect("grants do not resolve endpoints");
        let decoded = wire::Charter::decode(&wire::CEILINGS, &mut Reader::new(&bytes)).expect("charter");
        assert_eq!(*decoded.main().account(), main);
        assert_eq!(*decoded.models().get(0).unwrap().account(), main);
        assert_eq!(*decoded.models().get(1).unwrap().account(), Some(23));
    }
}

#[test]
fn endpoint_resolution_still_refuses_wrong_dialects_and_unknown_numbers() {
    let names = [EndpointName { number: 7, dialect: 4, name: b"primary".as_slice().into() }];
    for (number, dialect) in [(8, 4), (7, 3)] {
        let mut source = source();
        source.model.model.endpoint = number;
        source.model.model.dialect = dialect;
        let translated = charter(source, Box::new([]), true).expect("bounded charter");
        assert!(encode(translated, &names, &wire::CEILINGS).is_none());
    }
}

#[test]
fn open_mode_is_encoded_and_never_grants_delivery() {
    let mut value = source();
    value.mode = smith_domain_run::Mode::Open;
    value.contract.change = Some(crate::ChangeRule { checks_must_pass: false, fields: Box::new([]) });
    let value = charter(value, Box::new([]), false).unwrap();
    assert_eq!(value.mode, smith_domain_run::Mode::Open);
    assert!(value.grants.deliver.is_none());
    assert!(value.grants.wait);
}
