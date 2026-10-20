//! Independent observations of records, authentication and stream isolation (client.md, 14).
use jig_client_protocol_wire::v10 as wire;
use skein_lib::{Reader, Writer};

macro_rules! same {
    ($kind:ident, $encoded:expr, $crossed:expr) => {{
        let mut source = Reader::new($encoded);
        let mut received = Reader::new($crossed);
        let wanted = wire::$kind::decode(&wire::CEILINGS, &mut source).map_err(|_| Broken::Record)?;
        let actual = wire::$kind::decode(&wire::CEILINGS, &mut received).map_err(|_| Broken::Record)?;
        if source.remaining() != 0 || received.remaining() != 0 || actual != wanted {
            return Err(Broken::Record);
        }
        let mut writer = Writer::new(usize::try_from(actual.measure()).expect("bounded wire"));
        actual.encode(&mut writer).map_err(|_| Broken::Record)?;
        if writer.finish().as_ref() != $crossed {
            return Err(Broken::Record);
        }
    }};
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Broken {
    Record,
    Secret,
    Blocked,
}
#[derive(Default)]
pub struct Rules {
    secrets: Vec<Vec<u8>>,
    pub records: u64,
    pub domains: u64,
    pub cuts: u64,
    pub isolated: u64,
}
impl Rules {
    pub fn secret(&mut self, bytes: &[u8]) {
        if !self.secrets.iter().any(|value| value == bytes) {
            self.secrets.push(bytes.to_vec());
        }
    }
    /// Domain inputs are inspected as whole public values, never root state.
    /// # Errors
    /// Reports an authentication secret carried in that input.
    pub fn domain(&mut self, event: &impl std::fmt::Debug) -> Result<(), Broken> {
        let observed = format!("{event:?}");
        if self.secrets.iter().any(|bytes| observed.contains(&format!("{bytes:?}"))) {
            return Err(Broken::Secret);
        }
        self.domains += 1;
        Ok(())
    }
    /// # Errors
    /// Reports an altered or undecodable body after it crosses the stream.
    pub fn record(&mut self, path: &[u8], response: bool, encoded: &[u8], crossed: &[u8]) -> Result<(), Broken> {
        match (path, response) {
            (b"/jig/v1/request", false) => same!(Keyed, encoded, crossed),
            (b"/jig/v1/request", true) => same!(KeyedAnswer, encoded, crossed),
            (b"/jig/v1/read", false) => same!(Owned, encoded, crossed),
            (b"/jig/v1/read", true) => same!(ReadAnswer, encoded, crossed),
            _ => Self::watch_record(path, response, encoded, crossed)?,
        }
        self.records += 1;
        Ok(())
    }
    fn watch_record(path: &[u8], response: bool, encoded: &[u8], crossed: &[u8]) -> Result<(), Broken> {
        match (path, response) {
            (b"/jig/v1/watch", false) => same!(Open, encoded, crossed),
            (b"/jig/v1/watch", true) => same!(OpenAnswer, encoded, crossed),
            (b"/jig/v1/unwatch", false) => same!(WatchClose, encoded, crossed),
            (b"/jig/v1/unwatch", true) => same!(CloseAnswer, encoded, crossed),
            _ => Self::stream_record(path, response, encoded, crossed)?,
        }
        Ok(())
    }
    fn stream_record(path: &[u8], response: bool, encoded: &[u8], crossed: &[u8]) -> Result<(), Broken> {
        match (path, response) {
            (b"/jig/v1/stream-gone", false) => same!(Gone, encoded, crossed),
            (b"/jig/v1/stream", true) => same!(StreamEvent, encoded, crossed),
            (b"/jig/v1/sign-out", false) | (b"/jig/v1/stream-gone", true)
                if encoded.is_empty() && crossed.is_empty() => {}
            _ => Self::auth_record(path, response, encoded, crossed)?,
        }
        Ok(())
    }
    fn auth_record(path: &[u8], response: bool, encoded: &[u8], crossed: &[u8]) -> Result<(), Broken> {
        match (path, response) {
            (b"/jig/v1/sign-in/link", false) => same!(LinkSignIn, encoded, crossed),
            (b"/jig/v1/sign-in/link", true) => same!(SignInAnswer, encoded, crossed),
            (b"/jig/v1/sign-out", true) => same!(SignOutAnswer, encoded, crossed),
            _ => return Err(Broken::Record),
        }
        Ok(())
    }
    /// # Errors
    /// The independently observed other exchange must finish while the stream is unread.
    pub fn isolation(&mut self, unread: bool, other_answered: bool) -> Result<(), Broken> {
        if unread && !other_answered {
            return Err(Broken::Blocked);
        }
        if unread {
            self.isolated += 1;
        }
        Ok(())
    }
}
