//! Client listener, heartbeat and local sign-in link (configuration.md, 3).
use crate::{Address, Path, Problems, check, read};
use skein_json::Document;
use skein_lib::Duration;
#[derive(Debug)]
pub struct Clients {
    pub listener: Address,
    pub heartbeat: Duration,
    pub link: bool,
}
pub(crate) fn parse(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<Clients> {
    let mut object = check::object(document, at, path, problems)?;
    let listener = check::address(&mut object, document, b"listener", path, problems);
    if let Some(address) = &listener {
        check::local(address, &path.member(b"listener"), problems);
    }
    let heartbeat = check::duration(&mut object, document, b"heartbeat_ms", path, problems);
    let mut link = None;
    if let Some(at) = object.required(document, b"link", path, problems) {
        match read::boolean(document, at) {
            Ok(value) => link = Some(value),
            Err(what) => problems.push(path.member(b"link"), what),
        }
    }
    object.finish(document, path, problems);
    Some(Clients { listener: listener?, heartbeat: heartbeat?, link: link? })
}
