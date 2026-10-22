//! Boundary-only jig decisions and raw requests for negative referee proofs.
use super::{App, ClientId, Front, Observation, Wall, World};
impl<A: App, F: Front<A>> World<A, F> {
    pub(super) fn decision(&mut self, client: ClientId, request: &A::Request, acted: bool) {
        if let Some(ask) = A::decision(request) {
            if matches!(ask, jig_client::contract::Ask::Seen { .. }) {
                return;
            }
            let sent =
                A::sent(request).or_else(|| self.front.sent(request)).expect("keyed decision has a wire attempt");
            // The shell sees the wire attempt, rather than the application's private request-table name.
            let key = jig_client::Key { asker: jig_client::Asker::JIG, number: sent.attempt.raw() };
            // A retry sends no new person act; keep the first scene across reloads.
            let first = self.clients[client.0].decisions.insert(sent.key);
            let scene = if acted && first { self.clients[client.0].acted_scene.clone() } else { None };
            self.observe(Observation::Decision { client, key, ask: Box::new(ask.clone()), scene });
        }
    }
    /// Send a raw boundary request past the reference presentation for a referee proof.
    pub fn send_raw(&mut self, client: ClientId, request: A::Request) {
        self.decision(client, &request, false);
        let wall = Wall::from_nanos(1_000_000_000_000 + self.now.as_nanos());
        self.front.wire(client, self.now, wall, request);
    }
}

#[cfg(test)]
mod tests {
    use super::{App, Front};
    use crate::{Test, stories};
    use jig_client::contract::{Ask, SeenThrough};
    use jig_client_test_domain::Request;
    use skein_lib::Token;
    use skein_world::domain::Verdict;

    const REQUEST_KEY: [u8; 16] = [3; 16];

    fn send(ask: Ask) -> Request {
        Request::Client {
            request: jig_client::Request::Down {
                down: jig_client::Down::Send {
                    attempt: Token::new(7),
                    key: REQUEST_KEY,
                    request: jig_client::Ask::Jig(Box::new(ask)),
                    value: None,
                },
            },
        }
    }

    #[test]
    fn seen_without_a_sent_observation_is_not_a_decision() {
        let mut world = stories::world(1);
        let client = world.client(7, stories::inbox());
        let before = world.clients[client.0].decisions.clone();
        for through in [SeenThrough::Results { through: 1 }, SeenThrough::Conversation { task: 9, through: 1 }] {
            let request = send(Ask::Seen { through });
            assert!(matches!(Test::decision(&request), Some(Ask::Seen { .. })));
            assert!(Test::sent(&request).is_none());
            assert!(world.front.sent(&request).is_none());
            for acted in [false, true] {
                world.decision(client, &request, acted);
            }
        }
        assert_eq!(world.clients[client.0].decisions, before);
        assert_eq!(world.referee.verdict(), Verdict::Passed);
    }

    #[test]
    fn an_ordinary_request_without_a_card_is_still_a_decision() {
        let mut world = stories::world(1);
        let client = world.client(7, stories::inbox());
        let request = send(Ask::Take { project: 1, task: 9 });
        assert!(Test::sent(&request).expect("ordinary request has a wire attempt").card.is_none());
        assert!(!world.clients[client.0].decisions.contains(&REQUEST_KEY));
        world.decision(client, &request, false);
        assert!(world.clients[client.0].decisions.contains(&REQUEST_KEY));
        assert_eq!(world.referee.verdict(), Verdict::Passed);
    }
}
