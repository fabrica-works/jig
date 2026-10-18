//! Application mints, secrets erase fence, and component's terminal.
use super::{Event, Exchange, ExchangeState, Peer, SecretOwner, SecretsAnswer, Token, fixtures};
impl Peer {
    pub(super) fn close_socket(&mut self, entity: Token, state: ExchangeState) {
        *self.close_counts.entry(entity).or_default() += 1;
        if self.hold_closes && state == ExchangeState::Drained {
            self.closing_sockets.insert(entity);
            if let Some(exchange) = self.sockets.get_mut(&entity) {
                exchange.state = state;
            }
            return;
        }
        self.closing_sockets.remove(&entity);
        if entity == Token::new(99) {
            self.server.up(skein_io::Event::Closed { owner: Token::new(0) }, &mut self.server_out);
        }
        if let Some(exchange) = self.sockets.get_mut(&entity) {
            exchange.state = state;
            let owner = exchange.owner.expect("bound socket");
            self.server.up(skein_io::Event::Closed { owner }, &mut self.server_out);
        }
    }

    pub(super) fn erase_one(&mut self) {
        let owner = self.erases.pop_front().expect("erase requested");
        self.digests.remove(&owner);
        self.component.down(&self.env, Event::Secrets(SecretsAnswer::Erased { owner }), &mut self.out);
    }
    pub fn hold_closes(&mut self) {
        self.hold_closes = true;
    }
    pub fn hold_erases(&mut self) {
        self.hold_erases = true;
    }
    pub fn answer_erased(&mut self) {
        self.erase_one();
        self.pump();
    }
    #[must_use]
    pub fn has_digest(&self, sign_in: u64) -> bool {
        self.digests.contains_key(&SecretOwner::SignIn(sign_in))
    }
    pub fn mint(&mut self, identity: jig_domain_people::Identity) -> Token {
        let exchange = Token::new(self.next_mint);
        self.next_mint += 1;
        self.mint_identity = Some(identity.clone());
        self.event(Event::Mint { exchange, identity });
        exchange
    }
    pub fn minted(&mut self, exchange: Token) -> Option<(skein_lib::Secret, skein_lib::Wall)> {
        self.mints.remove(&exchange).map(|answer| answer.expect("minted credential"))
    }
    pub fn mint_refusal(&mut self, exchange: Token) -> Option<jig_protocol_engine_clients::MintRefusal> {
        self.mints.remove(&exchange).map(|answer| answer.expect_err("mint refused"))
    }
    #[must_use]
    pub fn closed(&self) -> usize {
        if self.server.closed() { self.closed } else { 0 }
    }
    #[must_use]
    pub fn socket_closes(&self, socket: Token) -> usize {
        self.close_counts.get(&socket).copied().unwrap_or(0)
    }
    #[must_use]
    pub fn stream_closed(&self, stream: [u8; 16]) -> bool {
        self.sockets[&self.streams[&stream]].state != ExchangeState::Active
    }
    #[must_use]
    pub fn stream_socket(&self, stream: [u8; 16]) -> Token {
        self.streams[&stream]
    }
    pub fn authenticated_post(&mut self, target: &[u8], body: Box<[u8]>) -> Token {
        let bearer = self.bearer.clone().expect("signed in");
        self.post(target, body, Some(&bearer))
    }
    pub fn accepted_link(&mut self, link: &[u8]) -> Token {
        use jig_client_protocol_wire::v8 as w;
        use skein_lib::{
            Writer,
            base64::{self, Alphabet},
        };
        let secret = link.split(|byte| *byte == b'=').next_back().expect("link secret");
        let secret = base64::decode(Alphabet::Url, secret, 32).expect("secret");
        let record = w::LinkSignIn::new(&self.env.limits.wire, w::LinkSignInParts { secret }).expect("link");
        let mut bytes = Writer::new(usize::try_from(record.measure()).expect("link size"));
        record.encode(&mut bytes).expect("link bytes");
        self.accepted_post(b"/jig/v1/sign-in/link", bytes.finish())
    }
    pub fn accepted_post(&mut self, target: &[u8], body: Box<[u8]>) -> Token {
        let socket = Token::new(self.next_socket);
        self.next_socket += 1;
        self.sockets.insert(socket, Exchange::new(skein_http::Method::Post, target, body, self.bearer.as_deref()));
        self.server.up(
            skein_io::Event::Accepted { owner: Token::new(0), socket, peer: fixtures::address() },
            &mut self.server_out,
        );
        socket
    }
    #[must_use]
    pub fn status(&self, socket: Token) -> Option<u16> {
        self.sockets[&socket].status
    }
    pub fn remember_sign_in(&mut self, socket: Token) {
        use skein_lib::{
            Writer,
            base64::{self, Alphabet},
        };
        let token = self.sign_in_answer(socket);
        let mut bearer = Writer::new(61);
        bearer.put(b"Bearer ").expect("prefix");
        base64::encode(Alphabet::Url, token.token().expose(), &mut bearer).expect("bearer");
        self.bearer = Some(bearer.finish());
    }
}
