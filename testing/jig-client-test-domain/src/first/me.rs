//! The family side watches its own `me`; the root routes only by asker.
use super::{Limits, Request, Side};
use jig_client::{
    Client, Handle, Latest, Link, Told, Watchable,
    contract::{Delivery, Subject},
};
use skein_lib::{Env, Queue};
#[must_use]
pub fn handle() -> Handle {
    Handle { asker: jig_client::Asker(u32::from(crate::FIRST)), number: u64::MAX }
}
/// Consume only this side's link and me notifications.
pub fn told(side: &mut Side, env: &Env<Limits>, client: &Client, told: &Told, out: &mut Queue<Request>) -> bool {
    match told {
        Told::Link { link } => {
            side.link = *link;
            match link {
                Link::Starting | Link::Live | Link::Offline => {
                    if !side.me {
                        side.me = true;
                        out.push(Request::Client {
                            asked: jig_client::Event::Watch { handle: handle(), subject: Watchable::Jig(Subject::Me) },
                        });
                    }
                }
                Link::SignedOut | Link::OutOfDate => {
                    if side.me {
                        side.me = false;
                        out.push(Request::Client { asked: jig_client::Event::Unwatch { handle: handle() } });
                    }
                    party(side, env, None, out);
                }
            }
            super::sync(side, env, out);
            side.changed();
        }
        Told::Changed { handle: value } if *value == handle() => {
            let Some(document) = client.document(*value) else { return true };
            let Some(Latest::Jig(Delivery::Me(frame))) = document.latest else { return true };
            party(side, env, Some(frame.party.number), out);
        }
        Told::Ended { handle: value, .. } if *value == handle() => {
            side.me = false;
            party(side, env, None, out);
        }
        Told::Changed { .. }
        | Told::Ended { .. }
        | Told::Page { .. }
        | Told::Text { .. }
        | Told::Pending { .. }
        | Told::Answered { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. } => return false,
    }
    true
}
fn party(side: &mut Side, env: &Env<Limits>, party: Option<u64>, out: &mut Queue<Request>) {
    if side.party != party {
        super::hide(side, out);
    }
    side.party = party;
    for (_, id) in &side.names {
        let object = side.objects.get_mut(*id).expect("named object exists");
        if object.terminal == Some(super::FamilyTerminal::SignedOut) && party.is_some() {
            object.terminal = None;
        }
    }
    super::sync(side, env, out);
    super::save(side, out);
    side.changed();
}
