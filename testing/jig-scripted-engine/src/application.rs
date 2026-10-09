//! Typed world hooks beside jig's documents; client/domain/testing.md, 4.
//! The world supplies its canonical digest. Owner and project join that digest in
//! the party's shared key space. Every hook entrance and output is validated;
//! application changes and jig changes become durable in the same numbered commit.
use crate::{Bounds, Change, Engine, Envelope, Invalid, Keyed, RequestIdentity, Retained, Truth, Up};
use jig_client::contract::{ReadNone, Refusal, WatchEnd};
use skein_lib::{Time, Token, Wall};
use std::fmt::Debug;

/// The authorization context for an application part.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplicationScope {
    Project(u32),
    Deployment,
}
/// All typed values crossing an application's protocol boundary.
/// State validates each retained document, not the sum of the world's truth.
#[derive(Debug)]
pub enum ApplicationValue<'a, A: Application> {
    Ask(&'a A::Ask),
    Answer(&'a A::Answer),
    Subject(&'a A::Subject),
    Document(&'a A::Document),
    State(&'a A),
}
/// The connector's answer and the jig requests it makes on the party's behalf.
/// Core requests use `Change::Request`; whole document replacements belong to
/// the story's `act` interface and cannot bypass party admission from this hook.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision<T> {
    pub answer: T,
    pub changes: Vec<Change>,
}
/// Ordinary Rust hooks implemented by an application's client world.
/// `decide` runs on a staged clone; refusal of a jig request discards that clone.
/// `fits` walks the whole decoded value, including nested text and capacities.
/// `document` serves both reads and watches; `marks` reports stale subjects.
/// Subjects name their project or deployment scope. The world carries that
/// scope at admission, and `document` checks its own subject and party visibility.
pub trait Application: Clone + Debug + PartialEq + Eq {
    type Ask: Clone + Debug + PartialEq + Eq;
    type Answer: Clone + Debug + PartialEq + Eq;
    type Subject: Clone + Debug + Ord;
    type Document: Clone + Debug + PartialEq + Eq;
    fn decide(&mut self, truth: &Truth, party: u64, ask: &Self::Ask, wall: Wall) -> Decision<Self::Answer>;
    fn document(&self, truth: &Truth, party: u64, subject: &Self::Subject) -> Option<Self::Document>;
    fn marks(&self, change: &Change) -> Vec<Self::Subject>;
    fn fits(&self, value: ApplicationValue<'_, Self>, bounds: &Bounds) -> bool;
}
/// No application payload can be constructed for the jig-only kit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Never {}
/// The default engine's empty application model; all its protocol types are uninhabited.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoApplication;
impl Application for NoApplication {
    type Ask = Never;
    type Answer = Never;
    type Subject = Never;
    type Document = Never;
    fn decide(&mut self, _: &Truth, _: u64, ask: &Never, _: Wall) -> Decision<Never> {
        match *ask {}
    }
    fn document(&self, _: &Truth, _: u64, subject: &Never) -> Option<Never> {
        match *subject {}
    }
    fn marks(&self, _: &Change) -> Vec<Never> {
        vec![]
    }
    fn fits(&self, value: ApplicationValue<'_, Self>, _: &Bounds) -> bool {
        match value {
            ApplicationValue::Ask(v)
            | ApplicationValue::Answer(v)
            | ApplicationValue::Subject(v)
            | ApplicationValue::Document(v) => match *v {},
            ApplicationValue::State(_) => true,
        }
    }
}
/// The application portion of the referee's numbered ledger.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplicationCommit<A: Application> {
    pub owner: u16,
    pub scope: ApplicationScope,
    pub digest: Option<[u8; 32]>,
    pub ask: Option<A::Ask>,
    pub answer: Option<Result<A::Answer, Refusal>>,
    pub requests: Vec<jig_client::contract::Ask>,
    pub state: A,
}
#[derive(Debug)]
pub(crate) struct Open<A: Application> {
    pub(crate) sign_in: u64,
    pub(crate) party: u64,
    scope: ApplicationScope,
    pub(crate) subject: A::Subject,
    pub(crate) dirty: bool,
    pub(crate) in_flight: bool,
    pub(crate) heard: Time,
    pub(crate) ready: Time,
}
impl<A: Application> Engine<A> {
    /// Application truth for a referee; it may include work awaiting durability.
    #[must_use]
    pub fn application(&self) -> &A {
        &self.application
    }
    fn scope_visible(&self, party: u64, scope: ApplicationScope) -> bool {
        match scope {
            ApplicationScope::Project(project) => self.truth.visible(party, project),
            ApplicationScope::Deployment => {
                self.truth.frames.get(&party).is_some_and(|frame| frame.deployment_role.is_some())
            }
        }
    }
    /// A typed application attempt in the same party/key space as jig's asks.
    #[expect(
        clippy::too_many_arguments,
        reason = "the world names owner, project, canonical digest and attempt explicitly"
    )]
    pub fn send_application(
        &mut self,
        now: Time,
        wall: Wall,
        sign_in: u64,
        key: [u8; 16],
        attempt: Token,
        owner: u16,
        project: u32,
        digest: [u8; 32],
        ask: A::Ask,
    ) -> Result<(), Invalid> {
        if !self.application.fits(ApplicationValue::Ask(&ask), &self.bounds) {
            return Err(Invalid::Payload);
        }
        self.attempts.insert(attempt, sign_in);
        if !self.online(now) || self.request_fault == Some(Envelope::Again) {
            self.request_fault = None;
            self.queue(now, 0, Up::ApplicationAnswer { attempt, envelope: Envelope::Again, answer: None });
            return Ok(());
        }
        let Some(party) = self.truth.sign_ins.get(&sign_in).copied() else {
            self.queue(now, 0, Up::ApplicationAnswer { attempt, envelope: Envelope::SignedOut, answer: None });
            return Ok(());
        };
        self.keys.retain(|_, keyed| keyed.retained_until.is_none_or(|until| now <= until));
        if let Some(old) = self.keys.get(&(party, key)) {
            let answer = if let (
                RequestIdentity::Application { owner: previous, project: space, digest: identity, .. },
                Retained::Application(answer),
            ) = (&old.ask, &old.answer)
            {
                if *previous == owner && *space == project && *identity == digest {
                    answer.clone()
                } else {
                    Err(Refusal::KeyConflict)
                }
            } else {
                Err(Refusal::KeyConflict)
            };
            let commit = old.commit;
            self.application_answer(now, commit, attempt, answer);
            return Ok(());
        }
        let (answer, requests, changes) = if self.truth.visible(party, project) {
            let mut application = self.application.clone();
            let decision = application.decide(&self.truth, party, &ask, wall);
            if !application.fits(ApplicationValue::Answer(&decision.answer), &self.bounds)
                || !application.fits(ApplicationValue::State(&application), &self.bounds)
            {
                self.attempts.remove(&attempt);
                return Err(Invalid::Payload);
            }
            match self.judge_application(party, wall, &decision.changes) {
                Ok((requests, changes)) => {
                    self.application = application;
                    self.mark_application(now);
                    (Ok(decision.answer), requests, changes)
                }
                Err(crate::application::Judged::Refused(refusal)) => (Err(refusal), vec![], vec![]),
                Err(crate::application::Judged::Invalid(error)) => {
                    self.attempts.remove(&attempt);
                    return Err(error);
                }
            }
        } else {
            (Err(Refusal::Unknown), vec![], vec![])
        };
        let commit = self.commit(now, wall, Some(party), Some(key), None, None, changes);
        self.pending.last_mut().expect("created commit").commit.application = Some(ApplicationCommit {
            owner,
            scope: ApplicationScope::Project(project),
            digest: Some(digest),
            ask: Some(ask),
            answer: Some(answer.clone()),
            requests,
            state: self.application.clone(),
        });
        self.keys.insert(
            (party, key),
            Keyed {
                ask: RequestIdentity::Application { owner, project, digest },
                answer: Retained::Application(answer.clone()),
                commit,
                retained_until: None,
            },
        );
        self.application_answer(now, commit, attempt, answer);
        Ok(())
    }
    fn application_answer(&mut self, now: Time, commit: u64, attempt: Token, answer: Result<A::Answer, Refusal>) {
        let envelope = self.request_fault.take().unwrap_or(Envelope::Final);
        self.queue(
            now,
            commit,
            Up::ApplicationAnswer {
                attempt,
                envelope,
                answer: if envelope == Envelope::Final { Some(answer) } else { None },
            },
        );
    }
    fn mark_application(&mut self, now: Time) {
        for watch in self.application_watches.values_mut() {
            if !watch.dirty {
                watch.ready = now.saturating_add(self.slow);
            }
            watch.dirty = true;
        }
    }
    fn judge_application(
        &mut self,
        party: u64,
        wall: Wall,
        changes: &[Change],
    ) -> Result<(Vec<jig_client::contract::Ask>, Vec<Change>), Judged> {
        let original = self.truth.clone();
        let mut requests = Vec::new();
        let mut effects = Vec::new();
        for change in changes {
            let Change::Request(ask) = change else {
                self.truth = original;
                return Err(Judged::Refused(Refusal::NotOffered));
            };
            if !crate::payload::fits(ask, &self.bounds) {
                self.truth = original;
                return Err(Judged::Invalid(Invalid::Payload));
            }
            let result = self.decide(party, ask, wall);
            match result {
                Ok((jig_client::contract::Answer::Done(_), changes)) => {
                    requests.push((**ask).clone());
                    effects.extend(changes);
                }
                Ok((jig_client::contract::Answer::Refused(refusal), _)) => {
                    self.truth = original;
                    return Err(Judged::Refused(refusal));
                }
                Err(error) => {
                    self.truth = original;
                    return Err(Judged::Invalid(error));
                }
            }
        }
        Ok((requests, effects))
    }
    /// Replaces the world's application model together with jig story changes.
    pub fn act_application(
        &mut self,
        now: Time,
        wall: Wall,
        owner: u16,
        project: u32,
        application: A,
        jig: Vec<Change>,
    ) -> Result<u64, Invalid> {
        self.act_scoped_application(now, wall, owner, ApplicationScope::Project(project), application, jig)
    }
    /// Moves project or deployment part facts through the same durability barrier.
    pub fn act_scoped_application(
        &mut self,
        now: Time,
        wall: Wall,
        owner: u16,
        scope: ApplicationScope,
        application: A,
        jig: Vec<Change>,
    ) -> Result<u64, Invalid> {
        if !self.online(now) {
            return Err(Invalid::Offline);
        }
        if !application.fits(ApplicationValue::State(&application), &self.bounds) {
            return Err(Invalid::Payload);
        }
        let mut truth = self.truth.clone();
        for change in &jig {
            change.validate(&self.bounds)?;
            truth.apply(change)?;
        }
        self.truth = truth;
        self.application = application;
        self.mark_application(now);
        let commit = self.commit(now, wall, None, None, None, None, jig);
        self.pending.last_mut().expect("created commit").commit.application = Some(ApplicationCommit {
            owner,
            scope,
            digest: None,
            ask: None,
            answer: None,
            requests: vec![],
            state: self.application.clone(),
        });
        Ok(commit)
    }
    /// Captures one application document behind the latest shared commit.
    pub fn read_application(
        &mut self,
        now: Time,
        sign_in: u64,
        token: Token,
        owner: u16,
        project: u32,
        read: &A::Subject,
    ) -> Result<(), Invalid> {
        self.read_scoped_application(now, sign_in, token, owner, ApplicationScope::Project(project), read)
    }
    /// Reads an application's project or deployment part.
    pub fn read_scoped_application(
        &mut self,
        now: Time,
        sign_in: u64,
        token: Token,
        _owner: u16,
        scope: ApplicationScope,
        read: &A::Subject,
    ) -> Result<(), Invalid> {
        if !self.application.fits(ApplicationValue::Subject(read), &self.bounds) {
            return Err(Invalid::Payload);
        }
        if !self.read_tokens.insert(token) {
            return Err(Invalid::Identity);
        }
        self.reads.insert(token, sign_in);
        let terminal = if !self.online(now) {
            Err(ReadNone::Unreachable)
        } else if let Some(party) = self.truth.sign_ins.get(&sign_in).copied() {
            if self.scope_visible(party, scope) {
                self.application.document(&self.truth, party, read).ok_or(ReadNone::Unknown)
            } else {
                Err(ReadNone::Unknown)
            }
        } else {
            Err(ReadNone::SignedOut)
        };
        let terminal = if let Ok(value) = terminal {
            if self.application.fits(ApplicationValue::Document(&value), &self.bounds) {
                Ok(value)
            } else {
                Err(ReadNone::Busy)
            }
        } else {
            terminal
        };
        self.queue(now, self.next_commit - 1, Up::ApplicationRead { token, terminal });
        Ok(())
    }
    /// Admits an application subject under the same global and party watch limits.
    pub fn open_application(
        &mut self,
        now: Time,
        sign_in: u64,
        token: Token,
        owner: u16,
        project: u32,
        subject: A::Subject,
    ) -> Result<(), Invalid> {
        self.open_scoped_application(now, sign_in, token, owner, ApplicationScope::Project(project), subject)
    }
    /// Watches an application's project or deployment part.
    pub fn open_scoped_application(
        &mut self,
        now: Time,
        sign_in: u64,
        token: Token,
        _owner: u16,
        scope: ApplicationScope,
        subject: A::Subject,
    ) -> Result<(), Invalid> {
        if !self.application.fits(ApplicationValue::Subject(&subject), &self.bounds) {
            return Err(Invalid::Payload);
        }
        if !self.watch_tokens.insert(token) {
            return Err(Invalid::Identity);
        }
        let Some(party) = self.truth.sign_ins.get(&sign_in).copied() else {
            self.queue(now, 0, Up::Ended { token, why: WatchEnd::SignedOut });
            return Ok(());
        };
        let count = self.watches.values().filter(|watch| watch.party == party).count()
            + self.application_watches.values().filter(|watch| watch.party == party).count();
        let why = if !self.online(now) {
            Some(WatchEnd::Dropped)
        } else if !self.scope_visible(party, scope) {
            Some(WatchEnd::Unknown)
        } else if self.watches.len() + self.application_watches.len() >= self.bounds.watches as usize
            || count >= self.bounds.watches_per_party as usize
        {
            Some(WatchEnd::Busy)
        } else {
            self.application.document(&self.truth, party, &subject).is_none().then_some(WatchEnd::Unknown)
        };
        if let Some(why) = why {
            self.queue(now, self.next_commit - 1, Up::Ended { token, why });
            return Ok(());
        }
        self.application_watches.insert(
            token,
            Open {
                sign_in,
                party,
                scope,
                subject,
                dirty: true,
                in_flight: false,
                heard: now,
                ready: now.saturating_add(self.slow),
            },
        );
        self.queue(now, 0, Up::Watch { token, event: jig_client::contract::WatchEvent::Opened });
        Ok(())
    }
    pub(crate) fn end_application_watch(&mut self, now: Time, token: Token, why: WatchEnd, commit: u64) {
        if self.application_watches.remove(&token).is_some() {
            self.due.retain(|due| !matches!(due.up,Up::App { watch:id,.. } | Up::Watch { token:id,.. } if id == token));
            self.queue(now, commit, Up::Ended { token, why });
        }
    }
    pub(crate) fn prepare_application_watches(&mut self, now: Time) {
        let durable = self.commits.last().map_or(0, |commit| commit.number);
        let tokens: Vec<_> = self.application_watches.keys().copied().collect();
        for token in tokens {
            let mut watch = self.application_watches.remove(&token).expect("selected watch");
            let why = if self.truth.sign_ins.get(&watch.sign_in) != Some(&watch.party) {
                Some(WatchEnd::SignedOut)
            } else if !self.scope_visible(watch.party, watch.scope) {
                Some(WatchEnd::NotAllowed)
            } else {
                None
            };
            if let Some(why) = why {
                self.application_watches.insert(token, watch);
                self.end_application_watch(now, token, why, self.next_commit - 1);
                continue;
            }
            if watch.dirty && !watch.in_flight && now >= watch.ready && durable == self.next_commit - 1 {
                match self.application.document(&self.truth, watch.party, &watch.subject) {
                    Some(document) if self.application.fits(ApplicationValue::Document(&document), &self.bounds) => {
                        watch.dirty = false;
                        watch.in_flight = true;
                        watch.heard = now;
                        self.queue(now, durable, Up::App { watch: token, document });
                    }
                    Some(_) => {
                        self.application_watches.insert(token, watch);
                        self.end_application_watch(now, token, WatchEnd::Busy, durable);
                        continue;
                    }
                    None => {
                        self.application_watches.insert(token, watch);
                        self.end_application_watch(now, token, WatchEnd::SubjectEnded, durable);
                        continue;
                    }
                }
            }
            if now >= watch.heard.saturating_add(self.bounds.heartbeat) {
                watch.heard = now;
                if !self.due.iter().any(
                    |due| matches!(due.up,Up::Watch { token:id,event:jig_client::contract::WatchEvent::Alive } if id==token),
                ) {
                    self.queue(now, 0, Up::Watch { token,event:jig_client::contract::WatchEvent::Alive });
                }
            }
            self.application_watches.insert(token, watch);
        }
    }
    pub(crate) fn application_watch_due(&self) -> impl Iterator<Item = Time> + '_ {
        self.application_watches.values().map(|watch| {
            let heartbeat = watch.heard.saturating_add(self.bounds.heartbeat);
            if watch.dirty && !watch.in_flight {
                heartbeat.min(self.pending.last().map_or(watch.ready, |pending| pending.due.max(watch.ready)))
            } else {
                heartbeat
            }
        })
    }
}

#[derive(Debug)]
enum Judged {
    Refused(Refusal),
    Invalid(Invalid),
}
