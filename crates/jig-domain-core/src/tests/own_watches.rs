//! The core names own watch subjects from live and restored sign-ins (protocol/client.md, 7).
use super::*;

fn identity(party: u8) -> people::Identity {
    people::Identity {
        key: people::IdentityKey { provider: 0, subject: Box::new([party]) },
        login: Box::from(&b"watcher"[..]),
        name: Box::from(&b"Watcher"[..]),
    }
}

fn sign_in(core: &mut Core, limits: &Limits, party: u8) -> (u64, Box<[Record]>) {
    let out = watch_outputs(
        core,
        limits,
        Event::SignIn { reply_to: ReplyTo::new(Token::new(90)), identity: identity(party) },
    );
    let mut sign_in = None;
    let mut records = List::with_capacity(u32::try_from(out.len()).expect("bounded sign-in outputs"));
    for request in out {
        match request {
            Request::Write(Write::Save(record)) => records.push(record).expect("decision records"),
            Request::Held(value) => {
                if let Held::PeopleReply { reply: people::Reply::SignedIn { sign_in: number, .. }, .. } = *value {
                    sign_in = Some(number);
                }
            }
            Request::Write(Write::Erase(_)) | Request::Ask { .. } | Request::Now(_) | Request::Decided => {}
        }
    }
    (sign_in.expect("live sign-in reply"), records.into_boxed())
}

fn signed_core() -> (Core, Limits, u64, Box<[Record]>) {
    let limits = limits();
    let mut configuration = config(41);
    configuration.owners = Box::new([]);
    let mut core = Core::new(configuration, &limits);
    drop(watch_outputs(&mut core, &limits, Event::People(people::Event::Restored)));
    let (sign_in, records) = sign_in(&mut core, &limits, 1);
    (core, limits, sign_in, records)
}

fn open(core: &mut Core, limits: &Limits, sign_in: u64, watcher: u64, subject: Watched) -> Box<[Request]> {
    watch_outputs(core, limits, Event::Watch { watcher: Token::new(watcher), sign_in, subject, window: window() })
}

fn named(core: &Core, watcher: u64) -> &Subject {
    let binding = core.view_subjects.get(&Token::new(watcher)).expect("admitted watch");
    &core.view_names.get(binding.subject).expect("bound subject").subject
}

fn frame_party(out: &[Request]) -> u64 {
    let Request::Held(value) = &out[0] else { panic!("held delivery") };
    let Held::Delivery { document, .. } = value.as_ref() else { panic!("document delivery") };
    let Document::Frame(frame) = document.as_ref() else { panic!("frame delivery") };
    frame.party
}

#[test]
fn an_own_subject_names_the_sign_ins_party() {
    let (mut core, limits, sign_in, _) = signed_core();
    for (label, subject, expected) in
        [("Frame", Watched::Frame, Subject::Frame { party: 1 }), ("Inbox", Watched::Inbox, Subject::Inbox { party: 1 })]
    {
        let out = open(&mut core, &limits, sign_in, 30, subject);
        assert_eq!(named(&core, 30), &expected, "{label}");
        assert_eq!(out.len(), 2, "{label}");
        drop(watch_outputs(&mut core, &limits, Event::Unwatch { watcher: Token::new(30) }));
        drop(watch_outputs(&mut core, &limits, Event::Delivered { watcher: Token::new(30) }));
    }
}

#[test]
fn an_own_subject_after_a_restart_names_the_restored_sign_ins_party() {
    let (original, limits, sign_in, records) = signed_core();
    let mut restored = Core::new(config(42), &limits);
    assert_eq!(restored.restart_begin(), RestartRequest::Step(RestartStep::LoadCore));
    let header = original.counters.deployment();
    assert_eq!(
        restored.restore_core(CoreRecord::Deployment(header), &limits),
        Restored::Deployment { commits: header.commits }
    );
    for record in records {
        match record {
            Record::People(record) => {
                drop(watch_outputs(&mut restored, &limits, Event::People(people::Event::Restore { record })));
            }
            Record::Core(record) => {
                assert_ne!(restored.restore_core(record, &limits), Restored::Rejected);
            }
            Record::Tasks(_) | Record::Notes(_) => panic!("sign-in saves only its people and header"),
        }
    }
    drop(watch_outputs(&mut restored, &limits, Event::People(people::Event::Restored)));
    assert_eq!(restored.restart_done(RestartStep::LoadCore), RestartRequest::Step(RestartStep::LoadInbox));
    let out = open(&mut restored, &limits, sign_in, 30, Watched::Frame);
    assert_eq!(named(&restored, 30), &Subject::Frame { party: 1 });
    assert_eq!(frame_party(&out), 1);
}

#[test]
fn two_parties_frames_stay_two_subjects() {
    let (mut core, limits, first, _) = signed_core();
    let (second, _) = sign_in(&mut core, &limits, 2);
    let first_out = open(&mut core, &limits, first, 30, Watched::Frame);
    let second_out = open(&mut core, &limits, second, 31, Watched::Frame);
    assert_eq!(named(&core, 30), &Subject::Frame { party: 1 });
    assert_eq!(named(&core, 31), &Subject::Frame { party: 2 });
    assert_eq!(core.view_names.len(), 2);
    assert_eq!(frame_party(&first_out), 1);
    assert_eq!(frame_party(&second_out), 2);
}

#[test]
fn an_own_subject_of_an_ended_sign_in_is_refused_signed_out() {
    let (mut core, limits, sign_in, _) = signed_core();
    drop(watch_outputs(
        &mut core,
        &limits,
        Event::People(people::Event::SignOut { sign_in, reply_to: ReplyTo::new(Token::new(91)) }),
    ));
    for (label, subject) in [("Frame", Watched::Frame), ("Inbox", Watched::Inbox)] {
        let out = open(&mut core, &limits, sign_in, 30, subject);
        assert_watch_refusal(&out[0], WatchRefusal::SignedOut);
        assert!(core.view_subjects.is_empty(), "{label}");
    }
}
