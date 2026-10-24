//! Independent owners share an issuer, and only the test carries the private file.
use crate::{
    fixtures,
    owner::{Heard, Owner, Trace},
    world::{self, Process, Story},
};
use jig_domain_accounts::{Answer, Change, Failure, Refusal};
use jig_protocol_accounts::moved;
use skein_fake_peers::oauth::Observation;
use skein_lib::{Duration, Secret, Time, Wall, bytes};
use skein_world::{Host, Memory, Outcome, Referee, World};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Wrong {
    Application,
    Issuer,
    Client,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoveStory {
    Written,
    Dead,
    Cancelled,
    NotOurs(Wrong),
    Restart,
}
struct Judge {
    story: MoveStory,
    phase: u8,
    file: Option<Secret>,
    deadline: Time,
    wake: Option<Time>,
    passed: bool,
    refreshed: bool,
}
impl Judge {
    fn new(story: MoveStory) -> Self {
        Self {
            story,
            phase: 0,
            file: None,
            deadline: Time::from_nanos(Duration::from_secs(150).as_nanos()),
            wake: None,
            passed: false,
            refreshed: false,
        }
    }
    fn arrive(&self, owner: &mut Owner) {
        let mut file = self.file.as_ref().expect("file taken").clone();
        if let MoveStory::NotOurs(wrong) = self.story {
            let mut decoded = moved::decode(file.expose(), &fixtures::limits().client.document).expect("valid file");
            match wrong {
                Wrong::Application => decoded.application = bytes::copy_of(b"another-application"),
                Wrong::Issuer => decoded.issuer = bytes::copy_of(b"https://another-issuer.example"),
                Wrong::Client => decoded.client_id = bytes::copy_of(b"another-client"),
            }
            file = moved::encode(&decoded, &fixtures::limits().client.document).expect("changed identity");
        }
        owner.carry_move(&file);
    }
    fn take_file(&mut self, owner: &mut Owner) {
        if let Some((number, source)) = owner.account_answers().iter().find_map(|answer| match answer {
            Answer::MovingOut { number, source, .. } => Some((*number, *source)),
            Answer::Finishing { .. }
            | Answer::MovedIn { .. }
            | Answer::ProgramDeclared { .. }
            | Answer::KeyKept { .. }
            | Answer::Done
            | Answer::Removed { .. }
            | Answer::Started { .. }
            | Answer::SignedOut { .. }
            | Answer::Refused { .. } => None,
        }) {
            assert!(!owner.source_usable(), "withdrawn source is unavailable");
            self.file = owner.moved_file(source).cloned();
            assert!(self.file.is_some(), "moving out carries the file");
            if self.story != MoveStory::Cancelled {
                owner.change(92, Change::Written { number });
            }
            self.phase = 2;
        }
    }
    fn advance(&mut self, now: Time, owners: &mut [&mut Owner]) {
        let a = &owners[0];
        match self.phase {
            0 if !a.unstarted() && !a.lent().is_empty() => {
                if self.story == MoveStory::Restart {
                    assert_eq!(a.lent()[0].generation, 2, "restart never lends the carried generation");
                    assert!(
                        now < Time::from_nanos(Duration::from_secs(200).as_nanos()),
                        "restart clears the refresh floor"
                    );
                    self.phase = 9;
                } else {
                    owners[0].change(91, Change::MoveOut { account: 1000 });
                    self.phase = 1;
                }
            }
            1 => {
                self.take_file(owners[0]);
            }
            2 => {
                let cancelled = self.story == MoveStory::Cancelled;
                if (cancelled && a.lent().len() == 2) || (!cancelled && a.account_answers().contains(&Answer::Done)) {
                    if cancelled {
                        assert!(a.source_usable(), "cancelled source is refreshed and usable again");
                        assert_eq!(a.lent()[1].generation, 2);
                        assert!(
                            now < Time::from_nanos(Duration::from_secs(200).as_nanos()),
                            "return clears the refresh floor"
                        );
                    } else {
                        assert!(owners[0].read_record().is_none(), "Written erased A's source row");
                    }
                    self.arrive(owners[1]);
                    self.phase = 3;
                }
            }
            3 => {
                let b = &owners[1];
                match self.story {
                    MoveStory::Cancelled if failed(b) => {
                        assert!(b.lent().is_empty(), "the cancelled file never lends");
                        assert!(owners[0].source_usable(), "A stays usable after the stale file is refused");
                        self.phase = 9;
                    }
                    MoveStory::NotOurs(_)
                        if b.account_answers().contains(&Answer::Refused { refusal: Refusal::NotOurs }) =>
                    {
                        assert!(b.lent().is_empty());
                        assert!(
                            owners[1].carried_value(owners[1].carried_move().expect("carried file")).is_some(),
                            "NotOurs never adopts the carried row"
                        );
                        assert!(owners[1].read_record().is_none());
                        self.phase = 9;
                    }
                    MoveStory::Written | MoveStory::Dead if !b.lent().is_empty() => {
                        assert_eq!(b.lent().len(), 1);
                        assert_eq!(b.lent()[0].generation, 2, "B refreshes before its first lend");
                        assert!(b.account_answers().iter().any(|answer| matches!(answer, Answer::MovedIn { .. })));
                        assert!(
                            now < Time::from_nanos(Duration::from_secs(200).as_nanos()),
                            "adoption clears the refresh floor"
                        );
                        assert!(owners[1].carried_value(owners[1].carried_move().expect("carried file")).is_none());
                        if self.story == MoveStory::Dead {
                            owners[2].carry_move(self.file.as_ref().expect("left behind"));
                            self.phase = 4;
                        } else {
                            self.phase = 9;
                        }
                    }
                    MoveStory::Written
                    | MoveStory::Dead
                    | MoveStory::Cancelled
                    | MoveStory::NotOurs(_)
                    | MoveStory::Restart => (),
                }
            }
            4 if failed(owners[2]) => {
                assert!(owners[2].lent().is_empty(), "the file left behind never lends");
                assert!(owners[1].source_usable(), "B stays usable");
                self.phase = 9;
            }
            _ => (),
        }
        if self.phase == 9 {
            for owner in owners {
                owner.close();
            }
            self.phase = 10;
        }
    }
}
fn failed(owner: &Owner) -> bool {
    owner.child_heard().iter().any(|event| matches!(event, Heard::Failed { failure: Failure::SignInAgain, .. }))
        && owner.account_record_erased()
}
impl Referee<Process> for Judge {
    fn act(&mut self, now: Time, processes: &mut [Process]) {
        let address = processes.iter().find_map(|process| match process {
            Process::Issuer(peer) => peer.address(),
            Process::Owner(_) | Process::Visitor(_) => None,
        });
        let mut owners = processes
            .iter_mut()
            .filter_map(|process| match process {
                Process::Owner(owner) => Some(owner.as_mut()),
                Process::Issuer(_) | Process::Visitor(_) => None,
            })
            .collect::<Vec<_>>();
        for owner in &mut owners {
            owner.address = address;
        }
        self.advance(now, &mut owners);
        let settled = owners.iter().all(|owner| owner.is_empty());
        if settled && self.phase == 10 {
            for process in processes {
                if let Process::Issuer(peer) = process {
                    peer.shutdown();
                }
            }
            self.phase = 11;
        }
    }
    fn observe(&mut self, now: Time, processes: &[Process]) {
        for process in processes {
            if let Process::Owner(owner) = process {
                crate::referee::kept_before_lent(owner);
                crate::referee::one_value_per_name(owner);
                crate::referee::no_value_outside_the_store(owner);
            }
        }
        self.passed = self.phase == 11 && processes.iter().all(Host::is_empty);
        let owners = processes
            .iter()
            .filter_map(|process| match process {
                Process::Owner(owner) => Some(owner.as_ref()),
                Process::Issuer(_) | Process::Visitor(_) => None,
            })
            .collect::<Vec<_>>();
        if !self.refreshed && !matches!(self.story, MoveStory::NotOurs(_)) {
            let (owner, grant) = match self.story {
                MoveStory::Cancelled => (owners[0], 1),
                MoveStory::Restart => (owners[0], 0),
                MoveStory::Written | MoveStory::Dead => (owners[1], 0),
                MoveStory::NotOurs(_) => unreachable!("excluded"),
            };
            if let Some((account, generation)) = owner
                .child_heard()
                .iter()
                .filter_map(|heard| match heard {
                    Heard::Granted { account, generation } => Some((*account, *generation)),
                    Heard::Expiring { .. } | Heard::Failed { .. } => None,
                })
                .nth(grant)
            {
                assert_eq!(generation, 2, "the first grant after Refresh disowns the moved generation");
                let trace = owner.trace();
                let kept = trace.iter().position(|entry| *entry == Trace::Kept { account, generation });
                let lent =
                    trace.iter().position(|entry| *entry == Trace::Heard(Heard::Granted { account, generation }));
                assert!(kept.is_some() && kept < lent, "the new generation is kept before its first grant");
                let observations = processes
                    .iter()
                    .find_map(|process| match process {
                        Process::Issuer(peer) => Some(peer.observations()),
                        Process::Owner(_) | Process::Visitor(_) => None,
                    })
                    .expect("issuer");
                assert_eq!(
                    observations.iter().filter(|event| matches!(event, Observation::Post { .. })).count(),
                    1,
                    "exactly one refresh before the first grant"
                );
                assert_eq!(
                    observations
                        .iter()
                        .filter(|event| matches!(event, Observation::Answered { status: 200, .. }))
                        .count(),
                    1
                );
                self.refreshed = true;
            }
        }
        let ready = match self.phase {
            0 => !owners[0].unstarted() && !owners[0].lent().is_empty(),
            1 => owners[0].account_answers().iter().any(|answer| matches!(answer, Answer::MovingOut { .. })),
            2 => {
                if self.story == MoveStory::Cancelled {
                    owners[0].lent().len() == 2
                } else {
                    owners[0].account_answers().contains(&Answer::Done)
                }
            }
            3 => {
                failed(owners[1])
                    || !owners[1].lent().is_empty()
                    || owners[1].account_answers().contains(&Answer::Refused { refusal: Refusal::NotOurs })
            }
            4 => failed(owners[2]),
            10 => owners.iter().all(|owner| owner.is_empty()),
            _ => false,
        };
        let starting = owners.iter().any(|owner| owner.unstarted())
            && processes.iter().any(|process| matches!(process, Process::Issuer(peer) if peer.address().is_some()));
        self.wake = (ready || starting).then_some(now);
    }
    fn next_deadline(&self) -> Option<Time> {
        self.wake.or(Some(self.deadline))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now >= self.deadline && !self.passed).then(|| format!("move {:?} stalled at phase {}", self.story, self.phase))
    }
    fn passed(&self) -> bool {
        self.passed
    }
}
#[must_use]
pub fn run_move(seed: u64, story: MoveStory) -> Outcome<Process> {
    let mut config = skein_sim::Config::calm();
    config.wall = Wall::EPOCH;
    let mut world = World::new(seed, config, Judge::new(story), Memory::Checked);
    world.spawn(|| world::issuer(Story::Granted));
    world.spawn(|| Process::Owner(Box::new(Owner::new_move(true, story == MoveStory::Restart))));
    if story != MoveStory::Restart {
        world.spawn(|| Process::Owner(Box::new(Owner::new_move(false, false))));
    }
    if story == MoveStory::Dead {
        world.spawn(|| Process::Owner(Box::new(Owner::new_move(false, false))));
    }
    world.run()
}
