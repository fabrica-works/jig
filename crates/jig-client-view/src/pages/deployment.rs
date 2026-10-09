//! Whole host, account and capacity facts, with connector slots (pages.md, 7).
use crate::{
    Builder, JigBindings, Role, Run, cards,
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};
use jig_client::Domain;
/// Show the admitted deployment document and its connector-owned parts.
pub fn deployment(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings) {
    super::heading(b, words, JigPhrase::Deployment, &[]);
    let Some(doc) = &client.deployment_page().document else {
        words.status(b, JigPhrase::LoadingObject, &[]);
        return;
    };
    b.open(Role::List);
    b.name(&words.say(JigPhrase::Deployment, &[]));
    for host in &doc.hosts {
        b.open(Role::Item);
        b.key(crate::Key { owner: crate::Owner::Jig, kind: 22, a: host.number, b: 0 });
        b.name(&words.say(JigPhrase::HostNumber, &[Hole::Number(host.number)]));
        words.text(b, JigPhrase::HostNumber, &[Hole::Number(host.number)]);
        words.status(b, if host.connected { JigPhrase::HostConnected } else { JigPhrase::HostDisconnected }, &[]);
        if let Some(grace) = host.grace {
            words.text(b, JigPhrase::Grace, &[Hole::Wall(words.adjusted_wall(grace))]);
        }
        words.text(b, JigPhrase::SlotsUsed, &[Hole::Number(u64::from(host.used))]);
        words.text(b, JigPhrase::SlotsFree, &[Hole::Number(u64::from(host.free))]);
        for chip in &host.runs {
            cards::chip::chip(b, client, chip, words, bindings);
        }
        if let Some(refused) = &host.refused {
            words.text(b, JigPhrase::Refused, &[]);
            b.text(Run::Plain, refused.bytes());
        }
        b.close();
    }
    b.close();
    b.open(Role::List);
    for account in &doc.accounts {
        b.open(Role::Item);
        b.key(crate::Key { owner: crate::Owner::Jig, kind: 23, a: u64::from(account.number), b: 0 });
        b.name(&words.say(JigPhrase::AccountNumber, &[Hole::Number(u64::from(account.number))]));
        words.text(b, JigPhrase::AccountNumber, &[Hole::Number(u64::from(account.number))]);
        words.text(b, if account.usable { JigPhrase::AccountUsable } else { JigPhrase::AccountUnavailable }, &[]);
        if let Some(reason) = &account.reason {
            b.text(Run::Plain, reason.bytes());
        }
        for chip in &account.runs {
            cards::chip::chip(b, client, chip, words, bindings);
        }
        if !account.waiting.is_empty() {
            words.text(b, JigPhrase::WaitingAccount, &[]);
        }
        for chip in &account.waiting {
            cards::chip::chip(b, client, chip, words, bindings);
        }
        b.close();
    }
    b.close();
    super::work::section(b, words, JigPhrase::Capacity);
    for cap in &doc.capacity {
        match cap.project {
            Some(project) => words.text(b, JigPhrase::ContextProject, &[Hole::Number(u64::from(project))]),
            None => words.text(b, JigPhrase::Deployment, &[]),
        }
        words.text(b, JigPhrase::TasksLive, &[Hole::Number(u64::from(cap.live))]);
        words.text(b, JigPhrase::CapacityLimit, &[Hole::Number(u64::from(cap.limit))]);
    }
    words.text(b, JigPhrase::ProposalsWaiting, &[Hole::Number(u64::from(doc.proposals))]);
    for why in &doc.refusals {
        words.refusal(b, why);
    }
    b.close();
    for part in &doc.parts {
        cards::common::part(b, part, words, bindings);
    }
    super::excess(b, client, words);
}
