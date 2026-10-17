//! The ordinary fronts encode family values before returning client callbacks.
use jig_client::{Family, Readout, Up};
use jig_ops_client_domain::{self as d, Event, infrastructure as infra, observability as obs};
use skein_lib::{RequestEnvelope, Token};
fn family(number: u16, bytes: Box<[u8]>) -> Family {
    Family { family: number, version: jig_ops_client_protocol_wire::VERSION, bytes }
}
pub fn answer(attempt: Token, envelope: RequestEnvelope, answer: Option<d::Answered>) -> Event {
    let answer = answer.map(|value| match value {
        d::Answered::Jig(value) => jig_client::Answered::Jig(value),
        d::Answered::Infrastructure(infra::Answer::EntranceRefused(why)) => {
            jig_client::Answered::Jig(jig_client::contract::Answer::Refused(why.into_refusal()))
        }
        d::Answered::Infrastructure(value) => {
            jig_client::Answered::Family(family(2, d::wire::answer(&value).expect("whole infrastructure answer")))
        }
    });
    Event::Client { event: jig_client::Event::Up { up: Up::Answer { attempt, envelope, answer, why: None } } }
}
macro_rules! side {
    ($name:ident, $d:ident, $wire:ident, $number:literal) => {
        fn $name(event: $d::Event) -> Option<Up> {
            Some(match event {
                $d::Event::Read { read, result } => Up::ReadDone {
                    read,
                    result: match result {
                        $d::ReadResult::Value(value) => Readout::Family {
                            family: $number,
                            record: Some(
                                d::wire::$wire::read_answer(&$d::ReadResult::Value(value)).expect("whole family read"),
                            ),
                        },
                        $d::ReadResult::Busy => {
                            Readout::Jig(jig_client::contract::ReadResult::None(jig_client::contract::ReadNone::Busy))
                        }
                        $d::ReadResult::SignedOut => Readout::Jig(jig_client::contract::ReadResult::None(
                            jig_client::contract::ReadNone::SignedOut,
                        )),
                        $d::ReadResult::NotAllowed => Readout::Family {
                            family: $number,
                            record: Some(
                                d::wire::$wire::read_answer(&$d::ReadResult::NotAllowed).expect("whole read refusal"),
                            ),
                        },
                        $d::ReadResult::Unknown => Readout::Family { family: $number, record: None },
                    },
                },
                $d::Event::Watch { watch, event } => match event {
                    $d::WatchEvent::Delivered(value) => Up::FamilyDelivered {
                        watch,
                        record: family($number, d::wire::$wire::document(&value).expect("whole family delivery")),
                    },
                    $d::WatchEvent::Opened => Up::Watched { watch, event: jig_client::contract::WatchEvent::Opened },
                    $d::WatchEvent::Alive => Up::Watched { watch, event: jig_client::contract::WatchEvent::Alive },
                },
                $d::Event::Ended { watch, end } => Up::WatchEnded {
                    watch,
                    end: match end {
                        $d::WatchEnd::NotAllowed => jig_client::contract::WatchEnd::NotAllowed,
                        $d::WatchEnd::SubjectEnded => jig_client::contract::WatchEnd::SubjectEnded,
                        $d::WatchEnd::Stopping => jig_client::contract::WatchEnd::Stopping,
                        $d::WatchEnd::Busy => jig_client::contract::WatchEnd::Busy,
                        $d::WatchEnd::Closed => jig_client::contract::WatchEnd::Closed,
                        $d::WatchEnd::Dropped => jig_client::contract::WatchEnd::Dropped,
                        $d::WatchEnd::Unknown => jig_client::contract::WatchEnd::Unknown,
                        $d::WatchEnd::SignedOut => jig_client::contract::WatchEnd::SignedOut,
                    },
                },
                _ => return None,
            })
        }
    };
}
// The source events are local to the fixture; only transport deliveries are converted.
side!(infrastructure, infra, infrastructure, 2);
side!(observability, obs, observability, 1);
pub fn events(
    values: Vec<(jig_client_world::harness::ClientId, Event)>,
) -> Vec<(jig_client_world::harness::ClientId, Event)> {
    let mut out = Vec::new();
    for (client, event) in values {
        let up = match event {
            Event::Jig(jig_client_view::Event::Hello { served }) => {
                out.push((
                    client,
                    Event::Client { event: jig_client::Event::Up { up: Up::Served { served, versions: None } } },
                ));
                for family in [1, 2] {
                    out.push((
                        client,
                        Event::Client {
                            event: jig_client::Event::Up {
                                up: Up::FamilyServed {
                                    family,
                                    versions: Some(jig_client::contract::VersionRange { first: 1, last: 1 }),
                                },
                            },
                        },
                    ));
                }
                continue;
            }
            Event::Jig(event) => {
                match jig_client_world::joint::transport::contract_event(&jig_client_test_domain::Event::Jig(event)) {
                    Some(up) => up,
                    None => continue,
                }
            }
            Event::Infrastructure(event) => match infrastructure(event) {
                Some(up) => up,
                None => continue,
            },
            Event::Observability(event) => match observability(event) {
                Some(up) => up,
                None => continue,
            },
            event @ (Event::Client { .. }
            | Event::SignedOut
            | Event::Act { .. }
            | Event::Went { .. }
            | Event::Start { .. }) => {
                out.push((client, event));
                continue;
            }
        };
        out.push((client, Event::Client { event: jig_client::Event::Up { up } }));
    }
    out
}
