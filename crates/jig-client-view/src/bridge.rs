//! Transitional translation onto the application-held client (root.md, 5).
use crate::{Domain, Event, Limits, Request, intents, link};
use jig_client::{Asker, Client, Handle, Key, Outcome, Progress, Readout, Told, Why, contract};
use skein_lib::{Env, Queue, Token};
#[must_use]
pub fn asked(request: &Request, salt: u64, watches: u32) -> Option<jig_client::Event> {
    let handle = |token: Token| Handle { asker: Asker::JIG, number: token.raw() };
    Some(match request {
        Request::Send { request, ask, .. } => jig_client::Event::Send {
            key: Key { asker: Asker::JIG, number: request.raw() ^ salt },
            request: jig_client::Ask::Jig(ask.clone()),
        },
        // Slab tokens carry a 32-bit slot and a 32-bit generation.
        // Read slots follow watch slots, preserving the complete generation.
        Request::Read { read, what } => jig_client::Event::Read {
            handle: Handle {
                asker: Asker::JIG,
                number: read
                    .raw()
                    .checked_add(u64::from(watches) << 32)
                    .expect("checked read slots follow watch slots"),
            },
            read: jig_client::Readable::Jig(what.clone()),
        },
        Request::Open { watch, subject } => {
            jig_client::Event::Watch { handle: handle(*watch), subject: jig_client::Watchable::Jig(subject.clone()) }
        }
        Request::Close { watch } => jig_client::Event::Unwatch { handle: handle(*watch) },
        Request::SignOut => jig_client::Event::SignOut,
        Request::Continue { .. }
        | Request::Show { .. }
        | Request::Save { .. }
        | Request::Erase { .. }
        | Request::SignIn { .. }
        | Request::ShowPart { .. }
        | Request::PartsLeft { .. }
        | Request::Link { .. }
        | Request::Confirmed { .. }
        | Request::Retention { .. }
        | Request::FrameIs { .. }
        | Request::Answered { .. }
        | Request::Unknown { .. }
        | Request::Fact { .. } => return None,
    })
}
pub fn told(domain: &mut Domain, env: &Env<Limits>, client: &Client, told: Told, salt: u64, out: &mut Queue<Request>) {
    let event = match told {
        Told::Changed { handle } => {
            let Some(document) = client.document(handle) else { return };
            let event = match document.state {
                jig_client::State::Behind => contract::WatchEvent::Behind,
                jig_client::State::Live => match document.latest {
                    Some(jig_client::Latest::Jig(delivery)) => contract::WatchEvent::Delivered(delivery.clone()),
                    Some(jig_client::Latest::Family(_)) | None => return,
                },
                jig_client::State::Opening | jig_client::State::Ended => return,
            };
            Event::Watch { watch: Token::new(handle.number), event }
        }
        Told::Pending { key } => {
            let Some(pending) = client.pending(key) else { return };
            let progress = match pending.progress {
                Progress::InFlight => intents::Progress::InFlight,
                Progress::Retrying { .. } => intents::Progress::Retrying,
                Progress::Parked | Progress::NotServed => intents::Progress::Parked,
                Progress::Unknown => intents::Progress::Unknown,
            };
            Event::Progress { request: Token::new(key.number ^ salt), progress }
        }
        Told::Answered { key, outcome } => match outcome {
            Outcome::Answered(jig_client::Answered::Jig(answer)) => {
                Event::Answered { request: Token::new(key.number ^ salt), answer }
            }
            Outcome::Answered(jig_client::Answered::Family(_))
            | Outcome::NotKept
            | Outcome::Limit
            | Outcome::Unconfirmed
            | Outcome::Unknown => {
                Event::Progress { request: Token::new(key.number ^ salt), progress: intents::Progress::Unknown }
            }
        },
        Told::Kept { key, .. } => {
            let Some(pending) = client.pending(key) else { return };
            let requests = domain
                .kept_requests
                .get(&pending.party)
                .copied()
                .unwrap_or(0)
                .checked_add(1)
                .expect("kept requests fit the client bound");
            Event::Kept { scope: pending.party, requests }
        }
        Told::Link { link: state } => {
            match state {
                jig_client::Link::Starting => {
                    starting(domain, env, out);
                    domain.set_link(link::Link::Starting, out);
                }
                jig_client::Link::Offline => domain.set_link(link::Link::Offline, out),
                jig_client::Link::Live => {
                    crate::step(domain, env, Event::Hello { served: true }, out);
                    domain.set_link(link::Link::Live, out);
                }
                jig_client::Link::SignedOut => crate::step(domain, env, Event::SignedOut, out),
                jig_client::Link::OutOfDate => crate::step(domain, env, Event::Hello { served: false }, out),
            }
            if client.link().sign_out_lost {
                crate::step(domain, env, Event::SignOutLost, out);
            }
            return;
        }
        Told::Ended { handle, why } => Event::Ended {
            watch: Token::new(handle.number),
            end: match why {
                Why::Unknown | Why::NotServed => contract::WatchEnd::Unknown,
                Why::NotAllowed => contract::WatchEnd::NotAllowed,
                Why::Busy => contract::WatchEnd::Busy,
                Why::SubjectEnded => contract::WatchEnd::SubjectEnded,
                Why::SignedOut => contract::WatchEnd::SignedOut,
                Why::Limit => contract::WatchEnd::Closed,
            },
        },
        Told::Page { handle, page } => Event::Read {
            read: Token::new(handle.number.checked_sub(u64::from(env.limits.watches) << 32).expect("view read slot")),
            result: match page {
                Readout::Jig(result) => result,
                Readout::Limit => contract::ReadResult::None(contract::ReadNone::Busy),
                Readout::NotServed { .. } | Readout::Family { .. } => {
                    contract::ReadResult::None(contract::ReadNone::Refused)
                }
            },
        },
        Told::Text { .. } | Told::Dropped { .. } => return,
    };
    crate::step(domain, env, event, out);
}

fn starting(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if !domain.started {
        return;
    }
    if let Some(id) = domain.frame_watch {
        let closing = match domain.watches.get(id) {
            Some(watch) => watch.state == crate::watches::Following::Closing,
            None => false,
        };
        if closing {
            crate::step(domain, env, Event::Ended { watch: id.token(), end: contract::WatchEnd::Closed }, out);
        }
    }
    domain.signin.returning = domain.frame_watch.is_none();
    crate::signin::returned(domain, out);
}

/// The client keeps the engine's closing obligation after Unwatch. The
/// presentation handle is already released and may retire in this step.
pub fn closed(domain: &mut Domain, env: &Env<Limits>, watch: Token, out: &mut Queue<Request>) {
    crate::step(domain, env, Event::Ended { watch, end: contract::WatchEnd::Closed }, out);
}
