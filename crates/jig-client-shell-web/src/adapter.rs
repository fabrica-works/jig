//! The mapping's calls carried by skein's transport (protocol/browser.md, 10).
use jig_client_protocol_http::Call;
use skein_ui_web::{fetch, store, transport};

/// A step's HTTP and small-store duties, before touching the platform.
#[derive(Debug)]
pub enum Asked {
    Exchange { id: u32, call: Call, bound: u32 },
    OpenStream { call: Call, piece: u32 },
    CloseStream,
    Save { key: Box<[u8]>, value: Box<[u8]> },
    Erase { key: Box<[u8]> },
}
/// A platform report to translate into the application's host event.
#[derive(Debug, PartialEq, Eq)]
pub enum Heard {
    Answered { exchange: u32, status: u16, body: Box<[u8]> },
    Failed { exchange: u32 },
    StreamOpened,
    StreamRefused { status: u16 },
    Piece { bytes: Box<[u8]> },
    StreamEnded,
    Stored { key: Box<[u8]> },
    NotStored { key: Box<[u8]> },
}
/// Hand the entire step to the transport, which writes before sending.
#[must_use]
pub fn requests(asked: Vec<Asked>) -> transport::Requests {
    let mut requests = transport::Requests::default();
    for ask in asked {
        match ask {
            Asked::Save { key, value } => requests.writes.push(store::Record { key, value }),
            Asked::Erase { key } => requests.removes.push(key),
            Asked::CloseStream => requests.stream = Some(transport::StreamDuty::Close),
            Asked::OpenStream { call, piece } => {
                requests.stream =
                    Some(transport::StreamDuty::Open { path: call.path, headers: headers(call.headers), bound: piece });
            }
            Asked::Exchange { id, call, bound } => {
                let headers =
                    headers(call.headers).into_iter().map(|(name, value)| (text(name), text(value))).collect();
                requests.exchanges.push(fetch::Exchange {
                    id,
                    path: call.path,
                    method: text(call.method.as_bytes().into()),
                    headers,
                    body: call.body,
                    bound,
                });
            }
        }
    }
    requests
}
fn headers(headers: skein_lib::List<skein_http::Header>) -> Vec<transport::Header> {
    let mut result = vec![(Box::from(b"Jig-Client".as_slice()), Box::from(b"browser".as_slice()))];
    for header in headers.into_boxed().into_vec() {
        result.push((header.name, header.value));
    }
    result
}
fn text(bytes: Box<[u8]>) -> String {
    String::from_utf8(bytes.into_vec()).expect("the HTTP mapping gives UTF-8")
}
#[must_use]
pub fn fetched(report: fetch::Report) -> Heard {
    match report {
        fetch::Report::Answered { exchange, status, body } => Heard::Answered { exchange, status, body },
        fetch::Report::Failed { exchange } => Heard::Failed { exchange },
        fetch::Report::StreamOpened => Heard::StreamOpened,
        fetch::Report::StreamRefused { status } => Heard::StreamRefused { status },
        fetch::Report::StreamPiece { bytes } => Heard::Piece { bytes },
        fetch::Report::StreamEnded => Heard::StreamEnded,
    }
}
#[must_use]
pub fn carried(report: transport::Carried) -> Heard {
    match report {
        transport::Carried::Written { key } => Heard::Stored { key },
        transport::Carried::Refused { key } => Heard::NotStored { key },
        transport::Carried::Failed { exchange } => Heard::Failed { exchange },
    }
}
