//! Boundary-only jig decisions and raw requests for negative referee proofs.
use super::{App, ClientId, Front, Observation, Wall, World};
impl<A: App, F: Front<A>> World<A, F> {
    pub(super) fn decision(&mut self, client: ClientId, request: &A::Request, acted: bool) {
        if let Some(ask) = A::decision(request) {
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
