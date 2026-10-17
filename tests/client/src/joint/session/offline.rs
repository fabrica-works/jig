//! A lost transport ends each watch attempt; the native client owns reopening.
use super::{ClientId, Event, Request, Session, client};
impl Session {
    pub(super) fn keep_reload_bootstrap(&mut self) {
        self.callbacks.retain(|(client, event)| {
            self.reloaded.contains(client)
                && self.driver.transport.sign_in(*client).is_some()
                && matches!(event, Event::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedIn } })
        });
    }

    pub(super) fn disconnected(&mut self, client: ClientId, request: Request) {
        match request {
            Request::Client {
                request:
                    jig_client::Request::Down {
                        down: jig_client::Down::Open { watch, .. } | jig_client::Down::Continue { watch, .. },
                    },
            } => self.callbacks.push((
                client,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::WatchEnded { watch, end: client::WatchEnd::Dropped },
                    },
                },
            )),
            request @ (Request::Save { .. }
            | Request::Erase { .. }
            | Request::Client { .. }
            | Request::Jig(_)
            | Request::First(_)
            | Request::Show { .. }
            | Request::SignIn { .. }
            | Request::SignOut
            | Request::Fact { .. }) => self.parked.entry(client).or_default().push(request),
        }
    }
}
