use super::{ClientId, Event, Session, Time, Wall, client, root};
impl Session {
    /// A normal callback taken by the shell releases only its normal flight.
    pub fn received(&mut self, client: ClientId, now: Time, wall: Wall, event: &Event) {
        self.stopping().expect("whole original stopped routes");
        if matches!(event, Event::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedIn } }) {
            self.reloaded.remove(&client);
        }
        if let Event::Client {
            event:
                jig_client::Event::Up { up: jig_client::Up::Watched { watch, event: client::WatchEvent::Delivered(value) } },
        } = event
            && !matches!(value, client::Delivery::Run(_))
            && let Some(watcher) = self.driver.transport.watch_token(client, *watch)
        {
            self.driver.referee.consumed(watcher);
            if let Some(event) = self.driver.transport.taken(client, *watch) {
                self.driver.send(now, wall, root::Event::Core(event));
            }
        }
        if let Event::First(jig_client_test_domain::first::Event::Watch {
            watch,
            event: jig_client_test_domain::family::WatchEvent::Delivered(_),
        }) = event
            && let Some(watcher) = self.driver.transport.application_watch_token(client, *watch)
        {
            self.driver.referee.consumed(watcher);
            if let Some(event) = self.driver.transport.application_taken(client, *watch) {
                self.driver.send(now, wall, root::Event::Core(event));
            }
        }
    }
}
