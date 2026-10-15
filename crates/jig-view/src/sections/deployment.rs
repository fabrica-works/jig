//! Whole host, account and capacity facts, with connector slots (pages.md, 7).
use crate::phrases::jig::JigPhrase;
use crate::ui::phrases::Hole;
use crate::ui::{Builder, Role, Run};
use crate::words::Words;
use crate::{JigBindings, cards};

use jig_domain_client::Domain;
/// Show the admitted deployment document and its connector-owned parts.
pub(crate) fn section(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings) -> bool {
    if !client.shows(jig_domain_client::pages::Section::Deployment) {
        return false;
    }
    crate::pages::work::section(b, words, JigPhrase::Deployment, crate::regions::JigRegion::Deployment);
    super::status(b, client, words, jig_domain_client::pages::Section::Deployment);
    if let Some(doc) = &client.deployment_page().owner {
        crate::owner_deployment::render(b, client, doc, words, bindings);
        b.close();
        return true;
    }
    let Some(doc) = &client.deployment_page().document else {
        words.status(b, JigPhrase::LoadingObject, &[]);
        b.close();
        return true;
    };
    region(b, words, crate::regions::JigRegion::Hosts, crate::owner_deployment_words::DeploymentPhrase::Hosts);
    b.open(Role::List);
    b.name(&words.say(JigPhrase::Deployment, &[]));
    for host in &doc.hosts {
        b.open(Role::Item);
        b.key(crate::ui::Key { owner: crate::Owner::Jig.opaque(), kind: 22, a: host.number, b: 0 });
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
    b.close();
    region(b, words, crate::regions::JigRegion::Accounts, crate::owner_deployment_words::DeploymentPhrase::Accounts);
    b.open(Role::List);
    for account in &doc.accounts {
        b.open(Role::Item);
        b.key(crate::ui::Key { owner: crate::Owner::Jig.opaque(), kind: 23, a: u64::from(account.number), b: 0 });
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
    b.close();
    crate::pages::work::section(b, words, JigPhrase::Capacity, crate::regions::JigRegion::Capacity);
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
    region(
        b,
        words,
        crate::regions::JigRegion::Connectors,
        crate::owner_deployment_words::DeploymentPhrase::Connectors,
    );
    for part in &doc.parts {
        cards::common::part(b, part, words, bindings);
    }
    b.close();
    b.close();
    true
}

fn region(
    b: &mut Builder,
    words: &Words,
    region: crate::regions::JigRegion,
    phrase: crate::owner_deployment_words::DeploymentPhrase,
) {
    b.open(Role::Region);
    b.region(region.region());
    b.name(&words.book.say(words.language, phrase.id(), &[]).expect("deployment region phrase checked").text);
}
