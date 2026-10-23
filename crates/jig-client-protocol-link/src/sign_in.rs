//! Signing belongs to the link (client/domain/client.md, 12).
use crate::{
    Link, Message, Request, Signing, With,
    component::{Attempt, ExchangeKind},
};
use jig_client_protocol_wire::v14 as w;
use skein_lib::{Queue, Reader, Secret, Time, Token, Writer};
impl Link {
    #[must_use]
    pub fn credential(&self) -> Option<&Secret> {
        self.credential.as_ref()
    }
    pub(super) fn sign_in(&mut self, _now: Time, with: With, out: &mut Queue<Request>) {
        match with {
            With::Held(credential) => {
                self.end_sign_in(out);
                self.generation = self.generation.saturating_add(1);
                self.credential = Some(credential);
            }
            With::Transport => {
                self.end_sign_in(out);
                self.generation = self.generation.saturating_add(1);
            }
            With::Service(credential) => {
                if credential.len() != 40 {
                    out.push(Request::Signing { signing: Signing::Refused });
                    return;
                }
                self.signing_exchange(ExchangeKind::SignIn, Message::SignInService, Some(credential), None, out);
            }
            With::Link { secret, held } => {
                let Ok(record) = w::LinkSignIn::new(
                    &self.limits.wire,
                    w::LinkSignInParts { secret: skein_lib::bytes::copy_of(secret.expose()) },
                ) else {
                    out.push(Request::Signing { signing: Signing::Refused });
                    return;
                };
                let mut writer = Writer::new(usize::try_from(record.measure()).expect("short sign-in"));
                record.encode(&mut writer).expect("measured sign-in");
                self.signing_exchange(
                    ExchangeKind::SignIn,
                    Message::SignInLink(writer.finish()),
                    held.clone(),
                    held,
                    out,
                );
            }
        }
    }
    pub(super) fn sign_out(&mut self, _now: Time, out: &mut Queue<Request>) {
        self.signing_exchange(
            ExchangeKind::SignOut { generation: self.generation },
            Message::SignOut,
            self.credential.clone(),
            None,
            out,
        );
    }
    fn signing_exchange(
        &mut self,
        kind: ExchangeKind,
        message: Message,
        credential: Option<Secret>,
        held: Option<Secret>,
        out: &mut Queue<Request>,
    ) {
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
            .insert(
                exchange,
                Attempt {
                    held,
                    token: exchange,
                    owner: w::Owner::Jig,
                    version: jig_client_protocol_wire::VERSION,
                    kind,
                },
            )
            .expect("checked exchange capacity");
        out.push(Request::Exchange { exchange, credential, message });
    }
    pub(super) fn signing_answer(
        &mut self,
        kind: ExchangeKind,
        held: Option<Secret>,
        body: &[u8],
        out: &mut Queue<Request>,
    ) {
        match kind {
            ExchangeKind::SignOut { generation } if generation != self.generation => return,
            ExchangeKind::SignOut { .. }
            | ExchangeKind::SignIn
            | ExchangeKind::Attempt { .. }
            | ExchangeKind::Read { .. }
            | ExchangeKind::WatchOpen { .. }
            | ExchangeKind::WatchClose { .. } => {}
        }
        let mut reader = Reader::new(body);
        let signing = {
            match kind {
                ExchangeKind::SignIn => match w::SignInAnswer::decode(&self.limits.wire, &mut reader) {
                    Ok(w::SignInAnswer::SignedIn(value))
                        if reader.remaining() == 0
                            && match value.token() {
                                Some(token) => token.len() == 40,
                                None => true,
                            } =>
                    {
                        let fields = value.into_parts();
                        self.end_sign_in(out);
                        self.credential = fields.token.or(held);
                        self.generation = self.generation.saturating_add(1);
                        Signing::SignedIn { party: fields.party }
                    }
                    Ok(w::SignInAnswer::NotReady) if reader.remaining() == 0 => Signing::Again,
                    Ok(w::SignInAnswer::Busy) if reader.remaining() == 0 => Signing::Busy,
                    Ok(w::SignInAnswer::Full(value)) if reader.remaining() == 0 => Signing::Full {
                        sign_ins: value.sign_ins(),
                        earliest: skein_lib::Wall::from_nanos(value.earliest()),
                        link_until: skein_lib::Wall::from_nanos(value.link_until()),
                    },
                    Ok(w::SignInAnswer::Refused) if reader.remaining() == 0 => Signing::Refused,
                    Ok(_) | Err(_) => Signing::Lost,
                },
                ExchangeKind::SignOut { .. } => match w::SignOutAnswer::decode(&self.limits.wire, &mut reader) {
                    Ok(w::SignOutAnswer::SignedOut) if reader.remaining() == 0 => {
                        self.end_sign_in(out);
                        Signing::SignedOut
                    }
                    Ok(w::SignOutAnswer::NotReady) if reader.remaining() == 0 => Signing::Again,
                    Ok(_) | Err(_) => Signing::Lost,
                },
                ExchangeKind::Attempt { .. }
                | ExchangeKind::Read { .. }
                | ExchangeKind::WatchOpen { .. }
                | ExchangeKind::WatchClose { .. } => unreachable!("signing exchange"),
            }
        };
        out.push(Request::Signing { signing });
    }
}
