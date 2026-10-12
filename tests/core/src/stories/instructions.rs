//! Starting facts for a charter instruction amendment and cold continuation.
use crate::{effects::World, reads::Parties};
use jig_domain_core::{self as core, documents};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

#[must_use]
pub fn world(seed: u64) -> World {
    let mut w = World::new(seed, false);
    w.reads = Some(Parties::new(w.store.applied));
    w
}
#[must_use]
pub fn row(w: &World, task: u64) -> &tasks::TaskRecord {
    match w.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Live(task)))) {
        Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))) => row,
        other => panic!("live task: {other:?}"),
    }
}
#[must_use]
pub fn change(words: &[u8]) -> people::Amendment {
    people::Amendment {
        addressed: None,
        instructions: Some(words.into()),
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        reason: b"new evidence".as_slice().into(),
    }
}
#[must_use]
pub fn task_document(w: &mut World, task: u64, request: u64) -> documents::Task {
    crate::reads::ask(w, request, core::reads::Read::Task { task });
    let core::reads::Answer::Document(document) = crate::reads::latest(w) else {
        panic!("document read");
    };
    let documents::Document::Task(doc) = &**document else {
        panic!("task document");
    };
    *doc.clone()
}
pub fn sign_in(w: &mut World) {
    w.send(root::Event::Core(core::Event::Account(jig_domain_accounts::Event::Add {
        account: 1,
        generation: 1,
        valid: Some(skein_lib::Duration::from_secs(60)),
    })));
    w.send(root::Event::Core(core::Event::Fleet(jig_domain_fleet::Event::Hello {
        channel: Token::new(7),
        hello: jig_domain_fleet::Hello {
            stop_bound: skein_lib::Duration::from_millis(100),
            slots: 2,
            workstreams: Box::new([]),
            hosting: Box::new([]),
        },
    })));
    w.send(root::Event::Core(core::Event::SignIn {
        reply_to: ReplyTo::new(Token::new(70_010)),
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
            login: b"person".as_slice().into(),
            name: b"Person".as_slice().into(),
        },
    }));
}
