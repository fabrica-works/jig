//! One admitted exchange and its answer (protocol/client.md, 4.2).
use crate::{Ended, Limits, Reply, Request};
use skein_lib::{Queue, Token};
#[derive(Debug)]
pub(crate) struct Exchange {
    pub name: Option<Token>,
    pub stream: bool,
    pub response: Option<Reply>,
    pub ended: Option<Ended>,
    pub closing: bool,
}
impl Exchange {
    pub fn new(exchange: Token, stream: bool) -> Self {
        Self { name: Some(exchange), stream, response: None, ended: None, closing: false }
    }
    pub fn respond(&mut self, reply: Reply, _limits: &Limits) {
        self.response = Some(reply);
    }
    pub fn close(&mut self, _out: &mut Queue<Request>) {
        if self.stream {
            self.ended = Some(Ended::Stopping);
        } else {
            self.closing = true;
        }
    }
}
