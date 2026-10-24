//! Outside grant and settlement checks shared by the stories, with
//! llm.md 8's referees for the driver and the secrets' store: no token
//! reaches a run before its record is kept; a name never means two values;
//! no value is in anything the owner keeps outside the secrets' store.
use crate::{
    fixtures,
    owner::{Heard, Owner, Trace},
    world::{Process, Story},
};
use skein_lib::{Duration, Time};
use skein_world::{Host, Referee};

#[derive(Debug)]
pub struct Judge {
    story: Story,
    wake: Option<Time>,
    deadline: Option<Time>,
    rejected: bool,
    pasted: u8,
    restarted: Option<()>,
    closing: bool,
    shutdown: bool,
    passed: Option<()>,
}
impl Judge {
    #[must_use]
    pub fn new(story: Story) -> Self {
        Self {
            story,
            wake: None,
            deadline: None,
            rejected: false,
            pasted: 0,
            restarted: None,
            closing: false,
            shutdown: false,
            passed: None,
        }
    }
}
impl Referee<Process> for Judge {
    fn act(&mut self, now: Time, processes: &mut [Process]) {
        self.deadline.get_or_insert(now.saturating_add(Duration::from_secs(180)));
        let address = processes.iter().find_map(|process| match process {
            Process::Issuer(peer) => peer.address(),
            Process::Owner(_) | Process::Visitor(_) => None,
        });
        let visit = processes.iter().find_map(|process| match process {
            Process::Owner(owner) => owner.visit().map(Box::<[u8]>::from),
            Process::Issuer(_) | Process::Visitor(_) => None,
        });
        let pasting = matches!(self.story, Story::PastedAddress | Story::PastedState | Story::SignInAgain);
        let can_visit = self.story != Story::SignInAgain || self.restarted.is_some();
        let mut redirect = [0_u8; 1024];
        let redirect_len = processes.iter().find_map(|process| match process {
            Process::Visitor(visitor) => visitor.redirect().map(|uri| {
                redirect[..uri.len()].copy_from_slice(uri);
                uri.len()
            }),
            Process::Owner(_) | Process::Issuer(_) => None,
        });
        let close_exchange = self.story == Story::ClosedExchange && crate::world::exchange_seen(processes);
        let answered = visit_answered(processes);
        let settled = processes.iter().any(|process| matches!(process, Process::Owner(owner) if owner.is_empty()))
            && processes.iter().all(|process| !matches!(process, Process::Visitor(visitor) if !visitor.is_empty()));
        for process in processes {
            match process {
                Process::Owner(owner) => {
                    owner.address = address;
                    if self.story == Story::SignInAgain && self.restarted.is_none() && visit.is_some() {
                        owner.sign_in_again();
                        self.restarted = Some(());
                    }
                    let retry = self.story == Story::PastedState
                        && self.pasted == 1
                        && owner.account_answers().iter().any(|answer| {
                            matches!(
                                answer,
                                jig_domain_accounts::Answer::Refused {
                                    refusal: jig_domain_accounts::Refusal::SignInRedirect {
                                        why: jig_domain_accounts::SignInRedirectWhy::State
                                    }
                                }
                            )
                        });
                    if pasting
                        && let Some(len) = redirect_len
                        && (self.pasted == 0 || retry)
                    {
                        if retry {
                            assert!(
                                owner.account_sign_in_ended().is_empty(),
                                "a refused paste leaves the sign-in waiting"
                            );
                        }
                        owner.hand_paste(&redirect[..len], self.story == Story::PastedState && self.pasted == 0);
                        self.pasted += 1;
                    }
                    if fixtures::rejects(self.story) && owner.child_heard().len() == 1 && !self.rejected {
                        owner.reject();
                        self.rejected = true;
                    }
                    if ends(self.story, owner, close_exchange, answered) && !self.closing {
                        owner.close();
                        self.closing = true;
                    }
                }
                Process::Issuer(peer) if settled && !self.shutdown => {
                    peer.shutdown();
                    self.shutdown = true;
                }
                Process::Issuer(_) => (),
                Process::Visitor(visitor) => {
                    if let Some(url) = &visit
                        && can_visit
                        && !visitor.handed()
                    {
                        if pasting {
                            visitor.stop_at_redirect();
                        }
                        visitor.hand(url);
                    }
                }
            }
        }
    }
    fn observe(&mut self, now: Time, processes: &[Process]) {
        let owner = processes
            .iter()
            .find_map(|process| match process {
                Process::Owner(owner) => Some(owner),
                Process::Issuer(_) | Process::Visitor(_) => None,
            })
            .expect("owner");
        kept_before_lent(owner);
        one_value_per_name(owner);
        for heard in owner.child_heard() {
            if let Heard::Granted { account, generation } = heard
                && !matches!(owner.child_heard().last(), Some(Heard::Failed { .. }))
            {
                assert!(
                    owner.account_removed() || owner.table_value(*account, *generation).is_some(),
                    "every granted generation is readable until expiry"
                );
            }
        }
        if owner.closed() {
            assert_eq!(owner.trace().last(), Some(&Trace::Closed), "Closed is last");
            assert_eq!(
                owner.became_usable(),
                !matches!(
                    self.story,
                    Story::DamagedRecord
                        | Story::RefusedGrant
                        | Story::NoRecord
                        | Story::AccountSignInMissingScope
                        | Story::AccountSignInFull
                ),
                "the real child accepted its grant only with a readable record; account sign-in ended {:?}",
                owner.account_sign_in_ended()
            );
            if self.story == Story::ClosedExchange {
                assert!(crate::world::exchange_finished(processes), "Closed follows the exchange's successful answer");
            }
            let expected = fixtures::expected(self.story);
            assert_eq!(owner.child_heard(), expected);
            no_value_outside_the_store(owner);
        }
        let paste_ready = matches!(self.story, Story::PastedAddress | Story::PastedState | Story::SignInAgain)
            && processes
                .iter()
                .any(|process| matches!(process, Process::Visitor(visitor) if visitor.redirect().is_some()))
            && (self.pasted == 0
                || (self.story == Story::PastedState && self.pasted == 1 && owner.account_answers().len() >= 2));
        self.wake = (paste_ready
            || (owner.unstarted()
                && processes
                    .iter()
                    .any(|process| matches!(process, Process::Issuer(peer) if peer.address().is_some())))
            || (owner.is_empty() && !self.shutdown)
            || (ends(self.story, owner, crate::world::exchange_seen(processes), visit_answered(processes))
                && !self.closing)
            || (owner.visit().is_some()
                && processes.iter().any(|process| matches!(process, Process::Visitor(visitor) if !visitor.handed())))
            || (self.story == Story::ClosedExchange && crate::world::exchange_seen(processes) && !self.closing)
            || (owner.child_heard().len() == 1 && !self.rejected && fixtures::rejects(self.story)))
        .then_some(now);
        self.passed = (self.shutdown && processes.iter().all(Host::is_empty)).then_some(());
    }
    fn next_deadline(&self) -> Option<Time> {
        self.wake.or(self.deadline)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (self.deadline.is_some_and(|due| now >= due) && self.passed.is_none())
            .then(|| "accounts world did not settle".to_owned())
    }
    fn passed(&self) -> bool {
        self.passed.is_some()
    }
}
/// Whether the owner's story is done and it may close: a sign-in's once its
/// visit is answered, an exchange's once it is seen, any other once the
/// child heard what it wanted.
fn ends(story: Story, owner: &Owner, exchange_seen: bool, _answered: bool) -> bool {
    match story {
        Story::SignIn | Story::PastedAddress | Story::PastedState | Story::SignInAgain | Story::AccountSignInVaryingPort => {
            !owner.child_heard().is_empty()
                || owner
                    .account_sign_in_ended()
                    .iter()
                    .any(|end| matches!(end.end, jig_domain_accounts::End::Failed { failure } if failure != jig_domain_accounts::Failure::Cancelled))
        }
        Story::AccountSignInMissingScope => !owner.account_sign_in_ended().is_empty() && owner.account_removed(),
        Story::AccountSignInFull => !owner.account_sign_in_ended().is_empty(),
        Story::AccountSignOut { .. } => owner.account_removed(),
        Story::RefusedRefresh | Story::RefusedGrant => {
            owner.account_record_erased() && owner.child_heard().len() >= fixtures::wanted(story)
        }
        Story::ClosedExchange => exchange_seen || owner.child_heard().len() >= fixtures::wanted(story),
        Story::Granted
        | Story::Refreshed
        | Story::Closed
        | Story::RateLimitedRefresh
        | Story::RejectedClient
        | Story::DamagedRecord
        | Story::NoRecord => owner.child_heard().len() >= fixtures::wanted(story),
    }
}
/// Whether the visitor's visit has its answer.
fn visit_answered(processes: &[Process]) -> bool {
    processes.iter().any(|process| matches!(process, Process::Visitor(visitor) if visitor.status().is_some()))
}
/// Each granted generation's record was read from the store at start, or
/// the store answered that it is durable, before the grant was heard.
pub(crate) fn kept_before_lent(owner: &Owner) {
    let trace = owner.trace();
    for (index, entry) in trace.iter().enumerate() {
        if let Trace::Heard(Heard::Granted { account, generation }) = entry {
            let kept = trace[..index].iter().any(|before| {
                matches!(before, Trace::Loaded { account: a, generation: g } | Trace::Kept { account: a, generation: g }
                    if a == account && g == generation)
            });
            assert!(kept, "no token reaches a run before its record is kept: {account} generation {generation}");
        }
    }
}
/// A source's generation names one value.
pub(crate) fn one_value_per_name(owner: &Owner) {
    let lent = owner.lent();
    for (index, first) in lent.iter().enumerate() {
        for second in &lent[index + 1..] {
            if (first.account, first.generation) == (second.account, second.generation) {
                assert_eq!(first.digest, second.digest, "a name never means two values");
            }
        }
    }
}
/// What the owner holds outside the store, its trace and the child's facts,
/// names no token; the driver files nothing (the owner's `iterate` checks).
pub(crate) fn no_value_outside_the_store(owner: &Owner) {
    let facts = format!("{:?}", owner.facts());
    let trace = format!("{:?}", owner.trace());
    for value in fixtures::VALUES {
        let value = std::str::from_utf8(value).expect("fixture values are text");
        assert!(!facts.contains(value) && !trace.contains(value), "no value outside the secrets' store");
    }
}
