use super::{ClientId, Event, Reply, Session, Time, Wall, core, packets, people, root};
impl Session {
    /// Retry sign-in using the supplied identity, not the expected party number.
    pub fn sign_in(&mut self, client: ClientId, now: Time, wall: Wall) {
        self.stopping().expect("whole original stopped routes");
        self.unsigned.remove(&client);
        self.sign_in_again.remove(&client);
        self.sign_in_failures.remove(&client);
        let event = self.driver.transport.start_sign_in(client);
        self.source(now, wall, root::Event::Core(event));
    }

    /// The source terminal of the latest provider exchange. It establishes no
    /// session and schedules no keyed or automatic sign-in retry (protocol/client.md, 8).
    #[must_use]
    pub fn sign_in_again(&self, client: ClientId) -> Option<people::Again> {
        self.sign_in_again.get(&client).copied()
    }

    /// The exact failure of the latest provider exchange. It neither creates a
    /// session nor retires an independently authenticated older session.
    #[must_use]
    pub fn sign_in_failure(&self, client: ClientId) -> Option<&packets::SignInFailure> {
        self.sign_in_failures.get(&client)
    }

    /// End the actual independent owner session through its durable `SignOut` door.
    /// An already absent session needs no fabricated owner reply.
    pub fn sign_out(&mut self, client: ClientId, now: Time, wall: Wall) {
        self.stopping().expect("whole original stopped routes");
        if self.offline() {
            self.callbacks.push((client, Event::Jig(jig_client_view::Event::SignOutLost)));
            return;
        }
        let Some(sign_in) = self.driver.transport.sign_in(client) else { return };
        let reply_to = self.driver.transport.reply_to(Reply::SignOut { client, sign_in });
        self.source(now, wall, root::Event::Core(core::Event::People(people::Event::SignOut { reply_to, sign_in })));
    }
}
