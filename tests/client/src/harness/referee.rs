//! Safety and liveness from shell, tree and durable boundary observations
//! (client/domain/testing.md, 6). No rule reads a root or a binding table.
use super::{CardKey, CardSeen, ClientId, Observation, Observed, Receipt, Seen, Sent};
use skein_lib::{Duration, Token};
use skein_world::domain::{Expectations, Judge};
use std::collections::{BTreeMap, BTreeSet};

/// Named request and link obligations, independent of attempts and reloads.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Deadline {
    /// The first send of one party's key must receive a final answer.
    Request(u64, [u8; 16]),
    /// Unreachable beyond a heartbeat must be shown offline.
    Offline(ClientId),
    /// Recovered truth must become visibly online.
    Online(ClientId),
}
type JudgeOf = Judge<Deadline, ()>;
type RequestKey = (u64, [u8; 16]);
type Card = (ClientId, CardKey);
#[derive(Debug)]
struct Pending {
    client: ClientId,
    card: Option<CardKey>,
    words: Option<(u64, Vec<u8>)>,
}
/// Seven safety rules with request and link deadlines in skein's referee.
#[derive(Debug)]
pub struct Rules {
    heartbeat: Duration,
    request_within: Duration,
    link_within: Duration,
    clients: BTreeSet<ClientId>,
    reachable: Option<bool>,
    durable: BTreeSet<u64>,
    made: BTreeMap<RequestKey, u64>,
    requests: BTreeMap<RequestKey, Pending>,
    attempts: BTreeMap<(ClientId, Token), RequestKey>,
    documents: BTreeMap<(ClientId, CardKey, u16), u64>,
    visible: BTreeMap<Card, (CardSeen, Option<u64>)>,
    answered: BTreeMap<Card, RequestKey>,
    stopped: BTreeMap<CardKey, u64>,
    reasoned: BTreeSet<Card>,
    kept: BTreeMap<(u64, u64), Vec<u8>>,
    ended: BTreeSet<u64>,
    promised: BTreeMap<(u64, u64), Vec<u8>>,
    acknowledged: BTreeMap<(u64, u64), Vec<u8>>,
}
impl Rules {
    /// Installs the world's spans; commit zero is its durable starting truth.
    #[must_use]
    pub fn new(heartbeat: Duration, request_within: Duration, link_within: Duration) -> Self {
        Self {
            heartbeat,
            request_within,
            link_within,
            clients: BTreeSet::new(),
            reachable: None,
            durable: BTreeSet::from([0]),
            made: BTreeMap::new(),
            requests: BTreeMap::new(),
            attempts: BTreeMap::new(),
            documents: BTreeMap::new(),
            visible: BTreeMap::new(),
            answered: BTreeMap::new(),
            stopped: BTreeMap::new(),
            reasoned: BTreeSet::new(),
            kept: BTreeMap::new(),
            ended: BTreeSet::new(),
            promised: BTreeMap::new(),
            acknowledged: BTreeMap::new(),
        }
    }
    fn truth(&mut self, observed: Observed, judge: &mut JudgeOf) {
        match observed {
            Observed::Durable { commit, at } => {
                judge.check(at <= judge.now(), "nothing shown done before durable: future write");
                judge.check(self.durable.insert(commit), "durable commit reported once");
            }
            Observed::Made { party, key, commit } => {
                judge.check(self.durable.contains(&commit), "made answer must be durable");
                judge.check(self.made.insert((party, key), commit).is_none(), "made once: repeated party key");
            }
            Observed::Kept { task, message, words, commit } => {
                judge.check(self.durable.contains(&commit), "kept words must be durable");
                judge.check(
                    self.ended.contains(&task)
                        || self.promised.get(&(task, message)).is_none_or(|wanted| wanted == &words),
                    "nothing sent lost: committed words changed",
                );
                self.kept.insert((task, message), words);
            }
            Observed::Retained { task, messages, commit } => {
                judge.check(self.durable.contains(&commit), "retained activity must be durable");
                self.kept.retain(|(t, m), _| *t != task || messages.contains(m));
                judge.check(
                    self.ended.contains(&task)
                        || self
                            .promised
                            .keys()
                            .filter(|(t, _)| *t == task)
                            .all(|key| self.kept.get(key) == self.promised.get(key)),
                    "nothing sent lost: previously sent words left activity",
                );
            }
            Observed::Stopped { card, commit } => {
                judge.check(self.durable.contains(&commit), "stopped object must be durable");
                self.stopped.insert(card, commit);
            }
            Observed::Ended { task, commit } => {
                judge.check(self.durable.contains(&commit), "ended task must be durable");
                self.ended.insert(task);
            }
            Observed::Reachable { live } => {
                if self.reachable == Some(live) {
                    return;
                }
                self.reachable = Some(live);
                for client in &self.clients {
                    if live {
                        judge.withdraw(&Deadline::Offline(*client));
                        judge.rearm(Deadline::Online(*client), self.link_within);
                    } else {
                        judge.withdraw(&Deadline::Online(*client));
                        judge.rearm(Deadline::Offline(*client), self.heartbeat);
                    }
                }
            }
        }
    }
    fn request(&mut self, client: ClientId, party: u64, sent: Sent, judge: &mut JudgeOf) {
        let key = (party, sent.key);
        let fresh = !self.requests.contains_key(&key);
        if fresh && let (Some(card), Some((kind, revision))) = (&sent.card, sent.revision) {
            let key = (client, card.clone());
            let shown = self
                .visible
                .get(&key)
                .and_then(|(_, revision)| *revision)
                .or_else(|| self.documents.get(&(client, card.clone(), kind)).copied());
            judge.check(
                self.visible.contains_key(&key) && shown == Some(revision),
                "what they saw is what they decided: revision differs",
            );
        }
        if fresh && let Some(card) = &sent.card {
            self.answered.remove(&(client, card.clone()));
            self.reasoned.remove(&(client, card.clone()));
        }
        if fresh || (!judge.is_pending(&Deadline::Request(party, sent.key)) && !self.made.contains_key(&key)) {
            judge.expect(Deadline::Request(party, sent.key), self.request_within);
        }
        self.requests.entry(key).or_insert(Pending { client, card: sent.card, words: sent.words });
        self.attempts.insert((client, sent.attempt), key);
    }
    fn delivered(&mut self, client: ClientId, receipt: Receipt, judge: &mut JudgeOf) {
        match receipt {
            Receipt::Card { key, revisions } => {
                for (kind, revision) in revisions {
                    self.documents
                        .entry((client, key.clone(), kind))
                        .and_modify(|prior| *prior = (*prior).max(revision))
                        .or_insert(revision);
                }
            }
            Receipt::Answer { attempt, final_answer, words } => {
                if final_answer && let Some(key) = self.attempts.get(&(client, attempt)) {
                    judge.check(
                        self.made.contains_key(key),
                        "nothing shown done before durable: final answer has no write",
                    );
                    judge.meet(&Deadline::Request(key.0, key.1));
                    if let Some((task, message)) = words
                        && let Some(Pending { words: Some((sent_task, text)), .. }) = self.requests.get(key)
                    {
                        judge.check(task == *sent_task, "sent words belong to their task");
                        self.acknowledged.insert((task, message), text.clone());
                    }
                    if let Some(pending) = self.requests.get(key)
                        && let Some(card) = &pending.card
                    {
                        self.answered.insert((client, card.clone()), *key);
                    }
                }
            }
            Receipt::Party { party } => {
                for (key, pending) in &self.requests {
                    if pending.client == client && key.0 != party {
                        judge.withdraw(&Deadline::Request(key.0, key.1));
                    }
                }
            }
            Receipt::Parked => {
                for (key, pending) in &self.requests {
                    if pending.client == client {
                        judge.withdraw(&Deadline::Request(key.0, key.1));
                    }
                }
            }
        }
    }
    fn shown(&mut self, client: ClientId, seen: Vec<Seen>, judge: &mut JudgeOf) {
        let mut cards = BTreeSet::new();
        if seen.contains(&Seen::HiddenPage) {
            judge.check(!seen.iter().any(|s| matches!(s, Seen::Card { .. })), "a hidden work page exposes a card");
            self.visible.retain(|(c, _), _| *c != client);
            self.reasoned.retain(|(c, _)| *c != client);
        }
        for seen in seen {
            match seen {
                Seen::Card { key, state, revision } => {
                    let card = (client, key.clone());
                    cards.insert(card.clone());
                    if matches!(&state, CardSeen::Done | CardSeen::Left { .. }) {
                        let answered = self.answered.get(&card).is_some_and(|key| self.made.contains_key(key));
                        judge.check(
                            answered || self.stopped.contains_key(&key),
                            "nothing shown done before durable: card outcome has no write",
                        );
                        let reason = match &state {
                            CardSeen::Done => true,
                            CardSeen::Left { reason } => *reason,
                            CardSeen::Waiting | CardSeen::Deciding => false,
                        };
                        judge.check(reason, "a card leaves for a reason: explanation absent");
                        if reason {
                            self.reasoned.insert(card.clone());
                        }
                    }
                    if matches!(state, CardSeen::Waiting | CardSeen::Deciding) {
                        self.reasoned.remove(&card);
                    }
                    self.visible.insert(card, (state, revision));
                }
                Seen::Words { task, message, words } => {
                    let kept = match message {
                        Some(message) => self.kept.get(&(task, message)) == Some(&words),
                        None => self.acknowledged.iter().any(|(key, wanted)| {
                            key.0 == task && wanted == &words && self.kept.get(key) == Some(wanted)
                        }),
                    };
                    judge.check(self.ended.contains(&task) || kept, "nothing sent lost: acknowledged words are absent");
                    let message = message.or_else(|| {
                        self.acknowledged
                            .iter()
                            .find(|(key, wanted)| key.0 == task && *wanted == &words)
                            .map(|(key, _)| key.1)
                    });
                    if let Some(message) = message {
                        self.promised.insert((task, message), words);
                    }
                }
                Seen::HiddenPage => {}
                Seen::Link { live } => {
                    judge.meet(&if live { Deadline::Online(client) } else { Deadline::Offline(client) });
                }
            }
        }
        let left: Vec<_> =
            self.visible.keys().filter(|(c, _)| *c == client).filter(|key| !cards.contains(*key)).cloned().collect();
        for key in left {
            judge.check(
                self.reasoned.remove(&key),
                format!("a card leaves for a reason: waiting card {key:?} disappeared"),
            );
            self.visible.remove(&key);
        }
    }
}
impl Expectations for Rules {
    type Seen = Observation;
    type Name = Deadline;
    type Stimulus = ();
    fn observe(&mut self, observation: Observation, judge: &mut JudgeOf) {
        match observation {
            Observation::Client { client } => {
                self.clients.insert(client);
                if let Some(live) = self.reachable {
                    judge.expect(
                        if live { Deadline::Online(client) } else { Deadline::Offline(client) },
                        if live { self.link_within } else { self.heartbeat },
                    );
                }
            }
            Observation::Truth(observed) => self.truth(observed, judge),
            Observation::Request { client, party, sent } => self.request(client, party, sent, judge),
            Observation::Delivered { client, receipt } => self.delivered(client, receipt, judge),
            Observation::Shown { client, seen } => self.shown(client, seen, judge),
            Observation::LeftPage { client } => {
                self.visible.retain(|(c, _), _| *c != client);
                self.reasoned.retain(|(c, _)| *c != client);
            }
            Observation::Bounded { heap, bound } => {
                judge.check(heap <= bound, "bounded: client and view exceeded the heap envelope");
            }
        }
    }
}
