//! Actual owner pressure/recovery and transport faults; no invented owner answers.
use super::{ClientId, Front, FrontFault, Protocol, Time, Wall, fixtures};
use skein_lib::Duration;
impl Protocol {
    pub(super) fn apply_fault(&mut self, now: Time, fault: FrontFault) {
        self.now = now;
        self.engine.clock(now, fixtures::wall(now));
        self.net.clock(now);
        match fault {
            FrontFault::Slow { span } => self.net.slow(self.last_client.expect("a client wired"), span),
            FrontFault::NoAnswer => self.net.cut_answer(),
            FrontFault::DropWatch => self.net.cut_stream(self.last_client.expect("a stream opened")),
            FrontFault::SignedOut => {
                let client = self.last_client.expect("authenticated client");
                self.sign_out(client, now, fixtures::wall(now));
            }
            fault @ (FrontFault::Busy
            | FrontFault::NotReady
            | FrontFault::Restart { .. }
            | FrontFault::Versions
            | FrontFault::ObjectGone) => self.owner_fault(now, &fault),
        }
    }
    fn owner_fault(&mut self, now: Time, fault: &FrontFault) {
        match fault {
            FrontFault::Busy => self.engine.driver.scripted_party(crate::stories::pressure(), now, fixtures::wall(now)),
            FrontFault::NotReady => {
                self.engine.driver.pause_restore(
                    jig_core_world::world::config(self.engine.seed),
                    now,
                    fixtures::wall(now),
                );
                self.release = Some(now.saturating_add(Duration::from_millis(100)));
            }
            FrontFault::Restart { offline } => {
                self.net.cut_all();
                self.advance(now, fixtures::wall(now));
                self.engine.driver.restart(jig_core_world::world::config(self.engine.seed), now, fixtures::wall(now));
                self.restart = Some(now.saturating_add(*offline));
                self.net.offline(self.restart.expect("recovery deadline"));
                self.reset_component();
            }
            FrontFault::Versions => {
                self.net.cut_all();
                self.advance(now, fixtures::wall(now));
                self.versions = true;
                self.reset_component();
            }
            FrontFault::ObjectGone
            | FrontFault::Slow { .. }
            | FrontFault::NoAnswer
            | FrontFault::DropWatch
            | FrontFault::SignedOut => panic!("ObjectGone belongs to the application's independent source"),
        }
    }
    fn reset_component(&mut self) {
        while self.engine.out.pop().is_some() {}
        let mut settings = fixtures::settings(self.engine.limits.core.documents.rows);
        if self.versions {
            settings.served = jig_client_world::test::list([
                jig_protocol_engine_clients::Served { family: None, first: u32::MAX, last: u32::MAX },
                jig_protocol_engine_clients::Served { family: Some(1), first: 1, last: 1 },
            ]);
        }
        self.engine.component =
            jig_protocol_engine_clients::Component::new(settings, &self.engine.env, &mut self.engine.out)
                .expect("bounded replacement");
    }
    pub(super) fn release_faults(&mut self, now: Time, wall: Wall) {
        if self.release.is_some_and(|until| until <= now) {
            self.release = None;
            self.engine.driver.finish_restore(now, wall);
        }
        if self.restart.is_some_and(|until| until <= now) {
            self.restart = None;
        }
    }
    pub fn select_stream(&mut self, client: ClientId) {
        self.last_client = Some(client);
    }
    #[must_use]
    pub fn deliveries(&self, client: ClientId) -> u64 {
        self.deliveries.get(&client).copied().unwrap_or(0)
    }
    pub fn cut_upload(&mut self) {
        self.net.cut_upload();
    }
    #[must_use]
    pub fn cuts(&self) -> u64 {
        self.net.cuts
    }
    #[must_use]
    pub fn rules(&self) -> &crate::referee::Rules {
        &self.engine.rules
    }
    #[must_use]
    pub fn answers(&self, client: ClientId) -> u64 {
        self.answers.get(&client).copied().unwrap_or(0)
    }
    pub fn check_isolation(&mut self, unread: ClientId, other: ClientId, before: u64) {
        self.engine
            .rules
            .isolation(self.net.unread(unread), self.answers(other) > before)
            .expect("a slow stream holds no other exchange");
    }
    #[must_use]
    pub fn requests_made(&self) -> usize {
        self.engine
            .driver
            .world
            .store
            .rows
            .values()
            .filter(|record| {
                matches!(
                    record,
                    jig_test_domain::Record::Core(jig_domain_core::Record::People(
                        jig_domain_people::Stored::Answer { .. }
                    ))
                )
            })
            .count()
    }
    #[must_use]
    pub fn words_reached_worker(&self, words: &[u8]) -> bool {
        self.engine.driver.world.peers.as_ref().expect("worker").workers[0]
            .messages
            .iter()
            .any(|(_, _, _, value)| value.as_ref() == words)
    }
}
