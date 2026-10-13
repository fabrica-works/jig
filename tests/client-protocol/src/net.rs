//! io records joined to shell connections by two seeded stream faces.
use jig_client_world::ClientId;
use jig_shell_native::Conn;
use skein_io::{Event, Request};
use skein_lib::{
    Token,
    stream::{Down, Up},
};
use skein_world::stream::Wire;
use std::collections::BTreeMap;
struct Socket {
    client: ClientId,
    conn: Conn,
    owner: Option<Token>,
    shell: Wire,
    engine: Wire,
    sent_shell: usize,
    sent_engine: usize,
}
pub(super) struct Net {
    sockets: BTreeMap<Token, Socket>,
    names: BTreeMap<(ClientId, Conn), Token>,
    next: u64,
    listener: Token,
}
impl Net {
    pub fn new() -> Self {
        Self { sockets: BTreeMap::new(), names: BTreeMap::new(), next: 100, listener: Token::new(0) }
    }
    pub fn down(&mut self, client: ClientId, conn: Conn, down: Down) -> Option<Event> {
        let event = if self.names.contains_key(&(client, conn)) {
            None
        } else {
            let socket = Token::new(self.next);
            self.next += 1;
            self.names.insert((client, conn), socket);
            self.sockets.insert(
                socket,
                Socket {
                    client,
                    conn,
                    owner: None,
                    shell: Wire::new(self.next),
                    engine: Wire::new(self.next + 1),
                    sent_shell: 0,
                    sent_engine: 0,
                },
            );
            Some(Event::Accepted { owner: self.listener, socket, peer: crate::fixtures::address() })
        };
        let socket = self.names[&(client, conn)];
        self.sockets.get_mut(&socket).expect("named socket").shell.take(down);
        event
    }
    pub fn io(&mut self, request: Request) -> Option<Event> {
        match request {
            Request::Listen { owner, addr } => {
                self.listener = owner;
                Some(Event::Listening { owner, listener: Token::new(99), addr })
            }
            Request::Bind { socket, owner } => {
                self.sockets.get_mut(&socket).expect("accepted socket").owner = Some(owner);
                None
            }
            Request::Stream { stream, down } => {
                self.sockets.get_mut(&stream).expect("bound socket").engine.take(down);
                None
            }
            Request::Close { entity } | Request::Abort { entity } => {
                let socket = self.sockets.remove(&entity).expect("owned socket");
                Some(Event::Closed { owner: socket.owner.expect("bound socket") })
            }
            Request::Reject { socket } => {
                self.sockets.remove(&socket);
                None
            }
            Request::Connect { .. }
            | Request::Spawn { .. }
            | Request::Signal { .. }
            | Request::Usage { .. }
            | Request::Window { .. }
            | Request::Output { .. } => panic!("clients component requested unrelated io"),
        }
    }
    pub fn advance(&mut self) -> (Vec<Event>, Vec<(ClientId, Conn, Up)>, bool) {
        let mut events = Vec::new();
        let mut clients = Vec::new();
        let mut changed = false;
        for socket in self.sockets.values_mut() {
            socket.engine.write(&socket.shell.received[socket.sent_shell..]);
            socket.sent_shell = socket.shell.received.len();
            socket.shell.write(&socket.engine.received[socket.sent_engine..]);
            socket.sent_engine = socket.engine.received.len();
            socket.engine.eof = socket.shell.finished;
            if let Some(up) = socket.shell.answer() {
                clients.push((socket.client, socket.conn, up));
                changed = true;
            }
            if let Some(owner) = socket.owner
                && let Some(up) = socket.engine.answer()
            {
                events.push(Event::Stream { owner, up });
                changed = true;
            }
        }
        (events, clients, changed)
    }
}
