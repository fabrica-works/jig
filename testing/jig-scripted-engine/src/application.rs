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
/// Facts visible to one application operation.
#[derive(Clone, Copy, Debug)]
pub struct Context<'a> {
    pub party: u64,
    pub owner: u16,
    pub scope: ApplicationScope,
    pub number: u64,
    pub at: Wall,
    pub bounds: Bounds,
    pub truth: &'a Truth,
}
/// All typed values crossing an application's protocol boundary.
#[derive(Debug)]
pub enum ApplicationValue<'a, A: Application> {
    Ask(&'a A::Ask),
    Answer(&'a A::Answer),
    Read(&'a A::Read),
    Subject(&'a A::Subject),
    Delivery(&'a A::Delivery),
    Change(&'a A::Change),
}
/// A decision's own answer and atomic changes in both vocabularies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplicationDecision<T, C> {
    pub answer: T,
    pub changes: Vec<C>,
    pub jig: Vec<Change>,
}
/// Ordinary Rust hooks implemented by an application's client world.
/// `fits` must walk the whole decoded value, including nested text, lists and
/// unused capacities. Refusals make no changes. `apply` runs on a staged clone;
/// restart restores the last durable clone, together with jig's truth.
pub trait Application: Clone + Debug + PartialEq + Eq {
    type Ask: Clone + Debug + PartialEq + Eq;
    type Answer: Clone + Debug + PartialEq + Eq;
    type Read: Clone + Debug + PartialEq + Eq;
    type Subject: Clone + Debug + PartialEq + Eq;
    type Delivery: Clone + Debug + PartialEq + Eq;
    type Change: Clone + Debug + PartialEq + Eq;
    fn fits(&self, value: ApplicationValue<'_, Self>, bounds: &Bounds) -> bool;
    fn decide(
        &self,
        context: Context<'_>,
        ask: &Self::Ask,
    ) -> Result<ApplicationDecision<Self::Answer, Self::Change>, Refusal>;
    fn read(&self, context: Context<'_>, read: &Self::Read) -> Result<Self::Delivery, ReadNone>;
    fn watch(&self, context: Context<'_>, subject: &Self::Subject) -> Result<Option<Self::Delivery>, WatchEnd>;
    fn apply(&mut self, change: &Self::Change) -> Result<(), Invalid>;
}
/// No application payload can be constructed for the jig-only kit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Never {}
/// The default kit contains only jig documents and requests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoApplication;
impl Application for NoApplication {
    type Ask = Never;
    type Answer = Never;
    type Read = Never;
    type Subject = Never;
    type Delivery = Never;
    type Change = Never;
    fn fits(&self, value: ApplicationValue<'_, Self>, _: &Bounds) -> bool {
        match value {
            ApplicationValue::Ask(v)
            | ApplicationValue::Answer(v)
            | ApplicationValue::Read(v)
            | ApplicationValue::Subject(v)
            | ApplicationValue::Delivery(v)
            | ApplicationValue::Change(v) => match *v {},
        }
    }
    fn decide(&self, _: Context<'_>, ask: &Never) -> Result<ApplicationDecision<Never, Never>, Refusal> {
        match *ask {}
    }
    fn read(&self, _: Context<'_>, read: &Never) -> Result<Never, ReadNone> {
        match *read {}
    }
    fn watch(&self, _: Context<'_>, subject: &Never) -> Result<Option<Never>, WatchEnd> {
        match *subject {}
    }
    fn apply(&mut self, change: &Never) -> Result<(), Invalid> {
        match *change {}
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
    pub changes: Vec<A::Change>,
}
/// A nonterminal application watch event, sharing jig's terminal channel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplicationEvent<T> {
    Opened,
    Delivered(T),
    Alive,
}
#[derive(Debug)]
pub(crate) struct Open<A: Application> {
    pub(crate) sign_in: u64,
    pub(crate) party: u64,
    owner: u16,
    scope: ApplicationScope,
    subject: A::Subject,
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
    fn context(&self, party: u64, owner: u16, project: u32, at: Wall) -> Context<'_> {
        self.scoped_context(party, owner, ApplicationScope::Project(project), at)
    }
    fn scoped_context(&self, party: u64, owner: u16, scope: ApplicationScope, at: Wall) -> Context<'_> {
        Context { party, owner, scope, number: self.next_commit, at, bounds: self.bounds, truth: &self.truth }
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
        let decision = if self.truth.visible(party, project) {
            self.application.decide(self.context(party, owner, project, wall), &ask)
        } else {
            Err(Refusal::Unknown)
        };
        let (answer, changes, jig) = match decision {
            Ok(decision) => {
                if !self.application.fits(ApplicationValue::Answer(&decision.answer), &self.bounds) {
                    self.attempts.remove(&attempt);
                    return Err(Invalid::Payload);
                }
                if let Err(error) = self.stage_application(&decision.changes, &decision.jig) {
                    self.attempts.remove(&attempt);
                    return Err(error);
                }
                (Ok(decision.answer), decision.changes, decision.jig)
            }
            Err(refusal) => {
                if !crate::payload::fits(&refusal, &self.bounds) {
                    self.attempts.remove(&attempt);
                    return Err(Invalid::Payload);
                }
                (Err(refusal), vec![], vec![])
            }
        };
        let commit = self.commit(now, wall, Some(party), Some(key), None, None, jig);
        self.pending.last_mut().expect("created commit").commit.application = Some(ApplicationCommit {
            owner,
            scope: ApplicationScope::Project(project),
            digest: Some(digest),
            ask: Some(ask),
            answer: Some(answer.clone()),
            changes,
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
    fn stage_application(&mut self, changes: &[A::Change], jig: &[Change]) -> Result<(), Invalid> {
        let mut application = self.application.clone();
        let mut truth = self.truth.clone();
        for change in changes {
            if !application.fits(ApplicationValue::Change(change), &self.bounds) {
                return Err(Invalid::Payload);
            }
            application.apply(change)?;
        }
        for change in jig {
            change.validate(&self.bounds)?;
            truth.apply(change)?;
        }
        self.application = application;
        self.truth = truth;
        Ok(())
    }
    /// Moves the application's story through the same durability barrier as jig.
    pub fn act_application(
        &mut self,
        now: Time,
        wall: Wall,
        owner: u16,
        project: u32,
        changes: Vec<A::Change>,
        jig: Vec<Change>,
    ) -> Result<u64, Invalid> {
        self.act_scoped_application(now, wall, owner, ApplicationScope::Project(project), changes, jig)
    }
    /// Moves project or deployment part facts through the shared ledger.
    pub fn act_scoped_application(
        &mut self,
        now: Time,
        wall: Wall,
        owner: u16,
        scope: ApplicationScope,
        changes: Vec<A::Change>,
        jig: Vec<Change>,
    ) -> Result<u64, Invalid> {
        if !self.online(now) {
            return Err(Invalid::Offline);
        }
        self.stage_application(&changes, &jig)?;
        let commit = self.commit(now, wall, None, None, None, None, jig);
        self.pending.last_mut().expect("created commit").commit.application =
            Some(ApplicationCommit { owner, scope, digest: None, ask: None, answer: None, changes });
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
        read: &A::Read,
    ) -> Result<(), Invalid> {
        self.read_scoped_application(now, sign_in, token, owner, ApplicationScope::Project(project), read)
    }
    /// Reads an application's project or deployment part.
    pub fn read_scoped_application(
        &mut self,
        now: Time,
        sign_in: u64,
        token: Token,
        owner: u16,
        scope: ApplicationScope,
        read: &A::Read,
    ) -> Result<(), Invalid> {
        if !self.application.fits(ApplicationValue::Read(read), &self.bounds) {
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
                self.application.read(self.scoped_context(party, owner, scope, Wall::EPOCH), read)
            } else {
                Err(ReadNone::Unknown)
            }
        } else {
            Err(ReadNone::SignedOut)
        };
        let terminal = if let Ok(value) = terminal {
            if self.application.fits(ApplicationValue::Delivery(&value), &self.bounds) {
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
        owner: u16,
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
            self.application.watch(self.scoped_context(party, owner, scope, Wall::EPOCH), &subject).err()
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
                owner,
                scope,
                subject,
                dirty: true,
                in_flight: false,
                heard: now,
                ready: now.saturating_add(self.slow),
            },
        );
        self.queue(now, 0, Up::ApplicationWatch { token, event: ApplicationEvent::Opened });
        Ok(())
    }
    pub(crate) fn end_application_watch(&mut self, now: Time, token: Token, why: WatchEnd, commit: u64) {
        if self.application_watches.remove(&token).is_some() {
            self.due.retain(|due| !matches!(due.up,Up::ApplicationWatch { token:id,.. } if id == token));
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
                match self
                    .application
                    .watch(self.scoped_context(watch.party, watch.owner, watch.scope, Wall::EPOCH), &watch.subject)
                {
                    Ok(Some(value)) if self.application.fits(ApplicationValue::Delivery(&value), &self.bounds) => {
                        watch.dirty = false;
                        watch.in_flight = true;
                        watch.heard = now;
                        self.queue(
                            now,
                            durable,
                            Up::ApplicationWatch { token, event: ApplicationEvent::Delivered(value) },
                        );
                    }
                    Ok(None) => {
                        watch.dirty = false;
                    }
                    Ok(Some(_)) => {
                        self.application_watches.insert(token, watch);
                        self.end_application_watch(now, token, WatchEnd::Busy, durable);
                        continue;
                    }
                    Err(why) => {
                        self.application_watches.insert(token, watch);
                        self.end_application_watch(now, token, why, durable);
                        continue;
                    }
                }
            }
            if now >= watch.heard.saturating_add(self.bounds.heartbeat) {
                watch.heard = now;
                if !self.due.iter().any(
                    |due| matches!(due.up,Up::ApplicationWatch { token:id,event:ApplicationEvent::Alive } if id==token),
                ) {
                    self.queue(now, 0, Up::ApplicationWatch { token, event: ApplicationEvent::Alive });
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
