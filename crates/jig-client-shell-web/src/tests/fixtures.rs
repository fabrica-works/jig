use crate::{adapter::Asked, sign_in::Route};
use jig_client_protocol_http::Call;
use jig_client_protocol_http::{Mapping, exchange as mapped};
use skein_http::Method;
use skein_lib::List;

pub const ID: u32 = 1;
pub const BOUND: u32 = 16;
pub const SECRET: &[u8] = b"BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc";
pub fn exchange(id: u32, body: &[u8]) -> Asked {
    // Use the mapping's actual call so the header check covers both components.
    let mapping = Mapping::new(b"/jig").expect("mapping");
    let call = mapped(&mapping, None, jig_client_protocol_link::Message::Read(body.into()));
    Asked::Exchange { id, call, bound: BOUND }
}
pub fn stream(piece: u32) -> Asked {
    Asked::OpenStream {
        call: Call {
            method: Method::Get,
            path: b"/jig/v1/stream".as_slice().into(),
            headers: List::with_capacity(0),
            body: Box::new([]),
        },
        piece,
    }
}
pub fn save(key: &[u8], value: &[u8]) -> Asked {
    Asked::Save { key: key.into(), value: value.into() }
}
pub fn route() -> Route<'static> {
    Route { path: b"/jig/v1/sign-in/link", home: b"/" }
}

#[derive(Default)]
pub struct Platform {
    pub calls: Vec<&'static str>,
}
impl skein_ui_web::platform::Platform for Platform {
    fn read_all(&self) -> Vec<skein_ui_web::store::Record> {
        Vec::new()
    }
    fn write(&mut self, _: &[u8], _: &[u8]) -> bool {
        self.calls.push("write");
        true
    }
    fn remove(&mut self, _: &[u8]) {
        self.calls.push("remove");
    }
    fn send(&mut self, _: &skein_ui_web::fetch::Exchange) {
        self.calls.push("send");
    }
    fn navigate(&mut self, _: &[u8]) {
        self.calls.push("navigate");
    }
    fn stream(&mut self, _: skein_ui_web::transport::StreamDuty) {
        self.calls.push("stream");
    }
}
