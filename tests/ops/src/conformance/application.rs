//! Ops-owned encodings and evidence from peers and the fake store.
use super::{observations::Store, r};
use jig_core as core;
use jig_core_people as people;
use jig_fake_store::Write;
use jig_ops_domain as root;
use jig_ops_domain_infrastructure as infra;
use skein_lib::{ReplyTo, Token};

pub fn requested(store: &Store, event: &mut root::Event) -> Option<r::Observed> {
    let root::Event::Application { reply_to, sign_in, key, digest, .. } = event else { return None };
    let person = store.rows.values().find_map(|row| match row {
        root::Record::Core(core::Record::People(people::Stored::SignIn { number, person, .. }))
            if number == sign_in =>
        {
            Some(*person)
        }
        root::Record::Core(_) | root::Record::Infrastructure(_) | root::Record::Observability(_) => None,
    })?;
    let reply = std::mem::replace(reply_to, ReplyTo::new(Token::new(0))).into_token();
    *reply_to = ReplyTo::new(reply);
    Some(r::Observed::ApplicationRequested {
        reply: reply.raw(),
        key: r::ApplicationKey { person, key: *key },
        connector: 2,
        digest: *digest,
    })
}

fn marker(row: &root::Record, token: u64) -> Option<(r::ApplicationKey, [u8; 32])> {
    if let root::Record::Core(core::Record::People(people::Stored::Answer {
        key,
        ask,
        outcome: people::Outcome::Applied { connector: 2, token: saved },
        ..
    })) = row
        && let people::Ask::Application { connector: 2, digest, .. } = ask.as_ref()
        && saved.raw() == token
    {
        Some((r::ApplicationKey { person: key.person, key: key.key }, *digest))
    } else {
        None
    }
}

pub fn submitted(store: &Store) -> Vec<r::Observed> {
    let Some(pending) = store.pending.back() else { return Vec::new() };
    let mut observed = Vec::new();
    for write in &pending.writes {
        if let Write::Save {
            row: root::Record::Infrastructure(infra::Record::ApplicationAnswer { token, until, outcome }),
            ..
        } = write
        {
            let (key, digest) = pending
                .writes
                .iter()
                .find_map(|write| match write {
                    Write::Save { row, .. } => marker(row, token.raw()),
                    Write::Erase(_) => None,
                })
                .or_else(|| store.rows.values().find_map(|row| marker(row, token.raw())))
                .expect("infrastructure terminal has its shared key metadata");
            observed.push(r::Observed::ApplicationSubmitted {
                commit: pending.number,
                answer: r::ApplicationAnswer {
                    key,
                    connector: 2,
                    digest,
                    token: token.raw(),
                    until: until.as_nanos(),
                    outcome: encoded(outcome),
                },
            });
        }
    }
    observed
}

pub fn encoded(outcome: &infra::ApplicationOutcome) -> Box<[u8]> {
    format!("{outcome:?}").into_bytes().into_boxed_slice()
}
