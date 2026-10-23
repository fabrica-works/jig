//! Shared client fixtures for the settings contract; tasks B/C supply the engine maps.
use jig_client_domain::contract as c;
use jig_client_protocol_link::{Limits, translate};
use jig_client_protocol_wire::v18 as w;
use jig_client_test_records::settings as values;
use skein_lib::{List, Writer};
macro_rules! encoded {
    ($value:expr) => {{
        let value = $value;
        let mut writer = Writer::new(usize::try_from(value.measure()).expect("wire size"));
        value.encode(&mut writer).expect("measured fixture");
        writer.finish()
    }};
}
fn answer(value: &w::Final, limits: &Limits) -> c::Answer {
    let record = encoded!(value);
    let owned =
        w::Owned::new(&limits.wire, w::OwnedParts { owner: w::Owner::Jig, version: 18, record }).expect("owned answer");
    translate::answers::answer(w::KeyedAnswer::FinalAnswer(owned), limits).expect("public answer")
}
fn outcome(value: &w::Outcome, limits: &Limits) -> Result<c::Outcome, ()> {
    let final_value =
        w::Final::Done(w::Done::new(&limits.wire, w::DoneParts { outcome: value.clone() }).expect("done"));
    match answer(&final_value, limits) {
        c::Answer::Done(value) => Ok(value),
        c::Answer::Refused { .. } => Err(()),
    }
}
fn refusal(value: &w::Refusal, limits: &Limits) -> Result<c::Refusal, ()> {
    let final_value = w::Final::Refused(
        w::Refused::new(&limits.wire, w::RefusedParts { refusal: value.clone(), at: None }).expect("refused"),
    );
    match answer(&final_value, limits) {
        c::Answer::Refused { refusal, .. } => Ok(refusal),
        c::Answer::Done(_) => Err(()),
    }
}
#[test]
fn settings_contract_fixtures_keep_the_ask_answers_documents_and_selectors() {
    let mut client = jig_client_world::test::limits().client;
    client.domain.rows = 8;
    let limits = Limits::new(client.domain, client.exchanges).expect("client limits");
    let t = &limits.wire;
    let ask = c::Ask::ChangeSettings(values::settings_change());
    let wire = translate::asks::ask(&ask, t).expect("settings ask");
    // The retained ask follows the same contract as a newly sent one.
    let w::Ask::ChangeSettings(wire) = wire else {
        panic!("settings ask");
    };
    assert_eq!(translate::settings::restore_change_settings(&wire, &limits), Ok(values::settings_change()));
    for read in [
        c::Read::Settings,
        c::Read::Setting(c::ReadSetting { setting: c::Setting::Rules, revision: 6 }),
        c::Read::Revision(c::ReadRevision { number: 6 }),
        c::Read::Revisions(c::ReadRevisions { before: Some(8) }),
    ] {
        translate::reads::read(&read, t).expect("read fixture");
    }
    assert_eq!(translate::watches::subject(&c::Subject::Settings, t).expect("settings watch"), w::Subject::Settings);
    let changed = w::Outcome::SettingsChanged(
        w::OutcomeSettingsChanged::new(t, w::OutcomeSettingsChangedParts { revision: 6 }).expect("changed"),
    );
    assert_eq!(outcome(&changed, &limits), Ok(c::Outcome::SettingsChanged(c::OutcomeSettingsChanged { revision: 6 })));
    let stale = w::Refusal::Stale(w::RefusalStale::new(t, w::RefusalStaleParts { newest: 9 }).expect("stale"));
    assert_eq!(refusal(&stale, &limits), Ok(c::Refusal::Stale(c::RefusalStale { newest: 9 })));
    let mut tasks = List::with_capacity(1);
    tasks.push(11).expect("one holder");
    let full = w::Refusal::Full(
        w::RefusalFull::new(t, w::RefusalFullParts { revision: 2, tasks: tasks.clone(), more: 3 }).expect("full"),
    );
    assert_eq!(refusal(&full, &limits), Ok(c::Refusal::Full(c::RefusalFull { revision: 2, tasks, more: 3 })));
    for fault in [
        c::SettingFault::Shape,
        c::SettingFault::Count,
        c::SettingFault::Reference,
        c::SettingFault::Rules,
        c::SettingFault::Budget,
        c::SettingFault::Grace,
        c::SettingFault::Owner,
    ] {
        let why = c::MalformedSettings { setting: c::Setting::Rules, fault };
        let wire = w::Refusal::Malformed(
            w::RefusalMalformed::new(
                t,
                w::RefusalMalformedParts {
                    at: 0,
                    why: w::Malformed::Settings(translate::settings::wire_malformed_settings(&why, t).expect("fault")),
                },
            )
            .expect("refusal"),
        );
        assert_eq!(refusal(&wire, &limits), Ok(c::Refusal::Malformed { at: 0, why: c::Malformed::Settings(why) }));
    }
    let unchanged = w::Refusal::Malformed(
        w::RefusalMalformed::new(t, w::RefusalMalformedParts { at: 0, why: w::Malformed::SettingsUnchanged })
            .expect("unchanged"),
    );
    assert_eq!(refusal(&unchanged, &limits), Ok(c::Refusal::Malformed { at: 0, why: c::Malformed::SettingsUnchanged }));
    let document = values::settings_document();
    let settings = translate::settings::wire_settings_document(&document, t).expect("settings");
    let bytes = encoded!(&w::ReadValue::Settings(settings.clone()));
    assert_eq!(translate::reads::value(&bytes, &limits), Ok(c::ReadValue::Settings(Box::new(document))));
    let bytes = encoded!(&w::DeliveryValue::Settings(settings));
    assert_eq!(
        translate::watches::delivery(&bytes, &limits),
        Ok(c::Delivery::Settings(Box::new(values::settings_document())))
    );
    let mut value = values::settings_values().into_boxed().into_vec();
    let document = c::SettingDocument { setting: c::Setting::Rules, revision: 6, value: value.pop().expect("rules") };
    let bytes =
        encoded!(&w::ReadValue::Setting(translate::settings::wire_setting_document(&document, t).expect("setting")));
    assert_eq!(translate::reads::value(&bytes, &limits), Ok(c::ReadValue::Setting(Box::new(document))));
    for answer in [
        c::RevisionAnswer::Revision(values::settings_revision()),
        c::RevisionAnswer::Forgotten(c::RevisionForgotten { oldest: 4 }),
        c::RevisionAnswer::Unknown,
    ] {
        let bytes = encoded!(&w::ReadValue::Revision(
            w::ReadRevisionAnswer::new(
                t,
                w::ReadRevisionAnswerParts {
                    value: translate::settings::wire_revision_answer(&answer, t).expect("revision")
                }
            )
            .expect("revision answer")
        ));
        assert_eq!(translate::reads::value(&bytes, &limits), Ok(c::ReadValue::Revision(Box::new(answer))));
    }
    let page = values::settings_page();
    let bytes = encoded!(&w::ReadValue::Revisions(translate::settings::wire_revisions_page(&page, t).expect("page")));
    assert_eq!(translate::reads::value(&bytes, &limits), Ok(c::ReadValue::Revisions(Box::new(page))));
}
