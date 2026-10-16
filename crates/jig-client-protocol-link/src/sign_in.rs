//! Signing belongs to the link (client/domain/client.md, 12).
use crate::{
    Link, Message, Request, Signing, With,
    component::{Attempt, ExchangeKind},
};
use jig_client_protocol_wire::v7 as w;
use skein_lib::{Queue, Reader, Secret, Time, Token, Writer};
impl Link {
    #[must_use]
    pub fn credential(&self) -> Option<&Secret> {
        self.credential.as_ref()
    }
    pub(super) fn sign_in(&mut self, now: Time, with: With, out: &mut Queue<Request>) {
        match with {
            With::Held(credential) => {
                self.generation = self.generation.saturating_add(1);
                self.credential = Some(credential);
            }
            With::Link(secret) => {
                let Ok(record) = w::LinkSignIn::new(
                    &self.limits.wire,
                    w::LinkSignInParts { secret: skein_lib::bytes::copy_of(secret.expose()) },
                ) else {
                    out.push(Request::Signing { signing: Signing::Refused });
                    return;
                };
                let mut writer = Writer::new(usize::try_from(record.measure()).expect("short sign-in"));
                record.encode(&mut writer).expect("measured sign-in");
                self.signing_exchange(now, ExchangeKind::SignIn, Message::SignInLink(writer.finish()), out);
            }
        }
    }
    pub(super) fn sign_out(&mut self, now: Time, out: &mut Queue<Request>) {
        self.signing_exchange(now, ExchangeKind::SignOut { generation: self.generation }, Message::SignOut, out);
    }
    fn signing_exchange(&mut self, _now: Time, kind: ExchangeKind, message: Message, out: &mut Queue<Request>) {
        let Some(next) = self.next.checked_add(1) else {
            out.push(Request::Signing { signing: Signing::Lost });
            return;
        };
        if self.attempts.len() == self.attempts.capacity() {
            out.push(Request::Signing { signing: Signing::Lost });
            return;
        }
        let exchange = Token::new(self.next);
        self.next = next;
        self.attempts
            .insert(exchange, Attempt { token: exchange, owner: w::Owner::Jig, version: 7, kind })
            .expect("checked exchange capacity");
        self.exchange(exchange, message, out);
    }
    pub(super) fn signing_answer(&mut self, kind: ExchangeKind, body: &[u8], out: &mut Queue<Request>) {
        match kind {
            ExchangeKind::SignOut { generation } if generation != self.generation => return,
            ExchangeKind::SignOut { .. }
            | ExchangeKind::SignIn
            | ExchangeKind::Attempt
            | ExchangeKind::Read
            | ExchangeKind::WatchOpen { .. }
            | ExchangeKind::WatchClose { .. } => {}
        }
        let mut reader = Reader::new(body);
        let signing = {
            match kind {
                ExchangeKind::SignIn => match w::SignInAnswer::decode(&self.limits.wire, &mut reader) {
                    Ok(w::SignInAnswer::SignedIn(value)) if reader.remaining() == 0 && value.token().len() == 40 => {
                        let fields = value.into_parts();
                        self.credential = Some(fields.token);
                        self.generation = self.generation.saturating_add(1);
                        Signing::SignedIn { party: fields.party }
                    }
                    Ok(w::SignInAnswer::NotReady) if reader.remaining() == 0 => Signing::Again,
                    Ok(w::SignInAnswer::Refused) if reader.remaining() == 0 => Signing::Refused,
                    Ok(_) | Err(_) => Signing::Lost,
                },
                ExchangeKind::SignOut { .. } => match w::SignOutAnswer::decode(&self.limits.wire, &mut reader) {
                    Ok(w::SignOutAnswer::SignedOut) if reader.remaining() == 0 => {
                        self.credential = None;
                        Signing::SignedOut
                    }
                    Ok(w::SignOutAnswer::NotReady) if reader.remaining() == 0 => Signing::Again,
                    Ok(_) | Err(_) => Signing::Lost,
                },
                ExchangeKind::Attempt
                | ExchangeKind::Read
                | ExchangeKind::WatchOpen { .. }
                | ExchangeKind::WatchClose { .. } => unreachable!("signing exchange"),
            }
        };
        out.push(Request::Signing { signing });
    }
}
