//! Local client refusals remain distinct from engine answers (view.md, 18.4).
use crate::{
    Limits, Request, Ui,
    intents::{self, NotSent, Origin},
    notices::kept::{self, Kind},
};
use jig_client::{Answered, Client, Key, Outcome};
use skein_lib::{Env, Id, Queue, Token};
pub(super) fn answered(
    domain: &mut Ui,
    env: &Env<Limits>,
    client: &Client,
    key: Key,
    outcome: Outcome,
    salt: u64,
    out: &mut Queue<Request>,
) {
    let token = Token::new(key.number ^ salt);
    if domain.kept_requests.remove(&key).is_some() {
        super::super::refresh_kept_page(domain, env, out, client);
        if !domain.intents.ids().contains_key(&Id::from_token(token)) {
            match outcome {
                Outcome::Answered(Answered::Jig(jig_client::contract::Answer::Refused(why))) => {
                    kept::show(domain, Kind::PageAnswer { answer: jig_client::contract::Answer::Refused(why) });
                }
                Outcome::Answered(Answered::Jig(jig_client::contract::Answer::Done(_)) | Answered::Family(_)) => {}
                Outcome::Unknown => kept::show(domain, Kind::KeptUnknown),
                Outcome::NotKept => kept::show(domain, Kind::NotSent(NotSent::NotKept)),
                Outcome::Limit => kept::show(domain, Kind::NotSent(NotSent::Limit)),
                Outcome::Unconfirmed => kept::show(domain, Kind::NotSent(NotSent::Unconfirmed)),
            }
            return;
        }
    }
    match outcome {
        Outcome::Answered(Answered::Jig(answer)) => intents::answered(domain, env, token, answer, out, client),
        Outcome::Answered(Answered::Family(_)) | Outcome::Unknown => {
            intents::progress(domain, token, intents::Progress::Unknown, out, client);
        }
        Outcome::NotKept => not_sent(domain, token, NotSent::NotKept),
        Outcome::Limit => not_sent(domain, token, NotSent::Limit),
        Outcome::Unconfirmed => not_sent(domain, token, NotSent::Unconfirmed),
    }
}
fn not_sent(domain: &mut Ui, token: Token, why: NotSent) {
    let Some(origin) = domain.intents.unknown(token) else { return };
    domain.conversation.words.refused(token);
    match origin {
        Origin::Card { object, .. } => match domain.objects.find(object) {
            Some(id) => {
                let object = domain.objects.get_mut(id).expect("named card exists");
                if !object.shown {
                    kept::show(domain, Kind::NotSent(why));
                }
                let object = domain.objects.get_mut(id).expect("named card exists");
                object.card = crate::objects::Card::NotSent { why };
                object.answer = None;
            }
            None => kept::show(domain, Kind::NotSent(why)),
        },
        Origin::Page { .. } => {
            if domain.chats.starting == Some(token) {
                domain.chats.starting = None;
                domain.chats.words = None;
            }
            crate::page::project::goals::unknown(domain, token);
            crate::page::project::owner_form::unknown(domain, token);
            kept::show(domain, Kind::NotSent(why));
        }
    }
    domain.changed();
}
